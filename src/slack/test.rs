//! Test module for Slack integration
//!
//! Simple test function to validate that common abstractions work with Slack

use crate::slack::api::SlackClient;
use crate::Result;

/// Test basic Slack integration using common abstractions
pub async fn test_slack_integration(workspace_domain: Option<String>) -> Result<()> {
    let domain = workspace_domain.unwrap_or_else(|| "postman.enterprise.slack.com".to_string());
    
    println!("Testing Slack Integration");
    println!("{}", "=".repeat(50));
    println!("Workspace: {}", domain);
    println!();
    
    let mut client = SlackClient::new(domain);
    
    // Test the integration
    match client.test_basic_functionality().await {
        Ok(_) => {
            println!("Success! Common abstractions work with Slack");
            Ok(())
        }
        Err(e) => {
            println!("Test failed: {}", e);
            println!();
            println!("Troubleshooting:");
            println!("1. Make sure Firefox is open");
            println!("2. Navigate to your Slack workspace");
            println!("3. Log in to Slack");
            println!("4. Try again");
            Err(e)
        }
    }
}
