use jiff::{Timestamp, Zoned};
use regex::Regex;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tracing::{debug, error, info};

use crate::gong::api::client::HttpClientPool;
use crate::gong::auth::GongAuthenticator;
use crate::gong::config::AppConfig;
use crate::gong::models::{
    Call, CallDirection, CallParticipant, Email, EmailDirection, EmailRecipient, ExtractionRange,
};
use crate::{CsCliError, Result};

/// Type alias for complex email filtering result
type EmailFilterResult = (Vec<HashMap<String, Value>>, HashMap<String, usize>);

/// Template markers for BDR/SPAM/Automation filtering
///
/// These phrases are commonly used in automated sales emails,
/// follow-up templates, and BDR outreach campaigns
const TEMPLATE_MARKERS: &[&str] = &[
    // Auto-reply and out-of-office patterns
    "automatic reply",
    "out of office",
    "out-of-office",
    "ooo",
    "will be out",
    "out of the office",
    "returning on",
    "limited access to email",
    "urgent matters",
];

/// Regex patterns for text normalization (compiled once for performance)
pub struct RegexPatterns {
    pub thread_prefix: Regex,
    pub url: Regex,
    pub greeting: Regex,
    pub company_at: Regex,
    pub account_manager: Regex,
    pub users_number: Regex,
    pub with_users: Regex,
    pub big_numbers: Regex,
    pub name_is: Regex,
    pub i_am: Regex,
    pub date_slash: Regex,
    pub date_month: Regex,
    pub whitespace: Regex,
}

impl RegexPatterns {
    pub fn new() -> Result<Self> {
        Ok(Self {
            thread_prefix: Regex::new(r"\b(re|fwd):\s*").map_err(|e| CsCliError::Generic(format!("Regex error: {e}")))?,
            url: Regex::new(r"https?://\S+").map_err(|e| CsCliError::Generic(format!("Regex error: {e}")))?,
            greeting: Regex::new(r"(?i)\b(hi|hello|hey|dear)\s+[a-z]+[,.-]?\s*").map_err(|e| CsCliError::Generic(format!("Regex error: {e}")))?,
            company_at: Regex::new(r"(?i)\bat\s+[a-z]+\s+with\b").map_err(|e| CsCliError::Generic(format!("Regex error: {e}")))?,
            account_manager: Regex::new(r"(?i)\b[a-z]+\'s\s+account\s+manager\b").map_err(|e| CsCliError::Generic(format!("Regex error: {e}")))?,
            users_number: Regex::new(r"(?i)\b\d+\.?\d*\s+users?\b").map_err(|e| CsCliError::Generic(format!("Regex error: {e}")))?,
            with_users: Regex::new(r"(?i)\bwith\s+\d+\.?\d*\s+users\b").map_err(|e| CsCliError::Generic(format!("Regex error: {e}")))?,
            big_numbers: Regex::new(r"\b\d{2,}\b").map_err(|e| CsCliError::Generic(format!("Regex error: {e}")))?,
            name_is: Regex::new(r"(?i)\bmy name is\s+[a-z]+\b").map_err(|e| CsCliError::Generic(format!("Regex error: {e}")))?,
            i_am: Regex::new(r"(?i)\bi am\s+[a-z]+\'s\b").map_err(|e| CsCliError::Generic(format!("Regex error: {e}")))?,
            date_slash: Regex::new(r"\b\d{1,2}/\d{1,2}/\d{2,4}\b").map_err(|e| CsCliError::Generic(format!("Regex error: {e}")))?,
            date_month: Regex::new(r"(?i)\b(january|february|march|april|may|june|july|august|september|october|november|december)\s+\d{1,2}(st|nd|rd|th)?\b").map_err(|e| CsCliError::Generic(format!("Regex error: {e}")))?,
            whitespace: Regex::new(r"\s+").map_err(|e| CsCliError::Generic(format!("Regex error: {e}")))?,
        })
    }
}

/// Filtering statistics for email processing
#[derive(Debug, Default, Clone)]
pub struct FilteringStats {
    /// Emails filtered by similarity
    pub similarity_filtered: usize,
    /// Emails filtered as BDR/templates
    pub bdr_filtered: usize,
    /// Total emails filtered
    pub total_filtered: usize,
    /// Calls filtered
    pub calls_filtered: usize,
    /// Calls filtered by duration
    pub calls_duration_filtered: usize,
    /// Noise filtered
    pub noise_filtered: usize,
}

/// Timeline extraction result
#[derive(Debug)]
pub struct TimelineResult {
    /// Extracted calls
    pub calls: Vec<Call>,
    /// Extracted emails (after filtering)
    pub emails: Vec<Email>,
    /// Filtering statistics
    pub stats: FilteringStats,
}

/// Extract account timeline with sophisticated email filtering
pub struct TimelineExtractor {
    /// HTTP client pool for making requests
    http_client: Arc<HttpClientPool>,
    /// Authentication manager
    auth: Arc<GongAuthenticator>,
    /// Application configuration
    _config: Option<AppConfig>,
    /// Chunk size in days for API requests
    chunk_days: i32,
    /// Filtering statistics (reset per extraction batch)
    filtered_stats: FilteringStats,
    /// Compiled regex patterns for performance
    regex_patterns: RegexPatterns,
}

impl TimelineExtractor {
    /// Create a new timeline extractor
    pub fn new(
        http_client: Arc<HttpClientPool>,
        auth: Arc<GongAuthenticator>,
        config: Option<AppConfig>,
        chunk_days: Option<i32>,
    ) -> Result<Self> {
        let chunk_days = chunk_days.unwrap_or(30);
        let regex_patterns = RegexPatterns::new()?;

        info!("Timeline extractor initialized with advanced filtering");

        Ok(Self {
            http_client,
            auth,
            _config: config,
            chunk_days,
            filtered_stats: FilteringStats::default(),
            regex_patterns,
        })
    }

    /// Extract all communications for an account within date range
    ///
    /// # Arguments
    /// * `account_id` - Account ID to extract timeline for
    /// * `start_date` - Start date for extraction
    /// * `end_date` - End date for extraction (defaults to now)
    ///
    /// # Returns
    /// Tuple of (calls, emails) with filtering applied
    pub async fn extract_account_timeline(
        &mut self,
        account_id: &str,
        start_date: Zoned,
        end_date: Option<Zoned>,
    ) -> Result<TimelineResult> {
        let end_date = end_date.unwrap_or_else(Zoned::now);

        info!(
            account_id = %account_id,
            start = %start_date,
            end = %end_date,
            "Extracting timeline"
        );

        // Create date range chunks
        let date_range = ExtractionRange::new(start_date.date(), end_date.date(), self.chunk_days);
        let chunks = date_range.chunk_by_days();

        debug!(chunks = chunks.len(), "Timeline chunked");

        // Fetch all chunks concurrently
        let mut chunk_results = Vec::new();
        for (start, end) in chunks {
            let chunk_range = ExtractionRange::new(start, end, self.chunk_days);
            let result = self.fetch_chunk(account_id, &chunk_range).await;
            chunk_results.push(result);
        }

        // Aggregate results
        let mut all_calls = Vec::new();
        let mut all_emails = Vec::new();

        for result in chunk_results {
            match result {
                Ok((calls, emails)) => {
                    all_calls.extend(calls);
                    all_emails.extend(emails);
                }
                Err(e) => {
                    error!(error = %e, "Chunk failed");
                    continue;
                }
            }
        }

        // Sort by date
        all_calls.sort_by_key(|c| c.scheduled_start.clone());
        all_emails.sort_by_key(|e| e.sent_at.clone());

        info!(
            account_id = %account_id,
            calls = all_calls.len(),
            emails = all_emails.len(),
            "Timeline extracted"
        );

        Ok(TimelineResult {
            calls: all_calls,
            emails: all_emails,
            stats: self.filtered_stats.clone(),
        })
    }

    /// Fetch a single timeline chunk with filtering
    ///
    /// # Arguments
    /// * `account_id` - Account ID to fetch for
    /// * `chunk` - Date range chunk to fetch
    ///
    /// # Returns
    /// Tuple of (calls, emails) for this chunk
    pub async fn fetch_chunk(
        &self,
        account_id: &str,
        chunk: &ExtractionRange,
    ) -> Result<(Vec<Call>, Vec<Email>)> {
        let base_url = self.auth.get_base_url()?;

        // Get API parameters from chunk
        let mut params = chunk.to_api_params(account_id);

        // Add required workspace parameters
        let workspace_id = self
            .auth
            .get_workspace_id()
            .unwrap_or("5562739194953732039");
        let team_id = "5359555372180789967"; // Default team ID

        params.insert("workspace-id".to_string(), workspace_id.to_string());
        params.insert("team-id".to_string(), team_id.to_string());

        let endpoint = format!("{base_url}/ajax/account/day-activities");
        let headers = self.auth.get_read_headers()?;

        // Update headers on HTTP client
        self.http_client.update_headers(headers).await?;

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
            if [401, 403].contains(&status_code) {
                self.auth.handle_auth_error(status_code, false).await?;
            }
            error!(
                account_id = %account_id,
                status = status_code,
                "Chunk fetch failed"
            );
            return Ok((Vec::new(), Vec::new()));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| CsCliError::ApiRequest(format!("Failed to read response: {e}")))?;

        let data: Value = serde_json::from_str(&response_text)
            .map_err(|e| CsCliError::ApiRequest(format!("Failed to parse JSON: {e}")))?;

        // Extract activities from date-based results
        let mut activities = Vec::new();
        let date_pattern = Regex::new(r"^\d{4}-\d{2}-\d{2}$")
            .map_err(|e| CsCliError::Generic(format!("Date regex error: {e}")))?;

        if let Some(data_obj) = data.as_object() {
            for (key, value) in data_obj {
                if date_pattern.is_match(key) {
                    if let Some(activities_array) = value.as_array() {
                        for activity in activities_array {
                            if let Some(activity_obj) = activity.as_object() {
                                if let Some(activity_type) =
                                    activity_obj.get("type").and_then(|t| t.as_str())
                                {
                                    if ["EMAIL", "CALL"].contains(&activity_type) {
                                        let mut activity_with_date = activity.clone();
                                        if !activity_obj.contains_key("date") {
                                            if let Some(obj) = activity_with_date.as_object_mut() {
                                                obj.insert(
                                                    "date".to_string(),
                                                    Value::String(key.clone()),
                                                );
                                            }
                                        }
                                        activities.push(activity_with_date);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Parse activities with advanced filtering
        let mut calls = Vec::new();
        let mut email_activities = Vec::new();

        // Parse calls and collect email activities
        for activity in activities {
            if let Some(activity_obj) = activity.as_object() {
                if let Some(activity_type) = activity_obj.get("type").and_then(|t| t.as_str()) {
                    match activity_type {
                        "CALL" | "call" => {
                            if let Ok(Some(call)) = self.parse_call(&activity, account_id) {
                                if self.should_include_call(&call) {
                                    calls.push(call);
                                }
                            }
                        }
                        "EMAIL" | "email" => {
                            email_activities.push((activity, account_id.to_string()));
                        }
                        _ => {}
                    }
                }
            }
        }

        // Process emails with advanced filtering
        let emails = if !email_activities.is_empty() {
            let raw_emails = self.process_emails_with_filtering(email_activities)?;
            raw_emails.into_iter().filter(|e| !e.is_automated).collect()
        } else {
            Vec::new()
        };

        Ok((calls, emails))
    }

    /// Call filtering logic - match behavior of direct customer search
    pub fn should_include_call(&self, call: &Call) -> bool {
        // Include all calls with basic validity checks
        !call.id.is_empty() && !call.title.is_empty()
    }

    /// Process emails with advanced BDR/SPAM/Automation filtering
    /// This is the CORE filtering logic that removes noise
    pub fn process_emails_with_filtering(
        &self,
        email_activities: Vec<(Value, String)>,
    ) -> Result<Vec<Email>> {
        if email_activities.is_empty() {
            return Ok(Vec::new());
        }

        // Group activities by sender for context-aware processing
        let mut sender_groups: HashMap<String, Vec<(Value, String)>> = HashMap::new();
        for (activity, account_id) in email_activities {
            if let Some(sender_email) = self.extract_sender_email(&activity) {
                sender_groups
                    .entry(sender_email)
                    .or_default()
                    .push((activity, account_id));
            }
        }

        // Process each sender group with full similarity context
        let mut all_emails = Vec::new();
        for (sender_email, activities) in sender_groups {
            let sender_emails = self.process_sender_emails(activities, &sender_email)?;
            all_emails.extend(sender_emails);
        }

        // Apply additional filtering by synopsis to remove duplicates and mass emails
        if !all_emails.is_empty() {
            // Convert to dict format for filtering
            let email_dicts: Vec<HashMap<String, Value>> = all_emails
                .iter()
                .map(|email| {
                    let mut dict = HashMap::new();
                    dict.insert("id".to_string(), Value::String(email.id.clone()));
                    dict.insert("subject".to_string(), Value::String(email.subject.clone()));
                    dict.insert(
                        "snippet".to_string(),
                        Value::String(email.snippet.as_ref().unwrap_or(&String::new()).clone()),
                    );
                    dict.insert(
                        "sent_at".to_string(),
                        Value::String(email.sent_at.to_string()),
                    );

                    // Create sender object
                    let mut sender_obj = serde_json::Map::new();
                    sender_obj.insert(
                        "email".to_string(),
                        Value::String(email.sender.email.clone()),
                    );
                    sender_obj.insert(
                        "title".to_string(),
                        Value::String(email.sender.name.as_ref().unwrap_or(&String::new()).clone()),
                    );
                    dict.insert("sender".to_string(), Value::Object(sender_obj));

                    dict
                })
                .collect();

            // Apply advanced synopsis filtering
            let (filtered_dicts, _filter_stats) = self.filter_emails_by_synopsis(email_dicts)?;
            let filtered_ids: HashSet<String> = filtered_dicts
                .iter()
                .filter_map(|dict| {
                    dict.get("id")
                        .and_then(|id| id.as_str())
                        .map(|s| s.to_string())
                })
                .collect();

            let filtered_emails: Vec<Email> = all_emails
                .into_iter()
                .filter(|e| filtered_ids.contains(&e.id))
                .collect();

            info!(
                original = filtered_emails.len() + _filter_stats.get("total").unwrap_or(&0),
                filtered = filtered_emails.len(),
                removed = _filter_stats.get("total").unwrap_or(&0),
                "Email filtering applied"
            );

            return Ok(filtered_emails);
        }

        Ok(all_emails)
    }

    /// Extract sender email from activity data
    pub fn extract_sender_email(&self, activity: &Value) -> Option<String> {
        let extended_data = activity.get("extendedData")?;
        let from_data = extended_data
            .get("from")
            .or_else(|| extended_data.get("byPerson"))?;
        from_data
            .get("email")
            .and_then(|e| e.as_str())
            .map(|s| s.to_string())
    }

    /// Process all emails from a single sender with automation detection
    pub fn process_sender_emails(
        &self,
        activities: Vec<(Value, String)>,
        sender_email: &str,
    ) -> Result<Vec<Email>> {
        // Parse all emails from this sender
        let mut sender_email_data = Vec::new();
        for (activity, account_id) in activities {
            if let Ok(Some(email_data)) = self.parse_email_basic(&activity, &account_id) {
                sender_email_data.push(email_data);
            }
        }

        // Apply automation detection with similarity context
        let mut processed_emails = Vec::new();
        for (i, email_data) in sender_email_data.iter().enumerate() {
            let subject = &email_data.subject;
            let snippet = email_data.snippet.as_deref().unwrap_or("");

            // Use unified automation detection
            let empty_title = String::new();
            let sender_title = email_data.sender.name.as_ref().unwrap_or(&empty_title);
            let (is_automated, is_template) =
                self.is_automated_content(subject, snippet, sender_email, sender_title);

            // Check similarity against other emails from same sender
            let mut final_is_automated = is_automated;
            let mut _final_is_template = is_template;

            if !is_automated && sender_email_data.len() > 1 {
                for (j, other_email_data) in sender_email_data.iter().enumerate() {
                    if i != j {
                        // Avoid self-comparison
                        let subject_sim = self.similarity_score(subject, &other_email_data.subject);
                        let empty_snippet_ref = String::new();
                        let other_snippet = other_email_data
                            .snippet
                            .as_ref()
                            .unwrap_or(&empty_snippet_ref);
                        let snippet_sim = self.similarity_score(snippet, other_snippet);
                        let max_similarity = subject_sim.max(snippet_sim);

                        if max_similarity >= 0.95 {
                            // High similarity threshold
                            final_is_automated = true;
                            _final_is_template = true;
                            break;
                        }
                    }
                }
            }

            // Create Email object with automation flags
            let mut email = email_data.clone();
            email.is_automated = final_is_automated;
            email.is_template = _final_is_template;

            processed_emails.push(email);
        }

        Ok(processed_emails)
    }

    /// Unified automation and template detection
    /// Returns: (is_automated, is_template)
    pub fn is_automated_content(
        &self,
        subject: &str,
        snippet: &str,
        sender_email: &str,
        sender_title: &str,
    ) -> (bool, bool) {
        if subject.is_empty() && snippet.is_empty() {
            return (false, false);
        }

        // Check for specific filtered senders and roles (highest confidence)
        let sender_email_lower = sender_email.to_lowercase();
        let sender_title_lower = sender_title.to_lowercase();

        // Filter out sales@postman.com
        if sender_email_lower == "sales@postman.com" {
            debug!(sender_email = %sender_email, "Filtering out email from sales@postman.com");
            return (true, true);
        }

        // Filter out anyone with "Account Development" in their title
        if sender_title_lower.contains("account development") {
            debug!(
                sender_email = %sender_email,
                sender_title = %sender_title,
                "Filtering out email from Account Development role"
            );
            return (true, true);
        }

        // Check for known automated senders (highest confidence)
        let automated_domains = [
            "academy@postman.com",
            "help@postman.com",
            "noreply@",
            "no-reply@",
        ];
        if automated_domains
            .iter()
            .any(|domain| sender_email_lower.contains(domain))
        {
            return (true, true);
        }

        // Check for auto-reply patterns (high-confidence automation)
        let subject_lower = subject.to_lowercase();
        let auto_reply_patterns = [
            "automatic reply:",
            "out-of-office",
            "out of office",
            "ooo ",
            "paternity leave",
            "maternity leave",
        ];

        if auto_reply_patterns
            .iter()
            .any(|pattern| subject_lower.contains(pattern))
        {
            return (true, true);
        }

        // Check for template language markers
        let content_lower = format!("{subject} {snippet}").to_lowercase();
        let has_template_markers = TEMPLATE_MARKERS
            .iter()
            .any(|marker| content_lower.contains(marker));

        (has_template_markers, has_template_markers)
    }

    /// Calculate text similarity using efficient Jaccard similarity
    pub fn similarity_score(&self, text1: &str, text2: &str) -> f64 {
        if text1.is_empty() || text2.is_empty() {
            return 0.0;
        }

        // Use normalized text for better comparison - collect strings to avoid borrowing issues
        let words1: HashSet<String> = text1
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        let words2: HashSet<String> = text2
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        if words1.is_empty() || words2.is_empty() {
            return 0.0;
        }

        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();

        if union > 0 {
            intersection as f64 / union as f64
        } else {
            0.0
        }
    }

    /// Enhanced normalization to detect template variations with personalization
    pub fn normalize_synopsis(&self, text: &str) -> String {
        if text.is_empty() {
            return String::new();
        }

        let mut t = text.to_lowercase();

        // Use precompiled regex patterns for better performance
        t = self
            .regex_patterns
            .thread_prefix
            .replace_all(&t, "")
            .to_string();
        t = self.regex_patterns.url.replace_all(&t, " URL ").to_string();
        t = self
            .regex_patterns
            .greeting
            .replace_all(&t, " GREETING ")
            .to_string();
        t = self
            .regex_patterns
            .company_at
            .replace_all(&t, " at COMPANY with ")
            .to_string();
        t = self
            .regex_patterns
            .account_manager
            .replace_all(&t, " COMPANY account manager ")
            .to_string();
        t = self
            .regex_patterns
            .users_number
            .replace_all(&t, " NUM users ")
            .to_string();
        t = self
            .regex_patterns
            .with_users
            .replace_all(&t, " with NUM users ")
            .to_string();
        t = self
            .regex_patterns
            .big_numbers
            .replace_all(&t, " NUM ")
            .to_string();
        t = self
            .regex_patterns
            .name_is
            .replace_all(&t, " my name is NAME ")
            .to_string();
        t = self
            .regex_patterns
            .i_am
            .replace_all(&t, " i am COMPANY ")
            .to_string();
        t = self
            .regex_patterns
            .date_slash
            .replace_all(&t, " DATE ")
            .to_string();
        t = self
            .regex_patterns
            .date_month
            .replace_all(&t, " DATE ")
            .to_string();
        t = self
            .regex_patterns
            .whitespace
            .replace_all(&t, " ")
            .trim()
            .to_string();

        t
    }

    /// Filter emails based on synopsis to remove duplicates and mass emails
    /// Returns: (filtered_emails, filter_stats)
    pub fn filter_emails_by_synopsis(
        &self,
        emails: Vec<HashMap<String, Value>>,
    ) -> Result<EmailFilterResult> {
        if emails.is_empty() {
            return Ok((Vec::new(), HashMap::from([("total".to_string(), 0)])));
        }

        let mut filtered_emails = Vec::new();
        let mut similarity_filtered = 0;
        let mut template_filtered = 0;

        // Group emails by sender
        let mut sender_groups: HashMap<String, Vec<HashMap<String, Value>>> = HashMap::new();
        for email in emails {
            let sender_email = email
                .get("sender")
                .and_then(|s| s.get("email"))
                .and_then(|e| e.as_str())
                .unwrap_or("")
                .to_lowercase();
            sender_groups.entry(sender_email).or_default().push(email);
        }

        // Process each sender group
        for (sender_email, sender_emails) in sender_groups {
            // Check for high-volume template senders
            let template_count = sender_emails
                .iter()
                .filter(|email| {
                    let subject = email.get("subject").and_then(|s| s.as_str()).unwrap_or("");
                    let snippet = email.get("snippet").and_then(|s| s.as_str()).unwrap_or("");
                    let sender_title = email
                        .get("sender")
                        .and_then(|s| s.get("title"))
                        .and_then(|t| t.as_str())
                        .unwrap_or("");

                    self.is_automated_content(subject, snippet, &sender_email, sender_title)
                        .1 // Use is_template result
                })
                .count();

            let template_rate = if sender_emails.is_empty() {
                0.0
            } else {
                template_count as f64 / sender_emails.len() as f64
            };
            let is_high_template_sender = sender_emails.len() >= 5 && template_rate >= 0.7;

            if is_high_template_sender {
                // Keep only the best representative for template senders
                let representative = self.select_blast_representative(&sender_emails);
                filtered_emails.push(representative);
                template_filtered += sender_emails.len() - 1;
                continue;
            }

            // For normal senders, group by content similarity
            let email_groups = self.group_emails_by_content_similarity(&sender_emails, 0.85)?;

            for group in email_groups {
                if group.len() > 1 && self.is_blast_pattern(&group)? {
                    // Keep one representative for similar content blasts
                    let representative = self.select_blast_representative(&group);
                    filtered_emails.push(representative);
                    similarity_filtered += group.len() - 1;
                } else {
                    // Keep all emails for conversations and unique content
                    filtered_emails.extend(group);
                }
            }
        }

        // Create filter stats
        let total_filtered = similarity_filtered + template_filtered;
        let mut filter_stats = HashMap::new();
        filter_stats.insert("similarity".to_string(), similarity_filtered);
        filter_stats.insert("template_mass".to_string(), template_filtered);
        filter_stats.insert("total".to_string(), total_filtered);

        Ok((filtered_emails, filter_stats))
    }

    /// Group emails by content similarity to detect blast patterns
    pub fn group_emails_by_content_similarity(
        &self,
        emails: &[HashMap<String, Value>],
        threshold: f64,
    ) -> Result<Vec<Vec<HashMap<String, Value>>>> {
        if emails.is_empty() {
            return Ok(Vec::new());
        }

        let mut groups = Vec::new();
        let mut processed_indices = HashSet::new();

        for (i, email) in emails.iter().enumerate() {
            if processed_indices.contains(&i) {
                continue;
            }

            // Start a new group with this email
            let mut group = vec![email.clone()];
            processed_indices.insert(i);

            let email_content = format!(
                "{} {}",
                email.get("snippet").and_then(|s| s.as_str()).unwrap_or(""),
                email.get("subject").and_then(|s| s.as_str()).unwrap_or("")
            );
            let normalized_content = self.normalize_synopsis(&email_content);

            // Find similar emails
            for (j, other_email) in emails.iter().enumerate() {
                if processed_indices.contains(&j) || i == j {
                    continue;
                }

                let other_content = format!(
                    "{} {}",
                    other_email
                        .get("snippet")
                        .and_then(|s| s.as_str())
                        .unwrap_or(""),
                    other_email
                        .get("subject")
                        .and_then(|s| s.as_str())
                        .unwrap_or("")
                );
                let normalized_other = self.normalize_synopsis(&other_content);

                // Use normalized content for better template detection
                let similarity = self.similarity_score(&normalized_content, &normalized_other);
                if similarity >= threshold {
                    group.push(other_email.clone());
                    processed_indices.insert(j);
                }
            }

            groups.push(group);
        }

        Ok(groups)
    }

    /// Detect if email group represents a blast pattern
    pub fn is_blast_pattern(&self, email_group: &[HashMap<String, Value>]) -> Result<bool> {
        if email_group.len() <= 1 {
            return Ok(false);
        }

        // Parse timestamps
        let mut timestamps = Vec::new();
        for email in email_group {
            if let Some(sent_at_str) = email.get("sent_at").and_then(|s| s.as_str()) {
                if let Ok(sent_at) = Zoned::strptime("%Y-%m-%dT%H:%M:%S", sent_at_str) {
                    timestamps.push(sent_at);
                }
            }
        }

        if timestamps.len() >= 2 {
            // Blast pattern: emails sent within 24 hours
            let min_time = timestamps.iter().min().unwrap();
            let max_time = timestamps.iter().max().unwrap();
            let time_diff = max_time
                .since(min_time)
                .map_err(|e| CsCliError::Generic(format!("Time calculation error: {e}")))?
                .abs();
            let one_day_seconds = 24 * 60 * 60; // 24 hours in seconds
            return Ok(time_diff.get_seconds() <= one_day_seconds); // 24 hours
        }

        // Default: multiple similar emails likely indicate blast pattern
        Ok(email_group.len() >= 2)
    }

    /// Select the best representative email from a blast group
    pub fn select_blast_representative(
        &self,
        blast_group: &[HashMap<String, Value>],
    ) -> HashMap<String, Value> {
        if blast_group.len() == 1 {
            return blast_group[0].clone();
        }

        // Priority: Prefer emails with longer content (more context)
        blast_group
            .iter()
            .max_by_key(|email| {
                let subject_len = email
                    .get("subject")
                    .and_then(|s| s.as_str())
                    .map(|s| s.len())
                    .unwrap_or(0);
                let snippet_len = email
                    .get("snippet")
                    .and_then(|s| s.as_str())
                    .map(|s| s.len())
                    .unwrap_or(0);
                subject_len + snippet_len
            })
            .unwrap_or(&blast_group[0])
            .clone()
    }

    /// Parse call activity into Call model
    pub fn parse_call(&self, activity: &Value, account_id: &str) -> Result<Option<Call>> {
        let Some(activity_obj) = activity.as_object() else {
            return Ok(None);
        };

        let empty_extended_data = serde_json::Map::new();
        let extended_data = activity_obj
            .get("extendedData")
            .and_then(|d| d.as_object())
            .unwrap_or(&empty_extended_data);

        // Debug: Log available fields to identify correct call ID
        debug!(
            activity_keys = ?activity_obj.keys().collect::<Vec<_>>(),
            extended_data_keys = ?extended_data.keys().collect::<Vec<_>>(),
            activity_id = ?activity_obj.get("id"),
            "Timeline call activity fields"
        );

        // Convert epoch time to datetime with fallbacks
        let scheduled_start =
            if let Some(epoch_time) = activity_obj.get("epochTime").and_then(|t| t.as_i64()) {
                Timestamp::from_second(epoch_time)
                    .map(|ts| ts.to_zoned(jiff::tz::TimeZone::system()))
                    .ok()
            } else {
                // Fallback to effectiveDateTime or date field
                let date_str = activity_obj
                    .get("effectiveDateTime")
                    .or_else(|| activity_obj.get("date"))
                    .and_then(|d| d.as_str());

                if let Some(date_str) = date_str {
                    Zoned::strptime("%Y-%m-%d", date_str).ok()
                } else {
                    None
                }
            };

        debug!(
            epoch_time = ?activity_obj.get("epochTime"),
            effective_date = ?activity_obj.get("effectiveDateTime"),
            scheduled_start = ?scheduled_start,
            "Timeline date extraction"
        );

        // Parse participants
        let mut participants = Vec::new();
        let mut participant_emails = HashSet::new();

        // Add detailed participant from extendedData.byPerson if available
        if let Some(by_person) = extended_data.get("byPerson").and_then(|p| p.as_object()) {
            if let Some(email) = by_person.get("email").and_then(|e| e.as_str()) {
                let participant = CallParticipant {
                    id: None,
                    name: by_person
                        .get("name")
                        .and_then(|n| n.as_str())
                        .unwrap_or(email.split('@').next().unwrap_or("Unknown"))
                        .to_string(),
                    email: Some(email.to_string()),
                    phone: by_person
                        .get("phone")
                        .and_then(|p| p.as_str())
                        .map(|s| s.to_string()),
                    title: by_person
                        .get("title")
                        .and_then(|t| t.as_str())
                        .map(|s| s.to_string()),
                    company: by_person
                        .get("companyName")
                        .and_then(|c| c.as_str())
                        .map(|s| s.to_string()),
                    is_internal: false,
                    speaking_time: None,
                    talk_ratio: None,
                };
                participants.push(participant);
                participant_emails.insert(email.to_string());
            }
        }

        // Add remaining participants from email list
        if let Some(participants_list) = activity_obj
            .get("participantsEmailList")
            .and_then(|p| p.as_array())
        {
            for participant_email in participants_list {
                if let Some(email) = participant_email.as_str() {
                    if !participant_emails.contains(email) {
                        let participant = CallParticipant {
                            id: None,
                            name: email.split('@').next().unwrap_or("Unknown").to_string(),
                            email: Some(email.to_string()),
                            phone: None,
                            title: None,
                            company: None,
                            is_internal: false,
                            speaking_time: None,
                            talk_ratio: None,
                        };
                        participants.push(participant);
                        participant_emails.insert(email.to_string());
                    }
                }
            }
        }

        // Determine direction
        let direction = activity_obj
            .get("direction")
            .and_then(|d| d.as_str())
            .map(|d| match d.to_uppercase().as_str() {
                "INBOUND" => CallDirection::Inbound,
                "OUTBOUND" => CallDirection::Outbound,
                "INTERNAL" => CallDirection::Internal,
                _ => CallDirection::Unknown,
            })
            .unwrap_or(CallDirection::Unknown);

        // Try multiple fields for call ID like customer search does
        let call_id = activity_obj
            .get("id")
            .or_else(|| activity_obj.get("callId"))
            .or_else(|| activity_obj.get("call_id"))
            .or_else(|| extended_data.get("callId"))
            .or_else(|| extended_data.get("id"))
            .and_then(|id| id.as_str())
            .unwrap_or("")
            .to_string();

        debug!(
            activity_id = ?activity_obj.get("id"),
            extracted_call_id = %call_id,
            activity_type = ?activity_obj.get("type"),
            "Timeline call ID extraction"
        );

        if call_id.is_empty() {
            return Ok(None);
        }

        let mut call = Call::new(
            call_id,
            account_id.to_string(),
            extended_data
                .get("title")
                .or_else(|| extended_data.get("contentTitle"))
                .and_then(|t| t.as_str())
                .unwrap_or("Call")
                .to_string(),
            direction,
            extended_data
                .get("duration")
                .and_then(|d| d.as_i64())
                .unwrap_or(0) as i32,
            scheduled_start.unwrap_or_else(Zoned::now),
        );

        // Set generated_title if available
        call.generated_title = extended_data
            .get("generatedTitle")
            .or_else(|| activity_obj.get("generatedTitle"))
            .and_then(|t| t.as_str())
            .map(|s| s.to_string());

        // Set customer_name if available
        call.customer_name = extended_data
            .get("customerName")
            .or_else(|| activity_obj.get("customerName"))
            .or_else(|| extended_data.get("accountName"))
            .and_then(|t| t.as_str())
            .map(|s| s.to_string());

        // Add participants to the call
        call.participants = participants;

        Ok(Some(call))
    }

    /// Parse basic email data from activity
    pub fn parse_email_basic(&self, activity: &Value, account_id: &str) -> Result<Option<Email>> {
        let Some(activity_obj) = activity.as_object() else {
            return Ok(None);
        };

        let empty_email_extended_data = serde_json::Map::new();
        let extended_data = activity_obj
            .get("extendedData")
            .and_then(|d| d.as_object())
            .unwrap_or(&empty_email_extended_data);

        // Convert epoch time to datetime
        let sent_at =
            if let Some(epoch_time) = activity_obj.get("epochTime").and_then(|t| t.as_i64()) {
                Timestamp::from_second(epoch_time)
                    .map(|ts| ts.to_zoned(jiff::tz::TimeZone::system()))
                    .unwrap_or_else(|_| Zoned::now())
            } else {
                Zoned::now()
            };

        // Parse sender
        let empty_from_data = serde_json::Map::new();
        let from_data = extended_data
            .get("from")
            .or_else(|| extended_data.get("byPerson"))
            .and_then(|d| d.as_object())
            .unwrap_or(&empty_from_data);

        let sender_email = from_data
            .get("email")
            .and_then(|e| e.as_str())
            .unwrap_or("unknown@example.com")
            .to_string();

        let sender_name = from_data
            .get("name")
            .and_then(|n| n.as_str())
            .map(|s| s.to_string());

        let sender_is_internal = sender_email.to_lowercase().contains("postman.com");

        // Parse recipients
        let mut recipients = Vec::new();
        if let Some(to_list) = extended_data.get("to").and_then(|t| t.as_array()) {
            for recipient_data in to_list {
                if let Some(recipient_obj) = recipient_data.as_object() {
                    let recipient = EmailRecipient {
                        email: recipient_obj
                            .get("email")
                            .and_then(|e| e.as_str())
                            .unwrap_or("")
                            .to_string(),
                        name: recipient_obj
                            .get("name")
                            .and_then(|n| n.as_str())
                            .map(|s| s.to_string()),
                        recipient_type: "to".to_string(),
                        is_internal: recipient_obj
                            .get("email")
                            .and_then(|e| e.as_str())
                            .map(|e| e.to_lowercase().contains("postman.com"))
                            .unwrap_or(false),
                        title: recipient_obj
                            .get("title")
                            .and_then(|t| t.as_str())
                            .map(|s| s.to_string()),
                        company: recipient_obj
                            .get("company")
                            .and_then(|c| c.as_str())
                            .map(|s| s.to_string()),
                    };
                    recipients.push(recipient);
                }
            }
        } else {
            // Fallback to participantsEmailList
            if let Some(participants_emails) = activity_obj
                .get("participantsEmailList")
                .and_then(|p| p.as_array())
            {
                for email_val in participants_emails {
                    if let Some(email_addr) = email_val.as_str() {
                        if email_addr != sender_email {
                            let recipient = EmailRecipient {
                                email: email_addr.to_string(),
                                name: Some(
                                    email_addr
                                        .split('@')
                                        .next()
                                        .unwrap_or("Unknown")
                                        .to_string(),
                                ),
                                recipient_type: "to".to_string(),
                                is_internal: email_addr.to_lowercase().contains("postman.com"),
                                title: None,
                                company: None,
                            };
                            recipients.push(recipient);
                        }
                    }
                }
            }
        }

        // Determine direction
        let direction = if sender_is_internal {
            EmailDirection::Outbound
        } else if recipients.iter().any(|r| r.is_internal) {
            EmailDirection::Inbound
        } else {
            EmailDirection::Internal
        };

        // Extract subject and snippet
        let subject = extended_data
            .get("subject")
            .or_else(|| extended_data.get("contentTitle"))
            .and_then(|s| s.as_str())
            .unwrap_or("No Subject")
            .to_string();

        let snippet = extended_data
            .get("synopsis")
            .or_else(|| extended_data.get("categoryPassiveVoice"))
            .and_then(|s| s.as_str())
            .map(|s| s.to_string());

        // Get the correct account ID from the activity
        let email_account_id = activity_obj
            .get("accountId")
            .and_then(|id| id.as_str())
            .unwrap_or(account_id);

        let email_id = activity_obj
            .get("id")
            .and_then(|id| id.as_str())
            .unwrap_or("")
            .to_string();

        if email_id.is_empty() {
            return Ok(None);
        }

        let mut email = Email::new(
            email_id,
            email_account_id.to_string(),
            subject,
            direction,
            sent_at,
            sender_email,
        );

        // Set additional fields that were extracted
        email.sender.name = sender_name;
        email.sender.is_internal = sender_is_internal;
        email.snippet = snippet;
        email.recipients = recipients;

        Ok(Some(email))
    }
}
