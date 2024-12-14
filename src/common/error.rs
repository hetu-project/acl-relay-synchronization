//! Module defining common error types and error handling utilities for the gateway application.
//!
//! This module provides a standardized way to handle errors using the `thiserror` crate.
//! It defines a custom `Error` enum for various error scenarios and includes helper
//! methods for extracting error codes and messages. A `Result` type alias is also
//! provided for convenience.use std::path::PathBuf;

use std::path::PathBuf;
use thiserror::Error;

/// A convenient type alias for results used throughout the gateway application.
/// This type simplifies returning a `Result` with the `Error` type.
///
/// # Examples
/// ```
/// fn example_function() -> Result<()> {
///     // Your logic here
///     Ok(())
/// }
/// ```
pub type Result<T> = std::result::Result<T, Error>;

/// Enumeration of predefined error codes for the gateway application.
/// These codes provide a standard way to classify errors.
///
/// # Variants
/// - `Success`: Indicates a successful operation, with an HTTP status code of 200.
#[repr(u16)]
#[derive(Debug)]
pub enum ErrorCodes {
    /// Represents a successful operation (HTTP 200).
    Success = 200,
}

/// Enumeration of possible errors in the gateway application.
///
/// This enum uses the `thiserror` crate to simplify error definitions. It supports errors
/// for missing configuration, I/O issues, tracing initialization errors, and custom errors.
///
/// # Variants
/// - `ConfigMissing`: Indicates a missing configuration file.
/// - `SerializationError`: Indicates a configuration file deserialize failed.
/// - `IoError`: Represents an I/O-related error.
/// - `TracingError`: Represents an error while initializing the tracing system.
/// - `CustomError`: Represents any custom error with a descriptive message.
#[derive(Error, Debug)]
pub enum Error {
    /// Missing operator configuration file at the specified path.
    #[error("No operator config found at this path: {0}")]
    ConfigMissing(PathBuf),

    #[error("Config deserialization error: {0}")]
    SerializationError(#[from] serde_yaml::Error),

    /// I/O error encountered during operations.
    #[error("Error while performing IO for the Operator: {0}")]
    IoError(#[from] std::io::Error),

    /// Error encountered while setting up the tracing system.
    #[error("Tracing error: {0}")]
    TracingError(#[from] tracing::dispatcher::SetGlobalDefaultError),

    /// Custom error with a descriptive string message.
    #[error("Custom error: {0}")]
    CustomError(String),

    /// Nostr Sdk keys error
    #[error(transparent)]
    NostrSdkKeyError(#[from] nostr_sdk::key::Error),

    /// Nostr Sdk client error
    #[error(transparent)]
    NostrSdkClientError(#[from] nostr_sdk::client::Error),

    /// Nostr Sdk database error
    #[error(transparent)]
    NostrSdkDBError(#[from] nostr_sdk::prelude::DatabaseError),

    /// Sea ORM database error
    #[error(transparent)]
    SeaOrmDBError(#[from] sea_orm::DbErr),
}

impl Error {
    /// Retrieves the error code associated with the current error variant.
    ///
    /// # Returns
    /// Returns the error code as a `u16`. Currently, all errors default to `ErrorCodes::Success`.
    ///
    /// # Examples
    /// ```
    /// let error = Error::CustomError("Something went wrong".to_string());
    /// assert_eq!(error.error_code(), 200);
    /// ```
    pub fn error_code(&self) -> u16 {
        match self {
            _ => ErrorCodes::Success as u16,
        }
    }

    /// Retrieves a human-readable error message for the current error variant.
    ///
    /// This method utilizes the `to_string` implementation provided by `thiserror`.
    ///
    /// # Returns
    /// Returns the error message as a `String`.
    ///
    /// # Examples
    /// ```
    /// let error = Error::ConfigMissing(PathBuf::from("/path/to/config"));
    /// assert_eq!(
    ///     error.error_message(),
    ///     "No operator config found at this path: /path/to/config"
    /// );
    /// ```
    pub fn error_message(&self) -> String {
        self.to_string()
    }
}
