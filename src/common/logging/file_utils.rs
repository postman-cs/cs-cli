//! Logging file utilities
//!
//! Common file operations for logging and log management.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use crate::CsCliError;

/// Write cookie validation log to file
pub fn write_cookie_validation_log(
    log_file_path: &PathBuf,
    cookie_log: &[String],
) -> Result<(), CsCliError> {
    // Write cookie log to file
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_file_path)
        .map_err(|e| CsCliError::FileIo(format!("Failed to create cookie log file: {}", e)))?;
    
    // Write header
    writeln!(file, "CS-CLI Cookie Validation Log")
        .map_err(|e| CsCliError::FileIo(format!("Failed to write to cookie log: {}", e)))?;
    
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    writeln!(file, "Generated: {}", now)
        .map_err(|e| CsCliError::FileIo(format!("Failed to write to cookie log: {}", e)))?;
    
    writeln!(file, "{}", "=".repeat(50))
        .map_err(|e| CsCliError::FileIo(format!("Failed to write to cookie log: {}", e)))?;
    
    writeln!(file)
        .map_err(|e| CsCliError::FileIo(format!("Failed to write to cookie log: {}", e)))?;
    
    // Write cookie details for each platform
    for platform_log in cookie_log {
        write!(file, "{}", platform_log)
            .map_err(|e| CsCliError::FileIo(format!("Failed to write to cookie log: {}", e)))?;
    }
    
    // Write summary
    writeln!(file, "=== SUMMARY ===")
        .map_err(|e| CsCliError::FileIo(format!("Failed to write to cookie log: {}", e)))?;
    
    Ok(())
}

/// Clean up old log files (keep only last N files)
pub fn cleanup_old_logs(log_dir: &PathBuf, keep_count: usize) -> Result<(), CsCliError> {
    if !log_dir.exists() {
        return Ok(());
    }
    
    let mut log_files: Vec<_> = std::fs::read_dir(log_dir)
        .map_err(|e| CsCliError::FileIo(format!("Failed to read log directory: {}", e)))?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()?.to_str()? == "txt" || path.extension()?.to_str()? == "log" {
                let metadata = entry.metadata().ok()?;
                Some((path, metadata.modified().ok()?))
            } else {
                None
            }
        })
        .collect();
    
    // Sort by modification time (newest first)
    log_files.sort_by(|a, b| b.1.cmp(&a.1));
    
    // Remove old files
    for (path, _) in log_files.into_iter().skip(keep_count) {
        if let Err(e) = std::fs::remove_file(&path) {
            tracing::warn!("Failed to remove old log file {}: {}", path.display(), e);
        }
    }
    
    Ok(())
}