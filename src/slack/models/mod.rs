//! Slack data models
//!
//! Simple data structures for Slack API responses

use serde::{Deserialize, Serialize};

/// Basic Slack message representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackMessage {
    pub ts: String,
    pub user: Option<String>,
    pub text: Option<String>,
    pub channel: Option<String>,
    pub thread_ts: Option<String>,
}

/// Basic Slack channel/conversation representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackConversation {
    pub id: String,
    pub name: Option<String>,
    pub is_channel: Option<bool>,
    pub is_private: Option<bool>,
    pub is_im: Option<bool>,
    pub num_members: Option<u32>,
}
