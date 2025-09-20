use super::EmailDirection;
use jiff::Zoned;
use serde::{Deserialize, Serialize};

fn default_recipient_type() -> String {
    "to".to_string()
}

/// Email recipient details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailRecipient {
    /// Recipient email address
    pub email: String,

    /// Recipient name
    pub name: Option<String>,

    /// Recipient type (to, cc, bcc, from)
    #[serde(default = "default_recipient_type", rename = "type")]
    pub recipient_type: String,

    /// Whether this is an internal recipient
    #[serde(default)]
    pub is_internal: bool,

    /// Job title
    pub title: Option<String>,

    /// Company affiliation
    pub company: Option<String>,
}

/// Email communication model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email {
    /// Email ID
    #[serde(alias = "emailId")]
    pub id: String,

    /// Account ID
    #[serde(alias = "accountId")]
    pub account_id: String,

    /// Email subject
    pub subject: String,

    /// Email direction
    pub direction: EmailDirection,

    /// Email sent timestamp
    #[serde(alias = "sentAt")]
    pub sent_at: Zoned,

    /// Email received timestamp (for inbound emails)
    #[serde(alias = "receivedAt")]
    pub received_at: Option<Zoned>,

    // Sender information
    /// Sender information
    pub sender: EmailRecipient,

    // Recipients
    /// Email recipients
    #[serde(default)]
    pub recipients: Vec<EmailRecipient>,

    // Content
    /// Email body (plain text)
    #[serde(alias = "body")]
    pub body_text: Option<String>,

    /// Email body (HTML)
    pub html_body: Option<String>,

    /// Email snippet/preview
    pub snippet: Option<String>,

    /// Email thread ID
    #[serde(alias = "threadId")]
    pub thread_id: Option<String>,

    /// Whether email is part of a thread
    pub in_thread: bool,

    // Analytics and metadata
    /// Email sentiment score
    pub sentiment: Option<f64>,

    /// Email summary/brief
    pub summary: Option<String>,

    /// Email topics
    #[serde(default)]
    pub topics: Vec<String>,

    /// Action items from email
    #[serde(default)]
    pub action_items: Vec<String>,

    /// Whether this email has attachments
    pub has_attachments: bool,

    /// Number of attachments
    pub attachment_count: i32,

    // Filtering metadata for BDR/SPAM detection
    /// Whether this email is likely automated
    #[serde(default)]
    pub is_automated: bool,

    /// Whether this is a template email
    #[serde(default)]
    pub is_template: bool,

    /// Template ID if this is a template email
    pub template_id: Option<String>,

    /// Campaign ID if part of a campaign
    pub campaign_id: Option<String>,

    /// Sequence ID if part of a sequence
    pub sequence_id: Option<String>,

    /// Number of times email was opened
    #[serde(default)]
    pub open_count: i32,

    /// Number of links clicked
    #[serde(default)]
    pub click_count: i32,

    /// Number of replies
    #[serde(default)]
    pub reply_count: i32,

    /// Whether this is likely a blast email
    pub is_blast: Option<bool>,

    /// Similarity score to other emails (for deduplication)
    pub similarity_score: Option<f64>,

    /// Email bounce status
    pub bounce_status: Option<String>,

    /// Email status (sent, delivered, opened, etc.)
    pub status: Option<String>,

    // Enhancement metadata
    /// Whether body content has been retrieved
    pub body_retrieved: bool,

    /// When this email record was last updated
    pub updated_at: Option<Zoned>,

    /// Vector embeddings for search
    pub embedding: Option<Vec<f32>>,
}

impl Email {
    /// Create a new email with minimal required fields
    pub fn new(
        id: String,
        account_id: String,
        subject: String,
        direction: EmailDirection,
        sent_at: Zoned,
        sender_email: String,
    ) -> Self {
        let sender = EmailRecipient {
            email: sender_email,
            name: None,
            recipient_type: "from".to_string(),
            is_internal: false,
            title: None,
            company: None,
        };

        Self {
            id,
            account_id,
            subject,
            direction,
            sent_at,
            received_at: None,
            sender,
            recipients: Vec::new(),
            body_text: None,
            html_body: None,
            snippet: None,
            thread_id: None,
            in_thread: false,
            sentiment: None,
            summary: None,
            topics: Vec::new(),
            action_items: Vec::new(),
            has_attachments: false,
            attachment_count: 0,
            is_automated: false,
            is_template: false,
            template_id: None,
            campaign_id: None,
            sequence_id: None,
            open_count: 0,
            click_count: 0,
            reply_count: 0,
            is_blast: None,
            similarity_score: None,
            bounce_status: None,
            status: None,
            body_retrieved: false,
            updated_at: None,
            embedding: None,
        }
    }

    /// Check if the email is inbound
    pub fn is_inbound(&self) -> bool {
        matches!(self.direction, EmailDirection::Inbound)
    }

    /// Check if the email is outbound
    pub fn is_outbound(&self) -> bool {
        matches!(self.direction, EmailDirection::Outbound)
    }

    /// Check if the email is internal
    pub fn is_internal(&self) -> bool {
        matches!(self.direction, EmailDirection::Internal)
    }

    /// Get external recipients (non-internal)
    pub fn external_recipients(&self) -> Vec<&EmailRecipient> {
        self.recipients.iter().filter(|r| !r.is_internal).collect()
    }

    /// Get internal recipients
    pub fn internal_recipients(&self) -> Vec<&EmailRecipient> {
        self.recipients.iter().filter(|r| r.is_internal).collect()
    }

    /// Check if email has content (body or HTML)
    pub fn has_content(&self) -> bool {
        self.body_text.is_some() || self.html_body.is_some()
    }

    /// Get the best available content (prefer plain text over HTML)
    pub fn best_content(&self) -> Option<&str> {
        self.body_text.as_deref().or(self.html_body.as_deref())
    }

    /// Check if this email should be filtered (automated/blast/template)
    pub fn should_filter(&self) -> bool {
        self.is_automated || self.is_template || self.is_blast.unwrap_or(false)
    }

    /// Get primary recipient (first TO recipient)
    pub fn primary_recipient(&self) -> Option<&EmailRecipient> {
        self.recipients
            .iter()
            .find(|r| r.recipient_type.to_lowercase() == "to")
    }
}
