use anyhow::{Context, Result};
use jiff::Zoned;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{error, info};

use crate::gong::models::{Call, Email};

/// Formatter for generating markdown reports from team calls and emails
pub struct CallMarkdownFormatter {
    /// Output directory for markdown files
    output_dir: PathBuf,
}

impl CallMarkdownFormatter {
    /// Create a new markdown formatter
    pub fn new(output_dir: Option<PathBuf>) -> Self {
        let output_dir = output_dir.unwrap_or_else(|| {
            // Default to desktop for easy access
            let home = dirs::home_dir().expect("Could not find home directory");
            home.join("Desktop").join("team-calls-output")
        });

        Self { output_dir }
    }

    /// Format a single call into markdown content
    pub fn format_call_to_markdown(&self, call: &Call) -> String {
        // Extract call information
        let title = &call.title;
        let customer = call
            .customer_name
            .as_deref()
            .unwrap_or("Unknown Customer");
        let date = &call.scheduled_start;
        let attendees = &call.participants;
        let transcript = call
            .transcript
            .as_deref()
            .unwrap_or("No transcript available");
        let call_id = &call.id;
        let call_url = call.recording_url.as_deref().unwrap_or("");

        // Format date as ISO string to match Python
        let formatted_date = date.strftime("%Y-%m-%dT%H:%M:%S").to_string();

        // Build markdown content - use title as header to match Python
        let mut markdown_content = format!(
            "# {title}\n\n**Customer:** {customer}\n**Date:** {formatted_date}\n**Call ID:** `{call_id}`"
        );

        // Add call URL if available
        if !call_url.is_empty() {
            markdown_content.push_str(&format!("\n**Call Link:** {call_url}"));
        }

        markdown_content.push_str("\n\n## Attendees\n\n");

        // Add attendees section
        if !attendees.is_empty() {
            for attendee in attendees {
                let name = &attendee.name;
                let title = attendee.title.as_deref().unwrap_or("");
                let company = attendee.company.as_deref().unwrap_or("");
                let email = attendee.email.as_deref().unwrap_or("");

                markdown_content.push_str(&format!("- **{name}**"));
                if !title.is_empty() {
                    markdown_content.push_str(&format!(" - {title}"));
                }
                if !company.is_empty() {
                    markdown_content.push_str(&format!(" ({company})"));
                }
                if !email.is_empty() {
                    markdown_content.push_str(&format!(" - {email}"));
                }
                markdown_content.push('\n');
            }
        } else {
            markdown_content.push_str("No attendee information available.\n");
        }

        // Add transcript section
        let cleaned_transcript = self.clean_transcript(transcript);
        let generated_time = Zoned::now().strftime("%Y-%m-%d %H:%M:%S").to_string();

        markdown_content.push_str(&format!(
            "\n\n## Transcript\n\n{cleaned_transcript}\n\n---\n*Generated on {generated_time}*\n"
        ));

        markdown_content
    }

    /// Save a call as a markdown file with proper naming
    pub fn save_call_markdown(&self, call: &Call) -> Result<PathBuf> {
        // Extract info for filename
        let date = &call.scheduled_start;

        // Format date for filename
        let file_date = self.format_date_for_filename(date);

        // Create filename - prefer generatedTitle, fallback to customer_name or extracted customer
        let call_id_suffix = if call.id.len() >= 8 {
            format!("-{}", &call.id[..8])
        } else {
            format!("-{}", call.id)
        };

        // Use generated_title if available, otherwise customer_name, otherwise extract from title
        let filename_base = if let Some(generated_title) = &call.generated_title {
            if !generated_title.trim().is_empty() {
                self.sanitize_filename(generated_title)
            } else if let Some(customer_name) = &call.customer_name {
                self.sanitize_filename(customer_name)
            } else {
                let customer = self.extract_customer_name(call);
                self.sanitize_filename(&customer)
            }
        } else if let Some(customer_name) = &call.customer_name {
            self.sanitize_filename(customer_name)
        } else {
            let customer = self.extract_customer_name(call);
            self.sanitize_filename(&customer)
        };

        let filename = format!("{filename_base}-{file_date}{call_id_suffix}.md");
        let filepath = self.output_dir.join(filename);

        // Generate markdown content
        let markdown_content = self.format_call_to_markdown(call);

        // Ensure output directory exists
        fs::create_dir_all(&self.output_dir).context("Failed to create output directory")?;

        // Write file
        fs::write(&filepath, markdown_content)
            .with_context(|| format!("Failed to write markdown file: {}", filepath.display()))?;

        info!("Saved call markdown to {}", filepath.display());
        Ok(filepath)
    }

    /// Save multiple calls as markdown files
    pub fn save_multiple_calls(
        &self,
        calls: &[Call],
        custom_dir_name: Option<&str>,
    ) -> Result<Vec<PathBuf>> {
        let mut saved_files = Vec::new();

        // Determine output directory - respect configured output_dir or use Desktop for CLI
        let output_dir = {
            // Check if we're using the default Desktop path (CLI behavior)
            let home = dirs::home_dir().expect("Could not find home directory");
            let default_desktop_path = home.join("Desktop").join("team-calls-output");

            if self.output_dir == default_desktop_path {
                // CLI mode: use Desktop with custom/dated subdirectory
                let desktop_path = home.join("Desktop");
                if let Some(custom_name) = custom_dir_name {
                    let sanitized_name = self.sanitize_filename(custom_name);
                    desktop_path.join(format!("ct_{sanitized_name}"))
                } else {
                    let today = Zoned::now().strftime("%Y-%m-%d").to_string();
                    desktop_path.join(format!("team-calls-{today}"))
                }
            } else {
                // Test mode: use configured output_dir directly
                self.output_dir.clone()
            }
        };

        fs::create_dir_all(&output_dir).context("Failed to create output directory")?;

        // Use output directory for saving files
        let temp_formatter = CallMarkdownFormatter::new(Some(output_dir.clone()));

        for call in calls {
            match temp_formatter.save_call_markdown(call) {
                Ok(filepath) => saved_files.push(filepath),
                Err(e) => {
                    error!("Failed to save call {}: {}", call.id, e);
                    continue;
                }
            }
        }

        info!(
            "Saved {} call markdown files to {}",
            saved_files.len(),
            output_dir.display()
        );

        Ok(saved_files)
    }

    /// Extract customer name from call data
    fn extract_customer_name(&self, call: &Call) -> String {
        // For now, extract from title patterns - this could be enhanced
        // with more sophisticated logic later
        let title = &call.title;

        // Pattern 1: "Customer + Something - Meeting" -> "Customer"
        if title.contains(" + ") && title.contains(" - ") {
            if let Some(customer_part) = title.split(" + ").next() {
                let trimmed = customer_part.trim();
                if !trimmed.is_empty() && trimmed.len() > 1 {
                    return trimmed.to_string();
                }
            }
        }

        // Pattern 2: "Customer - Something" -> "Customer"
        if title.contains(" - ") {
            let parts: Vec<&str> = title.split(" - ").collect();
            if parts.len() >= 2 {
                let customer_part = parts[0].trim();
                if !customer_part.is_empty() && customer_part.len() > 1 {
                    return customer_part.to_string();
                }
            }
        }

        // Pattern 3: Look for known patterns like "Postman + X"
        if let Some(remaining) = title.strip_prefix("Postman + ") {
            if let Some(dash_pos) = remaining.find(" - ") {
                let customer_part = remaining[..dash_pos].trim();
                if !customer_part.is_empty() {
                    return customer_part.to_string();
                }
            }
        }

        "Unknown Customer".to_string()
    }

    /// Format date for display
    fn format_date(&self, date: &Zoned) -> String {
        date.strftime("%B %d, %Y at %I:%M %p").to_string()
    }

    /// Format date for use in filename (includes time for uniqueness)
    fn format_date_for_filename(&self, date: &Zoned) -> String {
        // Format as YYYY-MM-DDtHHMMSS to match Python implementation
        date.strftime("%Y-%m-%dt%H%M%S").to_string()
    }

    /// Sanitize a string for use as a filename
    fn sanitize_filename(&self, filename: &str) -> String {
        if filename.is_empty() {
            return "unnamed".to_string();
        }

        // Step 1: Keep only letters, numbers, spaces, and basic punctuation
        let regex1 = Regex::new(r"[^a-zA-Z0-9\s\-._()]").unwrap();
        let sanitized = regex1.replace_all(filename, "");

        // Step 2: Replace whitespace and parentheses with hyphens
        let regex2 = Regex::new(r"[\s()]+").unwrap();
        let sanitized = regex2.replace_all(&sanitized, "-");

        // Step 3: Remove dots and underscores (keep only letters, numbers, hyphens)
        let regex3 = Regex::new(r"[._]+").unwrap();
        let sanitized = regex3.replace_all(&sanitized, "-");

        // Step 4: Collapse multiple consecutive hyphens into single hyphens
        let regex4 = Regex::new(r"-+").unwrap();
        let sanitized = regex4.replace_all(&sanitized, "-");

        // Step 5: Clean up and format
        let sanitized = sanitized.trim_matches(&['-', '.'][..]).to_lowercase();

        // Step 6: Limit length
        let sanitized = if sanitized.len() > 50 {
            sanitized[..50].trim_end_matches(&['-', '.'][..])
        } else {
            &sanitized
        };

        if sanitized.is_empty() {
            "unnamed".to_string()
        } else {
            sanitized.to_string()
        }
    }

    /// Clean and format transcript text for markdown
    fn clean_transcript(&self, transcript: &str) -> String {
        if transcript.is_empty() {
            return "No transcript available.".to_string();
        }

        let cleaned = transcript.trim();

        // Check if transcript is already properly formatted (contains **Speaker:** patterns)
        if cleaned.contains("**") && cleaned.contains(":**") {
            // Already formatted by the API client, just return with minimal cleaning
            // Remove excessive blank lines (more than 2 consecutive)
            let regex = Regex::new(r"\n\n\n+").unwrap();
            return regex.replace_all(cleaned, "\n\n").to_string();
        }

        // Legacy formatting for unformatted transcripts
        let lines: Vec<&str> = cleaned.lines().collect();
        let mut formatted_lines = Vec::new();

        for line in lines {
            let line = line.trim();
            if line.is_empty() {
                continue; // Skip empty lines, let natural spacing handle it
            }

            // Check if line starts with a speaker name pattern
            if line.contains(':') && line.split(':').next().unwrap_or("").len() < 50 {
                let parts: Vec<&str> = line.splitn(2, ':').collect();
                if parts.len() == 2 {
                    let speaker = parts[0].trim();
                    let text = parts[1].trim();
                    formatted_lines.push(format!("**{speaker}:** {text}"));
                } else {
                    formatted_lines.push(line.to_string());
                }
            } else {
                formatted_lines.push(line.to_string());
            }
        }

        formatted_lines.join("\n\n")
    }
}

/// Email formatting methods
impl CallMarkdownFormatter {
    /// Format a single email into markdown content
    pub fn format_email_to_markdown(&self, email: &Email) -> String {
        // Extract email information
        let subject = if email.subject.is_empty() {
            "No Subject"
        } else {
            &email.subject
        };
        let sender_name = email.sender.name.as_deref().unwrap_or("Unknown Sender");
        let sender_email = &email.sender.email;
        let sent_at = self.format_date(&email.sent_at);

        // Build sender info
        let mut sender_info = sender_name.to_string();
        if !sender_email.is_empty() {
            sender_info.push_str(&format!(" ({sender_email})"));
        }
        if let Some(title) = &email.sender.title {
            sender_info.push_str(&format!(" - {title}"));
        }
        if let Some(company) = &email.sender.company {
            sender_info.push_str(&format!(" @ {company}"));
        }

        // Build markdown content
        let mut markdown_content = format!(
            "## {}\n\n**From:** {}\n**Date:** {}\n**Direction:** {}\n**Email ID:** `{}`",
            subject,
            sender_info,
            sent_at,
            format!("{:?}", email.direction).to_lowercase(),
            email.id
        );

        // Add recipients section
        if !email.recipients.is_empty() {
            markdown_content.push_str("\n**To:** ");
            let recipient_names: Vec<String> = email
                .recipients
                .iter()
                .map(|recipient| {
                    let name = recipient
                        .name
                        .as_deref()
                        .unwrap_or_else(|| recipient.email.split('@').next().unwrap_or("Unknown"));
                    if !recipient.email.is_empty() {
                        format!("{} ({})", name, recipient.email)
                    } else {
                        name.to_string()
                    }
                })
                .collect();
            markdown_content.push_str(&recipient_names.join(", "));
        }

        // Add automation/template info if applicable
        if email.is_automated || email.is_template {
            markdown_content.push_str("\n**Type:** ");
            if email.is_template {
                markdown_content.push_str("Template/Automated");
            } else {
                markdown_content.push_str("Automated");
            }
        }

        // Add body content
        markdown_content.push_str("\n\n### Content\n\n");

        if let Some(body_text) = &email.body_text {
            if !body_text.trim().is_empty() {
                let cleaned_body = self.clean_email_body(body_text);
                markdown_content.push_str(&cleaned_body);
            }
        } else if let Some(snippet) = &email.snippet {
            if !snippet.trim().is_empty() {
                markdown_content.push_str(&format!(
                    "*[Preview only - full content not available]*\n\n{snippet}"
                ));
            }
        } else {
            markdown_content.push_str("*No content available*");
        }

        markdown_content.push_str("\n\n---\n");
        markdown_content
    }

    /// Format a batch of emails into a single markdown document
    pub fn format_emails_batch_to_markdown(
        &self,
        emails: &[Email],
        batch_num: usize,
        customer_name: &str,
    ) -> String {
        if emails.is_empty() {
            return "# No Emails\n\nNo emails found in this batch.".to_string();
        }

        // Sort emails by date (newest first)
        let mut sorted_emails = emails.to_vec();
        sorted_emails.sort_by(|a, b| b.sent_at.cmp(&a.sent_at));

        // Get date range for title - filter out fallback dates like Python does
        let date_range = if !sorted_emails.is_empty() {
            let valid_dates: Vec<&Zoned> = sorted_emails
                .iter()
                .map(|e| &e.sent_at)
                .filter(|date| {
                    // Exclude emails with dates very close to current time (likely fallback dates)
                    let now = Zoned::now();
                    let time_diff = now.since(*date).unwrap_or_default().abs();
                    // If email date is within 1 minute of current time, it's likely a fallback
                    time_diff.get_seconds() > 60
                })
                .collect();

            if !valid_dates.is_empty() {
                let oldest_date = valid_dates.iter().min().unwrap();
                let newest_date = valid_dates.iter().max().unwrap();
                format!(
                    "{} - {}",
                    oldest_date.strftime("%m/%d"),
                    newest_date.strftime("%m/%d/%Y")
                )
            } else {
                "Unknown Date Range".to_string()
            }
        } else {
            "Unknown Date Range".to_string()
        };

        // Build header
        let generated_time = Zoned::now().strftime("%B %d, %Y at %I:%M %p").to_string();
        let mut markdown_content = format!(
            "# {} - Emails Batch {}\n\n**Date Range:** {}  \n**Total Emails:** {}  \n**Generated:** {}  \n**Advanced BDR/SPAM filtering applied** - Templates, duplicates, and automation removed\n\n---\n\n",
            customer_name, batch_num, date_range, emails.len(), generated_time
        );

        // Add each email
        for (i, email) in sorted_emails.iter().enumerate() {
            markdown_content.push_str(&format!("### Email {}/{}\n\n", i + 1, emails.len()));
            markdown_content.push_str(&self.format_email_to_markdown(email));
            markdown_content.push('\n');
        }

        // Add footer
        markdown_content.push_str(&format!(
            "\n\n---\n*Batch {batch_num} of emails for {customer_name} - Generated by cs-transcript-cli*\n"
        ));

        markdown_content
    }

    /// Save emails as markdown files in batches of 20
    pub fn save_emails_as_markdown(
        &self,
        emails: &[Email],
        customer_name: &str,
        custom_dir_name: Option<&str>,
    ) -> Result<Vec<PathBuf>> {
        if emails.is_empty() {
            info!("No emails to save");
            return Ok(Vec::new());
        }

        // Determine output directory - respect configured output_dir or use Desktop for CLI
        let output_dir = {
            // Check if we're using the default Desktop path (CLI behavior)
            let home = dirs::home_dir().expect("Could not find home directory");
            let default_desktop_path = home.join("Desktop").join("team-calls-output");

            if self.output_dir == default_desktop_path {
                // CLI mode: use Desktop with custom subdirectory
                let desktop_path = home.join("Desktop");
                if let Some(custom_name) = custom_dir_name {
                    let sanitized_name = self.sanitize_filename(custom_name);
                    desktop_path.join(format!("ct_{sanitized_name}"))
                } else {
                    let sanitized_name = self.sanitize_filename(customer_name);
                    desktop_path.join(format!("ct_{sanitized_name}"))
                }
            } else {
                // Test mode: use configured output_dir directly
                self.output_dir.clone()
            }
        };

        fs::create_dir_all(&output_dir).context("Failed to create output directory for emails")?;

        // Sort emails by date for consistent batching
        let mut sorted_emails = emails.to_vec();
        sorted_emails.sort_by(|a, b| a.sent_at.cmp(&b.sent_at));

        let mut saved_files = Vec::new();
        let batch_size = 20;

        // Process emails in batches of 20
        for (batch_start, batch_emails) in sorted_emails.chunks(batch_size).enumerate() {
            let batch_num = batch_start + 1;

            // Calculate date range for filename
            let (opening_range, closing_range) = if !batch_emails.is_empty() {
                // Filter out emails with invalid/fallback dates like Python does
                let batch_dates: Vec<&Zoned> = batch_emails
                    .iter()
                    .map(|e| &e.sent_at)
                    .filter(|date| {
                        // Exclude emails with dates very close to current time (likely fallback dates)
                        let now = Zoned::now();
                        let time_diff = now.since(*date).unwrap_or_default().abs();
                        // If email date is within 1 minute of current time, it's likely a fallback
                        time_diff.get_seconds() > 60
                    })
                    .collect();

                if !batch_dates.is_empty() {
                    let oldest_date = batch_dates.iter().min().unwrap();
                    let newest_date = batch_dates.iter().max().unwrap();
                    (
                        oldest_date.strftime("%m-%d").to_string(),
                        newest_date.strftime("%m-%d").to_string(),
                    )
                } else {
                    ("unknown".to_string(), "unknown".to_string())
                }
            } else {
                ("unknown".to_string(), "unknown".to_string())
            };

            // Create filename with specified pattern: [customer]-emls-[opening range mm-dd]-[closing range mm-dd]
            let clean_customer = self.sanitize_filename(customer_name);
            let mut filename = format!("{clean_customer}-emls-{opening_range}-{closing_range}.md");

            // Handle duplicate filenames by adding batch number
            let mut filepath = output_dir.join(&filename);
            if filepath.exists() {
                filename = format!(
                    "{clean_customer}-emls-{opening_range}-{closing_range}-batch{batch_num}.md"
                );
                filepath = output_dir.join(&filename);
            }

            // Generate markdown content for this batch
            let markdown_content =
                self.format_emails_batch_to_markdown(batch_emails, batch_num, customer_name);

            // Save the batch file
            match fs::write(&filepath, markdown_content) {
                Ok(_) => {
                    saved_files.push(filepath.clone());
                    info!(
                        "Saved email batch {} ({} emails) to {}",
                        batch_num,
                        batch_emails.len(),
                        filepath.display()
                    );
                }
                Err(e) => {
                    error!("Failed to save email batch {}: {}", batch_num, e);
                    continue;
                }
            }
        }

        info!(
            "Saved {} emails across {} batch files to {}",
            emails.len(),
            saved_files.len(),
            output_dir.display()
        );

        Ok(saved_files)
    }

    /// Clean and format email body text for markdown
    fn clean_email_body(&self, body_text: &str) -> String {
        if body_text.trim().is_empty() {
            return "*No content available*".to_string();
        }

        let mut cleaned = body_text.trim().to_string();

        // Remove excessive whitespace but preserve paragraph breaks
        let regex1 = Regex::new(r"\n\s*\n\s*\n+").unwrap();
        cleaned = regex1.replace_all(&cleaned, "\n\n").to_string();

        let regex2 = Regex::new(r"[ \t]+").unwrap();
        cleaned = regex2.replace_all(&cleaned, " ").to_string();

        // Handle common email artifacts
        let regex3 = Regex::new(r"^[\s>]+").unwrap();
        cleaned = regex3.replace_all(&cleaned, "").to_string();

        let regex4 = Regex::new(r"(^|\n)On .* wrote:\s*$").unwrap();
        cleaned = regex4
            .replace_all(&cleaned, r"$1\n---\n\n**Previous conversation:**\n")
            .to_string();

        // Ensure reasonable length (truncate very long emails)
        const MAX_LENGTH: usize = 5000;
        if cleaned.len() > MAX_LENGTH {
            cleaned.truncate(MAX_LENGTH);
            cleaned.push_str("\n\n*[Email content truncated for readability]*");
        }

        cleaned
    }
}

/// Summary generation for calls
pub struct CallSummaryReporter;

impl CallSummaryReporter {
    /// Create a new summary reporter
    pub fn new() -> Self {
        Self
    }

    /// Generate a summary report of all extracted calls
    pub fn generate_summary_report(
        &self,
        calls_data: &[Call],
        output_path: Option<&Path>,
        resolved_customer_name: Option<&str>,
    ) -> Result<String> {
        let today = Zoned::now().strftime("%Y-%m-%d").to_string();
        let generated_time = Zoned::now().strftime("%B %d, %Y at %I:%M %p").to_string();

        let mut summary_content = format!(
            "# Team Calls Summary - {}\n\nGenerated on {}\n\n## Overview\n\n- **Total Calls:** {}\n- **Date Range:** Last 7 days\n- **Extraction Date:** {}\n\n## Calls by Customer\n\n",
            today, generated_time, calls_data.len(), today
        );

        // Group calls by customer
        let mut customer_calls = std::collections::HashMap::new();

        // If we have a resolved customer name, use it for all calls
        if let Some(resolved_name) = resolved_customer_name {
            customer_calls.insert(resolved_name.to_string(), calls_data.to_vec());
        } else {
            // Fallback to extracting from individual call data
            for call in calls_data {
                let customer = self.extract_customer_name(call);
                customer_calls
                    .entry(customer)
                    .or_insert_with(Vec::new)
                    .push(call.clone());
            }
        }

        // Add customer sections
        let mut customers: Vec<_> = customer_calls.keys().collect();
        customers.sort();

        for customer in customers {
            let calls = &customer_calls[customer];
            summary_content.push_str(&format!("### {} ({} calls)\n\n", customer, calls.len()));

            for call in calls {
                let title = &call.title;
                let date = self.extract_call_date(call);
                let formatted_date = self.format_date_summary(date);
                let call_id = &call.id;

                summary_content.push_str(&format!(
                    "- **{title}** - {formatted_date} (ID: `{call_id})\n"
                ));
            }

            summary_content.push('\n');
        }

        // Save if path provided
        if let Some(path) = output_path {
            fs::write(path, &summary_content)
                .with_context(|| format!("Failed to save summary report: {}", path.display()))?;
            info!("Summary report saved to {}", path.display());
        }

        Ok(summary_content)
    }

    /// Extract call date from multiple possible fields
    fn extract_call_date<'a>(&self, call: &'a Call) -> &'a Zoned {
        // Try actual_start first if available, otherwise use scheduled_start
        call.actual_start.as_ref().unwrap_or(&call.scheduled_start)
    }

    /// Extract customer name from call data with intelligent fallbacks
    fn extract_customer_name(&self, call: &Call) -> String {
        let title = &call.title;

        // Pattern 1: "Customer + Something - Meeting" -> "Customer"
        if title.contains(" + ") && title.contains(" - ") {
            if let Some(customer_part) = title.split(" + ").next() {
                let trimmed = customer_part.trim();
                if !trimmed.is_empty() && trimmed.len() > 1 {
                    return trimmed.to_string();
                }
            }
        }

        // Pattern 2: "Customer - Something" -> "Customer"
        if title.contains(" - ") {
            let parts: Vec<&str> = title.split(" - ").collect();
            if parts.len() >= 2 {
                let customer_part = parts[0].trim();
                if !customer_part.is_empty() && customer_part.len() > 1 {
                    return customer_part.to_string();
                }
            }
        }

        // Pattern 3: Look for known patterns like "Postman + X"
        if let Some(remaining) = title.strip_prefix("Postman + ") {
            if let Some(dash_pos) = remaining.find(" - ") {
                let customer_part = remaining[..dash_pos].trim();
                if !customer_part.is_empty() {
                    return customer_part.to_string();
                }
            }
        }

        "Unknown Customer".to_string()
    }

    /// Format date for summary display
    fn format_date_summary(&self, date: &Zoned) -> String {
        date.strftime("%m/%d/%Y").to_string()
    }
}

impl Default for CallSummaryReporter {
    fn default() -> Self {
        Self::new()
    }
}
