//! Common time and date range utilities
//!
//! Shared date/time handling that can be used by both Gong and Slack integrations.

use jiff::{civil::Date, Span, Zoned};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Date range for API retrieval with chunking support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalRange {
    /// Start date for retrieval
    pub start_date: Date,
    /// End date for retrieval
    pub end_date: Date,
    /// Maximum chunk size in days (for API rate limiting)
    pub chunk_days: i32,
}

impl RetrievalRange {
    /// Create a new retrieval range
    pub fn new(start_date: Date, end_date: Date, chunk_days: i32) -> Self {
        Self {
            start_date,
            end_date,
            chunk_days,
        }
    }

    /// Create range for the last N days
    pub fn last_days(days: i32) -> crate::Result<Self> {
        let now = Zoned::now();
        let end_date = now.date();
        let span = Span::new().days(days as i64);
        let start_date = end_date.saturating_sub(span);

        Ok(Self::new(start_date, end_date, 30)) // Default 30-day chunks
    }

    /// Create range for a specific month
    pub fn for_month(year: i16, month: i8) -> crate::Result<Self> {
        let start_date = Date::new(year, month, 1)
            .map_err(|e| crate::CsCliError::Generic(format!("Invalid date: {e}")))?;
        let end_date = start_date.last_of_month();

        Ok(Self::new(start_date, end_date, 30))
    }

    /// Split the range into smaller chunks for API requests
    pub fn chunk_by_days(&self) -> Vec<(Date, Date)> {
        let mut chunks = Vec::new();
        let mut current_start = self.start_date;

        while current_start <= self.end_date {
            let chunk_span = Span::new().days(self.chunk_days as i64);
            let chunk_end = std::cmp::min(current_start.saturating_add(chunk_span), self.end_date);

            chunks.push((current_start, chunk_end));

            // Move to next chunk (add 1 day to avoid overlap)
            let day_span = Span::new().days(1);
            current_start = chunk_end.saturating_add(day_span);

            // Break if we've reached the end to avoid infinite loop
            if chunk_end >= self.end_date {
                break;
            }
        }

        chunks
    }

    /// Convert to API parameters for day-activities endpoint (matching Python)
    pub fn to_api_params(&self, account_id: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("id".to_string(), account_id.to_string());
        params.insert("type".to_string(), "ACCOUNT".to_string());
        params.insert("day-from".to_string(), self.start_date.to_string());
        params.insert("day-to".to_string(), self.end_date.to_string());
        params
    }

    /// Get the total number of days in the range
    pub fn total_days(&self) -> i32 {
        (self.end_date - self.start_date).get_days() + 1
    }

    /// Split range into chunks (matching Python to_chunks method)
    pub fn to_chunks(&self, days: i32) -> Vec<RetrievalRange> {
        let mut chunks = Vec::new();
        let mut current = self.start_date;

        while current < self.end_date {
            let chunk_span = Span::new().days(days as i64);
            let chunk_end = std::cmp::min(current.saturating_add(chunk_span), self.end_date);

            chunks.push(RetrievalRange {
                start_date: current,
                end_date: chunk_end,
                chunk_days: days,
            });

            current = chunk_end;
        }

        chunks
    }
}
