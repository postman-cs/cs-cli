//! Common communication types
//!
//! Shared communication type enums that can be used by both Gong and Slack integrations.

use serde::{Deserialize, Serialize};

/// Communication types - extensible for multiple platforms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommunicationType {
    Call,
    Email,
    Meeting,
    Chat,
    Sms,
    // Future: can add Slack-specific types like Thread, DirectMessage, etc.
}

/// Call direction types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum CallDirection {
    Inbound,
    Outbound,
    Internal,
    Unknown,
}

/// Email direction types  
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EmailDirection {
    Inbound,
    Outbound,
    Internal,
}

/// Message direction types - generic for any messaging platform
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageDirection {
    Inbound,
    Outbound,
    Internal,
}

impl CommunicationType {
    /// Check if this communication type supports direction
    pub fn supports_direction(&self) -> bool {
        matches!(self, CommunicationType::Call | CommunicationType::Email | CommunicationType::Chat)
    }
    
    /// Check if this is a real-time communication type
    pub fn is_realtime(&self) -> bool {
        matches!(self, CommunicationType::Call | CommunicationType::Meeting | CommunicationType::Chat)
    }
}
