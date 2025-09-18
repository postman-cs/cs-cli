use console::style;
use inquire::{Select, Text};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::gong::api::client::HttpClientPool;
use crate::gong::auth::GongAuthenticator;
use crate::gong::config::AppConfig;
use crate::{CsCliError, Result};

/// Helper function to capitalize first letter of a string
fn capitalize_first_letter(s: &str) -> String {
    if s.is_empty() {
        return s.to_string();
    }

    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
    }
}

/// Customer search result data
#[derive(Debug, Clone)]
pub struct CustomerSearchResult {
    /// Customer/company name
    pub name: String,
    /// Account ID from Gong
    pub id: Option<String>,
    /// Raw API response data for debugging
    pub raw_data: Value,
}

/// Call information extracted from customer search
#[derive(Debug, Clone)]
pub struct CustomerCallInfo {
    /// Call ID
    pub id: Option<String>,
    /// Account ID associated with the call
    pub account_id: Option<String>,
    /// Call title
    pub title: String,
    /// Generated title from Gong
    pub generated_title: String,
    /// Customer name associated with the call
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

/// Result of getting customer calls with pagination
#[derive(Debug)]
pub struct CustomerCallsResult {
    /// List of calls
    pub calls: Vec<CustomerCallInfo>,
    /// Whether more results are available
    pub has_more: bool,
    /// Total count (if available)
    pub total_count: usize,
    /// Company names matched
    pub companies: Vec<String>,
    /// Account IDs found
    pub account_ids: Vec<String>,
}

/// Gong Customer Search API client for finding calls by customer name
pub struct GongCustomerSearchClient {
    /// HTTP client pool for making requests
    http_client: Arc<HttpClientPool>,
    /// Authentication manager
    auth: Arc<GongAuthenticator>,
    /// Application configuration
    _config: Option<AppConfig>,
    /// Workspace ID for API requests
    workspace_id: String,
}

impl GongCustomerSearchClient {
    /// Create a new customer search client
    pub fn new(
        http_client: Arc<HttpClientPool>,
        auth: Arc<GongAuthenticator>,
        config: Option<AppConfig>,
    ) -> Result<Self> {
        // Get workspace ID from authenticator (must be real, not default)
        let workspace_id = auth
            .get_workspace_id()
            .ok_or_else(|| {
                CsCliError::Configuration(
                    "No workspace ID available - authentication may not be complete".to_string(),
                )
            })?
            .to_string();

        info!(workspace_id = %workspace_id, "Using workspace ID for customer search");

        Ok(Self {
            http_client,
            auth,
            _config: config,
            workspace_id,
        })
    }

    /// Search for customers using autocomplete API with case-insensitive fallback
    ///
    /// # Arguments
    /// * `partial_name` - Partial customer/company name to search for
    ///
    /// # Returns
    /// Vector of customer search results with company names and account IDs
    pub async fn search_customers(&self, partial_name: &str) -> Result<Vec<CustomerSearchResult>> {
        // Try original case first, then fallback to other cases if no results
        let search_variations = vec![
            partial_name.to_string(),              // Original case
            partial_name.to_lowercase(),           // Lowercase
            capitalize_first_letter(partial_name), // Capitalized
            partial_name.to_uppercase(),           // Uppercase
        ];

        // Remove duplicates
        let mut unique_variations = Vec::new();
        for variation in search_variations {
            if !unique_variations.contains(&variation) {
                unique_variations.push(variation);
            }
        }

        // Try each variation until we get results
        for (i, variation) in unique_variations.iter().enumerate() {
            info!(
                partial_name = %partial_name,
                variation = %variation,
                attempt = i + 1,
                "Searching for customers with case variation"
            );

            let results = self.search_customers_exact_case(variation).await?;
            if !results.is_empty() {
                info!(
                    partial_name = %partial_name,
                    variation = %variation,
                    results_count = results.len(),
                    "Found customers with case variation"
                );
                return Ok(results);
            }
        }

        info!(partial_name = %partial_name, "No results found with any case variation");
        Ok(Vec::new())
    }

    /// Search for customers using exact case (internal method)
    async fn search_customers_exact_case(
        &self,
        partial_name: &str,
    ) -> Result<Vec<CustomerSearchResult>> {
        let base_url = self.auth.get_base_url()?;
        let url = format!("{base_url}/conversations/ajax/text-filter-suggestions");

        let mut params = HashMap::new();
        params.insert("workspace-id".to_string(), self.workspace_id.clone());
        params.insert(
            "filter-name".to_string(),
            "LeadCompanyOrAccount".to_string(),
        );
        params.insert("partial-text".to_string(), partial_name.to_string());

        // Get authenticated headers (matching Python exactly)
        let mut headers = self.auth.get_authenticated_headers(true).await?;

        // Add required headers for autocomplete requests (matching Python exactly)
        headers.insert(
            "accept".to_string(),
            "application/json; charset=utf-8".to_string(),
        );
        headers.insert("sec-fetch-dest".to_string(), "empty".to_string());
        headers.insert("sec-fetch-mode".to_string(), "cors".to_string());
        headers.insert("sec-fetch-site".to_string(), "same-origin".to_string());

        // Update headers on HTTP client
        self.http_client.update_headers(headers).await?;

        // CRITICAL: Also set the session cookies on the HTTP client!
        // The authenticator has the cookies but they're not automatically transferred
        let cookies = self.auth.get_session_cookies()?;
        debug!(
            "Setting {} session cookies on HTTP client for customer search",
            cookies.len()
        );
        self.http_client.set_cookies(cookies).await?;

        // Build URL with query parameters
        let query_string = params
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");
        let full_url = format!("{url}?{query_string}");

        let response = self.http_client.get(&full_url).await?;

        let status_code = response.status().as_u16();

        if status_code == 401 {
            let _auth_state = self.auth.get_auth_state();
        }

        if response.status().is_success() {
            let response_text = response
                .text()
                .await
                .map_err(|e| CsCliError::ApiRequest(format!("Failed to read response: {e}")))?;

            // Use sonic-rs for fast JSON parsing
            let data: Value = serde_json::from_str(&response_text)
                .map_err(|e| CsCliError::ApiRequest(format!("Failed to parse JSON: {e}")))?;

            // Extract suggestion data including account IDs
            let empty_suggestions = Vec::new();
            let suggestions = data
                .get("suggestions")
                .and_then(|s| s.as_array())
                .unwrap_or(&empty_suggestions);

            debug!(
                "Customer search API returned {} suggestions for '{}'",
                suggestions.len(),
                partial_name
            );

            if suggestions.is_empty() {
                debug!(
                    "No suggestions found, full response structure: {:?}",
                    data.as_object().map(|o| o.keys().collect::<Vec<_>>())
                );
            } else {
                for (i, suggestion) in suggestions.iter().take(3).enumerate() {
                    debug!("Suggestion {}: {:?}", i + 1, suggestion);
                }
            }

            let mut customer_results = Vec::new();
            for item in suggestions {
                if let Some(obj) = item.as_object() {
                    let name = obj
                        .get("text")
                        .and_then(|t| t.as_str())
                        .unwrap_or("")
                        .to_string();

                    let id = obj
                        .get("id")
                        .or_else(|| obj.get("value"))
                        .or_else(|| obj.get("accountId"))
                        .and_then(|i| i.as_str())
                        .map(|s| s.to_string());

                    customer_results.push(CustomerSearchResult {
                        name,
                        id,
                        raw_data: item.clone(),
                    });
                }
            }

            Ok(customer_results)
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
                "Customer search failed"
            );
            Ok(Vec::new())
        }
    }

    /// Resolve customer name to list of possible company names and account IDs
    ///
    /// # Arguments
    /// * `customer_name` - Customer name to resolve
    ///
    /// # Returns
    /// Tuple of (company_names, account_ids) that can be used in filtering
    pub async fn resolve_customer_companies(
        &self,
        customer_name: &str,
    ) -> Result<(Vec<String>, Vec<String>)> {
        // Search for customer - API returns full data with IDs
        let customer_data = self.search_customers(customer_name).await?;

        if customer_data.is_empty() {
            warn!(customer_name = %customer_name, "No results found for customer");
            return Ok((Vec::new(), Vec::new()));
        }

        // Find best matches - exact match first, then partial matches
        let mut exact_matches = Vec::new();
        for customer in &customer_data {
            // First try exact case-insensitive match
            if customer.name.eq_ignore_ascii_case(customer_name) {
                exact_matches.push(customer);
            }
        }

        // If no exact matches, fall back to substring matching (like Python)
        if exact_matches.is_empty() {
            for customer in &customer_data {
                if customer
                    .name
                    .to_lowercase()
                    .contains(&customer_name.to_lowercase())
                {
                    exact_matches.push(customer);
                }
            }
        }

        // If we have exact matches, prefer those, otherwise use all suggestions
        let final_matches = if exact_matches.is_empty() {
            customer_data.iter().collect()
        } else {
            exact_matches
        };

        // Extract company names and account IDs
        let company_names: Vec<String> = final_matches
            .iter()
            .map(|c| c.name.clone())
            .filter(|name| !name.is_empty())
            .collect();

        let account_ids: Vec<String> = final_matches
            .iter()
            .filter_map(|c| c.id.as_ref())
            .cloned()
            .collect();

        info!(
            customer_name = %customer_name,
            company_count = company_names.len(),
            account_id_count = account_ids.len(),
            "Resolved customer to companies with account IDs"
        );

        Ok((company_names, account_ids))
    }

    /// Prompt user to select the correct company when multiple matches are found
    ///
    /// # Arguments
    /// * `customer_name` - Original customer name searched for
    /// * `company_names` - List of matching company names
    ///
    /// # Returns
    /// Selected company name, "SEARCH_AGAIN" to search again, or None if cancelled
    pub async fn select_customer_company(
        &self,
        customer_name: &str,
        company_names: &[String],
    ) -> Result<Option<String>> {
        if company_names.is_empty() {
            return Ok(None);
        }

        if company_names.len() == 1 {
            // Only one match, use it automatically
            let selected = &company_names[0];
            println!(
                "\n{} {}",
                style("Found customer:").green().bold(),
                style(selected).white().bold()
            );
            return Ok(Some(selected.clone()));
        }

        // Multiple matches - show selection
        println!(
            "\n{} {} {} {}",
            style("I found").yellow(),
            style(company_names.len()).yellow().bold(),
            style("companies matching").yellow(),
            style(format!("'{customer_name}'")).yellow().bold()
        );
        println!("{}", style("Which one are you looking for?").dim());

        // Prepare options for selection
        let display_count = std::cmp::min(company_names.len(), 10);
        let mut options: Vec<String> = company_names[..display_count].to_vec();

        if company_names.len() > 10 {
            println!(
                "\n{}",
                style(format!(
                    "Showing first 10 of {} matches",
                    company_names.len()
                ))
                .dim()
            );
        }

        // Add special options
        options.push("None of these - search again".to_string());
        options.push("Cancel and exit".to_string());

        // Create selection dialog
        let selection = Select::new("Select a company", options.clone())
            .with_starting_cursor(0)
            .prompt()
            .map_err(|e| CsCliError::Generic(format!("Selection failed: {e}")))?;

        if selection == "Cancel and exit" {
            // User selected "Cancel and exit"
            println!(
                "\n{}",
                style("Cancelled - no files will be extracted.").yellow()
            );
            Ok(None)
        } else if selection == "None of these - search again" {
            // User selected "None of these - search again"
            Ok(Some("SEARCH_AGAIN".to_string()))
        } else if company_names.contains(&selection) {
            // User selected a company
            println!(
                "\n{} {}\n",
                style(" Selected:").green().bold(),
                style(&selection).white().bold()
            );
            Ok(Some(selection))
        } else {
            Err(CsCliError::Generic("Invalid selection".to_string()))
        }
    }

    /// Get calls filtered by customer name with pagination
    ///
    /// # Arguments
    /// * `customer_name` - Customer name to filter by
    /// * `page_size` - Number of calls per page (default 10, same as Gong UI)
    /// * `calls_offset` - Offset for pagination
    /// * `interactive` - Whether to prompt for selection when multiple matches found
    ///
    /// # Returns
    /// Result containing calls data and pagination info
    pub async fn get_customer_calls(
        &self,
        customer_name: &str,
        page_size: usize,
        calls_offset: usize,
        interactive: bool,
    ) -> Result<CustomerCallsResult> {
        // First resolve customer name to company names and account IDs
        let (mut company_names, mut account_ids) =
            self.resolve_customer_companies(customer_name).await?;

        if company_names.is_empty() {
            warn!(customer_name = %customer_name, "Could not resolve customer name");
            println!(
                "{} {}",
                style("No customers found matching").red().bold(),
                style(format!("'{customer_name}'")).red().bold()
            );
            return Ok(CustomerCallsResult {
                calls: Vec::new(),
                has_more: false,
                total_count: 0,
                companies: Vec::new(),
                account_ids: Vec::new(),
            });
        }

        // Handle interactive selection if enabled
        if interactive {
            #[allow(non_snake_case)]
            match self
                .select_customer_company(customer_name, &company_names)
                .await?
            {
                None => {
                    // User cancelled
                    return Ok(CustomerCallsResult {
                        calls: Vec::new(),
                        has_more: false,
                        total_count: 0,
                        companies: Vec::new(),
                        account_ids: Vec::new(),
                    });
                }
                Some(selected) if selected == "SEARCH_AGAIN" => {
                    // User wants to search for a different customer
                    println!("\n{}", style("Let's try a different search.").cyan());
                    let new_customer: String = Text::new("Enter the customer name")
                        .prompt()
                        .map_err(|e| CsCliError::Generic(format!("Input failed: {e}")))?;
                    return Box::pin(self.get_customer_calls(
                        &new_customer,
                        page_size,
                        calls_offset,
                        interactive,
                    ))
                    .await;
                }
                Some(selected_company) => {
                    // Use the selected company - need to find its account ID
                    if let Some(selected_index) = company_names
                        .iter()
                        .position(|name| name == &selected_company)
                    {
                        company_names = vec![selected_company];
                        // Keep only the corresponding account ID
                        if selected_index < account_ids.len() {
                            account_ids = vec![account_ids[selected_index].clone()];
                        }
                    }
                }
            }
        }

        // Build the search filter payload using serde_json
        let search_filter = serde_json::json!({
            "search": {
                "type": "And",
                "filters": [{
                    "type": "LeadCompanyOrAccount",
                    "names": company_names
                }]
            }
        });

        let payload = serde_json::json!({
            "pageSize": page_size,
            "callsOffset": calls_offset,
            "callsSearchJson": serde_json::to_string(&search_filter)
                .map_err(|e| CsCliError::Generic(format!("JSON serialization failed: {e}")))?
        });

        let base_url = self.auth.get_base_url()?;
        let url = format!("{base_url}/conversations/ajax/results");

        // Show progress to user
        if calls_offset == 0 {
            println!("{}", style("Downloading calls and emails...").dim());
        }

        info!(
            customer = %customer_name,
            companies = ?company_names,
            page_size = page_size,
            offset = calls_offset,
            "Fetching customer calls"
        );

        // Get authenticated headers
        let mut headers = self.auth.get_authenticated_headers(true).await?;

        // Add required headers for POST requests
        headers.insert(
            "accept".to_string(),
            "application/json; charset=utf-8".to_string(),
        );
        headers.insert("content-type".to_string(), "application/json".to_string());
        headers.insert("sec-fetch-dest".to_string(), "empty".to_string());
        headers.insert("sec-fetch-mode".to_string(), "cors".to_string());
        headers.insert("sec-fetch-site".to_string(), "same-origin".to_string());

        // Update headers on HTTP client
        self.http_client.update_headers(headers).await?;

        // CRITICAL: Also set the session cookies on the HTTP client!
        let cookies = self.auth.get_session_cookies()?;
        self.http_client.set_cookies(cookies).await?;

        // Build query parameters
        let query_params = [("workspace-id", self.workspace_id.as_str())];
        let query_string = query_params
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");
        let full_url = format!("{url}?{query_string}");

        // Convert payload to JSON string for POST body
        let json_body = serde_json::to_string(&payload)
            .map_err(|e| CsCliError::ApiRequest(format!("Failed to serialize payload: {e}")))?;

        let response = self.http_client.post(&full_url, Some(&json_body)).await?;

        if response.status().is_success() {
            let response_text = response
                .text()
                .await
                .map_err(|e| CsCliError::ApiRequest(format!("Failed to read response: {e}")))?;

            // Use sonic-rs for fast JSON parsing
            let data: Value = serde_json::from_str(&response_text)
                .map_err(|e| CsCliError::ApiRequest(format!("Failed to parse JSON: {e}")))?;

            // Extract calls from response
            let calls = self.extract_calls_from_response(&data)?;

            info!(
                customer = %customer_name,
                calls_retrieved = calls.len(),
                "Successfully retrieved calls"
            );

            // Extract unique account IDs from the calls we just retrieved
            let mut extracted_account_ids = std::collections::HashSet::new();
            for call in &calls {
                if let Some(account_id) = &call.account_id {
                    extracted_account_ids.insert(account_id.clone());
                }
            }

            // Use extracted account IDs if we found any, otherwise fall back to what we had
            let final_account_ids = if !extracted_account_ids.is_empty() {
                info!(
                    account_ids_extracted = extracted_account_ids.len(),
                    "Extracted unique account IDs from calls"
                );
                extracted_account_ids.into_iter().collect()
            } else {
                account_ids
            };

            Ok(CustomerCallsResult {
                calls: calls.clone(),
                has_more: calls.len() == page_size, // Assume more if we got a full page
                total_count: calls.len(),           // This might be available in response
                companies: company_names,
                account_ids: final_account_ids,
            })
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
                "Customer calls API request failed"
            );

            Ok(CustomerCallsResult {
                calls: Vec::new(),
                has_more: false,
                total_count: 0,
                companies: Vec::new(),
                account_ids: Vec::new(),
            })
        }
    }

    /// Extract call data from the conversations API response
    ///
    /// # Arguments
    /// * `api_response` - Raw API response data
    ///
    /// # Returns
    /// Vector of call info structures
    pub fn extract_calls_from_response(
        &self,
        api_response: &Value,
    ) -> Result<Vec<CustomerCallInfo>> {
        let mut calls = Vec::new();

        // Check for the actual response structure from Gong customer search API
        let calls_data = if let Some(stream) = api_response.get("callItemsStream") {
            debug!(items = "callItemsStream", "Found call items in response");
            stream
        } else if let Some(items) = api_response.get("items") {
            debug!(items = "items", "Found items in response");
            items
        } else if let Some(calls) = api_response.get("calls") {
            debug!(items = "calls", "Found calls in response");
            calls
        } else if let Some(results) = api_response.get("results") {
            debug!(items = "results", "Found results in response");
            results
        } else if let Some(data) = api_response.get("data") {
            debug!(items = "data", "Found data in response");
            data
        } else {
            // Try to find calls in the response structure
            if let Some(obj) = api_response.as_object() {
                debug!(keys = ?obj.keys().collect::<Vec<_>>(), "Exploring API response structure");
            }
            return Ok(Vec::new());
        };

        let empty_calls = Vec::new();
        let calls_array = calls_data.as_array().unwrap_or(&empty_calls);

        for item in calls_array {
            if let Some(item_obj) = item.as_object() {
                // Extract title and customer info
                let title = item_obj
                    .get("title")
                    .or_else(|| item_obj.get("name"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
                    .to_string();

                let mut customer_name = item_obj
                    .get("customerAccountName")
                    .or_else(|| item_obj.get("accountName"))
                    .or_else(|| item_obj.get("customer"))
                    .and_then(|c| c.as_str())
                    .unwrap_or("")
                    .to_string();

                // Extract account ID from crmData > accounts > gongId
                let mut account_id: Option<String> = None;

                // Check for customer info and account ID in CRM data
                if let Some(crm_data) = item_obj.get("crmData") {
                    if let Some(crm_obj) = crm_data.as_object() {
                        // Look for accounts array
                        if let Some(accounts) = crm_obj.get("accounts") {
                            if let Some(accounts_array) = accounts.as_array() {
                                if let Some(first_account) = accounts_array.first() {
                                    if let Some(account_obj) = first_account.as_object() {
                                        // Extract the gongId from the account
                                        account_id = account_obj
                                            .get("gongId")
                                            .and_then(|id| id.as_str())
                                            .map(|s| s.to_string());

                                        if customer_name.is_empty() {
                                            customer_name = account_obj
                                                .get("name")
                                                .and_then(|n| n.as_str())
                                                .unwrap_or("")
                                                .to_string();
                                        }
                                    }
                                }
                            }
                        }

                        // Fallback to other CRM data fields
                        if customer_name.is_empty() {
                            customer_name = crm_obj
                                .get("accountName")
                                .or_else(|| crm_obj.get("companyName"))
                                .and_then(|n| n.as_str())
                                .unwrap_or("")
                                .to_string();
                        }
                    }
                }

                // If still no customer name, extract from title
                if customer_name.is_empty() && !title.is_empty() {
                    // Look for patterns like "CustomerName - meeting" or "Meeting with CustomerName"
                    if title.contains(" - ") {
                        if let Some(potential_customer) = title.split(" - ").next() {
                            let potential_customer = potential_customer.trim();
                            // Only use if it looks like a company name
                            if potential_customer.len() > 3
                                && !["call", "meeting", "sync", "demo"]
                                    .contains(&potential_customer.to_lowercase().as_str())
                            {
                                customer_name = potential_customer.to_string();
                            }
                        }
                    } else if title.to_lowercase().contains("with ") {
                        // Try to extract after "with"
                        let lowercase_title = title.to_lowercase();
                        let parts: Vec<&str> = lowercase_title.split("with ").collect();
                        if parts.len() > 1 {
                            if let Some(word) = parts[1].split_whitespace().next() {
                                if word.len() > 3 {
                                    customer_name = word.to_string();
                                }
                            }
                        }
                    }
                }

                // Extract participants
                let participants = item_obj
                    .get("participants")
                    .or_else(|| item_obj.get("attendees"))
                    .and_then(|p| p.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|participant| {
                                participant
                                    .as_str()
                                    .or_else(|| participant.get("name").and_then(|n| n.as_str()))
                            })
                            .map(|s| s.to_string())
                            .collect()
                    })
                    .unwrap_or_else(Vec::new);

                let call_info = CustomerCallInfo {
                    id: item_obj
                        .get("id")
                        .or_else(|| item_obj.get("callId"))
                        .or_else(|| item_obj.get("call_id"))
                        .and_then(|id| id.as_str())
                        .map(|s| s.to_string()),
                    account_id,
                    title,
                    generated_title: item_obj
                        .get("generatedTitle")
                        .and_then(|t| t.as_str())
                        .unwrap_or("")
                        .to_string(),
                    customer_name,
                    date: item_obj
                        .get("effectiveStartDateTime")
                        .or_else(|| item_obj.get("startDate"))
                        .or_else(|| item_obj.get("date"))
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
                        .or_else(|| item_obj.get("url"))
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
                        "Found customer call"
                    );
                    calls.push(call_info);
                } else {
                    debug!(item_keys = ?item_obj.keys().collect::<Vec<_>>(), "Skipping item without valid ID");
                }
            }
        }

        debug!(
            extracted_calls = calls.len(),
            "Extracted calls from customer search response"
        );
        Ok(calls)
    }
}
