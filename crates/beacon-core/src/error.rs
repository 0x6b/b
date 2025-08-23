//! Error types for the planner library.

use std::path::PathBuf;

use thiserror::Error;

/// Comprehensive error type for all planner operations.
#[derive(Error, Debug)]
pub enum PlannerError {
    /// Database connection or query errors
    #[error("Database error: {message}")]
    Database {
        message: String,
        #[source]
        source: rusqlite::Error,
    },

    /// Plan not found for the given ID
    #[error("Plan with ID {id} not found")]
    PlanNotFound { id: u64 },

    /// Step not found for the given ID
    #[error("Step with ID {id} not found")]
    StepNotFound { id: u64 },

    /// File system operation errors
    #[error("File system error at path '{path}': {source}")]
    FileSystem {
        path: PathBuf,
        source: std::io::Error,
    },

    /// XDG directory specification errors
    #[error("XDG directory error: {0}")]
    XdgDirectory(String),

    /// Invalid input validation errors
    #[error("Invalid input for field '{field}': {reason}")]
    InvalidInput { field: String, reason: String },

    /// Serialization/deserialization errors
    #[error("Serialization error: {source}")]
    Serialization {
        #[from]
        source: serde_json::Error,
    },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    Configuration { message: String },
}

impl PlannerError {
    /// Creates a new database error with additional context
    pub fn database_error(message: &str, source: rusqlite::Error) -> Self {
        Self::Database {
            message: message.to_string(),
            source,
        }
    }

    /// Creates a new input validation error
    pub fn invalid_input(field: &str, reason: &str) -> Self {
        Self::InvalidInput {
            field: field.to_string(),
            reason: reason.to_string(),
        }
    }

    /// Creates a new serialization error
    pub fn serialization_error(source: serde_json::Error) -> Self {
        Self::Serialization { source }
    }
}

/// Extension trait for Result to provide concise error mapping
pub trait ResultExt<T> {
    /// Map database errors with a message
    fn db_err(self, message: &str) -> Result<T>;
    
    /// Map configuration errors with a message
    fn config_err(self, message: &str) -> Result<T>;
}

impl<T> ResultExt<T> for std::result::Result<T, rusqlite::Error> {
    fn db_err(self, message: &str) -> Result<T> {
        self.map_err(|e| PlannerError::database_error(message, e))
    }
    
    fn config_err(self, message: &str) -> Result<T> {
        self.map_err(|e| PlannerError::Configuration {
            message: format!("{}: {}", message, e),
        })
    }
}

/// Result type alias for planner operations
pub type Result<T> = std::result::Result<T, PlannerError>;
