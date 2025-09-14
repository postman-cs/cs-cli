use super::{Call, CommunicationType, Email};
use jiff::Zoned;
use serde::{Deserialize, Serialize};

/// Generic communication wrapper for unified handling of calls and emails
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Communication {
    /// Type of communication
    #[serde(rename = "type")]
    pub communication_type: CommunicationType,

    /// Communication ID
    pub id: String,

    /// Account ID
    pub account_id: String,

    /// When the communication occurred
    pub occurred_at: Zoned,

    /// Number of participants
    pub participants_count: usize,

    // Type-specific data
    /// Call data (if type is Call)
    pub call: Option<Call>,

    /// Email data (if type is Email)
    pub email: Option<Email>,

    // Common fields
    /// Communication title
    pub title: String,

    /// Summary of the communication
    pub summary: Option<String>,

    /// Sentiment score
    pub sentiment: Option<f64>,

    /// Whether this is an internal communication
    #[serde(default)]
    pub is_internal: bool,

    // Caching metadata
    /// When this record was fetched
    #[serde(default = "zoned_now")]
    pub fetched_at: Zoned,

    /// Whether full content has been fetched
    #[serde(default)]
    pub enhanced: bool,
}

fn zoned_now() -> Zoned {
    Zoned::now()
}

impl Communication {
    /// Create a Communication from a Call
    pub fn from_call(call: Call) -> Self {
        let participants_count = call.participants.len();
        let is_internal = call.is_internal();
        let title = call.title.clone();
        let summary = call.summary.clone();
        let sentiment = call.sentiment;
        let id = call.id.clone();
        let account_id = call.account_id.clone();
        let occurred_at = call.scheduled_start.clone();

        Self {
            communication_type: CommunicationType::Call,
            id,
            account_id,
            occurred_at,
            participants_count,
            call: Some(call),
            email: None,
            title,
            summary,
            sentiment,
            is_internal,
            fetched_at: Zoned::now(),
            enhanced: false,
        }
    }

    /// Create a Communication from an Email
    pub fn from_email(email: Email) -> Self {
        let participants_count = email.recipients.len() + 1; // Recipients + sender
        let is_internal = email.is_internal();
        let title = email.subject.clone();
        let summary = email.summary.clone();
        let sentiment = email.sentiment;
        let id = email.id.clone();
        let account_id = email.account_id.clone();
        let occurred_at = email.sent_at.clone();

        Self {
            communication_type: CommunicationType::Email,
            id,
            account_id,
            occurred_at,
            participants_count,
            call: None,
            email: Some(email),
            title,
            summary,
            sentiment,
            is_internal,
            fetched_at: Zoned::now(),
            enhanced: false,
        }
    }

    /// Check if this is a call
    pub fn is_call(&self) -> bool {
        matches!(self.communication_type, CommunicationType::Call)
    }

    /// Check if this is an email
    pub fn is_email(&self) -> bool {
        matches!(self.communication_type, CommunicationType::Email)
    }

    /// Get the duration if this is a call
    pub fn duration(&self) -> Option<i32> {
        self.call.as_ref().map(|c| c.duration)
    }

    /// Get the underlying Call if this is a call communication
    pub fn as_call(&self) -> Option<&Call> {
        self.call.as_ref()
    }

    /// Get the underlying Email if this is an email communication
    pub fn as_email(&self) -> Option<&Email> {
        self.email.as_ref()
    }

    /// Check if content has been fully fetched
    pub fn is_enhanced(&self) -> bool {
        self.enhanced
    }

    /// Mark as enhanced after fetching full content
    pub fn mark_enhanced(&mut self) {
        self.enhanced = true;
        self.fetched_at = Zoned::now();
    }
}
