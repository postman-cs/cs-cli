use super::CallDirection;
use jiff::Zoned;
use serde::{Deserialize, Serialize};

/// Call participant details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallParticipant {
    /// Participant ID
    pub id: Option<String>,

    /// Participant name
    pub name: String,

    /// Email address
    pub email: Option<String>,

    /// Phone number
    pub phone: Option<String>,

    /// Job title
    pub title: Option<String>,

    /// Company affiliation
    pub company: Option<String>,

    /// Whether this is an internal participant
    pub is_internal: bool,

    /// Speaking time in seconds
    pub speaking_time: Option<f64>,

    /// Talk ratio (0.0 to 1.0)
    pub talk_ratio: Option<f64>,
}

/// Call communication model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Call {
    /// Call ID
    #[serde(alias = "callId")]
    pub id: String,

    /// Account ID
    #[serde(alias = "accountId")]
    pub account_id: String,

    /// Call title
    pub title: String,

    /// Generated title from API (used for filename generation)
    #[serde(alias = "generatedTitle")]
    pub generated_title: Option<String>,

    /// Customer name (resolved from text-suggestions or retrieved)
    #[serde(alias = "customerName")]
    pub customer_name: Option<String>,

    /// Call direction
    pub direction: CallDirection,

    /// Duration in seconds
    pub duration: i32,

    /// Scheduled start time
    #[serde(alias = "scheduledStart")]
    pub scheduled_start: Zoned,

    /// Actual start time
    #[serde(alias = "actualStart")]
    pub actual_start: Option<Zoned>,

    /// Recording URL
    #[serde(alias = "recordingUrl")]
    pub recording_url: Option<String>,

    /// Transcript URL
    #[serde(alias = "transcriptUrl")]
    pub transcript_url: Option<String>,

    /// Call brief/summary
    #[serde(alias = "callBrief")]
    pub call_brief: Option<String>,

    // Recording status indicators
    /// Call status (e.g., "COMPLETED")
    pub status: Option<String>,

    /// Call type (e.g., "WEB_CONFERENCE")
    pub call_type: Option<String>,

    // Participants
    /// Call participants
    #[serde(default)]
    pub participants: Vec<CallParticipant>,

    /// Host ID
    #[serde(alias = "hostId")]
    pub host_id: Option<String>,

    /// Host name
    #[serde(alias = "hostName")]
    pub host_name: Option<String>,

    // Analytics
    /// Sentiment score
    pub sentiment: Option<f64>,

    /// Overall talk ratio
    pub talk_ratio: Option<f64>,

    /// Longest monologue duration in seconds
    pub longest_monologue: Option<i32>,

    /// Interactivity score
    pub interactivity: Option<f64>,

    /// Number of questions asked
    pub questions_asked: Option<i32>,

    // Content
    /// Call transcript
    pub transcript: Option<String>,

    /// Call summary
    pub summary: Option<String>,

    /// Topics discussed
    #[serde(default)]
    pub topics: Vec<String>,

    /// Action items identified
    #[serde(default)]
    pub action_items: Vec<String>,
}

impl Call {
    /// Create a new call with minimal required fields
    pub fn new(
        id: String,
        account_id: String,
        title: String,
        direction: CallDirection,
        duration: i32,
        scheduled_start: Zoned,
    ) -> Self {
        Self {
            id,
            account_id,
            title,
            generated_title: None,
            customer_name: None,
            direction,
            duration,
            scheduled_start,
            actual_start: None,
            recording_url: None,
            transcript_url: None,
            call_brief: None,
            status: None,
            call_type: None,
            participants: Vec::new(),
            host_id: None,
            host_name: None,
            sentiment: None,
            talk_ratio: None,
            longest_monologue: None,
            interactivity: None,
            questions_asked: None,
            transcript: None,
            summary: None,
            topics: Vec::new(),
            action_items: Vec::new(),
        }
    }

    /// Check if the call has a recording
    pub fn has_recording(&self) -> bool {
        self.recording_url.is_some()
    }

    /// Check if the call has a transcript
    pub fn has_transcript(&self) -> bool {
        self.transcript_url.is_some() || self.transcript.is_some()
    }

    /// Get the actual call duration (from actual_start to end)
    pub fn actual_duration(&self) -> Option<jiff::Span> {
        self.actual_start
            .as_ref()
            .map(|_start| jiff::Span::new().seconds(self.duration as i64))
    }

    /// Check if this is an internal call
    pub fn is_internal(&self) -> bool {
        matches!(self.direction, CallDirection::Internal)
    }

    /// Get external participants (non-internal)
    pub fn external_participants(&self) -> Vec<&CallParticipant> {
        self.participants
            .iter()
            .filter(|p| !p.is_internal)
            .collect()
    }

    /// Get internal participants
    pub fn internal_participants(&self) -> Vec<&CallParticipant> {
        self.participants.iter().filter(|p| p.is_internal).collect()
    }
}
