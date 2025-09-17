//! Main CLI orchestration and coordination
//!
//! This module provides the main entry point for the CS-CLI application,
//! coordinating argument parsing, interactive mode, and extraction workflows.

pub mod args;
pub mod interactive;

pub use args::*;
pub use interactive::*;

use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{info, warn};

use crate::common::auth::unlock_keychain_with_cli_password;
use crate::gong::api::client::HttpClientPool;
use crate::gong::api::customer::CustomerCallInfo;
use crate::gong::api::customer::GongCustomerSearchClient;
use crate::gong::api::email::EmailEnhancer;
use crate::gong::api::library::{CallDetailsFetcher, GongLibraryClient, LibraryCallInfo};
use crate::gong::api::timeline::TimelineExtractor;
use crate::gong::auth::GongAuthenticator;
use crate::gong::config::AppConfig;
use crate::gong::models::{Call, CallDirection, Email};
use crate::gong::output::markdown::{CallMarkdownFormatter, CallSummaryReporter};
use crate::Result;
use clap::{CommandFactory, Parser};
use clap_complete::{generate, Shell};

/// Convert LibraryCallInfo to Call model
fn convert_library_call_info_to_call(info: LibraryCallInfo) -> Call {
    use crate::gong::models::CallParticipant;

    // Parse the date string to Zoned
    // Try parsing with timezone first, then without (as local time)
    let scheduled_start = jiff::Zoned::strptime(&info.date, "%Y-%m-%dT%H:%M:%S%.fZ")
        .or_else(|_| jiff::Zoned::strptime(&info.date, "%Y-%m-%dT%H:%M:%SZ"))
        .or_else(|_| {
            // Parse as local time for dates without timezone (matching Python behavior)
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
        account_id: "unknown".to_string(), // Not available in LibraryCallInfo
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
        direction: CallDirection::Outbound, // Default assumption
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
        transcript: None, // Not available in LibraryCallInfo
        summary: None,
        topics: Vec::new(),
        action_items: Vec::new(),
    }
}

/// Convert CustomerCallInfo to Call model
fn convert_customer_call_info_to_call(info: CustomerCallInfo) -> Call {
    use crate::gong::models::CallParticipant;

    // Parse the date string to Zoned
    // Try parsing with timezone first, then without (as local time)
    let scheduled_start = jiff::Zoned::strptime(&info.date, "%Y-%m-%dT%H:%M:%S%.fZ")
        .or_else(|_| jiff::Zoned::strptime(&info.date, "%Y-%m-%dT%H:%M:%SZ"))
        .or_else(|_| {
            // Parse as local time for dates without timezone (matching Python behavior)
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
        account_id: info.account_id.unwrap_or_else(|| "unknown".to_string()),
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
        direction: CallDirection::Outbound, // Default assumption
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
        transcript: None, // Not available in CustomerCallInfo
        summary: None,
        topics: Vec::new(),
        action_items: Vec::new(),
    }
}

/// Configuration file structure for persistent settings
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CliConfig {
    /// Previously used team call stream ID
    pub team_call_stream_id: Option<String>,
}

/// Configuration file path
fn config_file_path() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".cs-cli-config.json")
}

/// Load saved configuration from disk
pub fn load_config() -> CliConfig {
    let config_path = config_file_path();

    if !config_path.exists() {
        return CliConfig::default();
    }

    match fs::read_to_string(&config_path) {
        Ok(contents) => match serde_json::from_str(&contents) {
            Ok(config) => config,
            Err(e) => {
                warn!("Failed to parse config file: {}, using defaults", e);
                CliConfig::default()
            }
        },
        Err(e) => {
            warn!("Failed to read config file: {}, using defaults", e);
            CliConfig::default()
        }
    }
}

/// Save configuration to disk
pub fn save_config(config: &CliConfig) -> Result<()> {
    let config_path = config_file_path();

    let contents = serde_json::to_string_pretty(config).map_err(|e| {
        crate::CsCliError::Configuration(format!("Failed to serialize config: {e}"))
    })?;

    fs::write(&config_path, contents)
        .map_err(|e| crate::CsCliError::FileIo(format!("Failed to save config: {e}")))?;

    Ok(())
}

/// Main orchestrator for team calls extraction with all API clients
pub struct TeamCallsExtractor {
    config: AppConfig,
    http: Option<HttpClientPool>,
    auth: Option<GongAuthenticator>,
    library_client: Option<GongLibraryClient>,
    details_fetcher: Option<CallDetailsFetcher>,
    customer_search_client: Option<GongCustomerSearchClient>,
    timeline_extractor: Option<TimelineExtractor>,
    email_enhancer: Option<EmailEnhancer>,
    formatter: CallMarkdownFormatter,
    summary_reporter: CallSummaryReporter,
}

impl TeamCallsExtractor {
    /// Create a new extractor instance
    pub fn new(config: AppConfig) -> Self {
        Self {
            config,
            http: None,
            auth: None,
            library_client: None,
            details_fetcher: None,
            customer_search_client: None,
            timeline_extractor: None,
            email_enhancer: None,
            formatter: CallMarkdownFormatter::new(None),
            summary_reporter: CallSummaryReporter::new(),
        }
    }

    /// Initialize all components (setup phase)
    pub async fn setup(&mut self) -> Result<()> {
        println!(
            "{}",
            "Setting up CS-CLI extractor...".truecolor(255, 142, 100)
        );

        // Initialize HTTP client pool
        self.http = Some(HttpClientPool::new(Some(self.config.http.clone())).await?);

        // Initialize authenticator and authenticate
        let mut auth = GongAuthenticator::new(self.config.auth.clone()).await?;
        let authenticated = auth.authenticate().await?;
        if !authenticated {
            return Err(crate::CsCliError::Authentication(
                "Authentication failed: No valid Gong browser session detected".to_string(),
            ));
        }
        self.auth = Some(auth);

        // Set cookies on all HTTP clients in the pool
        if let Some(auth) = &self.auth {
            let auth_state = auth.get_auth_state();
            if auth_state.get("authenticated") == Some(&"true".to_string()) {
                if let Some(http) = &self.http {
                    // Get cookies from auth state and set them on the pool
                    if let Ok(session_cookies) = auth.get_session_cookies() {
                        http.set_cookies(session_cookies.clone()).await?;
                        info!(
                            cookies_count = session_cookies.len(),
                            "Cookies set on HTTP client pool"
                        );
                    }
                }
            }
        }

        // Initialize all API clients with Arc wrappers
        let http_arc = Arc::new(self.http.take().unwrap());
        let auth_arc = Arc::new(self.auth.take().unwrap());

        self.library_client = Some(GongLibraryClient::new(
            http_arc.clone(),
            auth_arc.clone(),
            Some(self.config.clone()),
        ));
        self.details_fetcher = Some(CallDetailsFetcher::new(
            http_arc.clone(),
            auth_arc.clone(),
            Some(self.config.clone()),
        ));
        self.customer_search_client = Some(GongCustomerSearchClient::new(
            http_arc.clone(),
            auth_arc.clone(),
            Some(self.config.clone()),
        )?);
        self.timeline_extractor = Some(TimelineExtractor::new(
            http_arc.clone(),
            auth_arc.clone(),
            Some(self.config.clone()),
            None,
        )?);
        self.email_enhancer = Some(EmailEnhancer::new(
            http_arc.clone(),
            auth_arc.clone(),
            Some(self.config.clone()),
            None,
        ));

        println!("{}", "Setup complete!".truecolor(255, 255, 255));
        Ok(())
    }

    /// Extract team calls from call stream
    pub async fn extract_team_calls(
        &self,
        stream_id: &str,
        days: Option<u32>,
        from_date: Option<&str>,
        to_date: Option<&str>,
    ) -> Result<Vec<Call>> {
        let library_client = self.library_client.as_ref().ok_or_else(|| {
            crate::CsCliError::Generic("Library client not initialized".to_string())
        })?;

        // Determine date range display message
        let date_range_desc = if from_date.is_some() || to_date.is_some() {
            format!(
                "from {} to {}",
                from_date.unwrap_or("beginning"),
                to_date.unwrap_or("now")
            )
        } else {
            let days_value = days.unwrap_or(7);
            format!("last {days_value} days")
        };

        println!(
            "{}",
            format!("Extracting team calls from {date_range_desc}...").cyan()
        );

        // Create progress indicator
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        pb.set_message("Fetching call stream...");
        pb.enable_steady_tick(std::time::Duration::from_millis(100));

        // Get all calls with pagination
        let mut all_calls = Vec::new();
        let mut offset = 0;
        const PAGE_SIZE: usize = 10;

        loop {
            pb.set_message(format!(
                "Fetching calls (page {})...",
                offset / PAGE_SIZE + 1
            ));

            // Pass date parameters based on what was provided
            let response = if from_date.is_some() || to_date.is_some() {
                library_client
                    .get_library_calls(Some(stream_id), None, from_date, to_date, offset)
                    .await?
            } else {
                library_client
                    .get_library_calls(Some(stream_id), days.map(|d| d as i32), None, None, offset)
                    .await?
            };

            if response.calls.is_empty() {
                break;
            }

            for call_info in response.calls {
                let call = convert_library_call_info_to_call(call_info);
                all_calls.push(call);
            }

            offset += PAGE_SIZE;
            pb.set_message(format!("Found {} calls so far...", all_calls.len()));
        }

        pb.finish_with_message(format!("Found {} calls in stream", all_calls.len()));

        if all_calls.is_empty() {
            println!("{}", "No calls found in call stream!".yellow());
            return Ok(Vec::new());
        }

        // Enhance calls with detailed information (transcripts)
        let pb = ProgressBar::new(all_calls.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );
        pb.set_message("Fetching call details...");

        // Fetch detailed information including transcripts
        let details_fetcher = self.details_fetcher.as_ref().ok_or_else(|| {
            crate::CsCliError::Generic("Details fetcher not initialized".to_string())
        })?;

        let mut enhanced_calls = Vec::new();
        for (i, call) in all_calls.iter().enumerate() {
            pb.set_message(format!(
                "Fetching details for call {}/{}",
                i + 1,
                all_calls.len()
            ));

            if let Some(details) = details_fetcher.get_call_details(&call.id).await? {
                let mut enhanced_call = call.clone();

                // Add transcript from detailed call data
                enhanced_call.transcript = if !details.transcript.is_empty() {
                    Some(details.transcript)
                } else {
                    Some("No transcript available.".to_string())
                };

                // Use attendees from detailed call data (which has actual names)
                enhanced_call.participants = details
                    .attendees
                    .into_iter()
                    .map(|attendee| {
                        use crate::gong::models::CallParticipant;
                        CallParticipant {
                            id: None,
                            name: attendee.name,
                            email: if !attendee.email.is_empty() {
                                Some(attendee.email)
                            } else {
                                None
                            },
                            phone: None,
                            title: if !attendee.title.is_empty() {
                                Some(attendee.title)
                            } else {
                                None
                            },
                            company: if !attendee.company.is_empty() {
                                Some(attendee.company)
                            } else {
                                None
                            },
                            is_internal: false,
                            speaking_time: None,
                            talk_ratio: None,
                        }
                    })
                    .collect();

                // Add generated title for intelligent file naming if not already present
                if !details.generated_title.is_empty()
                    && enhanced_call.title != details.generated_title
                {
                    enhanced_call.title = details.generated_title;
                }

                enhanced_calls.push(enhanced_call);
            } else {
                // If details fetch failed, keep the original call with no transcript
                let mut enhanced_call = call.clone();
                enhanced_call.transcript = Some("No transcript available.".to_string());
                enhanced_calls.push(enhanced_call);
            }

            pb.inc(1);
        }

        pb.finish_with_message("Call details fetching complete");

        println!(
            "{}",
            format!("Successfully extracted {} team calls", enhanced_calls.len()).green()
        );
        Ok(enhanced_calls)
    }

    /// Extract customer calls with date range support
    pub async fn extract_customer_calls(
        &self,
        name: &str,
        days: Option<u32>,
        from_date: Option<&str>,
        to_date: Option<&str>,
    ) -> Result<(Vec<Call>, String)> {
        let customer_client = self.customer_search_client.as_ref().ok_or_else(|| {
            crate::CsCliError::Generic("Customer search client not initialized".to_string())
        })?;

        // Determine date range display message
        let date_range_desc = if from_date.is_some() || to_date.is_some() {
            format!(
                "from {} to {}",
                from_date.unwrap_or("beginning"),
                to_date.unwrap_or("now")
            )
        } else {
            let days_value = days.unwrap_or(90); // Default to 90 days for customer
            format!("last {days_value} days")
        };

        println!(
            "{}",
            format!("Extracting calls for '{name}' from {date_range_desc}...").cyan()
        );

        // Calculate date range for smart pagination
        let (start_date, end_date) = if from_date.is_some() || to_date.is_some() {
            // Use custom date range
            let start = from_date.and_then(|d| jiff::Zoned::strptime(d, "%Y-%m-%d").ok());
            let end = to_date.and_then(|d| {
                jiff::Zoned::strptime(d, "%Y-%m-%d").ok().map(|z| {
                    // Add 23:59:59 to include the entire end date
                    z.saturating_add(jiff::Span::new().hours(23).minutes(59).seconds(59))
                })
            });
            (start, end)
        } else if let Some(days_value) = days {
            // Use days_back
            let now = jiff::Zoned::now();
            let start = now
                .checked_sub(jiff::Span::new().days(days_value as i64))
                .ok();
            (start, None)
        } else {
            (None, None)
        };

        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        pb.set_message("Searching for customer...");
        pb.enable_steady_tick(std::time::Duration::from_millis(100));

        // Get customer calls with smart pagination
        let mut all_calls = Vec::new();
        let mut offset = 0;
        let mut resolved_name = name.to_string();
        let mut should_stop_pagination = false;
        const PAGE_SIZE: usize = 10;

        loop {
            pb.set_message(format!(
                "Fetching customer calls (page {})...",
                offset / PAGE_SIZE + 1
            ));

            let response = customer_client
                .get_customer_calls(name, PAGE_SIZE, offset, false)
                .await?;

            // Capture resolved customer name from first response
            if offset == 0 && !response.companies.is_empty() {
                resolved_name = response.companies[0].clone();
                pb.set_message(format!("Resolved customer: '{resolved_name}'"));
            }

            if response.calls.is_empty() {
                break;
            }

            // Filter calls and check if we should stop pagination
            for call_info in response.calls {
                let call = convert_customer_call_info_to_call(call_info);

                // Check if call is too old (smart pagination)
                if let Some(ref start) = start_date {
                    if call.scheduled_start < *start {
                        should_stop_pagination = true;
                        break;
                    }
                }

                // Apply date filtering
                let include_call = match (&start_date, &end_date) {
                    (Some(start), Some(end)) => {
                        call.scheduled_start >= *start && call.scheduled_start <= *end
                    }
                    (Some(start), None) => call.scheduled_start >= *start,
                    (None, Some(end)) => call.scheduled_start <= *end,
                    (None, None) => true,
                };

                if include_call {
                    all_calls.push(call);
                }
            }

            if should_stop_pagination {
                pb.set_message("Stopping pagination - reached calls outside date range");
                break;
            }

            offset += PAGE_SIZE;
        }

        pb.finish_with_message(format!(
            "Found {} calls for '{}'",
            all_calls.len(),
            resolved_name
        ));

        println!(
            "{}",
            format!("Successfully extracted {} customer calls", all_calls.len()).green()
        );
        Ok((all_calls, resolved_name))
    }

    /// Extract customer communications (calls + emails)
    pub async fn extract_customer_communications(
        &mut self,
        name: &str,
        days: u32,
        include_emails: bool,
        emails_only: bool,
        fetch_email_bodies: bool,
    ) -> Result<(Vec<Call>, Vec<Email>, String)> {
        println!(
            "{}",
            format!("Extracting communications for '{name}' from last {days} days...").cyan()
        );

        if emails_only {
            println!(
                "{}",
                "Extracting only emails (calls will be ignored)".yellow()
            );
        } else if include_emails {
            println!("{}", "Filtering emails to remove blasts and spam".yellow());
        }

        // Step 1: Calculate date range
        let now = jiff::Zoned::now();
        let start_date = now
            .checked_sub(jiff::Span::new().days(days as i64))
            .unwrap_or(now.clone());
        let end_date = Some(now);

        // Step 2: Find customer accounts using customer search API
        let customer_client = self.customer_search_client.as_ref().ok_or_else(|| {
            crate::CsCliError::Generic("Customer search client not initialized".to_string())
        })?;

        // Get customer calls to extract account IDs
        let customer_response = customer_client
            .get_customer_calls(name, 10, 0, false)
            .await?;

        // Capture resolved customer name from response
        let resolved_customer_name = if !customer_response.companies.is_empty() {
            customer_response.companies[0].clone()
        } else {
            name.to_string()
        };

        println!(
            "{}",
            format!("Resolved customer name: '{resolved_customer_name}'").green()
        );

        // Get account IDs from the response
        let account_ids = customer_response.account_ids;

        if account_ids.is_empty() {
            println!(
                "{}",
                format!("No accounts found for customer '{name}'").red()
            );
            return Ok((Vec::new(), Vec::new(), resolved_customer_name));
        }

        println!(
            "{}",
            format!("Found {} accounts for customer '{name}'", account_ids.len()).green()
        );

        // Step 3: Use timeline extractor to get communications from these accounts
        let timeline_extractor = self.timeline_extractor.as_mut().ok_or_else(|| {
            crate::CsCliError::Generic("Timeline extractor not initialized".to_string())
        })?;

        let mut all_calls: Vec<Call> = Vec::new();
        let mut all_emails: Vec<Email> = Vec::new();

        for account_id in &account_ids {
            match timeline_extractor
                .extract_account_timeline(account_id, start_date.clone(), end_date.clone())
                .await
            {
                Ok(timeline_result) => {
                    if !emails_only {
                        all_calls.extend(timeline_result.calls);
                    }
                    if include_emails || emails_only {
                        all_emails.extend(timeline_result.emails);
                    }
                }
                Err(e) => {
                    warn!(
                        account_id = %account_id,
                        error = %e,
                        "Failed to extract timeline for account"
                    );
                    continue;
                }
            }
        }

        println!(
            "{}",
            format!(
                "Timeline extraction complete: {} calls, {} emails",
                all_calls.len(),
                all_emails.len()
            )
            .green()
        );

        // Step 4: Enhance email bodies if requested
        let mut emails = all_emails;
        if (include_emails || emails_only) && fetch_email_bodies && !emails.is_empty() {
            if let Some(email_enhancer) = &self.email_enhancer {
                println!(
                    "{}",
                    "Fetching email body content...".truecolor(255, 142, 100)
                );

                let pb = ProgressBar::new(emails.len() as u64);
                pb.set_style(
                    ProgressStyle::default_bar()
                        .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                        .unwrap()
                        .progress_chars("#>-"),
                );
                pb.set_message("Fetching email bodies...");

                emails = email_enhancer
                    .enhance_emails_with_progress(emails, true)
                    .await
                    .map_err(|e| {
                        crate::CsCliError::Generic(format!("Failed to enhance emails: {e}"))
                    })?;

                pb.finish_with_message("Email body enhancement complete");
                println!(
                    "{}",
                    "Email body enhancement complete".truecolor(255, 255, 255)
                );
            }
        }

        // Step 5: For calls, get detailed information (transcripts) if not emails-only
        let mut detailed_calls = Vec::new();
        if !emails_only && !all_calls.is_empty() {
            let details_fetcher = self.details_fetcher.as_ref().ok_or_else(|| {
                crate::CsCliError::Generic("Details fetcher not initialized".to_string())
            })?;

            let pb = ProgressBar::new(all_calls.len() as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                    .unwrap()
                    .progress_chars("#>-"),
            );
            pb.set_message("Fetching call details...");

            for (i, call) in all_calls.iter().enumerate() {
                pb.set_message(format!(
                    "Fetching details for call {}/{}",
                    i + 1,
                    all_calls.len()
                ));

                let mut enhanced_call = call.clone();

                // Get detailed call information (mainly for transcript)
                if let Some(details) = details_fetcher.get_call_details(&call.id).await? {
                    enhanced_call.transcript = if !details.transcript.is_empty() {
                        Some(details.transcript)
                    } else {
                        Some("No transcript available.".to_string())
                    };

                    // Add generated title for intelligent file naming
                    if !details.generated_title.is_empty() {
                        enhanced_call.generated_title = Some(details.generated_title.clone());
                        // Also update title if it's generic
                        if enhanced_call.title == "Call" || enhanced_call.title.is_empty() {
                            enhanced_call.title = details.generated_title;
                        }
                    }
                } else {
                    enhanced_call.transcript = Some("No transcript available.".to_string());
                }

                // Set the resolved customer name on the call
                enhanced_call.customer_name = Some(resolved_customer_name.clone());

                detailed_calls.push(enhanced_call);
                pb.inc(1);
            }

            pb.finish_with_message("Call details fetching complete");
        }

        println!(
            "{}",
            format!(
                "Successfully extracted {} calls and {} emails for customer '{}'",
                detailed_calls.len(),
                emails.len(),
                resolved_customer_name
            )
            .green()
        );

        Ok((detailed_calls, emails, resolved_customer_name))
    }

    /// Save calls as markdown files
    pub fn save_calls_as_markdown(
        &self,
        calls: &[Call],
        customer_name: Option<&str>,
    ) -> Result<Vec<PathBuf>> {
        if calls.is_empty() {
            return Ok(Vec::new());
        }

        println!(
            "{}",
            "Generating markdown files...".truecolor(255, 142, 100)
        );

        let saved_files = self
            .formatter
            .save_multiple_calls(calls, customer_name)
            .map_err(|e| crate::CsCliError::Generic(format!("Failed to save calls: {e}")))?;

        println!(
            "{}",
            format!("Saved {} markdown files", saved_files.len()).green()
        );

        // Generate summary report
        if let Some(first_file) = saved_files.first() {
            if let Some(output_dir) = first_file.parent() {
                let summary_path = output_dir.join("SUMMARY.md");
                self.summary_reporter
                    .generate_summary_report(calls, Some(&summary_path), customer_name)
                    .map_err(|e| {
                        crate::CsCliError::Generic(format!("Failed to generate summary: {e}"))
                    })?;
                println!(
                    "{}",
                    format!("Summary report saved to {}", summary_path.display()).green()
                );
            }
        }

        Ok(saved_files)
    }

    /// Save calls as markdown files with resolved customer name for summary
    pub fn save_calls_as_markdown_with_resolved_name(
        &self,
        calls: &[Call],
        customer_name: Option<&str>,
        resolved_customer_name: Option<&str>,
    ) -> Result<Vec<PathBuf>> {
        if calls.is_empty() {
            return Ok(Vec::new());
        }

        println!(
            "{}",
            "Generating markdown files...".truecolor(255, 142, 100)
        );

        let saved_files = self
            .formatter
            .save_multiple_calls(calls, customer_name)
            .map_err(|e| crate::CsCliError::Generic(format!("Failed to save calls: {e}")))?;

        println!(
            "{}",
            format!("Saved {} markdown files", saved_files.len()).green()
        );

        // Generate summary report with resolved customer name for better accuracy
        if let Some(first_file) = saved_files.first() {
            if let Some(output_dir) = first_file.parent() {
                let summary_path = output_dir.join("SUMMARY.md");
                // Use resolved_customer_name if provided, otherwise fall back to customer_name
                let name_for_summary = resolved_customer_name.or(customer_name);
                self.summary_reporter
                    .generate_summary_report(calls, Some(&summary_path), name_for_summary)
                    .map_err(|e| {
                        crate::CsCliError::Generic(format!("Failed to generate summary: {e}"))
                    })?;
                println!(
                    "{}",
                    format!("Summary report saved to {}", summary_path.display()).green()
                );
            }
        }

        Ok(saved_files)
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

        println!(
            "{}",
            "Generating email markdown files...".truecolor(255, 142, 100)
        );

        let saved_files = self
            .formatter
            .save_emails_as_markdown(emails, customer_name, Some(customer_name))
            .map_err(|e| crate::CsCliError::Generic(format!("Failed to save emails: {e}")))?;

        println!(
            "{}",
            format!(
                "Saved {} emails across {} batch files",
                emails.len(),
                saved_files.len()
            )
            .green()
        );

        Ok(saved_files)
    }

    /// Cleanup resources
    pub async fn cleanup(&mut self) {
        // Cleanup HTTP client if needed
        // Note: HttpClientPool may not have a cleanup method,
        // resources should be cleaned up when dropped
        self.http = None;
        self.auth = None;
        self.library_client = None;
        self.details_fetcher = None;
        self.customer_search_client = None;
        self.timeline_extractor = None;
        self.email_enhancer = None;
    }
}

/// Main CLI runner function
pub async fn run_cli() -> Result<()> {
    // Parse command line arguments
    let args = CliArgs::parse();
    let command = args.parse_command()?;

    // Initialize logging based on debug flag
    // Use try_init to avoid panic if subscriber already set
    if args.debug {
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .try_init();
    } else {
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .try_init();
    }

    // Unlock macOS keychain once at startup (prompt if needed)
    if cfg!(target_os = "macos") {
        let password = match &args.keychain_password {
            Some(p) => p.clone(),
            None => {
                println!(
                    "{}",
                    "Make sure you're logged into Gong in your browser!".truecolor(255, 142, 100)
                );
                rpassword::prompt_password(&format!(
                    "{}",
                    "Enter your Macbook password: ".truecolor(111, 44, 186)
                ))
                .map_err(|e| {
                    crate::CsCliError::Authentication(format!("Failed to read password: {}", e))
                })?
            }
        };

        unlock_keychain_with_cli_password(&password)?;
        info!("Keychain unlocked for session");
    }

    // Load application configuration
    let mut cli_config = load_config();
    let app_config = AppConfig::from_env()?;

    // Route to appropriate handler based on command
    match command {
        ParsedCommand::Interactive => {
            // Launch interactive mode
            let interactive_command = interactive_mode()?;
            execute_command(interactive_command, &mut cli_config, app_config).await
        }
        ParsedCommand::Customer { .. } | ParsedCommand::Team { .. } => {
            execute_command(command, &mut cli_config, app_config).await
        }
        ParsedCommand::Completion { shell } => {
            generate_completion(shell);
            Ok(())
        }
    }
}

/// Generate shell completion script
fn generate_completion(shell: Shell) {
    let mut app = CliArgs::command();
    let bin_name = app.get_name().to_string();
    generate(shell, &mut app, bin_name, &mut std::io::stdout());
}

/// Execute the parsed command
async fn execute_command(
    command: ParsedCommand,
    cli_config: &mut CliConfig,
    app_config: AppConfig,
) -> Result<()> {
    // Initialize extractor
    let mut extractor = TeamCallsExtractor::new(app_config);

    // Setup components
    extractor.setup().await?;

    let mut saved_files = Vec::new();

    match command {
        ParsedCommand::Team {
            stream_id, days, ..
        } => {
            // Handle team extraction
            let stream_id = match stream_id {
                Some(id) => id,
                None => {
                    // Interactive team mode
                    let team_command =
                        interactive_team_mode(cli_config.team_call_stream_id.clone())?;
                    if let ParsedCommand::Team {
                        stream_id: Some(id),
                        ..
                    } = team_command
                    {
                        // Save the stream ID for future use
                        cli_config.team_call_stream_id = Some(id.clone());
                        save_config(cli_config)?;
                        id
                    } else {
                        return Err(crate::CsCliError::Generic(
                            "Failed to get stream ID".to_string(),
                        ));
                    }
                }
            };

            let days = days.unwrap_or(7);
            let calls = extractor
                .extract_team_calls(&stream_id, Some(days), None, None)
                .await?;

            if !calls.is_empty() {
                let call_files = extractor.save_calls_as_markdown_with_resolved_name(
                    &calls,
                    Some("Team"),
                    Some("Team"),
                )?;
                saved_files.extend(call_files);
            }

            // Display results
            println!();
            println!("{}", "Extraction Complete!".bold().truecolor(255, 108, 55));
            println!("Extracted {} team calls", calls.len());
            println!("Saved {} markdown files", saved_files.len());
        }

        ParsedCommand::Customer {
            name,
            days,
            content_type,
            emails_only,
            fetch_email_bodies,
            ..
        } => {
            // Handle customer extraction
            let days = days.unwrap_or(90);

            let (calls, emails, resolved_name) =
                if matches!(content_type, ContentType::Emails | ContentType::Both) {
                    // Extract communications (calls + emails)
                    extractor
                        .extract_customer_communications(
                            &name,
                            days,
                            true, // include_emails
                            emails_only,
                            fetch_email_bodies,
                        )
                        .await?
                } else {
                    // Extract calls only
                    let (calls, resolved_name) = extractor
                        .extract_customer_calls(&name, Some(days), None, None)
                        .await?;
                    (calls, Vec::new(), resolved_name)
                };

            // Save results
            if !calls.is_empty() && !emails_only {
                let call_files = extractor.save_calls_as_markdown_with_resolved_name(
                    &calls,
                    Some(&resolved_name),
                    Some(&resolved_name),
                )?;
                saved_files.extend(call_files);
            }

            if !emails.is_empty() {
                let email_files = extractor.save_emails_as_markdown(&emails, &resolved_name)?;
                saved_files.extend(email_files);
            }

            // Display results
            println!();
            println!("{}", "Extraction Complete!".bold().truecolor(255, 108, 55));

            if emails_only {
                println!("Extracted {} emails for '{}'", emails.len(), resolved_name);
                if !emails.is_empty() {
                    let emails_with_bodies = emails
                        .iter()
                        .filter(|e| e.body_text.as_ref().is_some_and(|b| !b.trim().is_empty()))
                        .count();
                    println!(
                        "{}/{} emails have full body content",
                        emails_with_bodies,
                        emails.len()
                    );
                }
            } else if matches!(content_type, ContentType::Both) {
                println!(
                    "Extracted {} calls and {} emails for '{}'",
                    calls.len(),
                    emails.len(),
                    resolved_name
                );
                if !emails.is_empty() {
                    let emails_with_bodies = emails
                        .iter()
                        .filter(|e| e.body_text.as_ref().is_some_and(|b| !b.trim().is_empty()))
                        .count();
                    println!(
                        "Advanced BDR/SPAM filtering applied - {}/{} emails have full content",
                        emails_with_bodies,
                        emails.len()
                    );
                }
            } else {
                println!("Extracted {} calls for '{}'", calls.len(), resolved_name);
            }

            println!("Saved {} markdown files", saved_files.len());
        }

        ParsedCommand::Interactive => {
            // This should not happen as interactive is handled above
            unreachable!("Interactive mode should be handled before this point");
        }
        ParsedCommand::Completion { .. } => {
            // This should not happen as completion is handled above
            unreachable!("Completion should be handled before this point");
        }
    }

    // Show output directory
    if !saved_files.is_empty() {
        if let Some(output_directory) = saved_files.first().and_then(|f| f.parent()) {
            println!(
                "Output directory: {}",
                output_directory.display().to_string().bold()
            );
        }
    }

    // Cleanup
    extractor.cleanup().await;

    Ok(())
}
