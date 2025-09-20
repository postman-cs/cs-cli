//! Gong data retrieval and processing logic
//!
//! This module handles all Gong-specific retrieval functionality including
//! calls, emails, and transcripts. It's designed to be UI-agnostic and can
//! be used by any interface (CLI, TUI, API, etc.)

use crate::gong::api::client::HttpClientPool;
use crate::gong::api::customer::CustomerCallInfo;
use crate::gong::api::customer::GongCustomerSearchRetriever;
use crate::gong::api::email::EmailRetriever;
use crate::gong::api::library::{CallDetailsRetriever, GongLibraryRetriever, LibraryCallInfo};
use crate::gong::api::timeline::TimelineRetriever;
use crate::gong::auth::GongAuthenticator;
use crate::gong::config::AppConfig;
use crate::gong::models::{Call, CallDirection, Email};
use crate::gong::output::markdown::{CallMarkdownRenderer, CallSummaryReporter};
use crate::Result;
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{info, warn};

/// Convert LibraryCallInfo to Call model
fn convert_library_call_info_to_call(info: LibraryCallInfo) -> Call {
    use crate::gong::models::CallParticipant;

    // Parse the date string to Zoned
    let scheduled_start = jiff::Zoned::strptime(&info.date, "%Y-%m-%dT%H:%M:%S%.fZ")
        .or_else(|_| jiff::Zoned::strptime(&info.date, "%Y-%m-%dT%H:%M:%SZ"))
        .or_else(|_| {
            // Parse as local time for dates without timezone
            jiff::civil::DateTime::strptime(&info.date, "%Y-%m-%dT%H:%M:%S")
                .and_then(|dt| dt.to_zoned(jiff::tz::TimeZone::system()))
        })
        .or_else(|_| jiff::Zoned::strptime(&info.date, "%Y-%m-%d %H:%M:%S"))
        .or_else(|_| jiff::Zoned::strptime(&info.date, "%Y-%m-%d"))
        .unwrap_or_else(|_| jiff::Zoned::now());

    // Convert participant strings to CallParticipant structs
    let participants: Vec<CallParticipant> = info
        .participants
        .into_iter()
        .map(|name| CallParticipant {
            id: None,
            name,
            email: None,
            phone: None,
            title: None,
            company: None,
            is_internal: false, // Default assumption
            speaking_time: None,
            talk_ratio: None,
        })
        .collect();

    Call {
        id: info.id.unwrap_or_else(|| "unknown".to_string()),
        account_id: "unknown".to_string(),
        title: if info.generated_title.is_empty() {
            info.title.clone()
        } else {
            info.generated_title.clone()
        },
        generated_title: if info.generated_title.is_empty() {
            None
        } else {
            Some(info.generated_title)
        },
        customer_name: if info.customer_name.is_empty() {
            None
        } else {
            Some(info.customer_name)
        },
        direction: CallDirection::Outbound,
        duration: info.duration,
        scheduled_start,
        actual_start: None,
        recording_url: if info.call_url.is_empty() {
            None
        } else {
            Some(info.call_url)
        },
        transcript_url: None,
        call_brief: None,
        status: None,
        call_type: None,
        participants,
        host_id: None,
        host_name: None, // Not available in LibraryCallInfo
        sentiment: None,
        talk_ratio: None,
        longest_monologue: None,
        interactivity: None,
        questions_asked: None,
        transcript: None, // Will be filled after fetching details
        summary: None,
        topics: Vec::new(),
        action_items: Vec::new(),
    }
}

/// Main orchestrator for Gong data retrieval
pub struct TeamCallsRetriever {
    config: AppConfig,
    http: Option<Arc<HttpClientPool>>,
    auth: Option<Arc<GongAuthenticator>>,
    library_client: Option<GongLibraryRetriever>,
    details_fetcher: Option<CallDetailsRetriever>,
    customer_search_client: Option<GongCustomerSearchRetriever>,
    timeline_retriever: Option<TimelineRetriever>,
    email_enhancer: Option<EmailRetriever>,
    formatter: CallMarkdownRenderer,
    summary_reporter: CallSummaryReporter,
    quiet: bool, // Suppress console output when true
}

impl TeamCallsRetriever {
    /// Create a new retriever instance
    pub fn new(config: AppConfig) -> Self {
        Self {
            config,
            http: None,
            auth: None,
            library_client: None,
            details_fetcher: None,
            customer_search_client: None,
            timeline_retriever: None,
            email_enhancer: None,
            formatter: CallMarkdownRenderer::new(None),
            summary_reporter: CallSummaryReporter::new(),
            quiet: false,
        }
    }

    /// Enable quiet mode (suppress console output)
    pub fn set_quiet(&mut self, quiet: bool) {
        self.quiet = quiet;
    }

    /// Initialize retriever with pre-authenticated components
    /// This allows reusing authentication from other parts of the system
    pub async fn setup_with_auth(
        &mut self,
        http: Arc<HttpClientPool>,
        auth: Arc<GongAuthenticator>,
    ) -> Result<()> {
        self.print("Setting up retrieval components...".to_string());

        // Store Arc references
        self.http = Some(Arc::clone(&http));
        self.auth = Some(Arc::clone(&auth));

        // Initialize API clients
        self.library_client = Some(GongLibraryRetriever::new(
            Arc::clone(&http),
            Arc::clone(&auth),
            Some(self.config.clone()),
        ));

        self.details_fetcher = Some(CallDetailsRetriever::new(
            Arc::clone(&http),
            Arc::clone(&auth),
            Some(self.config.clone()),
        ));

        self.customer_search_client = Some(GongCustomerSearchRetriever::new(
            Arc::clone(&http),
            Arc::clone(&auth),
            Some(self.config.clone()),
        )?);

        self.timeline_retriever = Some(TimelineRetriever::new(
            Arc::clone(&http),
            Arc::clone(&auth),
            Some(self.config.clone()),
            Some(30), // Default chunk days
        )?);

        self.email_enhancer = Some(EmailRetriever::new(
            Arc::clone(&http),
            Arc::clone(&auth),
            Some(self.config.clone()),
            None, // Default batch size
        ));

        self.print("✅ Retrieval system ready".green().to_string());
        Ok(())
    }

    /// Print a message if not in quiet mode
    fn print(&self, message: String) {
        if !self.quiet {
            println!("{message}");
        }
    }

    /// Setup all required API clients
    pub async fn setup(&mut self) -> Result<()> {
        self.print(format!(
            "Initializing {} retrieval system...",
            "CS-CLI".truecolor(255, 255, 255)
        ));

        // Initialize Gong HTTP client and auth
        let http = HttpClientPool::new_gong_pool(Some(self.config.http.clone())).await?;
        let mut auth = GongAuthenticator::new(self.config.auth.clone()).await?;

        // Authenticate
        self.print("Authenticating with Gong...".cyan().to_string());
        let authenticated = auth.authenticate().await?;

        if !authenticated {
            return Err(crate::common::error::types::CsCliError::Authentication(
                "Failed to authenticate with Gong".to_string(),
            ));
        }

        // Set cookies
        if let Ok(session_cookies) = auth.get_session_cookies() {
            http.set_cookies(session_cookies).await?;
        }

        // Create Arc references for sharing
        let http_arc = Arc::new(http);
        let auth_arc = Arc::new(auth);

        // Store Arc references
        self.http = Some(Arc::clone(&http_arc));
        self.auth = Some(Arc::clone(&auth_arc));

        // Initialize API clients
        self.library_client = Some(GongLibraryRetriever::new(
            Arc::clone(&http_arc),
            Arc::clone(&auth_arc),
            Some(self.config.clone()),
        ));

        self.details_fetcher = Some(CallDetailsRetriever::new(
            Arc::clone(&http_arc),
            Arc::clone(&auth_arc),
            Some(self.config.clone()),
        ));

        self.customer_search_client = Some(GongCustomerSearchRetriever::new(
            Arc::clone(&http_arc),
            Arc::clone(&auth_arc),
            Some(self.config.clone()),
        )?);

        self.timeline_retriever = Some(TimelineRetriever::new(
            Arc::clone(&http_arc),
            Arc::clone(&auth_arc),
            Some(self.config.clone()),
            Some(30), // Default chunk days
        )?);

        self.email_enhancer = Some(EmailRetriever::new(
            Arc::clone(&http_arc),
            Arc::clone(&auth_arc),
            Some(self.config.clone()),
            None, // Default batch size
        ));

        self.print("Setup complete!".truecolor(255, 255, 255).to_string());
        Ok(())
    }

    /// Retrieve team calls for a given stream
    pub async fn retrieve_team_calls(
        &mut self,
        stream_id: &str,
        days: Option<u32>,
        _limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<Call>> {
        let library_client = self.library_client.as_ref().ok_or_else(|| {
            crate::common::error::types::CsCliError::Generic(
                "Library client not initialized".to_string(),
            )
        })?;

        let details_fetcher = self.details_fetcher.as_ref().ok_or_else(|| {
            crate::common::error::types::CsCliError::Generic(
                "Details fetcher not initialized".to_string(),
            )
        })?;

        self.print(format!(
            "{}",
            format!("Fetching calls from stream ID: {stream_id}").cyan()
        ));

        // Fetch calls from library
        let library_result = library_client
            .get_library_calls(
                Some(stream_id),
                days.map(|d| d as i32),
                None,
                None,
                offset.unwrap_or(0),
            )
            .await?;
        let library_calls = library_result.calls;

        if library_calls.is_empty() {
            self.print("No calls found in call stream!".yellow().to_string());
            return Ok(Vec::new());
        }

        // Progress bar for transcript fetching
        let pb = if !self.quiet {
            let pb = ProgressBar::new(library_calls.len() as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                    .unwrap_or_else(|_| ProgressStyle::default_bar())
                    .progress_chars("#>-"),
            );
            pb.set_message("Fetching transcripts...");
            Some(pb)
        } else {
            None
        };

        // Fetch detailed information and transcripts for each call
        let mut detailed_calls = Vec::new();
        for call_info in library_calls {
            if let Some(pb) = &pb {
                pb.set_message(format!("Processing: {}", call_info.title.clone()));
            }

            // Convert to Call model first
            let mut call = convert_library_call_info_to_call(call_info);

            // Fetch transcript if available
            if call.id != "unknown" {
                match details_fetcher.get_call_details(&call.id).await {
                    Ok(Some(details)) => {
                        if !details.transcript.is_empty() {
                            call.transcript = Some(details.transcript);
                        }
                    }
                    Ok(None) => {
                        warn!("No details found for call {}", call.id);
                    }
                    Err(e) => {
                        warn!("Failed to fetch transcript for {}: {}", call.id, e);
                    }
                }
            }

            detailed_calls.push(call);

            if let Some(pb) = &pb {
                pb.inc(1);
            }
        }

        if let Some(pb) = &pb {
            pb.finish_with_message("All transcripts fetched!");
        }

        self.print(
            format!("Found {} team calls", detailed_calls.len())
                .green()
                .to_string(),
        );

        Ok(detailed_calls)
    }

    /// Retrieve customer calls
    pub async fn retrieve_customer_calls(
        &mut self,
        name: &str,
        _days: Option<u32>,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<(Vec<Call>, String)> {
        self.print(
            format!("Searching for calls with '{name}'...")
                .cyan()
                .to_string(),
        );

        let customer_search_client = self.customer_search_client.as_ref().ok_or_else(|| {
            crate::common::error::types::CsCliError::Generic(
                "Customer search client not initialized".to_string(),
            )
        })?;

        let details_fetcher = self.details_fetcher.as_ref().ok_or_else(|| {
            crate::common::error::types::CsCliError::Generic(
                "Details fetcher not initialized".to_string(),
            )
        })?;

        // Search for the customer
        let customers = customer_search_client.search_customers(name).await?;
        if customers.is_empty() {
            self.print(
                format!("No customers found matching '{name}'")
                    .yellow()
                    .to_string(),
            );
            return Ok((Vec::new(), name.to_string()));
        }

        let customer = &customers[0];
        let resolved_name = customer.name.clone();

        self.print(
            format!(
                "Found customer: {} (ID: {})",
                customer.name,
                customer.id.as_deref().unwrap_or("unknown")
            )
            .green()
            .to_string(),
        );

        // Fetch customer calls
        let call_infos = customer_search_client
            .get_customer_calls(
                &customer.name,
                limit.unwrap_or(10),
                offset.unwrap_or(0),
                false,
            )
            .await?;

        if call_infos.calls.is_empty() {
            self.print(
                format!("No calls found for '{}'", customer.name)
                    .yellow()
                    .to_string(),
            );
            return Ok((Vec::new(), resolved_name));
        }

        self.print(
            format!(
                "Found {} calls for {}",
                call_infos.calls.len(),
                customer.name
            )
            .green()
            .to_string(),
        );

        // Convert to Call models and fetch transcripts
        let mut all_calls = Vec::new();

        let pb = if !self.quiet {
            let pb = ProgressBar::new(call_infos.calls.len() as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                    .unwrap_or_else(|_| ProgressStyle::default_bar())
                    .progress_chars("#>-"),
            );
            pb.set_message("Fetching call details...");
            Some(pb)
        } else {
            None
        };

        for info in call_infos.calls {
            if let Some(pb) = &pb {
                pb.set_message(format!("Processing: {}", info.title));
            }

            // Fetch transcript and create Call with transcript
            let call_id = info.id.as_ref().unwrap_or(&"unknown".to_string()).clone();
            let mut call = self.convert_customer_call_info(info);

            // Try to enrich with transcript
            match details_fetcher.get_call_details(&call_id).await {
                Ok(Some(details)) => {
                    if !details.transcript.is_empty() {
                        call.transcript = Some(details.transcript);
                    }
                }
                Ok(None) => {
                    warn!("No details found for call {}", call_id);
                }
                Err(e) => {
                    warn!("Failed to fetch transcript for {}: {}", call_id, e);
                }
            };

            all_calls.push(call);

            if let Some(pb) = &pb {
                pb.inc(1);
            }
        }

        if let Some(pb) = &pb {
            pb.finish_with_message("All calls fetched!");
        }

        self.print(
            format!(
                "Fetched {} calls with transcripts for {}",
                all_calls.len(),
                customer.name
            )
            .green()
            .to_string(),
        );

        Ok((all_calls, resolved_name))
    }

    /// Retrieve customer communications (calls + emails)
    pub async fn retrieve_customer_communications(
        &mut self,
        name: &str,
        days: u32,
        include_emails: bool,
        emails_only: bool,
        fetch_email_bodies: bool,
    ) -> Result<(Vec<Call>, Vec<Email>, String)> {
        self.print(
            format!("Retrieving communications for '{name}' from last {days} days...")
                .cyan()
                .to_string(),
        );

        if emails_only {
            self.print(
                "Email-only mode: Skipping call retrieval"
                    .yellow()
                    .to_string(),
            );
        }

        // Retrieve calls if not email-only mode
        let (mut calls, resolved_name) = if !emails_only {
            self.retrieve_customer_calls(name, Some(days), None, None)
                .await?
        } else {
            // Still need to resolve customer name
            let customer_search_client = self.customer_search_client.as_ref().ok_or_else(|| {
                crate::common::error::types::CsCliError::Generic(
                    "Customer search client not initialized".to_string(),
                )
            })?;

            let customers = customer_search_client.search_customers(name).await?;
            let resolved = if !customers.is_empty() {
                customers[0].name.clone()
            } else {
                name.to_string()
            };
            (Vec::new(), resolved)
        };

        // Extract emails if requested
        let emails = if include_emails || emails_only {
            self.print(
                "Filtering emails to remove blasts and spam"
                    .yellow()
                    .to_string(),
            );
            self.retrieve_customer_emails(&resolved_name, days, fetch_email_bodies)
                .await?
        } else {
            Vec::new()
        };

        // Sort calls by date (newest first)
        calls.sort_by(|a, b| b.scheduled_start.cmp(&a.scheduled_start));

        self.print(
            format!(
                "Retrieval complete: {} calls, {} emails",
                calls.len(),
                emails.len()
            )
            .green()
            .to_string(),
        );

        Ok((calls, emails, resolved_name.to_string()))
    }

    /// Retrieve customer emails
    async fn retrieve_customer_emails(
        &mut self,
        name: &str,
        days: u32,
        fetch_bodies: bool,
    ) -> Result<Vec<Email>> {
        self.print(
            format!("Searching for emails with '{name}'...")
                .cyan()
                .to_string(),
        );

        let customer_search_client = self.customer_search_client.as_ref().ok_or_else(|| {
            crate::common::error::types::CsCliError::Generic(
                "Customer search client not initialized".to_string(),
            )
        })?;

        let timeline_retriever = self.timeline_retriever.as_mut().ok_or_else(|| {
            crate::common::error::types::CsCliError::Generic(
                "Timeline retriever not initialized".to_string(),
            )
        })?;

        let email_enhancer = self.email_enhancer.as_ref().ok_or_else(|| {
            crate::common::error::types::CsCliError::Generic(
                "Email enhancer not initialized".to_string(),
            )
        })?;

        // Search for the customer by name to get their ID
        let customers = customer_search_client.search_customers(name).await?;
        let customer_id = if let Some(customer) = customers.first() {
            customer
                .id
                .as_ref()
                .expect("Customer should always have an ID")
                .clone()
        } else {
            return Ok(Vec::new()); // No customer found, skip email retrieval
        };

        // Search for emails using timeline retrieval
        let timeline_result = timeline_retriever
            .retrieve_account_timeline(
                &customer_id,
                jiff::Zoned::now().saturating_sub(jiff::Span::new().days(days as i64)),
                None,
            )
            .await?;
        let mut emails = timeline_result.emails;

        if emails.is_empty() {
            self.print(format!("No emails found for '{name}'").yellow().to_string());
            return Ok(Vec::new());
        }

        self.print(format!("Found {} emails", emails.len()).green().to_string());

        // Fetch email bodies if requested
        if fetch_bodies {
            self.print("Fetching email bodies...".cyan().to_string());
            emails = email_enhancer
                .retrieve_emails_with_progress(emails, true)
                .await?;
        }

        // Sort by date (newest first)
        emails.sort_by(|a, b| b.sent_at.cmp(&a.sent_at));

        Ok(emails)
    }

    /// Convert CustomerCallInfo to Call
    fn convert_customer_call_info(&self, info: CustomerCallInfo) -> Call {
        use crate::gong::models::CallParticipant;

        // Parse date
        let scheduled_start = jiff::Zoned::strptime(&info.date, "%Y-%m-%dT%H:%M:%S%.fZ")
            .or_else(|_| jiff::Zoned::strptime(&info.date, "%Y-%m-%dT%H:%M:%SZ"))
            .or_else(|_| {
                jiff::civil::DateTime::strptime(&info.date, "%Y-%m-%dT%H:%M:%S")
                    .and_then(|dt| dt.to_zoned(jiff::tz::TimeZone::system()))
            })
            .unwrap_or_else(|_| jiff::Zoned::now());

        // Convert participants
        let participants: Vec<CallParticipant> = info
            .participants
            .into_iter()
            .map(|name| CallParticipant {
                id: None,
                name,
                email: None,
                phone: None,
                title: None,
                company: None,
                is_internal: false,
                speaking_time: None,
                talk_ratio: None,
            })
            .collect();

        Call {
            id: info.id.unwrap_or_else(|| "unknown".to_string()),
            account_id: info.account_id.unwrap_or_else(|| "unknown".to_string()),
            title: info.title.clone(),
            generated_title: if info.generated_title.is_empty() {
                None
            } else {
                Some(info.generated_title)
            },
            customer_name: if info.customer_name.is_empty() {
                None
            } else {
                Some(info.customer_name)
            },
            direction: CallDirection::Outbound,
            duration: info.duration,
            scheduled_start,
            actual_start: None,
            recording_url: if info.call_url.is_empty() {
                None
            } else {
                Some(info.call_url)
            },
            transcript_url: None,
            call_brief: None,
            status: None,
            call_type: None,
            participants,
            host_id: None,
            host_name: None,
            sentiment: None,
            talk_ratio: None,
            longest_monologue: None,
            interactivity: None,
            questions_asked: None,
            transcript: None, // Will be filled after fetching details
            summary: None,
            topics: Vec::new(),
            action_items: Vec::new(),
        }
    }

    /// Save calls as markdown files
    pub fn save_calls_as_markdown_with_resolved_name(
        &self,
        calls: &[Call],
        original_query: Option<&str>,
        resolved_name: Option<&str>,
    ) -> Result<Vec<PathBuf>> {
        let name = resolved_name
            .or(original_query)
            .unwrap_or("unknown_customer");

        self.print(
            format!("Saving {} call transcripts...", calls.len())
                .cyan()
                .to_string(),
        );

        let saved = self
            .formatter
            .save_multiple_calls(calls, Some(name))
            .map_err(|e| crate::common::error::types::CsCliError::Generic(e.to_string()))?;

        for file in &saved {
            if let Some(name) = file.file_name() {
                self.print(
                    format!("  📄 {}", name.to_string_lossy())
                        .green()
                        .to_string(),
                );
            }
        }

        // Generate summary report
        let output_dir = if let Some(first_file) = saved.first() {
            first_file.parent()
        } else {
            None
        };
        self.summary_reporter
            .generate_summary_report(calls, output_dir, Some(name))
            .map_err(|e| crate::common::error::types::CsCliError::Generic(e.to_string()))?;

        self.print(
            format!("Saved {} markdown files", saved.len())
                .green()
                .to_string(),
        );

        Ok(saved)
    }

    /// Save emails as markdown files
    pub fn save_emails_as_markdown(
        &self,
        emails: &[Email],
        customer_name: &str,
    ) -> Result<Vec<PathBuf>> {
        if emails.is_empty() {
            return Ok(Vec::new());
        }

        self.print(
            format!("Saving {} emails...", emails.len())
                .cyan()
                .to_string(),
        );

        let saved = self
            .formatter
            .save_emails_as_markdown(emails, customer_name, None)
            .map_err(|e| crate::common::error::types::CsCliError::Generic(e.to_string()))?;

        for file in &saved {
            if let Some(name) = file.file_name() {
                self.print(
                    format!("  📄 {}", name.to_string_lossy())
                        .green()
                        .to_string(),
                );
            }
        }

        Ok(saved)
    }

    /// Cleanup resources
    pub async fn cleanup(&self) {
        info!("Cleaning up retrieval resources...");
        // Any cleanup logic if needed
    }
}
