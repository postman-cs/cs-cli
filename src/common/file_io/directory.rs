//! Directory management utilities
//!
//! Common directory creation, copying, and management patterns.

use crate::{CsCliError, Result};
use crate::common::sanitize_filename;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Create directory with error handling
pub fn create_directory(path: &Path) -> Result<()> {
    fs::create_dir_all(path)
        .map_err(|e| CsCliError::FileIo(format!("Failed to create directory {}: {}", path.display(), e)))?;
    
    debug!("Created directory: {}", path.display());
    Ok(())
}

/// Create directory and return the path
pub fn create_directory_path(path: &Path) -> Result<PathBuf> {
    create_directory(path)?;
    Ok(path.to_path_buf())
}

/// Create customer-specific directory on Desktop
pub fn create_customer_directory(customer_name: &str) -> Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| CsCliError::FileIo("Could not find home directory".to_string()))?;
    
    let desktop_path = home.join("Desktop");
    let sanitized_name = sanitize_filename(customer_name);
    let customer_dir = desktop_path.join(format!("ct_{}", sanitized_name));
    
    create_directory(&customer_dir)?;
    info!("Created customer directory: {}", customer_dir.display());
    Ok(customer_dir)
}

/// Create temporary directory with unique name
pub fn create_temp_directory(prefix: &str) -> Result<PathBuf> {
    let temp_dir = std::env::temp_dir()
        .join(format!("{}-{}", prefix, std::process::id()));
    
    create_directory(&temp_dir)?;
    debug!("Created temporary directory: {}", temp_dir.display());
    Ok(temp_dir)
}

/// Copy directory recursively
pub fn copy_directory_recursive(source: &Path, dest: &Path) -> Result<u64> {
    if !source.exists() {
        return Err(CsCliError::FileIo(format!("Source directory does not exist: {}", source.display())));
    }

    create_directory(dest)?;

    let mut total_bytes = 0u64;

    for entry in fs::read_dir(source)
        .map_err(|e| CsCliError::FileIo(format!("Failed to read directory {}: {}", source.display(), e)))? {
        let entry = entry
            .map_err(|e| CsCliError::FileIo(format!("Failed to read directory entry: {}", e)))?;
        let source_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if source_path.is_dir() {
            // Recursively copy subdirectory
            match copy_directory_recursive(&source_path, &dest_path) {
                Ok(bytes) => total_bytes += bytes,
                Err(e) => debug!("Failed to copy subdirectory {}: {}", source_path.display(), e),
            }
        } else {
            // Copy file
            match fs::copy(&source_path, &dest_path) {
                Ok(bytes) => {
                    total_bytes += bytes;
                    debug!("Copied file: {} ({} bytes)", entry.file_name().to_string_lossy(), bytes);
                }
                Err(e) => debug!("Failed to copy file {}: {}", source_path.display(), e),
            }
        }
    }

    Ok(total_bytes)
}

/// Clean up directory (remove all contents)
pub fn cleanup_directory(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    
    for entry in fs::read_dir(path)
        .map_err(|e| CsCliError::FileIo(format!("Failed to read directory {}: {}", path.display(), e)))? {
        let entry = entry
            .map_err(|e| CsCliError::FileIo(format!("Failed to read directory entry: {}", e)))?;
        let entry_path = entry.path();
        
        if entry_path.is_dir() {
            fs::remove_dir_all(&entry_path)
                .map_err(|e| CsCliError::FileIo(format!("Failed to remove directory {}: {}", entry_path.display(), e)))?;
        } else {
            fs::remove_file(&entry_path)
                .map_err(|e| CsCliError::FileIo(format!("Failed to remove file {}: {}", entry_path.display(), e)))?;
        }
    }
    
    debug!("Cleaned up directory: {}", path.display());
    Ok(())
}

/// Remove directory and all contents
pub fn remove_directory(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_dir_all(path)
            .map_err(|e| CsCliError::FileIo(format!("Failed to remove directory {}: {}", path.display(), e)))?;
        debug!("Removed directory: {}", path.display());
    }
    Ok(())
}

/// Check if directory is empty
pub fn is_directory_empty(path: &Path) -> Result<bool> {
    if !path.exists() {
        return Ok(true);
    }
    
    let mut entries = fs::read_dir(path)
        .map_err(|e| CsCliError::FileIo(format!("Failed to read directory {}: {}", path.display(), e)))?;
    
    Ok(entries.next().is_none())
}