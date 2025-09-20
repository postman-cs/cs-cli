//! Path utilities
//!
//! Common path manipulation and validation patterns.

use crate::{CsCliError, Result};
use std::path::{Path, PathBuf};
use regex::Regex;

/// Sanitize filename by removing invalid characters
pub fn sanitize_filename(filename: &str) -> String {
    if filename.trim().is_empty() {
        return "unnamed".to_string();
    }
    
    // Remove or replace invalid filename characters
    let sanitized = filename
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect::<String>();
    
    // Remove excessive whitespace and special characters
    let re = Regex::new(r"[._]+").unwrap();
    let sanitized = re.replace_all(&sanitized, "_");
    
    // Collapse multiple consecutive underscores
    let re = Regex::new(r"_+").unwrap();
    let sanitized = re.replace_all(&sanitized, "_");
    
    // Remove leading/trailing underscores and spaces
    let sanitized = sanitized.trim_matches(|c| c == ' ' || c == '_');
    
    // Limit length
    let sanitized = if sanitized.len() > 100 {
        format!("{}...", &sanitized[..97])
    } else {
        sanitized.to_string()
    };
    
    if sanitized.is_empty() {
        "unnamed".to_string()
    } else {
        sanitized
    }
}

/// Format date for filename (YYYY-MM-DD)
pub fn format_date_for_filename(date: &str) -> String {
    // Try to parse various date formats and convert to YYYY-MM-DD
    if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(date) {
        parsed.format("%Y-%m-%d").to_string()
    } else if let Ok(parsed) = chrono::NaiveDateTime::parse_from_str(date, "%Y-%m-%d %H:%M:%S") {
        parsed.format("%Y-%m-%d").to_string()
    } else {
        // Fallback to current date if parsing fails
        chrono::Local::now().format("%Y-%m-%d").to_string()
    }
}

/// Get file extension
pub fn get_file_extension(path: &Path) -> Option<String> {
    path.extension()?.to_str()?.to_string().into()
}

/// Change file extension
pub fn change_file_extension(path: &Path, new_extension: &str) -> PathBuf {
    let mut path_buf = path.to_path_buf();
    path_buf.set_extension(new_extension);
    path_buf
}

/// Get filename without extension
pub fn get_filename_without_extension(path: &Path) -> Option<String> {
    path.file_stem()?.to_str()?.to_string().into()
}

/// Join paths safely
pub fn join_paths(base: &Path, relative: &str) -> PathBuf {
    base.join(relative)
}

/// Get relative path from base
pub fn get_relative_path(path: &Path, base: &Path) -> Result<PathBuf> {
    path.strip_prefix(base)
        .map(|p| p.to_path_buf())
        .map_err(|e| CsCliError::FileIo(format!("Failed to get relative path: {}", e)))
}

/// Check if path is absolute
pub fn is_absolute_path(path: &Path) -> bool {
    path.is_absolute()
}

/// Convert to absolute path
pub fn to_absolute_path(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        std::env::current_dir()
            .map_err(|e| CsCliError::FileIo(format!("Failed to get current directory: {}", e)))?
            .join(path)
            .canonicalize()
            .map_err(|e| CsCliError::FileIo(format!("Failed to canonicalize path: {}", e)))
    }
}

/// Create unique filename by adding counter if file exists
pub fn create_unique_filename(base_path: &Path) -> PathBuf {
    if !base_path.exists() {
        return base_path.to_path_buf();
    }
    
    let stem = get_filename_without_extension(base_path).unwrap_or_else(|| "file".to_string());
    let extension = get_file_extension(base_path).unwrap_or_else(|| "txt".to_string());
    let parent = base_path.parent().unwrap_or(Path::new("."));
    
    let mut counter = 1;
    loop {
        let new_name = format!("{}_{}.{}", stem, counter, extension);
        let new_path = parent.join(new_name);
        
        if !new_path.exists() {
            return new_path;
        }
        
        counter += 1;
        
        // Prevent infinite loop
        if counter > 1000 {
            break;
        }
    }
    
    // Fallback with timestamp
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let new_name = format!("{}_{}.{}", stem, timestamp, extension);
    parent.join(new_name)
}