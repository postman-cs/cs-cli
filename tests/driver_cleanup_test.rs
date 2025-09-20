//! Integration tests for driver cleanup functionality
//! Tests that driver processes are properly cleaned up on exit, panic, and port conflicts

use cs_cli::common::drivers::DriverManager;
use std::process::Command;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

/// Helper function to check if a process with given name is running
fn is_process_running(process_name: &str) -> bool {
    let output = Command::new("pgrep")
        .arg("-f")
        .arg(process_name)
        .output()
        .expect("Failed to execute pgrep");

    // pgrep returns 0 if found, 1 if not found
    output.status.success()
}

/// Helper to count running driver processes
fn count_driver_processes() -> usize {
    let drivers = ["chromedriver", "geckodriver", "safaridriver"];
    drivers.iter().filter(|&d| is_process_running(d)).count()
}

#[tokio::test]
async fn test_driver_cleanup_on_normal_exit() {
    let initial_count = count_driver_processes();

    // Create and use driver manager
    {
        match DriverManager::new() {
            Ok(manager) => {
                let manager = Arc::new(manager);

                // Try to start a driver (may fail if browser not installed, that's ok)
                let _ = manager.connect(None).await;

                // Give it time to start
                sleep(Duration::from_secs(1)).await;

                // Explicitly cleanup
                let _ = manager.cleanup_all_drivers().await;
            }
            Err(_) => {
                // Driver manager not available on this system
                return;
            }
        }
    }

    // Give processes time to terminate
    sleep(Duration::from_secs(2)).await;

    let final_count = count_driver_processes();
    assert_eq!(
        initial_count, final_count,
        "Driver processes not cleaned up properly. Initial: {initial_count}, Final: {final_count}"
    );
}

#[tokio::test]
async fn test_driver_cleanup_on_drop() {
    let initial_count = count_driver_processes();

    // Create driver manager in a scope
    {
        match DriverManager::new() {
            Ok(manager) => {
                // Try to start a driver
                let _ = manager.connect(None).await;

                // Give it time to start
                sleep(Duration::from_secs(1)).await;

                // Manager goes out of scope, Drop should cleanup
            }
            Err(_) => {
                // Driver manager not available on this system
                return;
            }
        }
    }

    // Give Drop impl time to cleanup
    sleep(Duration::from_secs(2)).await;

    let final_count = count_driver_processes();
    assert_eq!(
        initial_count, final_count,
        "Drop trait not cleaning up drivers. Initial: {initial_count}, Final: {final_count}"
    );
}

#[tokio::test]
async fn test_port_conflict_handling() {
    let manager = match DriverManager::new() {
        Ok(m) => Arc::new(m),
        Err(_) => {
            // Driver manager not available on this system
            return;
        }
    };

    // Try to connect multiple times - should handle port conflicts
    let mut connections = vec![];
    for _ in 0..3 {
        match manager.connect(None).await {
            Ok(client) => connections.push(client),
            Err(_) => {
                // It's ok if connection fails (no browser installed)
                break;
            }
        }
    }

    // Cleanup all
    let _ = manager.cleanup_all_drivers().await;

    // Verify cleanup worked
    sleep(Duration::from_secs(2)).await;

    // All connections should be cleaned up
    assert!(
        !is_process_running("chromedriver") &&
        !is_process_running("geckodriver") &&
        !is_process_running("safaridriver"),
        "Some driver processes still running after cleanup"
    );
}

#[test]
fn test_driver_manager_is_send_sync() {
    // Compile-time test to ensure DriverManager can be used across threads
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<DriverManager>();
}

#[tokio::test]
async fn test_cleanup_idempotency() {
    let manager = match DriverManager::new() {
        Ok(m) => Arc::new(m),
        Err(_) => {
            // Driver manager not available on this system
            return;
        }
    };

    // Cleanup should be safe to call multiple times
    for _ in 0..3 {
        let result = manager.cleanup_all_drivers().await;
        assert!(result.is_ok(), "Cleanup should not fail on repeated calls");
    }
}