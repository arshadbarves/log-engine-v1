# Log Engine v1

## Overview
Log Engine v1 is a high-performance, asynchronous logging library for Rust. It supports multiple log levels, concurrent logging tasks, and customizable configurations through YAML files.

## Features
- Asynchronous logging
- Multiple log levels (DEBUG, INFO, WARN, ERROR, FATAL)
- Configurable through YAML files
- Metrics server for monitoring
- Supports concurrent logging tasks

## Installation
Add the following to your `Cargo.toml`:

```toml
[dependencies]
log_engine_v1 = "0.1.0"
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
```

## Usage

### Initialization
Initialize the logger with a configuration file and a secure key:

```rust
use log_engine_v1::logger::Logger;

#[tokio::main]
async fn main() {
    let logger = Logger::new("config/benchmark_config.yaml", b"anexampleverysecurekey123456789012")
        .await
        .expect("Failed to initialize logger");
}
```

### Logging
Log messages at different levels:

```rust
logger.debug("Debug message", None);
logger.info("Info message", None);
logger.warn("Warning message", None);
logger.error("Error message", None);
logger.fatal("Fatal message", None);
```

### Benchmark Example
Run a benchmark to test the logging performance:

```rust
use log_engine_v1::logger::Logger;
use serde_json::json;
use std::time::Instant;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let logger = Logger::new("config/benchmark_config.yaml", b"anexampleverysecurekey123456789012")
        .await
        .expect("Failed to initialize logger");

    let metrics_clone = logger.metrics.clone();
    tokio::spawn(async move {
        if let Err(e) = metrics_clone.serve_metrics("127.0.0.1:9100").await {
            eprintln!("Metrics server error: {}", e);
        }
    });

    let total_logs = 1_000_000;
    let concurrent_tasks = 4;
    let logs_per_task = total_logs / concurrent_tasks;

    println!("Starting benchmark: {} logs across {} tasks", total_logs, concurrent_tasks);

    let start_time = Instant::now();

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

    for handle in handles {
        handle.await.unwrap();
    }

    println!("All log messages enqueued. Waiting for processing to complete...");
    sleep(Duration::from_secs(10)).await;

    let elapsed = start_time.elapsed();

    let logs_processed = logger.metrics.logs_processed.load(std::sync::atomic::Ordering::SeqCst);
    let errors = logger.metrics.errors.load(std::sync::atomic::Ordering::SeqCst);
    let queue_size = logger.metrics.queue_size.load(std::sync::atomic::Ordering::SeqCst);

    let throughput = logs_processed as f64 / elapsed.as_secs_f64();
    let average_latency = elapsed.as_secs_f64() / logs_processed as f64 * 1_000_000.0;

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

    println!("Benchmark completed.");
}
```

### Configuration
Example configuration file `config/benchmark_config.yaml`:

```yaml
level: "TRACE"
filters: {}
handlers:
  - type_: "console"
    config:
      capacity: 1_000_000
formatter: "text"
plugins: []
```

## License
This project is licensed under the MIT License.
```