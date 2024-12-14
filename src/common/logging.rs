//! Module for initializing and managing the application's logging system.
//! It supports logging to both the console and rolling log files with optional
//! environment-based log level configuration.

use crate::common::consts;
use crate::common::error;
use chrono::Local;
use std::fs;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter};

/// Initializes the logging system for the application.
///
/// This function sets up logging to both the console and log files. Log files are
/// rolled manually (not automatically by size or time) and are stored in the
/// specified directory. The logging level can be controlled via the `RUST_LOG`
/// environment variable or defaults to `info`.
///
/// # Arguments
///
/// * `log_dir` - Path to the directory where log files will be stored.
///
/// # Returns
///
/// Returns a `error::Result<()>` indicating success or failure.
///
/// # Errors
///
/// Returns an error if the log directory cannot be created or if there is an issue
/// initializing the tracing subscriber.
///
/// # Example
///
/// ```
/// logging_init("/path/to/logs").unwrap();
/// ```
pub fn logging_init(log_dir: &str) -> error::Result<()> {
    let log_file = format!(
        "{}_{}.log",
        Local::now().format(consts::LOG_TIME_FORMAT),
        consts::LOG_BASE_NAME
    );

    // Create a rolling file appender that does not rotate automatically.
    let file_appender = RollingFileAppender::new(Rotation::NEVER, log_dir, log_file);
    //let (file_writer, _guard) = non_blocking(file_appender);
    let file_writer = file_appender;

    // Ensure the log directory exists, create if necessary.
    fs::create_dir_all(log_dir)?;

    // Define a logging layer for writing to log files with timestamps and line numbers.
    let file_layer = fmt::Layer::default()
        .with_writer(file_writer)
        .with_line_number(true)
        .with_ansi(false); // Disable ANSI colors for log files.

    // Define a logging layer for console output with timestamps and line numbers.
    let stdout_layer = fmt::Layer::default()
        .with_writer(std::io::stdout)
        .with_line_number(true);

    // Get the logging level from the environment or use the default.
    let rust_log = std::env::var(consts::LOG_KEY_ENV)
        .unwrap_or_else(|_| consts::LOG_DEFAULT_LEVEL.to_string());

    // Create a tracing subscriber with environment-based filtering and layered output.
    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::new(rust_log))
        .with(stdout_layer)
        .with(file_layer);

    // Set the global default subscriber for tracing.
    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}
