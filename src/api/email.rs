use futures::future::join_all;
use indicatif::{ProgressBar, ProgressStyle};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::api::client::HttpClientPool;
use crate::auth::GongAuthenticator;
use crate::config::AppConfig;
use crate::models::Email;
use crate::output::html::HTMLProcessor;
use crate::{CsCliError, Result};

/// Progress callback function type for email enhancement
pub type ProgressCallback = Box<dyn Fn(usize, usize) + Send + Sync>;

/// Enhanced email content processor with body fetching
pub struct EmailEnhancer {
    /// HTTP client pool for making requests
    http_client: Arc<HttpClientPool>,
    /// Authentication manager
    auth: Arc<GongAuthenticator>,
    /// Application configuration
    _config: Option<AppConfig>,
    /// Batch size for concurrent processing
    batch_size: usize,
    /// HTML processor for converting email bodies
    html_processor: HTMLProcessor,
}

impl EmailEnhancer {
    /// Create a new email enhancer
    pub fn new(
        http_client: Arc<HttpClientPool>,
        auth: Arc<GongAuthenticator>,
        config: Option<AppConfig>,
        batch_size: Option<usize>,
    ) -> Self {
        let batch_size = batch_size.unwrap_or(50);
        let html_processor = HTMLProcessor::new();

        Self {
            http_client,
            auth,
            _config: config,
            batch_size,
            html_processor,
        }
    }

    /// Enhance emails with full body content
    ///
    /// # Arguments
    /// * `emails` - List of emails to enhance
    /// * `fetch_bodies` - Whether to fetch full email bodies
    /// * `progress_callback` - Optional callback function to report progress (completed_count, total_count)
    ///
    /// # Returns
    /// List of enhanced emails with body content
    pub async fn enhance_emails_with_bodies(
        &self,
        emails: Vec<Email>,
        fetch_bodies: bool,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<Vec<Email>> {
        if emails.is_empty() {
            return Ok(Vec::new());
        }

        info!(
            count = emails.len(),
            fetch_bodies = fetch_bodies,
            "Starting email enhancement"
        );

        // Filter emails that need enhancement (no body content) - collect indices to avoid borrowing issues
        let enhancement_indices: Vec<usize> = emails
            .iter()
            .enumerate()
            .filter(|(_, email)| {
                email.body_text.is_none()
                    || email
                        .body_text
                        .as_ref()
                        .map(|b| b.trim().is_empty())
                        .unwrap_or(true)
            })
            .map(|(i, _)| i)
            .collect();

        if enhancement_indices.is_empty() {
            info!("No emails need body enhancement");
            return Ok(emails);
        }

        info!(count = enhancement_indices.len(), "Emails to enhance");

        // Create progress bar if no callback provided
        let progress_bar = if progress_callback.is_none() {
            let pb = ProgressBar::new(enhancement_indices.len() as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} {msg}")
                    .unwrap()
                    .progress_chars("##-")
            );
            pb.set_message("Fetching email bodies...");
            Some(pb)
        } else {
            None
        };

        // Process in batches to control memory usage
        let mut enhanced_emails = HashMap::new();
        let total_batches = enhancement_indices.len().div_ceil(self.batch_size);

        for (batch_num, chunk) in enhancement_indices.chunks(self.batch_size).enumerate() {
            let batch_start = batch_num * self.batch_size;
            let batch_end = std::cmp::min(batch_start + chunk.len(), enhancement_indices.len());

            debug!(
                batch_num = batch_num + 1,
                total_batches = total_batches,
                batch_size = chunk.len(),
                "Processing email batch"
            );

            // Process batch concurrently
            let tasks: Vec<_> = chunk
                .iter()
                .map(|&idx| self.enhance_single_email(&emails[idx], fetch_bodies))
                .collect();

            let results = join_all(tasks).await;

            // Collect successful results
            for (i, result) in results.into_iter().enumerate() {
                let email_id = &emails[chunk[i]].id;
                match result {
                    Ok(Some(enhanced_email)) => {
                        enhanced_emails.insert(email_id.clone(), enhanced_email);
                    }
                    #[allow(non_snake_case)]
                    Ok(None) => {
                        debug!(email_id = %email_id, "Email enhancement returned None");
                    }
                    Err(e) => {
                        error!(email_id = %email_id, error = %e, "Email enhancement failed");
                    }
                }
            }

            // Update progress
            if let Some(callback) = &progress_callback {
                callback(batch_end, enhancement_indices.len());
            }
            if let Some(pb) = &progress_bar {
                pb.set_position(batch_end as u64);
            }
        }

        // Finish progress bar
        if let Some(pb) = progress_bar {
            pb.finish_with_message("Email enhancement complete");
        }

        // Update original emails list with enhanced data
        let mut result_emails = emails;
        for email in &mut result_emails {
            if let Some(enhanced_email) = enhanced_emails.remove(&email.id) {
                *email = enhanced_email;
            }
        }

        info!(
            enhanced = enhanced_emails.len(),
            total = enhancement_indices.len(),
            "Email enhancement completed"
        );

        Ok(result_emails)
    }

    /// Enhance a single email with full body content
    ///
    /// # Arguments
    /// * `email` - Email to enhance
    /// * `fetch_body` - Whether to fetch full email body
    ///
    /// # Returns
    /// Enhanced email or None if enhancement failed
    pub async fn enhance_single_email(
        &self,
        email: &Email,
        fetch_body: bool,
    ) -> Result<Option<Email>> {
        if !fetch_body {
            return Ok(Some(email.clone()));
        }

        let headers = self.auth.get_read_headers()?;
        let base_url = self.auth.get_base_url()?;

        // Update headers on HTTP client
        self.http_client.update_headers(headers).await?;

        // Fetch email content using the email-expanded endpoint
        let endpoint = format!("{base_url}/ajax/account/email-expanded");
        let workspace_id = self
            .auth
            .get_workspace_id()
            .unwrap_or("5562739194953732039");

        let mut params = HashMap::new();
        params.insert("id".to_string(), email.id.clone());
        params.insert("account-id".to_string(), email.account_id.clone());
        params.insert("customer-type".to_string(), "ACCOUNT".to_string());
        params.insert("workspace-id".to_string(), workspace_id.to_string());

        // Build URL with query parameters
        let query_string = params
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");
        let full_url = format!("{endpoint}?{query_string}");

        let response = self.http_client.get(&full_url).await?;

        if !response.status().is_success() {
            let status_code = response.status().as_u16();

            // Fallback to snippet if available
            if let Some(snippet) = &email.snippet {
                if !snippet.trim().is_empty() {
                    let mut enhanced_email = email.clone();
                    enhanced_email.body_text = Some(snippet.clone());
                    enhanced_email.body_fetched = true;

                    info!(
                        email_id = %email.id,
                        status_code = status_code,
                        "Using snippet fallback for email"
                    );
                    return Ok(Some(enhanced_email));
                }
            }

            warn!(
                email_id = %email.id,
                status_code = status_code,
                "Email enhancement failed - no fallback available"
            );

            // Return the original email instead of None
            return Ok(Some(email.clone()));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| CsCliError::ApiRequest(format!("Failed to read response: {e}")))?;

        let content: Value = serde_json::from_str(&response_text)
            .map_err(|e| CsCliError::ApiRequest(format!("Failed to parse JSON: {e}")))?;

        let mut enhanced_email = email.clone();

        // Process HTML content to clean text
        if fetch_body {
            if let Some(html_content) = content.get("body").and_then(|b| b.as_str()) {
                let mut full_html_content = html_content.to_string();

                // Include quoted conversation if available
                if let Some(quote) = content.get("quote").and_then(|q| q.as_str()) {
                    full_html_content = format!("{full_html_content}<hr/>{quote}");
                }

                // Convert HTML to text using processor
                match self.html_processor.process_content(&full_html_content) {
                    Ok(converted_text) => {
                        if !converted_text.trim().is_empty() {
                            enhanced_email.body_text = Some(converted_text);
                            enhanced_email.body_fetched = true;
                        }
                    }
                    Err(e) => {
                        warn!(
                            email_id = %email.id,
                            error = %e,
                            "HTML processing failed, using raw HTML"
                        );
                        enhanced_email.body_text = Some(full_html_content);
                        enhanced_email.body_fetched = true;
                    }
                }
            } else {
                // Body is None or empty - use snippet as fallback
                if let Some(snippet) = &email.snippet {
                    if !snippet.trim().is_empty() {
                        enhanced_email.body_text = Some(snippet.clone());
                        enhanced_email.body_fetched = true;

                        debug!(
                            email_id = %email.id,
                            "Email has no body content, using snippet"
                        );
                    }
                }
            }
        }

        // Update metadata if available
        if let Some(subject) = content.get("subject").and_then(|s| s.as_str()) {
            enhanced_email.subject = subject.to_string();
        }
        if let Some(snippet) = content.get("synopsis").and_then(|s| s.as_str()) {
            enhanced_email.snippet = Some(snippet.to_string());
        }

        Ok(Some(enhanced_email))
    }

    /// Enhance emails with progress bar (convenience method)
    ///
    /// # Arguments
    /// * `emails` - List of emails to enhance
    /// * `fetch_bodies` - Whether to fetch full email bodies
    ///
    /// # Returns
    /// List of enhanced emails with progress indication
    pub async fn enhance_emails_with_progress(
        &self,
        emails: Vec<Email>,
        fetch_bodies: bool,
    ) -> Result<Vec<Email>> {
        self.enhance_emails_with_bodies(emails, fetch_bodies, None)
            .await
    }

    /// Get enhancement statistics
    ///
    /// # Arguments
    /// * `emails` - List of emails to analyze
    ///
    /// # Returns
    /// Statistics about how many emails need enhancement: (total, needs_enhancement, already_enhanced)
    pub fn get_enhancement_stats(&self, emails: &[Email]) -> (usize, usize, usize) {
        let total = emails.len();
        let needs_enhancement = emails
            .iter()
            .filter(|email| {
                email.body_text.is_none()
                    || email
                        .body_text
                        .as_ref()
                        .map(|b| b.trim().is_empty())
                        .unwrap_or(true)
            })
            .count();
        let already_enhanced = total - needs_enhancement;

        (total, needs_enhancement, already_enhanced)
    }

    /// Check if an email needs body enhancement
    ///
    /// # Arguments
    /// * `email` - Email to check
    ///
    /// # Returns
    /// True if email needs body content fetching
    pub fn needs_enhancement(&self, email: &Email) -> bool {
        email.body_text.is_none()
            || email
                .body_text
                .as_ref()
                .map(|b| b.trim().is_empty())
                .unwrap_or(true)
    }

    /// Create a progress callback that updates an indicatif progress bar
    ///
    /// # Arguments
    /// * `progress_bar` - Progress bar to update
    ///
    /// # Returns
    /// Progress callback function
    pub fn create_progress_callback(&self, progress_bar: ProgressBar) -> ProgressCallback {
        Box::new(move |completed, _total| {
            progress_bar.set_position(completed as u64);
        })
    }
}
