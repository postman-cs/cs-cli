//! Rate limiting utilities with jitter
//!
//! Provides intelligent delays and jitter to avoid anti-spam detection

use rand::Rng;
use std::time::Duration;
use tokio::time::sleep;

/// Add jitter to avoid predictable request patterns
pub async fn sleep_with_jitter(base_ms: u64, jitter_percent: f64) {
    let mut rng = rand::thread_rng();

    // Calculate jitter range
    let jitter_range = (base_ms as f64 * jitter_percent) as u64;
    let min_delay = base_ms.saturating_sub(jitter_range / 2);
    let max_delay = base_ms + (jitter_range / 2);

    // Generate random delay within range
    let actual_delay = rng.gen_range(min_delay..=max_delay);

    sleep(Duration::from_millis(actual_delay)).await;
}

/// Conservative delay for search requests with jitter
pub async fn search_delay() {
    // Base 1.5 seconds with ±30% jitter (1.05s - 1.95s)
    sleep_with_jitter(1500, 0.3).await;
}

/// Conservative delay for API requests with jitter  
pub async fn api_delay() {
    // Base 1 second with ±25% jitter (0.75s - 1.25s)
    sleep_with_jitter(1000, 0.25).await;
}

/// Longer delay for channel exploration with jitter
pub async fn channel_delay() {
    // Base 2 seconds with ±20% jitter (1.6s - 2.4s)
    sleep_with_jitter(2000, 0.2).await;
}

/// Burst protection - longer delay after multiple rapid requests
pub async fn burst_protection_delay() {
    // Base 5 seconds with ±10% jitter (4.5s - 5.5s)
    sleep_with_jitter(5000, 0.1).await;
}
