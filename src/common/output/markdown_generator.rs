//! Common markdown generation utilities
//!
//! Provides shared markdown generation patterns that can be used by all platforms
//! to eliminate duplication in output formatting.

use crate::Result;
use regex::Regex;
use std::path::{Path, PathBuf};
use tracing::{error, info};

/// Common markdown generation utilities
pub struct MarkdownGenerator {
    output_dir: PathBuf,
}

impl MarkdownGenerator {
    /// Create new markdown generator with output directory
    pub fn new(output_dir: Option<PathBuf>) -> Self {
        let output_dir = output_dir.unwrap_or_else(|| {
            let home = dirs::home_dir().expect("Could not find home directory");
            home.join("Desktop").join("cs-cli-output")
        });
        
        Self { output_dir }
    }
    
    /// Get output directory
    pub fn output_dir(&self) -> &PathBuf {
        &self.output_dir
    }
    
    /// Create customer-specific output directory using common utilities
    pub fn create_customer_dir(&self, customer_name: &str) -> Result<PathBuf> {
        crate::common::file_io::create_customer_directory(customer_name)
    }
    
    /// Sanitize filename using common utilities
    pub fn sanitize_filename(&self, filename: &str) -> String {
        crate::common::file_io::sanitize_filename(filename)
    }
    
    /// Format date for filename using common utilities
    pub fn format_date_for_filename(&self, date: &str) -> String {
        crate::common::file_io::format_date_for_filename(date)
    }
    
    /// Create markdown header with metadata
    pub fn create_metadata_header(
        &self,
        title: &str,
        platform: &str,
        date: &str,
        participants: Option<&[String]>,
        additional_metadata: Option<&std::collections::HashMap<String, String>>,
    ) -> String {
        let mut header = format!("# {}\n\n", title);
        
        // Add metadata table
        header.push_str("| Field | Value |\n");
        header.push_str("|-------|-------|\n");
        header.push_str(&format!("| Platform | {} |\n", platform));
        header.push_str(&format!("| Date | {} |\n", date));
        
        if let Some(participants) = participants {
            header.push_str(&format!("| Participants | {} |\n", participants.join(", ")));
        }
        
        if let Some(metadata) = additional_metadata {
            for (key, value) in metadata {
                header.push_str(&format!("| {} | {} |\n", key, value));
            }
        }
        
        header.push_str("\n---\n\n");
        header
    }
    
    /// Clean and format text content for markdown
    pub fn clean_text_content(&self, content: &str) -> String {
        if content.trim().is_empty() {
            return "*No content available*".to_string();
        }
        
        // Basic text cleaning
        let cleaned = content
            .replace('\r', "") // Remove carriage returns
            .replace('\t', "    ") // Convert tabs to spaces
            .lines()
            .map(|line| line.trim_end()) // Remove trailing whitespace
            .collect::<Vec<_>>()
            .join("\n");
        
        // Remove excessive blank lines (more than 2 consecutive)
        let re = Regex::new(r"\n{3,}").unwrap();
        re.replace_all(&cleaned, "\n\n").to_string()
    }
    
    /// Format HTML content to markdown (basic conversion)
    pub fn html_to_markdown(&self, html_content: &str) -> String {
        // Use htmd for HTML to markdown conversion if available
        // This is a simplified version - platforms can override with more sophisticated conversion
        if html_content.trim().is_empty() {
            return "*No content available*".to_string();
        }
        
        // Basic HTML tag removal and formatting
        let cleaned = html_content
            .replace("<br>", "\n")
            .replace("<br/>", "\n")
            .replace("<br />", "\n")
            .replace("<p>", "\n")
            .replace("</p>", "\n")
            .replace("<div>", "\n")
            .replace("</div>", "\n");
        
        // Remove HTML tags (basic regex)
        let re = Regex::new(r"<[^>]+>").unwrap();
        let text_only = re.replace_all(&cleaned, "");
        
        self.clean_text_content(&text_only)
    }
    
    /// Save content to file using common utilities
    pub fn save_content_to_file(&self, filepath: &Path, content: &str) -> Result<()> {
        crate::common::file_io::write_file(filepath, content)
    }
    
    /// Batch process items and save to multiple files
    pub fn save_batch_files<T>(
        &self,
        items: &[T],
        batch_size: usize,
        filename_prefix: &str,
        content_generator: impl Fn(&[T], usize) -> String,
    ) -> Result<Vec<PathBuf>> {
        if items.is_empty() {
            info!("No items to save");
            return Ok(Vec::new());
        }
        
        let mut saved_files = Vec::new();
        
        for (batch_index, batch) in items.chunks(batch_size).enumerate() {
            let batch_num = batch_index + 1;
            let filename = format!("{}-batch-{}.md", filename_prefix, batch_num);
            let filepath = self.output_dir.join(&filename);
            
            let content = content_generator(batch, batch_num);
            
            match self.save_content_to_file(&filepath, &content) {
                Ok(_) => {
                    saved_files.push(filepath);
                    info!("Saved batch {} ({} items) to {}", batch_num, batch.len(), filename);
                }
                Err(e) => {
                    error!("Failed to save batch {}: {}", batch_num, e);
                    continue;
                }
            }
        }
        
        info!("Saved {} items across {} batch files", items.len(), saved_files.len());
        Ok(saved_files)
    }
}

/// Platform-specific markdown formatting
pub struct PlatformMarkdownFormatter;

impl PlatformMarkdownFormatter {
    /// Format Gong-specific content
    pub fn format_gong_content(&self, title: &str, call_data: &str, email_data: Option<&str>) -> String {
        let mut content = format!("# {}\n\n", title);
        
        if !call_data.is_empty() {
            content.push_str("## Call Information\n\n");
            content.push_str(call_data);
            content.push_str("\n\n");
        }
        
        if let Some(email_data) = email_data {
            content.push_str("## Email Information\n\n");
            content.push_str(email_data);
            content.push_str("\n\n");
        }
        
        content
    }
    
    /// Format Slack-specific content
    pub fn format_slack_content(&self, title: &str, conversation_data: &str, channel_info: Option<&str>) -> String {
        let mut content = format!("# {}\n\n", title);
        
        if let Some(channel_info) = channel_info {
            content.push_str("## Channel Information\n\n");
            content.push_str(channel_info);
            content.push_str("\n\n");
        }
        
        content.push_str("## Conversation\n\n");
        content.push_str(conversation_data);
        content.push_str("\n\n");
        
        content
    }
    
    /// Format Gainsight-specific content
    pub fn format_gainsight_content(&self, title: &str, record_data: &str, relationship_data: Option<&str>) -> String {
        let mut content = format!("# {}\n\n", title);
        
        content.push_str("## Record Information\n\n");
        content.push_str(record_data);
        content.push_str("\n\n");
        
        if let Some(relationship_data) = relationship_data {
            content.push_str("## Relationship Data\n\n");
            content.push_str(relationship_data);
            content.push_str("\n\n");
        }
        
        content
    }
}