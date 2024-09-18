#[cfg(test)]
mod integration_tests {
    use crate::logger::Logger;
    use serde_json::json;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_logging_flow() {
        let logger = Logger::new("./config/config.yaml", b"anexampleverysecurekey123456789012")
            .await
            .unwrap();

        logger.info("Application started", Some(json!({"user": "test_user"})));
        logger.debug("Debugging mode enabled", None);
        logger.warn(
            "Low disk space",
            Some(json!({"disk": "C:", "free_space": "500MB"})),
        );
        logger.error(
            "Failed to connect to database",
            Some(json!({"db_host": "localhost"})),
        );
        logger.fatal("Unrecoverable error encountered", None);

        // Allow some time for async logging
        sleep(Duration::from_secs(2)).await;

        // Further assertions can be made based on the handlers' states
        // For example, checking if the in-memory handler has the expected logs
    }
}
