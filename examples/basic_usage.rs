use log_engine_v1::logger::Logger;
use log_engine_v1::{log_error, log_info};
use serde_json::json;
use tokio::signal;

#[tokio::main]
async fn main() {
    // Initialize Logger
    let logger = Logger::new("config/config.yaml", b"anexampleverysecurekey123456789012")
        .await
        .expect("Failed to initialize logger");

    // Log messages with different levels
    logger.debug(
        "This is a debug message",
        Some(json!({"debug_info": "details"})),
    );
    logger.info("User logged in", Some(json!({"user_id": 12345})));
    logger.warn("Memory usage is high", Some(json!({"memory": "80%"})));
    logger.error(
        "Failed to load resource",
        Some(json!({"resource": "texture.png"})),
    );
    logger.fatal("System crash imminent", None);

    // Use logging macros
    log_info!(logger, "Application has reached point {}", "X");
    log_error!(logger, "An error occurred: {}", "OutOfMemory");

    // Optionally, start metrics server in a separate task
    let metrics_logger = logger.metrics.clone();
    tokio::spawn(async move {
        if let Err(e) = metrics_logger.serve_metrics("127.0.0.1:9000").await {
            eprintln!("Metrics server error: {}", e);
        }
    });

    // Keep the application running to allow async logging
    tokio::select! {
        _ = signal::ctrl_c() => {
            logger.info("Application shutting down", None);
        },
    }
}
