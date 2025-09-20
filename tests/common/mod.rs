//! Common test utilities and configuration
//!
//! This module provides shared test helpers, fixtures, and configuration
//! for the regression test suite.

use cs_cli::common::config::{AuthSettings, HttpSettings};
use cs_cli::gong::models::{Call, Email, CallParticipant, EmailRecipient};
use jiff::Zoned;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test environment configuration
pub struct TestConfig {
    /// Customer name for testing
    pub customer_name: String,
    /// Number of days to look back
    pub days_back: u32,
    /// Whether to use real API (from env var)
    pub use_real_api: bool,
    /// Test data directory
    pub fixtures_dir: PathBuf,
    /// Temporary output directory
    pub temp_dir: Option<TempDir>,
}

impl TestConfig {
    /// Create test configuration from environment
    pub fn from_env() -> Self {
        Self {
            customer_name: env::var("TEST_CUSTOMER_NAME")
                .unwrap_or_else(|_| "TestCustomer".to_string()),
            days_back: env::var("TEST_DAYS_BACK")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            use_real_api: true, // Always use real API for production-ready tests
            fixtures_dir: PathBuf::from("tests/fixtures"),
            temp_dir: None,
        }
    }

    /// Create test configuration with temporary directory
    pub fn with_temp_dir() -> Self {
        let mut config = Self::from_env();
        config.temp_dir = Some(TempDir::new().expect("Failed to create temp dir"));
        config
    }

    /// Get output directory path
    pub fn output_dir(&self) -> PathBuf {
        if let Some(ref temp) = self.temp_dir {
            temp.path().to_path_buf()
        } else {
            PathBuf::from("tests/output")
        }
    }
}

/// Test data fixtures
pub struct TestFixtures;

impl TestFixtures {
    /// Create a sample Call for testing
    pub fn sample_call() -> Call {
        Call {
            id: "test-call-123".to_string(),
            account_id: "account-456".to_string(),
            title: "Test Call - Project Discussion".to_string(),
            generated_title: Some("AI Generated Title".to_string()),
            customer_name: Some("Test Customer".to_string()),
            direction: cs_cli::gong::models::CallDirection::Outbound,
            duration: 1800, // 30 minutes
            scheduled_start: Zoned::now(),
            actual_start: Some(Zoned::now()),
            recording_url: Some("https://gong.io/recording/123".to_string()),
            transcript_url: Some("https://gong.io/transcript/123".to_string()),
            call_brief: Some("Discussion about project timeline and requirements".to_string()),
            status: Some("completed".to_string()),
            call_type: Some("customer_call".to_string()),
            participants: vec![
                CallParticipant {
                    id: Some("part-1".to_string()),
                    name: "John Doe".to_string(),
                    email: Some("john@customer.com".to_string()),
                    phone: None,
                    title: Some("Engineering Manager".to_string()),
                    company: Some("Customer Inc".to_string()),
                    is_internal: false,
                    speaking_time: Some(900),
                    talk_ratio: Some(0.5),
                },
                CallParticipant {
                    id: Some("part-2".to_string()),
                    name: "Jane Smith".to_string(),
                    email: Some("jane@ourcompany.com".to_string()),
                    phone: None,
                    title: Some("Customer Success Manager".to_string()),
                    company: Some("Our Company".to_string()),
                    is_internal: true,
                    speaking_time: Some(900),
                    talk_ratio: Some(0.5),
                },
            ],
            transcript: None,
            summary: None,
            action_items: vec![],
            keywords: vec![],
            topics: vec![],
            questions: vec![],
            owner_id: None,
            owner_name: None,
            workspace_id: Some("ws-789".to_string()),
            media_url: None,
            metadata: HashMap::new(),
            source: None,
        }
    }

    /// Create a sample Email for testing
    pub fn sample_email() -> Email {
        use cs_cli::gong::models::EmailDirection;

        Email {
            id: "email-456".to_string(),
            account_id: "account-456".to_string(),
            subject: "Re: Project Update".to_string(),
            direction: EmailDirection::Inbound,
            sent_at: Zoned::now(),
            from: EmailRecipient {
                email: "john@customer.com".to_string(),
                name: Some("John Doe".to_string()),
                domain: Some("customer.com".to_string()),
            },
            to: vec![EmailRecipient {
                email: "jane@ourcompany.com".to_string(),
                name: Some("Jane Smith".to_string()),
                domain: Some("ourcompany.com".to_string()),
            }],
            cc: vec![],
            bcc: vec![],
            html_content: Some("<p>Here's the latest update on the project...</p>".to_string()),
            text_content: Some("Here's the latest update on the project...".to_string()),
            attachments: vec![],
            thread_id: Some("thread-789".to_string()),
            message_id: Some("msg-123".to_string()),
            in_reply_to: Some("msg-122".to_string()),
            references: vec!["msg-120".to_string(), "msg-121".to_string()],
            workspace_id: Some("ws-789".to_string()),
            metadata: HashMap::new(),
        }
    }

    /// Create sample HTML content for testing conversion
    pub fn sample_html_email() -> String {
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <style>
                body { font-family: Arial, sans-serif; }
                .important { color: red; font-weight: bold; }
            </style>
        </head>
        <body>
            <h1>Project Update</h1>
            <p>Hello team,</p>
            <p>Here are the key updates:</p>
            <ul>
                <li>Feature A is <span class="important">complete</span></li>
                <li>Feature B is in progress</li>
                <li>Feature C is scheduled for next week</li>
            </ul>
            <p>Please review and let me know if you have questions.</p>
            <p>Best regards,<br>John</p>
            <script>console.log('tracking');</script>
        </body>
        </html>
        "#.to_string()
    }

    /// Create a sample CSRF token response
    pub fn sample_csrf_response() -> String {
        r#"{
            "csrf_token": "test-csrf-token-123456",
            "user": {
                "id": "user-123",
                "email": "test@example.com"
            },
            "workspace": {
                "id": "ws-789",
                "name": "Test Workspace"
            }
        }"#.to_string()
    }

    /// Create sample cookie data
    pub fn sample_cookies() -> Vec<cs_cli::common::auth::Cookie> {
        vec![
            cs_cli::common::auth::Cookie {
                name: "gong_session".to_string(),
                value: "test-session-value".to_string(),
                domain: Some(".gong.io".to_string()),
                path: Some("/".to_string()),
                expires: Some(1735689600.0), // Future date
                http_only: true,
                secure: true,
                same_site: Some("Lax".to_string()),
            },
            cs_cli::common::auth::Cookie {
                name: "gong_cell".to_string(),
                value: "us-14496".to_string(),
                domain: Some(".gong.io".to_string()),
                path: Some("/".to_string()),
                expires: None,
                http_only: false,
                secure: true,
                same_site: None,
            },
        ]
    }
}

/// Test assertion helpers
pub mod assertions {
    use super::*;

    /// Assert that a Call has valid structure
    pub fn assert_valid_call(call: &Call) {
        assert!(!call.id.is_empty(), "Call ID should not be empty");
        assert!(!call.title.is_empty(), "Call title should not be empty");
        assert!(call.duration >= 0, "Call duration should be non-negative");
        assert!(!call.participants.is_empty(), "Call should have participants");

        for participant in &call.participants {
            assert!(!participant.name.is_empty(), "Participant name should not be empty");
        }
    }

    /// Assert that an Email has valid structure
    pub fn assert_valid_email(email: &Email) {
        assert!(!email.id.is_empty(), "Email ID should not be empty");
        assert!(!email.subject.is_empty(), "Email subject should not be empty");
        assert!(!email.from.email.is_empty(), "Email from address should not be empty");
        assert!(!email.to.is_empty(), "Email should have recipients");

        // Should have either HTML or text content
        assert!(
            email.html_content.is_some() || email.text_content.is_some(),
            "Email should have content"
        );
    }

    /// Assert that markdown output is valid
    pub fn assert_valid_markdown(content: &str) {
        // Should have frontmatter
        assert!(content.starts_with("---"), "Should have YAML frontmatter");
        assert!(content.contains("---\n"), "Should close frontmatter");

        // Should have common markdown elements
        let has_headers = content.contains("#");
        let has_sections = content.contains("##");

        assert!(has_headers || has_sections, "Should have headers or sections");
    }
}

/// Mock server helpers for API testing
pub mod mocks {
    use super::*;

    /// Setup a mock Gong API server
    pub async fn setup_mock_server() -> mockito::ServerGuard {
        let mut server = mockito::Server::new_async().await;

        // Mock authentication endpoint
        server
            .mock("GET", "/v2/widget-accounts-data")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(TestFixtures::sample_csrf_response())
            .create_async()
            .await;

        server
    }

    /// Create mock timeline response
    pub fn mock_timeline_response() -> String {
        r#"{
            "calls": [],
            "emails": [],
            "total": 0,
            "hasMore": false
        }"#.to_string()
    }

    /// Create mock customer search response
    pub fn mock_customer_search_response() -> String {
        r#"{
            "customers": [
                {
                    "id": "cust-123",
                    "name": "Test Customer",
                    "workspace_id": "ws-789"
                }
            ]
        }"#.to_string()
    }
}

/// File system test helpers
pub mod fs_helpers {
    use super::*;

    /// Create test output directory structure
    pub fn create_test_output_structure(base: &Path) -> std::io::Result<()> {
        let customer_dir = base.join("ct_test_customer");
        fs::create_dir_all(&customer_dir)?;

        // Create sample markdown files
        let call_file = customer_dir.join("2024-01-15_call_test.md");
        let email_file = customer_dir.join("2024-01-15_email_test.md");

        fs::write(&call_file, "# Test Call\nContent here...")?;
        fs::write(&email_file, "# Test Email\nContent here...")?;

        Ok(())
    }

    /// Verify output directory structure
    pub fn verify_output_structure(base: &Path, customer: &str) -> bool {
        let customer_dir = base.join(format!("ct_{}", customer));

        if !customer_dir.exists() || !customer_dir.is_dir() {
            return false;
        }

        // Check for markdown files
        let entries: Vec<_> = fs::read_dir(&customer_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "md")
                    .unwrap_or(false)
            })
            .collect();

        !entries.is_empty()
    }
}

/// Environment setup for tests
pub fn setup_test_env() {
    // Set up test environment variables if needed
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "cs_cli=debug");
    }

    // Initialize tracing for tests (ignore if already initialized)
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let _ = tracing_subscriber::fmt::try_init();
    });
}

/// Run test with timeout
pub async fn with_timeout<F, T>(duration_secs: u64, f: F) -> Result<T, String>
where
    F: std::future::Future<Output = T>,
{
    tokio::time::timeout(
        std::time::Duration::from_secs(duration_secs),
        f
    )
    .await
    .map_err(|_| format!("Test timed out after {} seconds", duration_secs))
}