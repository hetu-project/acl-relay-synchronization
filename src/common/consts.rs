//! # Constants Module
//!
//! This module defines all application-wide constants used throughout the project.
//! Keeping constants centralized in this module promotes reusability, maintainability,
//! and avoids magic numbers or hardcoded strings scattered across the codebase.
//!
//! ## Usage
//! Import the constants as needed in your modules to ensure consistent values are used.
//! ```rust
//! use crate::consts::{CONSTANT_NAME};
//! ```

/// Format string for timestamp used in log file names.
pub const LOG_TIME_FORMAT: &str = "%Y-%m-%d_%H-%M-%S";

/// log dir for log files.
pub const LOG_PATH: &str = "logs";

/// Base name for log files.
pub const LOG_BASE_NAME: &str = "app";

/// Environment variable key to override the default logging level.
pub const LOG_KEY_ENV: &str = "RUST_LOG";

/// Default logging level if `RUST_LOG` environment variable is not set.
pub const LOG_DEFAULT_LEVEL: &str = "info";

/// client version
pub const CLI_VERSION: &str = "1.0";
