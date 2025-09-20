//! File operations utilities
//!
//! Common file reading, writing, and management patterns.

use crate::{CsCliError, Result};
use std::fs;
use std::path::Path;
use tracing::{debug, info};

/// Write content to file with error handling
pub fn write_file(path: &Path, content: &str) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        crate::common::file_io::create_directory(parent)?;
    }
    
    fs::write(path, content)
        .map_err(|e| CsCliError::FileIo(format!("Failed to write file {}: {}", path.display(), e)))?;
    
    info!("Wrote file: {}", path.display());
    Ok(())
}

/// Read file content with error handling
pub fn read_file(path: &Path) -> Result<String> {
    fs::read_to_string(path)
        .map_err(|e| CsCliError::FileIo(format!("Failed to read file {}: {}", path.display(), e)))
}

/// Copy file with error handling
pub fn copy_file(source: &Path, dest: &Path) -> Result<u64> {
    // Ensure destination directory exists
    if let Some(parent) = dest.parent() {
        crate::common::file_io::create_directory(parent)?;
    }
    
    fs::copy(source, dest)
        .map_err(|e| CsCliError::FileIo(format!("Failed to copy file {} to {}: {}", source.display(), dest.display(), e)))
}

/// Check if file exists
pub fn file_exists(path: &Path) -> bool {
    path.exists() && path.is_file()
}

/// Get file size
pub fn get_file_size(path: &Path) -> Result<u64> {
    let metadata = fs::metadata(path)
        .map_err(|e| CsCliError::FileIo(format!("Failed to get metadata for {}: {}", path.display(), e)))?;
    Ok(metadata.len())
}

/// Create file with content if it doesn't exist
pub fn create_file_if_not_exists(path: &Path, content: &str) -> Result<bool> {
    if file_exists(path) {
        return Ok(false); // File already exists
    }
    
    write_file(path, content)?;
    Ok(true) // File was created
}

/// Append content to file
pub fn append_to_file(path: &Path, content: &str) -> Result<()> {
    use std::fs::OpenOptions;
    use std::io::Write;
    
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        crate::common::file_io::create_directory(parent)?;
    }
    
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| CsCliError::FileIo(format!("Failed to open file {} for appending: {}", path.display(), e)))?;
    
    file.write_all(content.as_bytes())
        .map_err(|e| CsCliError::FileIo(format!("Failed to append to file {}: {}", path.display(), e)))?;
    
    debug!("Appended to file: {}", path.display());
    Ok(())
}

/// Remove file with error handling
pub fn remove_file(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_file(path)
            .map_err(|e| CsCliError::FileIo(format!("Failed to remove file {}: {}", path.display(), e)))?;
        debug!("Removed file: {}", path.display());
    }
    Ok(())
}

/// Create temporary file with content
pub fn create_temp_file(_prefix: &str, content: &str) -> Result<std::path::PathBuf> {
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    let mut temp_file = NamedTempFile::new()
        .map_err(|e| CsCliError::FileIo(format!("Failed to create temporary file: {}", e)))?;
    
    temp_file.write_all(content.as_bytes())
        .map_err(|e| CsCliError::FileIo(format!("Failed to write to temporary file: {}", e)))?;
    
    let path = temp_file.path().to_path_buf();
    temp_file.persist(&path)
        .map_err(|e| CsCliError::FileIo(format!("Failed to persist temporary file: {}", e)))?;
    
    debug!("Created temporary file: {}", path.display());
    Ok(path)
}