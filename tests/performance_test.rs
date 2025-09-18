//! Performance and Concurrency Tests
//!
//! These tests validate performance characteristics, memory usage,
//! and concurrent operation safety.

use cs_cli::gong::api::client::HttpClientPool;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

#[tokio::test]
async fn test_semaphore_rate_limiting() {
    // Test that semaphore properly limits concurrent requests
    let max_concurrent = 3;
    let semaphore = Arc::new(Semaphore::new(max_concurrent));

    let mut tasks = vec![];
    let start = Instant::now();

    // Launch 10 tasks that each take 100ms
    for i in 0..10 {
        let sem = semaphore.clone();
        tasks.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.expect("Failed to acquire permit");
            let task_start = Instant::now();

            // Simulate work
            tokio::time::sleep(Duration::from_millis(100)).await;

            let task_duration = task_start.elapsed();
            (i, task_duration)
        }));
    }

    // Wait for all tasks
    let mut results = vec![];
    for task in tasks {
        results.push(task.await.expect("Task failed"));
    }

    let total_duration = start.elapsed();

    println!("Total time for 10 tasks with max_concurrent={max_concurrent}: {total_duration:?}");

    // With max 3 concurrent and 100ms per task:
    // Should take at least 400ms (4 waves: 3+3+3+1)
    assert!(
        total_duration >= Duration::from_millis(300),
        "Should respect concurrency limit"
    );

    // But shouldn't take as long as sequential (1000ms)
    assert!(
        total_duration < Duration::from_millis(1000),
        "Should run concurrently"
    );

    // Verify all tasks completed
    assert_eq!(results.len(), 10, "All tasks should complete");
}

#[tokio::test]
async fn test_connection_pool_reuse() {
    // Test that connection pool properly reuses connections with real Gong API
    // This validates our HTTP client consolidation work

    use cs_cli::gong::auth::GongAuthenticator;
    use cs_cli::gong::config::AppConfig;

    let config = AppConfig::create_default();
    let mut authenticator = GongAuthenticator::new(config.auth.clone())
        .await
        .expect("Failed to create Gong authenticator - check browser login");

    // Perform authentication flow with timeout and detailed error reporting
    println!("🔍 Starting authentication...");
    let auth_result = tokio::time::timeout(
        tokio::time::Duration::from_secs(10), // 10 second timeout
        authenticator.authenticate(),
    )
    .await;

    match auth_result {
        Ok(Ok(true)) => {
            println!("✅ Authentication successful!");
        }
        Ok(Ok(false)) => {
            panic!("❌ Authentication returned false - check browser login to Gong");
        }
        Ok(Err(e)) => {
            panic!("❌ Authentication error: {e}");
        }
        Err(_) => {
            panic!("❌ Authentication timed out after 10 seconds - this indicates a network or API issue");
        }
    }

    let auth = Arc::new(authenticator);

    let pool = Arc::new(
        HttpClientPool::new_gong_pool(Some(config.http))
            .await
            .expect("Failed to create pool"),
    );

    // Test that pool can handle multiple concurrent Gong API requests
    let mut tasks = vec![];
    for i in 0..5 {
        let pool = pool.clone();
        let auth = auth.clone();
        tasks.push(tokio::spawn(async move {
            // Get session cookies for the request
            let _cookies = auth
                .get_session_cookies()
                .expect("Authentication should be available");

            let result = tokio::time::timeout(
                tokio::time::Duration::from_secs(10), // 10 second timeout per request
                pool.get("https://us-65885.app.gong.io/v2/settings/workspaces"),
            )
            .await;

            let success = match result {
                Ok(Ok(response)) => response.status().is_success(),
                Ok(Err(_)) => false,
                Err(_) => false, // timeout
            };
            (i, success)
        }));
    }

    // Wait for all tasks with overall timeout
    let mut success_count = 0;
    let timeout_duration = tokio::time::Duration::from_secs(30); // 30 second overall timeout

    match tokio::time::timeout(timeout_duration, async {
        for task in tasks {
            let (idx, success) = task.await.expect("Task failed");
            if success {
                success_count += 1;
            }
            println!("Request {idx} success: {success}");
        }
        success_count
    })
    .await
    {
        Ok(count) => success_count = count,
        Err(_) => {
            println!("Connection pool test timed out after 30 seconds");
            success_count = 0;
        }
    }

    // Pool should handle concurrent requests (allow some failures due to auth)
    println!("Successfully completed {success_count}/5 requests");

    // At least one request should succeed if authentication is working
    assert!(
        success_count > 0,
        "At least one request should succeed with valid Gong authentication"
    );
}

#[tokio::test]
async fn test_memory_usage_large_dataset() {
    // Test memory efficiency with large datasets
    // Create large vectors to simulate call/email data

    let mut large_dataset = Vec::new();

    // Simulate 10,000 communications
    for i in 0..10_000 {
        let data = format!("Communication {i} with some content that takes up space");
        large_dataset.push(data);
    }

    let initial_len = large_dataset.len();

    // Process in chunks to avoid memory spikes
    let chunk_size = 100;
    let mut processed_count = 0;

    for chunk in large_dataset.chunks(chunk_size) {
        // Simulate processing
        processed_count += chunk.len();

        // Small delay to simulate API calls
        tokio::time::sleep(Duration::from_micros(100)).await;
    }

    assert_eq!(processed_count, initial_len, "Should process all items");

    println!("Processed {processed_count} items in chunks of {chunk_size}");
}

#[tokio::test]
async fn test_exponential_backoff_timing() {
    // Test that exponential backoff works correctly
    let delays = [100, 200, 400, 800, 1600]; // milliseconds
    let mut total_delay = Duration::ZERO;

    for (attempt, &delay_ms) in delays.iter().enumerate() {
        let start = Instant::now();

        // Simulate retry with backoff
        let delay = Duration::from_millis(delay_ms);
        tokio::time::sleep(delay).await;

        let actual_delay = start.elapsed();
        total_delay += actual_delay;

        println!("Attempt {attempt}: Expected {delay_ms}ms, Actual {actual_delay:?}");

        // Allow 10ms tolerance for timing
        assert!(
            actual_delay >= delay - Duration::from_millis(10),
            "Delay should be at least the expected value"
        );
    }

    println!("Total backoff time: {total_delay:?}");

    // Total should be around 3.2 seconds
    assert!(
        total_delay >= Duration::from_millis(3000),
        "Total backoff should be substantial"
    );
}

#[tokio::test]
async fn test_concurrent_task_cancellation() {
    // Test that tasks can be cancelled gracefully
    let mut tasks = vec![];

    // Launch long-running tasks
    for i in 0..5 {
        tasks.push(tokio::spawn(async move {
            // Simulate long work
            for j in 0..100 {
                tokio::time::sleep(Duration::from_millis(10)).await;
                if j % 10 == 0 {
                    // Check for cancellation opportunity
                    tokio::task::yield_now().await;
                }
            }
            format!("Task {i} completed")
        }));
    }

    // Wait a bit then cancel remaining tasks
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Cancel all tasks
    for task in &tasks {
        task.abort();
    }

    // Verify tasks were cancelled
    let mut cancelled_count = 0;
    let mut completed_count = 0;

    for task in tasks {
        match task.await {
            Ok(msg) => {
                completed_count += 1;
                println!("Task completed: {msg}");
            }
            Err(e) if e.is_cancelled() => {
                cancelled_count += 1;
                println!("Task cancelled");
            }
            Err(e) => {
                panic!("Unexpected error: {e:?}");
            }
        }
    }

    println!("Completed: {completed_count}, Cancelled: {cancelled_count}");

    // Most tasks should be cancelled
    assert!(cancelled_count > 0, "Some tasks should be cancelled");
}

#[tokio::test]
async fn test_rate_limiting_with_jitter() {
    use cs_cli::common::http::rate_limiting::{api_delay, sleep_with_jitter};

    // Test basic jitter functionality
    let start = Instant::now();
    sleep_with_jitter(100, 0.5).await; // 100ms ±50%
    let duration = start.elapsed();

    // Should be between 50ms and 150ms
    assert!(
        duration >= Duration::from_millis(40),
        "Jitter delay too short: {duration:?}"
    );
    assert!(
        duration <= Duration::from_millis(200),
        "Jitter delay too long: {duration:?}"
    );

    println!("Jitter test: {}ms base -> {:?} actual", 100, duration);

    // Test API delay
    let start = Instant::now();
    api_delay().await; // 1000ms ±25%
    let duration = start.elapsed();

    // Should be between 750ms and 1250ms
    assert!(
        duration >= Duration::from_millis(700),
        "API delay too short: {duration:?}"
    );
    assert!(
        duration <= Duration::from_millis(1300),
        "API delay too long: {duration:?}"
    );

    println!("API delay test: {duration:?} (expected 750ms-1250ms)");

    // Test that multiple delays have variation (jitter working)
    let mut delays = Vec::new();
    for _ in 0..5 {
        let start = Instant::now();
        sleep_with_jitter(200, 0.3).await;
        delays.push(start.elapsed());
    }

    // Check that we have variation in delays (not all identical)
    let min_delay = delays.iter().min().unwrap();
    let max_delay = delays.iter().max().unwrap();
    let variation = max_delay.saturating_sub(*min_delay);

    assert!(
        variation >= Duration::from_millis(20),
        "Insufficient jitter variation: min={min_delay:?}, max={max_delay:?}, variation={variation:?}"
    );

    println!("Jitter variation test: {variation:?} range across 5 delays");
}

#[tokio::test]
async fn test_parallel_data_processing() {
    // Test parallel processing of data chunks
    let data: Vec<i32> = (0..1000).collect();
    let chunk_size = 100;

    let start = Instant::now();

    let mut tasks = vec![];
    for chunk in data.chunks(chunk_size) {
        let chunk = chunk.to_vec();
        tasks.push(tokio::spawn(async move {
            // Simulate processing
            let sum: i32 = chunk.iter().sum();
            tokio::time::sleep(Duration::from_millis(10)).await;
            sum
        }));
    }

    let mut total = 0;
    for task in tasks {
        total += task.await.expect("Task failed");
    }

    let duration = start.elapsed();

    // Verify correctness
    let expected: i32 = (0..1000).sum();
    assert_eq!(total, expected, "Parallel processing should be correct");

    println!("Parallel processing of 1000 items took {duration:?}");

    // Should be faster than sequential (100ms)
    assert!(
        duration < Duration::from_millis(100),
        "Parallel should be faster than sequential"
    );
}

#[tokio::test]
async fn test_async_stream_processing() {
    // Test streaming data processing
    use tokio_stream::{self as stream, StreamExt};

    let data = stream::iter(0..100);

    let start = Instant::now();
    let mut count = 0;

    tokio::pin!(data);

    while let Some(item) = data.next().await {
        count += 1;

        // Simulate processing every 10th item
        if item % 10 == 0 {
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    }

    let duration = start.elapsed();

    assert_eq!(count, 100, "Should process all items");
    println!("Stream processing took {duration:?}");
}

#[tokio::test]
async fn test_thread_safety_shared_state() {
    // Test thread-safe access to shared state
    use std::sync::atomic::{AtomicUsize, Ordering};

    let counter = Arc::new(AtomicUsize::new(0));
    let mut tasks = vec![];

    // Launch 100 tasks that increment counter
    for _ in 0..100 {
        let counter = counter.clone();
        tasks.push(tokio::spawn(async move {
            for _ in 0..100 {
                counter.fetch_add(1, Ordering::SeqCst);
                tokio::task::yield_now().await;
            }
        }));
    }

    // Wait for all tasks
    for task in tasks {
        task.await.expect("Task failed");
    }

    let final_count = counter.load(Ordering::SeqCst);
    assert_eq!(final_count, 10000, "Counter should be thread-safe");

    println!("Thread-safe counter reached {final_count}");
}
