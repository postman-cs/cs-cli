//! Slack workspace explorer
//!
//! Tool for exploring and analyzing Slack workspace structure, conversations,
//! and customer-related content to understand integration opportunities.

use crate::common::auth::SessionManager;
use crate::common::http::{BrowserHttpClient, HttpClient, search_delay, channel_delay};
use crate::common::config::HttpSettings;
use crate::slack::auth::{SlackAuth, SlackSession};
use crate::slack::models::{SlackConversation, SlackMessage};
use crate::{CsCliError, Result};
use serde_json::Value;
use std::collections::{HashMap, BTreeMap};
use tracing::{info, warn};

/// Slack workspace explorer and analyzer
pub struct SlackExplorer {
    auth: SlackAuth,
    http_client: Option<BrowserHttpClient>,
    session: Option<SlackSession>,
}

impl SlackExplorer {
    /// Create new Slack explorer
    pub fn new(workspace_domain: String) -> Self {
        let auth = SlackAuth::new(workspace_domain);
        
        Self {
            auth,
            http_client: None,
            session: None,
        }
    }

    /// Initialize authentication and HTTP client
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing Slack explorer...");
        
        // Authenticate
        self.auth.authenticate().await?;
        
        // Create HTTP client matching the browser used for authentication
        let browser_to_impersonate = self.auth.detected_browser.as_deref().unwrap_or("firefox");
        
        let http_config = HttpSettings {
            pool_size: 1,
            max_concurrency_per_client: 3,
            timeout_seconds: 30.0,
            max_clients: Some(1),
            global_max_concurrency: Some(3),
            enable_http3: true,   // Use HTTP/3 for better performance and modern browser matching
            force_http3: false,   // Allow fallback to HTTP/2 if Slack doesn't support HTTP/3
            tls_version: None,
            impersonate_browser: browser_to_impersonate.to_string(),
        };
        
        info!("Explorer HTTP client impersonating: {}", browser_to_impersonate);
        
        let client = BrowserHttpClient::new(http_config).await?;
        
        // Set authentication data
        if let Some(session) = self.auth.get_session_data() {
            client.set_cookies(session.cookies.clone()).await?;
            client.set_headers(self.auth.get_auth_headers()).await?;
            self.session = Some(session.clone());
        }
        
        self.http_client = Some(client);
        
        info!("Slack explorer initialized successfully");
        Ok(())
    }

    /// Get basic workspace information
    pub async fn get_workspace_info(&self) -> Result<HashMap<String, String>> {
        let client = self.http_client.as_ref()
            .ok_or_else(|| CsCliError::Generic("HTTP client not initialized".to_string()))?;
            
        let session = self.session.as_ref()
            .ok_or_else(|| CsCliError::Authentication("No active session".to_string()))?;

        let form_data = format!("token={}", session.xoxc_token);
        
        let response = client.post(
            &format!("{}/api/auth.test", session.workspace_url),
            Some(&form_data)
        ).await?;

        let json: Value = response.json().await
            .map_err(|e| CsCliError::ApiRequest(format!("Failed to parse auth.test response: {}", e)))?;

        let mut info = HashMap::new();
        
        if let Some(obj) = json.as_object() {
            for (key, value) in obj.iter() {
                if let Some(str_val) = value.as_str() {
                    info.insert(key.clone(), str_val.to_string());
                }
            }
        }
        
        Ok(info)
    }

    /// Get all conversations in the workspace
    pub async fn get_all_conversations(&self) -> Result<Vec<SlackConversation>> {
        let client = self.http_client.as_ref()
            .ok_or_else(|| CsCliError::Generic("HTTP client not initialized".to_string()))?;
            
        let session = self.session.as_ref()
            .ok_or_else(|| CsCliError::Authentication("No active session".to_string()))?;

        // Use client.counts endpoint which gives us comprehensive conversation data
        let form_data = format!("token={}", session.xoxc_token);
        
        let response = client.post(
            &format!("{}/api/client.counts", session.workspace_url),
            Some(&form_data)
        ).await?;

        let json: Value = response.json().await
            .map_err(|e| CsCliError::ApiRequest(format!("Failed to parse conversations response: {}", e)))?;

        let mut conversations = Vec::new();
        
        if let Some(channels) = json.get("channels").and_then(|v| v.as_array()) {
            for channel in channels {
                if let Ok(conv) = serde_json::from_value::<SlackConversation>(channel.clone()) {
                    conversations.push(conv);
                }
            }
        }
        
        Ok(conversations)
    }

    /// Get messages from a specific conversation
    pub async fn get_conversation_messages(&self, channel_id: &str, limit: u32) -> Result<Vec<SlackMessage>> {
        let client = self.http_client.as_ref()
            .ok_or_else(|| CsCliError::Generic("HTTP client not initialized".to_string()))?;
            
        let session = self.session.as_ref()
            .ok_or_else(|| CsCliError::Authentication("No active session".to_string()))?;

        let form_data = format!("token={}&channel={}&limit={}", 
            session.xoxc_token, channel_id, limit);
        
        let response = client.post(
            &format!("{}/api/conversations.history", session.workspace_url),
            Some(&form_data)
        ).await?;

        let json: Value = response.json().await
            .map_err(|e| CsCliError::ApiRequest(format!("Failed to parse messages response: {}", e)))?;

        let mut messages = Vec::new();
        
        if let Some(msgs) = json.get("messages").and_then(|v| v.as_array()) {
            for msg in msgs {
                if let Ok(slack_msg) = serde_json::from_value::<SlackMessage>(msg.clone()) {
                    messages.push(slack_msg);
                }
            }
        }
        
        Ok(messages)
    }

    /// Search messages across the workspace
    pub async fn search_messages(&self, query: &str, count: u32) -> Result<Vec<Value>> {
        let client = self.http_client.as_ref()
            .ok_or_else(|| CsCliError::Generic("HTTP client not initialized".to_string()))?;
            
        let session = self.session.as_ref()
            .ok_or_else(|| CsCliError::Authentication("No active session".to_string()))?;

        let form_data = format!("token={}&query={}&count={}", 
            session.xoxc_token, urlencoding::encode(query), count);
        
        let response = client.post(
            &format!("{}/api/search.messages", session.workspace_url),
            Some(&form_data)
        ).await?;

        let json: Value = response.json().await
            .map_err(|e| CsCliError::ApiRequest(format!("Failed to parse search response: {}", e)))?;

        let mut messages = Vec::new();
        
        if let Some(search_results) = json.get("messages")
            .and_then(|v| v.get("matches"))
            .and_then(|v| v.as_array()) {
            messages.extend(search_results.iter().cloned());
        }
        
        Ok(messages)
    }

    /// Analyze conversation types and get statistics
    pub async fn analyze_conversations(&self) -> Result<ConversationAnalysis> {
        println!("Analyzing workspace conversations...");
        
        let conversations = self.get_all_conversations().await?;
        let mut analysis = ConversationAnalysis::new();
        
        for conv in &conversations {
            analysis.total_conversations += 1;
            
            // Categorize by type
            if conv.is_im.unwrap_or(false) {
                analysis.direct_messages += 1;
            } else if conv.is_private.unwrap_or(false) {
                analysis.private_channels += 1;
            } else {
                analysis.public_channels += 1;
            }
            
            // Analyze channel names for patterns
            if let Some(name) = &conv.name {
                let name_lower = name.to_lowercase();
                
                if name_lower.contains("customer") || name_lower.contains("client") || 
                   name_lower.contains("support") || name_lower.contains("success") {
                    analysis.customer_related_channels.push(conv.clone());
                }
                
                if name_lower.contains("engineering") || name_lower.contains("dev") || 
                   name_lower.contains("tech") {
                    analysis.engineering_channels.push(conv.clone());
                }
                
                if name_lower.contains("sales") || name_lower.contains("revenue") || 
                   name_lower.contains("business") {
                    analysis.sales_channels.push(conv.clone());
                }
                
                if name_lower.contains("general") || name_lower.contains("random") || 
                   name_lower.contains("announce") {
                    analysis.general_channels.push(conv.clone());
                }
            }
        }
        
        println!("   Analyzed {} conversations", analysis.total_conversations);
        println!("      📱 Direct messages: {}", analysis.direct_messages);
        println!("      🔒 Private channels: {}", analysis.private_channels);
        println!("      🌐 Public channels: {}", analysis.public_channels);
        println!("      👥 Customer-related: {}", analysis.customer_related_channels.len());
        println!("      🔧 Engineering: {}", analysis.engineering_channels.len());
        println!("      💰 Sales: {}", analysis.sales_channels.len());
        
        Ok(analysis)
    }

    /// Search for customer-related discussions
    pub async fn find_customer_stories(&self) -> Result<CustomerStoryAnalysis> {
        println!("Searching for customer stories and discussions...");
        
        let mut analysis = CustomerStoryAnalysis::new();
        
        // Search terms related to customers and issues
        let search_terms = vec![
            "customer issue",
            "customer problem", 
            "customer feedback",
            "bug report",
            "escalation",
            "customer complaint",
            "support ticket",
            "customer call",
            "customer demo",
            "feature request",
            "integration issue",
            "API problem",
            "customer onboarding",
            "customer success",
            "churn risk",
            "renewal",
            "contract",
            "enterprise customer",
            "premium customer",
            "customer meeting"
        ];
        
        for term in search_terms {
            println!("   Searching for: '{}'", term);
            
            match self.search_messages(term, 20).await {
                Ok(messages) => {
                    if !messages.is_empty() {
                        println!("      Found {} messages", messages.len());
                        analysis.search_results.insert(term.to_string(), messages);
                    } else {
                        println!("      📭 No messages found");
                    }
                }
                Err(e) => {
                    warn!("Failed to search for '{}': {}", term, e);
                }
            }
            
            // Conservative delay with jitter to avoid anti-spam triggers
            search_delay().await;
        }
        
        Ok(analysis)
    }

    /// Explore high-activity channels for customer content
    pub async fn explore_customer_channels(&self, analysis: &ConversationAnalysis) -> Result<Vec<ChannelExploration>> {
        println!("Exploring customer-related channels...");
        
        let mut explorations = Vec::new();
        
        // Focus on customer-related channels first
        for channel in &analysis.customer_related_channels {
            if let Some(name) = &channel.name {
                println!("   📱 Exploring #{}", name);
                
                let messages = match self.get_conversation_messages(&channel.id, 50).await {
                    Ok(msgs) => msgs,
                    Err(e) => {
                        warn!("Failed to get messages from {}: {}", name, e);
                        continue;
                    }
                };
                
                let message_count = messages.len();
                let topics = self.extract_topics_from_messages(&messages);
                let recent_messages: Vec<SlackMessage> = messages.into_iter().take(10).collect();
                
                let exploration = ChannelExploration {
                    channel: channel.clone(),
                    message_count,
                    recent_messages,
                    topics_discussed: topics,
                };
                
                println!("      {} recent messages", exploration.message_count);
                
                explorations.push(exploration);
                
                // Conservative delay with jitter to avoid session termination
                channel_delay().await;
            }
        }
        
        Ok(explorations)
    }

    /// Extract topics and keywords from messages
    fn extract_topics_from_messages(&self, messages: &[SlackMessage]) -> Vec<String> {
        let mut topics = HashMap::new();
        
        let customer_keywords = vec![
            "customer", "client", "user", "account", "enterprise", "support", 
            "issue", "problem", "bug", "feature", "request", "feedback",
            "demo", "onboarding", "integration", "API", "error", "failure",
            "escalation", "urgent", "critical", "outage", "downtime",
            "success", "renewal", "churn", "contract", "deal", "sales"
        ];
        
        for message in messages {
            if let Some(text) = &message.text {
                let text_lower = text.to_lowercase();
                
                for keyword in &customer_keywords {
                    if text_lower.contains(keyword) {
                        *topics.entry(keyword.to_string()).or_insert(0) += 1;
                    }
                }
            }
        }
        
        // Return top topics sorted by frequency
        let mut topic_vec: Vec<(String, usize)> = topics.into_iter().collect();
        topic_vec.sort_by(|a, b| b.1.cmp(&a.1));
        
        topic_vec.into_iter()
            .take(10)
            .map(|(topic, count)| format!("{} ({})", topic, count))
            .collect()
    }

    /// Get detected browser for debugging
    pub fn get_detected_browser(&self) -> Option<&String> {
        self.auth.detected_browser.as_ref()
    }

    /// Complete workspace exploration
    pub async fn explore_workspace(&mut self) -> Result<WorkspaceExploration> {
        println!("Starting comprehensive Slack workspace exploration");
        println!("{}", "=".repeat(60));
        
        // Initialize
        self.initialize().await?;
        
        // Get workspace info
        let workspace_info = self.get_workspace_info().await?;
        println!("📍 Workspace: {}", workspace_info.get("team").unwrap_or(&"Unknown".to_string()));
        println!("👤 User: {}", workspace_info.get("user").unwrap_or(&"Unknown".to_string()));
        println!();
        
        // Analyze conversations
        let conversation_analysis = self.analyze_conversations().await?;
        println!();
        
        // Search for customer stories
        let customer_stories = self.find_customer_stories().await?;
        println!();
        
        // Explore customer channels in detail
        let channel_explorations = self.explore_customer_channels(&conversation_analysis).await?;
        println!();
        
        Ok(WorkspaceExploration {
            workspace_info,
            conversation_analysis,
            customer_stories,
            channel_explorations,
        })
    }
}

/// Analysis of conversation types and patterns
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConversationAnalysis {
    pub total_conversations: usize,
    pub direct_messages: usize,
    pub private_channels: usize,
    pub public_channels: usize,
    pub customer_related_channels: Vec<SlackConversation>,
    pub engineering_channels: Vec<SlackConversation>,
    pub sales_channels: Vec<SlackConversation>,
    pub general_channels: Vec<SlackConversation>,
}

impl ConversationAnalysis {
    fn new() -> Self {
        Self {
            total_conversations: 0,
            direct_messages: 0,
            private_channels: 0,
            public_channels: 0,
            customer_related_channels: Vec::new(),
            engineering_channels: Vec::new(),
            sales_channels: Vec::new(),
            general_channels: Vec::new(),
        }
    }
}

/// Analysis of customer-related content and stories
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CustomerStoryAnalysis {
    pub search_results: BTreeMap<String, Vec<Value>>,
    pub total_customer_messages: usize,
    pub common_issues: Vec<String>,
    pub escalations_found: usize,
}

impl CustomerStoryAnalysis {
    fn new() -> Self {
        Self {
            search_results: BTreeMap::new(),
            total_customer_messages: 0,
            common_issues: Vec::new(),
            escalations_found: 0,
        }
    }
}

/// Detailed exploration of a specific channel
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChannelExploration {
    pub channel: SlackConversation,
    pub message_count: usize,
    pub recent_messages: Vec<SlackMessage>,
    pub topics_discussed: Vec<String>,
}

/// Complete workspace exploration results
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkspaceExploration {
    pub workspace_info: HashMap<String, String>,
    pub conversation_analysis: ConversationAnalysis,
    pub customer_stories: CustomerStoryAnalysis,
    pub channel_explorations: Vec<ChannelExploration>,
}
