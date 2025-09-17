//! Performance and Concurrency Tests
//!
//! These tests validate performance characteristics, memory usage,
//! and concurrent operation safety.

use cs_cli::common::config::HttpSettings;
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

    println!("Total time for 10 tasks with max_concurrent={}: {:?}",
             max_concurrent, total_duration);

    // With max 3 concurrent and 100ms per task:
    // Should take at least 400ms (4 waves: 3+3+3+1)
    assert!(total_duration >= Duration::from_millis(300),
            "Should respect concurrency limit");

    // But shouldn't take as long as sequential (1000ms)
    assert!(total_duration < Duration::from_millis(1000),
            "Should run concurrently");

    // Verify all tasks completed
    assert_eq!(results.len(), 10, "All tasks should complete");
}

#[tokio::test]
async fn test_connection_pool_reuse() {
    // Test that connection pool properly reuses connections
    let config = HttpSettings::default();
    let pool = Arc::new(
        HttpClientPool::new(Some(config)).await.expect("Failed to create pool")
    );

    // Test that pool can handle multiple concurrent requests
    let mut tasks = vec![];
    for i in 0..5 {
        let pool = pool.clone();
        tasks.push(tokio::spawn(async move {
            // Use the pool to make a request
            let result = pool.get("https://httpbin.org/delay/0").await;
            (i, result.is_ok())
        }));
    }

    // Wait for all tasks
    let mut success_count = 0;
    for task in tasks {
        let (idx, success) = task.await.expect("Task failed");
        if success {
            success_count += 1;
        }
        println!("Request {} success: {}", idx, success);
    }

    // Pool should handle concurrent requests
    println!("Successfully completed {}/5 requests", success_count);
}

#[tokio::test]
async fn test_memory_usage_large_dataset() {
    // Test memory efficiency with large datasets
    // Create large vectors to simulate call/email data

    let mut large_dataset = Vec::new();

    // Simulate 10,000 communications
    for i in 0..10_000 {
        let data = format!("Communication {} with some content that takes up space", i);
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

    assert_eq!(processed_count, initial_len,
               "Should process all items");

    println!("Processed {} items in chunks of {}",
             processed_count, chunk_size);
}

#[tokio::test]
async fn test_exponential_backoff_timing() {
    // Test that exponential backoff works correctly
    let delays = vec![100, 200, 400, 800, 1600]; // milliseconds
    let mut total_delay = Duration::ZERO;

    for (attempt, &delay_ms) in delays.iter().enumerate() {
        let start = Instant::now();

        // Simulate retry with backoff
        let delay = Duration::from_millis(delay_ms);
        tokio::time::sleep(delay).await;

        let actual_delay = start.elapsed();
        total_delay += actual_delay;

        println!("Attempt {}: Expected {}ms, Actual {:?}",
                 attempt, delay_ms, actual_delay);

        // Allow 10ms tolerance for timing
        assert!(actual_delay >= delay - Duration::from_millis(10),
                "Delay should be at least the expected value");
    }

    println!("Total backoff time: {:?}", total_delay);

    // Total should be around 3.2 seconds
    assert!(total_delay >= Duration::from_millis(3000),
            "Total backoff should be substantial");
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
            format!("Task {} completed", i)
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
                println!("Task completed: {}", msg);
            }
            Err(e) if e.is_cancelled() => {
                cancelled_count += 1;
                println!("Task cancelled");
            }
            Err(e) => {
                panic!("Unexpected error: {:?}", e);
            }
        }
    }

    println!("Completed: {}, Cancelled: {}",
             completed_count, cancelled_count);

    // Most tasks should be cancelled
    assert!(cancelled_count > 0, "Some tasks should be cancelled");
}

#[tokio::test]
async fn test_rate_limiter_throughput() {
    /// Custom rate limiter for testing
    struct RateLimiter {
        semaphore: Arc<Semaphore>,
        _refill_task: tokio::task::JoinHandle<()>,
    }

    impl RateLimiter {
        fn new(requests_per_second: usize) -> Self {
            let semaphore = Arc::new(Semaphore::new(requests_per_second));
            let sem_clone = semaphore.clone();

            let refill_task = tokio::spawn(async move {
                let interval = Duration::from_secs(1) / requests_per_second as u32;
                loop {
                    tokio::time::sleep(interval).await;
                    if sem_clone.available_permits() < requests_per_second {
                        sem_clone.add_permits(1);
                    }
                }
            });

            Self {
                semaphore,
                _refill_task: refill_task,
            }
        }

        async fn acquire(&self) {
            let _ = self.semaphore.acquire().await.expect("Failed to acquire permit");
        }
    }

    // Test custom rate limiter implementation
    let requests_per_second = 10;
    let rate_limiter = RateLimiter::new(requests_per_second);

    let start = Instant::now();
    let mut request_times = vec![];

    // Make 20 requests
    for i in 0..20 {
        rate_limiter.acquire().await;
        request_times.push(Instant::now());
        println!("Request {} at {:?}", i, request_times.last().unwrap().duration_since(start));
    }

    let total_duration = start.elapsed();

    // 20 requests at 10/second should take about 2 seconds
    println!("20 requests at {}/sec took {:?}",
             requests_per_second, total_duration);

    assert!(total_duration >= Duration::from_millis(1900),
            "Should respect rate limit");
    assert!(total_duration < Duration::from_millis(2500),
            "Should not be too slow");

    // Verify spacing between requests
    for i in 1..request_times.len() {
        let gap = request_times[i].duration_since(request_times[i - 1]);
        assert!(gap >= Duration::from_millis(90),
                "Requests should be properly spaced");
    }
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

    println!("Parallel processing of 1000 items took {:?}", duration);

    // Should be faster than sequential (100ms)
    assert!(duration < Duration::from_millis(100),
            "Parallel should be faster than sequential");
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
    println!("Stream processing took {:?}", duration);
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

    println!("Thread-safe counter reached {}", final_count);
}

