use jiff::{Span, Zoned};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::gong::api::client::HttpClientPool;
use crate::gong::auth::GongAuthenticator;
use crate::gong::config::AppConfig;
use crate::{CsCliError, Result};

/// Library calls API response
#[derive(Debug)]
pub struct LibraryCallsResult {
    /// List of calls from the library
    pub calls: Vec<LibraryCallInfo>,
}

/// Call information from library API
#[derive(Debug, Clone)]
pub struct LibraryCallInfo {
    /// Call ID
    pub id: Option<String>,
    /// Call title
    pub title: String,
    /// Generated title from Gong
    pub generated_title: String,
    /// Customer name (deduplicated)
    pub customer_name: String,
    /// Call date/time
    pub date: String,
    /// Call duration in seconds
    pub duration: i32,
    /// List of participants
    pub participants: Vec<String>,
    /// URL to the call recording
    pub call_url: String,
    /// Raw API response data
    pub raw_data: Value,
}

/// Detailed call information with transcript
#[derive(Debug, Clone)]
pub struct DetailedCallInfo {
    /// Call ID
    pub id: String,
    /// Call title
    pub title: String,
    /// Generated title from Gong
    pub generated_title: String,
    /// Customer name
    pub customer_name: String,
    /// Call date/time
    pub date: String,
    /// List of attendees with details
    pub attendees: Vec<CallAttendee>,
    /// Full call transcript
    pub transcript: String,
}

/// Call attendee information
#[derive(Debug, Clone)]
pub struct CallAttendee {
    /// Attendee name
    pub name: String,
    /// Job title
    pub title: String,
    /// Company name
    pub company: String,
    /// Email address
    pub email: String,
}

/// Gong Library API client for extracting call data
pub struct GongLibraryClient {
    /// HTTP client pool for making requests
    http_client: Arc<HttpClientPool>,
    /// Authentication manager
    auth: Arc<GongAuthenticator>,
    /// Application configuration
    _config: Option<AppConfig>,
}

impl GongLibraryClient {
    /// Create a new library client
    pub fn new(
        http_client: Arc<HttpClientPool>,
        auth: Arc<GongAuthenticator>,
        config: Option<AppConfig>,
    ) -> Self {
        Self {
            http_client,
            auth,
            _config: config,
        }
    }

    /// Get calls from a Gong call stream for the specified date range with pagination
    ///
    /// # Arguments
    /// * `call_stream_id` - Call stream ID to query (default: "195005774106634129")
    /// * `days_back` - Number of days back from today (ignored if from_date/to_date provided)
    /// * `from_date` - Start date in YYYY-MM-DD format (optional)
    /// * `to_date` - End date in YYYY-MM-DD format (optional)
    /// * `offset` - Pagination offset for retrieving calls
    ///
    /// # Returns
    /// Result containing calls data
    pub async fn get_library_calls(
        &self,
        call_stream_id: Option<&str>,
        days_back: Option<i32>,
        from_date: Option<&str>,
        to_date: Option<&str>,
        offset: usize,
    ) -> Result<LibraryCallsResult> {
        let stream_id = call_stream_id.unwrap_or("195005774106634129");
        let base_url = self.auth.get_base_url()?;
        let url = format!("{base_url}/callstream/read-content");

        // Determine date range based on provided parameters
        let (from_date_str, to_date_str) = if from_date.is_some() || to_date.is_some() {
            // Use provided dates (can be empty for unlimited range)
            (
                from_date.unwrap_or("").to_string(),
                to_date.unwrap_or("").to_string(),
            )
        } else {
            // Calculate date range from days_back parameter
            let days_back = days_back.unwrap_or(7); // Default to 7 days
            let now = Zoned::now();
            let to_date_obj = now.date();
            let from_date_obj = to_date_obj.saturating_sub(Span::new().days(days_back as i64));

            (from_date_obj.to_string(), to_date_obj.to_string())
        };

        // Build query parameters
        let mut params = HashMap::new();
        params.insert("call-stream-id".to_string(), stream_id.to_string());
        params.insert("offset".to_string(), offset.to_string());
        params.insert("from-date".to_string(), from_date_str.clone());
        params.insert("to-date".to_string(), to_date_str.clone());

        info!(
            call_stream_id = %stream_id,
            offset = offset,
            from_date = %from_date_str,
            to_date = %to_date_str,
            days_back = ?days_back,
            "Fetching call stream data with date range and pagination support"
        );

        // Get authenticated headers including CSRF token
        let mut headers = self.auth.get_authenticated_headers(true).await?;

        // Add required headers based on working curl command
        headers.insert("accept".to_string(), "application/json".to_string());
        headers.insert("content-type".to_string(), "application/json".to_string());
        headers.insert(
            "referer".to_string(),
            format!("{base_url}/callstream/read-content?call-stream-id={stream_id}"),
        );
        headers.insert("sec-fetch-dest".to_string(), "empty".to_string());
        headers.insert("sec-fetch-mode".to_string(), "cors".to_string());
        headers.insert("sec-fetch-site".to_string(), "same-origin".to_string());

        // Update headers on HTTP client
        self.http_client.update_headers(headers).await?;

        // Build URL with query parameters
        let query_string = params
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");
        let full_url = format!("{url}?{query_string}");

        let response = self.http_client.get(&full_url).await?;

        if response.status().is_success() {
            let response_text = response
                .text()
                .await
                .map_err(|e| CsCliError::ApiRequest(format!("Failed to read response: {e}")))?;

            // Use sonic-rs for fast JSON parsing
            let data: Value = serde_json::from_str(&response_text)
                .map_err(|e| CsCliError::ApiRequest(format!("Failed to parse JSON: {e}")))?;

            // Extract calls from the response
            let calls = self.extract_call_ids_and_metadata(&data)?;

            debug!(
                calls_received = calls.len(),
                "Successfully received call stream data"
            );

            Ok(LibraryCallsResult { calls })
        } else {
            let status_code = response.status().as_u16();
            let response_text = response.text().await.unwrap_or_default();
            let truncated_text = if response_text.len() > 500 {
                &response_text[..500]
            } else {
                &response_text
            };

            error!(
                status_code = status_code,
                response_text = %truncated_text,
                "Call stream API request failed"
            );

            Ok(LibraryCallsResult { calls: Vec::new() })
        }
    }

    /// Extract call IDs and basic metadata from the call stream API response
    ///
    /// # Arguments
    /// * `api_response` - Raw API response data
    ///
    /// # Returns
    /// Vector of call info with IDs and basic metadata
    pub fn extract_call_ids_and_metadata(
        &self,
        api_response: &Value,
    ) -> Result<Vec<LibraryCallInfo>> {
        let mut calls = Vec::new();

        // Based on call stream API response structure
        if let Some(folder_content) = api_response.get("folderContent") {
            if let Some(call_data) = folder_content.get("calls") {
                if let Some(calls_array) = call_data.as_array() {
                    for item in calls_array {
                        if let Some(item_obj) = item.as_object() {
                            // Clean customer name to remove duplications
                            let raw_customer_name = item_obj
                                .get("customerAccountName")
                                .and_then(|c| c.as_str())
                                .unwrap_or("");
                            let clean_customer_name =
                                self.deduplicate_customer_name(raw_customer_name);

                            // Extract participants
                            let participants = item_obj
                                .get("participants")
                                .and_then(|p| p.as_array())
                                .map(|arr| {
                                    arr.iter()
                                        .filter_map(|participant| participant.as_str())
                                        .map(|s| s.to_string())
                                        .collect()
                                })
                                .unwrap_or_else(Vec::new);

                            let call_info = LibraryCallInfo {
                                id: item_obj
                                    .get("id")
                                    .and_then(|id| id.as_str())
                                    .map(|s| s.to_string()),
                                title: item_obj
                                    .get("title")
                                    .and_then(|t| t.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                                generated_title: item_obj
                                    .get("generatedTitle")
                                    .and_then(|t| t.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                                customer_name: clean_customer_name,
                                date: item_obj
                                    .get("effectiveStartDateTime")
                                    .and_then(|d| d.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                                duration: item_obj
                                    .get("duration")
                                    .and_then(|d| d.as_i64())
                                    .unwrap_or(0) as i32,
                                participants,
                                call_url: item_obj
                                    .get("callUrl")
                                    .and_then(|u| u.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                                raw_data: item.clone(),
                            };

                            // Only add if we have a valid call ID
                            if call_info.id.is_some() {
                                debug!(
                                    title = %call_info.title,
                                    customer = %call_info.customer_name,
                                    date = %call_info.date,
                                    "Found call"
                                );
                                calls.push(call_info);
                            }
                        }
                    }
                }
            }
        }

        debug!(
            extracted_calls = calls.len(),
            "Extracted call IDs from call stream response"
        );
        Ok(calls)
    }

    /// Remove duplicated customer names from comma-separated lists
    ///
    /// Examples:
    /// - "AT&T, AT&T, AT&T, AT&T, AT&T" -> "AT&T"
    /// - "Deloitte, Deloitte, Deloitte" -> "Deloitte"
    /// - "rbcnomail.com, RBC, RBC, RBC, RBC, RBC" -> "rbcnomail.com, RBC"
    ///
    /// # Arguments
    /// * `customer_name` - Raw customer name string, potentially with duplicates
    ///
    /// # Returns
    /// Cleaned customer name with duplicates removed
    pub fn deduplicate_customer_name(&self, customer_name: &str) -> String {
        if customer_name.is_empty() {
            return customer_name.to_string();
        }

        // Split by comma and clean each part
        let parts: Vec<String> = customer_name
            .split(',')
            .map(|part| part.trim().to_string())
            .collect();

        // Remove duplicates while preserving order
        let mut seen = HashSet::new();
        let mut unique_parts = Vec::new();

        for part in parts {
            // Case-insensitive deduplication
            let part_lower = part.to_lowercase();
            if !seen.contains(&part_lower) && !part.is_empty() {
                seen.insert(part_lower);
                unique_parts.push(part);
            }
        }

        unique_parts.join(", ")
    }

    /// Filter calls to only include those from the specified number of days back
    ///
    /// # Arguments
    /// * `calls` - List of call info structures
    /// * `days_back` - Number of days back to include
    ///
    /// # Returns
    /// Filtered list of calls
    pub fn filter_calls_by_date(
        &self,
        calls: &[LibraryCallInfo],
        days_back: i32,
    ) -> Result<Vec<LibraryCallInfo>> {
        if calls.is_empty() || days_back <= 0 {
            return Ok(calls.to_vec());
        }

        // Calculate cutoff date
        let now = Zoned::now();
        let cutoff_date = now.saturating_sub(Span::new().days(days_back as i64));

        let mut filtered_calls = Vec::new();

        for call in calls {
            let call_date_str = &call.date;
            if call_date_str.is_empty() {
                // Keep calls without dates to be safe
                filtered_calls.push(call.clone());
                continue;
            }

            // Try to parse the date string using various formats
            let date_formats = [
                "%Y/%m/%d %H:%M:%S",     // Gong userTimezoneActivityTime format
                "%Y-%m-%dT%H:%M:%S.%fZ", // ISO with microseconds
                "%Y-%m-%dT%H:%M:%SZ",    // ISO without microseconds
                "%Y-%m-%d %H:%M:%S",     // Standard format
                "%Y-%m-%d",              // Date only
            ];

            let mut parsed_date: Option<Zoned> = None;
            for format in &date_formats {
                if let Ok(parsed) = jiff::fmt::strtime::parse(format, call_date_str) {
                    if let Ok(zoned) = parsed.to_zoned() {
                        parsed_date = Some(zoned);
                        break;
                    }
                }
            }

            match parsed_date {
                Some(call_date) => {
                    if call_date >= cutoff_date {
                        filtered_calls.push(call.clone());
                    }
                }
                #[allow(non_snake_case)]
                None => {
                    // Couldn't parse date, keep the call to be safe
                    filtered_calls.push(call.clone());
                }
            }
        }

        debug!(
            original_count = calls.len(),
            filtered_count = filtered_calls.len(),
            days_back = days_back,
            "Filtered calls by date"
        );

        Ok(filtered_calls)
    }
}

/// Fetches detailed call information using existing pipeline tools
pub struct CallDetailsFetcher {
    /// HTTP client pool for making requests
    http_client: Arc<HttpClientPool>,
    /// Authentication manager
    auth: Arc<GongAuthenticator>,
    /// Application configuration
    _config: Option<AppConfig>,
}

impl CallDetailsFetcher {
    /// Create a new call details fetcher
    pub fn new(
        http_client: Arc<HttpClientPool>,
        auth: Arc<GongAuthenticator>,
        config: Option<AppConfig>,
    ) -> Self {
        Self {
            http_client,
            auth,
            _config: config,
        }
    }

    /// Get detailed call information including transcript
    ///
    /// # Arguments
    /// * `call_id` - Gong call ID
    ///
    /// # Returns
    /// Detailed call information or None if failed
    pub async fn get_call_details(&self, call_id: &str) -> Result<Option<DetailedCallInfo>> {
        // Get dynamic base URL from authenticator
        let base_url = self.auth.get_base_url()?;
        let url = format!("{base_url}/call/detailed-transcript");

        let mut params = HashMap::new();
        params.insert("call-id".to_string(), call_id.to_string());

        debug!(call_id = %call_id, "Fetching call details");

        // Get authenticated headers
        let headers = self.auth.get_authenticated_headers(true).await?;

        // Update headers on HTTP client
        self.http_client.update_headers(headers).await?;

        // Build URL with query parameters
        let query_string = params
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");
        let full_url = format!("{url}?{query_string}");

        let response = self.http_client.get(&full_url).await?;

        if response.status().is_success() {
            let response_text = response
                .text()
                .await
                .map_err(|e| CsCliError::ApiRequest(format!("Failed to read response: {e}")))?;

            // Use sonic-rs for fast JSON parsing
            let data: Value = serde_json::from_str(&response_text)
                .map_err(|e| CsCliError::ApiRequest(format!("Failed to parse JSON: {e}")))?;

            let call_details = self.extract_call_details(&data, call_id)?;
            Ok(Some(call_details))
        } else {
            let status_code = response.status().as_u16();
            warn!(
                call_id = %call_id,
                status_code = status_code,
                "Failed to fetch call details"
            );
            Ok(None)
        }
    }

    /// Extract and structure call details from API response
    ///
    /// # Arguments
    /// * `api_response` - Raw API response data
    /// * `call_id` - Call ID for logging
    ///
    /// # Returns
    /// Structured call details
    pub fn extract_call_details(
        &self,
        api_response: &Value,
        call_id: &str,
    ) -> Result<DetailedCallInfo> {
        // Debug: Log the actual API response structure
        if let Some(obj) = api_response.as_object() {
            debug!(
                call_id = %call_id,
                response_keys = ?obj.keys().collect::<Vec<_>>(),
                "Call details API response structure"
            );
        }

        // Initialize call info with defaults
        let mut call_info = DetailedCallInfo {
            id: call_id.to_string(),
            title: String::new(),
            generated_title: String::new(),
            customer_name: String::new(),
            date: String::new(),
            attendees: Vec::new(),
            transcript: String::new(),
        };

        // Extract title and generatedTitle
        if let Some(title) = api_response.get("callTitle").and_then(|t| t.as_str()) {
            call_info.title = title.to_string();
        }

        // Try to extract generatedTitle from various fields
        // Priority: generatedTitle > callTitle > callBrief
        let generated_title = api_response
            .get("generatedTitle")
            .or_else(|| api_response.get("callTitle"))
            .or_else(|| api_response.get("callBrief"))
            .and_then(|t| t.as_str())
            .unwrap_or("");
        call_info.generated_title = generated_title.to_string();

        debug!(
            call_id = %call_id,
            call_title = ?api_response.get("callTitle"),
            generated_title = ?api_response.get("generatedTitle"),
            call_brief = ?api_response.get("callBrief"),
            extracted_title = %generated_title,
            "Call title extraction"
        );

        // Extract customer name from callCustomers
        if let Some(customers) = api_response.get("callCustomers") {
            if let Some(customers_array) = customers.as_array() {
                if let Some(customer) = customers_array.first() {
                    if let Some(customer_obj) = customer.as_object() {
                        if let Some(name) = customer_obj.get("name").and_then(|n| n.as_str()) {
                            call_info.customer_name = name.to_string();
                        }
                    } else if let Some(customer_str) = customer.as_str() {
                        call_info.customer_name = customer_str.to_string();
                    }
                }
            } else if let Some(customer_str) = customers.as_str() {
                call_info.customer_name = customer_str.to_string();
            }
        }

        // Extract date
        if let Some(when) = api_response.get("when").and_then(|w| w.as_str()) {
            call_info.date = when.to_string();
        }

        // Extract attendees from multiple participant categories
        let mut attendees = Vec::new();

        // Company participants (internal team)
        if let Some(company_participants) = api_response.get("companyParticipants") {
            self.extract_participants_from_group(
                company_participants,
                api_response
                    .get("callCompanyName")
                    .and_then(|c| c.as_str())
                    .unwrap_or(""),
                &mut attendees,
            );
        }

        // Customer participants (client attendees)
        if let Some(customer_participants) = api_response.get("customerParticipants") {
            self.extract_participants_from_group(customer_participants, "", &mut attendees);
        }

        // Unknown participants
        if let Some(unknown_participants) = api_response.get("unknownParticipants") {
            if let Some(unknown_array) = unknown_participants.as_array() {
                for participant in unknown_array {
                    if let Some(participant_obj) = participant.as_object() {
                        let name = self.extract_participant_name(participant_obj);
                        if !name.is_empty() {
                            let attendee = CallAttendee {
                                name,
                                title: participant_obj
                                    .get("title")
                                    .and_then(|t| t.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                                company: String::new(),
                                email: participant_obj
                                    .get("emailAddress")
                                    .and_then(|e| e.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                            };
                            attendees.push(attendee);
                        }
                    }
                }
            }
        }

        call_info.attendees = attendees;

        // Extract and clean transcript from monologues
        if let Some(monologues) = api_response.get("monologues") {
            if let Some(monologues_array) = monologues.as_array() {
                let mut transcript_parts = Vec::new();

                for monologue in monologues_array {
                    if let Some(monologue_obj) = monologue.as_object() {
                        let mut speaker_name = "Unknown Speaker".to_string();

                        // Try to get speaker name - check multiple possible fields
                        if let Some(name) =
                            monologue_obj.get("speakerName").and_then(|n| n.as_str())
                        {
                            if !name.is_empty() {
                                speaker_name = name.to_string();
                            }
                        } else if let Some(short_name) = monologue_obj
                            .get("speakerShortName")
                            .and_then(|n| n.as_str())
                        {
                            // Look up full name using shortNamesLookup
                            if let Some(lookup) = api_response.get("shortNamesLookup") {
                                if let Some(lookup_obj) = lookup.as_object() {
                                    if let Some(full_info) = lookup_obj.get(short_name) {
                                        speaker_name = full_info
                                            .get("name")
                                            .and_then(|n| n.as_str())
                                            .unwrap_or(short_name)
                                            .to_string();
                                    } else {
                                        speaker_name = short_name.to_string();
                                    }
                                }
                            } else {
                                speaker_name = short_name.to_string();
                            }
                        }

                        // Get the monologue text - check multiple possible structures
                        let mut monologue_text: Option<String> = None;

                        // Try direct text field first
                        if let Some(text) = monologue_obj.get("text").and_then(|t| t.as_str()) {
                            if !text.trim().is_empty() {
                                monologue_text = Some(text.trim().to_string());
                            }
                        }

                        // Fallback to sentences structure
                        if monologue_text.is_none() {
                            if let Some(sentences) = monologue_obj.get("sentences") {
                                if let Some(sentences_array) = sentences.as_array() {
                                    let mut sentence_texts = Vec::new();
                                    for sentence in sentences_array {
                                        if let Some(sentence_obj) = sentence.as_object() {
                                            if let Some(text) =
                                                sentence_obj.get("text").and_then(|t| t.as_str())
                                            {
                                                sentence_texts.push(text.to_string());
                                            }
                                        } else if let Some(sentence_str) = sentence.as_str() {
                                            sentence_texts.push(sentence_str.to_string());
                                        }
                                    }

                                    if !sentence_texts.is_empty() {
                                        monologue_text = Some(sentence_texts.join(" "));
                                    }
                                }
                            }
                        }

                        // Add to transcript if we have text
                        if let Some(text) = monologue_text {
                            transcript_parts.push(format!("**{speaker_name}:** {text}"));
                        }
                    }
                }

                if !transcript_parts.is_empty() {
                    call_info.transcript = transcript_parts.join("\n\n");
                    debug!(
                        call_id = %call_id,
                        monologue_count = transcript_parts.len(),
                        "Extracted transcript with monologues"
                    );
                }
            }
        }

        Ok(call_info)
    }

    /// Extract participants from a participant group (company or customer)
    fn extract_participants_from_group(
        &self,
        participants_data: &Value,
        default_company: &str,
        attendees: &mut Vec<CallAttendee>,
    ) {
        match participants_data {
            Value::Object(participants_obj) => {
                // Handle dictionary structure (company -> participants)
                for (company_name, participants) in participants_obj {
                    if let Some(participants_array) = participants.as_array() {
                        for participant in participants_array {
                            if let Some(participant_obj) = participant.as_object() {
                                let name = self.extract_participant_name(participant_obj);
                                if !name.is_empty() {
                                    let attendee = CallAttendee {
                                        name,
                                        title: participant_obj
                                            .get("title")
                                            .and_then(|t| t.as_str())
                                            .unwrap_or("")
                                            .to_string(),
                                        company: if company_name.is_empty() {
                                            default_company.to_string()
                                        } else {
                                            company_name.clone()
                                        },
                                        email: participant_obj
                                            .get("emailAddress")
                                            .and_then(|e| e.as_str())
                                            .unwrap_or("")
                                            .to_string(),
                                    };
                                    attendees.push(attendee);
                                }
                            }
                        }
                    }
                }
            }
            Value::Array(participants_array) => {
                // Handle list structure
                for participant in participants_array {
                    if let Some(participant_obj) = participant.as_object() {
                        let name = self.extract_participant_name(participant_obj);
                        if !name.is_empty() {
                            let attendee = CallAttendee {
                                name,
                                title: participant_obj
                                    .get("title")
                                    .and_then(|t| t.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                                company: participant_obj
                                    .get("companyName")
                                    .and_then(|c| c.as_str())
                                    .unwrap_or(default_company)
                                    .to_string(),
                                email: participant_obj
                                    .get("emailAddress")
                                    .and_then(|e| e.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                            };
                            attendees.push(attendee);
                        }
                    }
                }
            }
            _ => {
                debug!("Unexpected participant data structure");
            }
        }
    }

    /// Extract participant name from various possible name fields
    ///
    /// # Arguments
    /// * `participant` - Participant data object
    ///
    /// # Returns
    /// Full name string or empty string if no name found
    pub fn extract_participant_name(&self, participant: &serde_json::Map<String, Value>) -> String {
        // Try different name field combinations
        let first_name = participant
            .get("firstName")
            .and_then(|f| f.as_str())
            .unwrap_or("")
            .trim();
        let last_name = participant
            .get("lastName")
            .and_then(|l| l.as_str())
            .unwrap_or("")
            .trim();

        // Full name field
        if let Some(name) = participant.get("name").and_then(|n| n.as_str()) {
            let name = name.trim();
            if !name.is_empty() {
                return name.to_string();
            }
        }

        // First and last name combination
        if !first_name.is_empty() && !last_name.is_empty() {
            return format!("{first_name} {last_name}");
        } else if !first_name.is_empty() {
            return first_name.to_string();
        } else if !last_name.is_empty() {
            return last_name.to_string();
        }

        // Try other possible name fields
        if let Some(full_name) = participant.get("fullName").and_then(|n| n.as_str()) {
            let full_name = full_name.trim();
            if !full_name.is_empty() {
                return full_name.to_string();
            }
        }

        if let Some(display_name) = participant.get("displayName").and_then(|n| n.as_str()) {
            let display_name = display_name.trim();
            if !display_name.is_empty() {
                return display_name.to_string();
            }
        }

        String::new()
    }
}
