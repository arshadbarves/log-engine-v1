use log_engine_v1::logger::Logger;
use serde_json::json;
use std::time::Instant;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    // Initialize Logger with a benchmark-specific configuration
    let logger = Logger::new("config/benchmark_config.yaml", b"anexampleverysecurekey123456789012")
        .await
        .expect("Failed to initialize logger");

    // Start the metrics server in a separate task
    let metrics_clone = logger.metrics.clone();
    tokio::spawn(async move {
        if let Err(e) = metrics_clone.serve_metrics("127.0.0.1:9100").await {
            eprintln!("Metrics server error: {}", e);
        }
    });

    // Define benchmark parameters
    let total_logs = 1_000_000; // Total number of log messages to send
    let concurrent_tasks = 4;    // Number of concurrent logging tasks
    let logs_per_task = total_logs / concurrent_tasks;

    println!("Starting benchmark: {} logs across {} tasks", total_logs, concurrent_tasks);

    // Start timing
    let start_time = Instant::now();

    // Spawn concurrent logging tasks
    let mut handles = Vec::new();
    for i in 0..concurrent_tasks {
        let logger_clone = logger.clone();
        let handle = tokio::spawn(async move {
            for j in 0..logs_per_task {
                logger_clone.debug(
                    &format!("Benchmark log message {} from task {}", j, i),
                    Some(json!({"task_id": i, "message_id": j})),
                );
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Allow some time for all logs to be processed
    // Alternatively, implement a mechanism to wait until all logs are processed
    println!("All log messages enqueued. Waiting for processing to complete...");
    sleep(Duration::from_secs(10)).await;

    // End timing
    let elapsed = start_time.elapsed();

    // Retrieve metrics
    let logs_processed = logger.metrics.logs_processed.load(std::sync::atomic::Ordering::SeqCst);
    let errors = logger.metrics.errors.load(std::sync::atomic::Ordering::SeqCst);
    let queue_size = logger.metrics.queue_size.load(std::sync::atomic::Ordering::SeqCst);

    // Calculate throughput and latency
    let throughput = logs_processed as f64 / elapsed.as_secs_f64();
    let average_latency = elapsed.as_secs_f64() / logs_processed as f64 * 1_000_000.0; // in microseconds

    println!("Benchmark Results:");
    println!("Total Logs Sent: {}", total_logs);
    println!("Logs Processed: {}", logs_processed);
    println!("Errors: {}", errors);
    println!("Final Queue Size: {}", queue_size);
    println!("Elapsed Time: {:.2} seconds", elapsed.as_secs_f64());
    println!("Throughput: {:.2} logs/sec", throughput);
    println!("Average Latency: {:.2} µs/log", average_latency);

    let metrics_logger = logger.metrics.clone();
    tokio::spawn(async move {
        if let Err(e) = metrics_logger.serve_metrics("127.0.0.1:9000").await {
            eprintln!("Metrics server error: {}", e);
        }
    });

    // Exit the benchmark
    println!("Benchmark completed.");
}
