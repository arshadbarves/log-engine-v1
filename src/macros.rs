#[macro_export]
macro_rules! log_debug {
    ($logger:expr, $msg:expr) => {
        $logger.debug($msg, None);
    };
    ($logger:expr, $msg:expr, $($arg:tt)*) => {
        $logger.debug(&format!($msg, $($arg)*), Some(serde_json::json!({})));
    };
}

#[macro_export]
macro_rules! log_info {
    ($logger:expr, $msg:expr) => {
        $logger.info($msg, None);
    };
    ($logger:expr, $msg:expr, $($arg:tt)*) => {
        $logger.info(&format!($msg, $($arg)*), Some(serde_json::json!({})));
    };
}

#[macro_export]
macro_rules! log_warn {
    ($logger:expr, $msg:expr) => {
        $logger.warn($msg, None);
    };
    ($logger:expr, $msg:expr, $($arg:tt)*) => {
        $logger.warn(&format!($msg, $($arg)*), Some(serde_json::json!({})));
    };
}

#[macro_export]
macro_rules! log_error {
    ($logger:expr, $msg:expr) => {
        $logger.error($msg, None);
    };
    ($logger:expr, $msg:expr, $($arg:tt)*) => {
        $logger.error(&format!($msg, $($arg)*), Some(serde_json::json!({})));
    };
}

#[macro_export]
macro_rules! log_fatal {
    ($logger:expr, $msg:expr) => {
        $logger.fatal($msg, None);
    };
    ($logger:expr, $msg:expr, $($arg:tt)*) => {
        $logger.fatal(&format!($msg, $($arg)*), Some(serde_json::json!({})));
    };
}
