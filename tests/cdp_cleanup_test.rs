//! Integration tests for CDP browser cleanup functionality
//! Tests that browser processes are properly cleaned up on exit, panic, and signal handling

use cs_cli::common::auth::CdpBrowserManager;
use cs_cli::common::auth::cdp_browser_manager::Browser;
use std::process::Command;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

/// Helper function to check if a process with given name is running
fn is_process_running(process_name: &str) -> bool {
    let output = Command::new("pgrep")
        .arg("-f")
        .arg(process_name)
        .output();
    
    match output {
        Ok(output) => output.status.success() && !output.stdout.is_empty(),
        Err(_) => false,
    }
}

/// Helper to count running browser processes
fn count_browser_processes() -> usize {
    let browsers = ["Google Chrome", "Chrome", "chromium"];
    browsers.iter().filter(|&b| is_process_running(b)).count()
}

#[tokio::test]
async fn test_cdp_cleanup_on_normal_exit() {
    let initial_count = count_browser_processes();

    // Create and use CDP browser manager
    {
        let mut manager = CdpBrowserManager::new();
        
        // Try to launch a browser (may fail if browser not installed, that's ok)
        if let Ok(_) = manager.launch_browser(Browser::Chrome, None).await {
            // Give browser time to start
            sleep(Duration::from_secs(1)).await;
            
            // Clean up explicitly
            let _ = manager.close().await;
        }
    }

    // Give processes time to clean up
    sleep(Duration::from_secs(2)).await;

    let final_count = count_browser_processes();
    
    // Allow for some variance in process counts due to system activity
    assert!(
        final_count <= initial_count + 1,
        "Browser processes not cleaned up properly. Initial: {initial_count}, Final: {final_count}"
    );
}

#[tokio::test]
async fn test_cdp_cleanup_on_drop() {
    let initial_count = count_browser_processes();

    // Create CDP browser manager in a scope
    {
        let mut manager = CdpBrowserManager::new();
        
        // Try to launch a browser
        if let Ok(_) = manager.launch_browser(Browser::Chrome, None).await {
            // Give browser time to start
            sleep(Duration::from_secs(1)).await;
            
            // Manager will be dropped here, triggering cleanup
        }
    }

    // Give processes time to clean up
    sleep(Duration::from_secs(2)).await;

    let final_count = count_browser_processes();
    
    // Allow for some variance in process counts due to system activity
    assert!(
        final_count <= initial_count + 1,
        "Drop trait not cleaning up browsers. Initial: {initial_count}, Final: {final_count}"
    );
}

#[tokio::test]
async fn test_global_cleanup_functionality() {
    // Test the global cleanup functionality
    let manager = CdpBrowserManager::new();
    
    // Try to launch a browser
    if let Ok(_) = manager.launch_browser(Browser::Chrome, None).await {
        // Give browser time to start
        sleep(Duration::from_secs(1)).await;
        
        // Test global cleanup
        let _ = CdpBrowserManager::cleanup_all_browsers().await;
        
        // Give processes time to clean up
        sleep(Duration::from_secs(1)).await;
    }

    // Verify no browser processes are running
    assert!(
        !is_process_running("Google Chrome") && !is_process_running("Chrome"),
        "Some browser processes still running after cleanup"
    );
}

fn test_cdp_browser_manager_is_send_sync() {
    // Compile-time test to ensure CdpBrowserManager can be used across threads
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<CdpBrowserManager>();
}

#[tokio::test]
async fn test_browser_manager_creation() {
    let manager = CdpBrowserManager::new();
    
    // Test that manager can be created without issues
    assert!(!manager.has_browser());
    
    // Test cleanup doesn't panic
    let result = manager.close().await;
    assert!(result.is_ok());
}