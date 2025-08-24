//! Error types for the planner library.

use std::fmt;
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

/// Builder for creating database errors with optional context.
pub struct DatabaseErrorBuilder {
    message: String,
}

impl DatabaseErrorBuilder {
    /// Create a new database error builder with a message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// Build the error with the given source.
    pub fn with_source(self, source: rusqlite::Error) -> PlannerError {
        PlannerError::Database {
            message: self.message,
            source,
        }
    }
}

/// Builder for creating input validation errors.
pub struct InvalidInputBuilder {
    field: String,
}

impl InvalidInputBuilder {
    /// Create a new invalid input error builder for a field.
    pub fn new(field: impl Into<String>) -> Self {
        Self {
            field: field.into(),
        }
    }

    /// Build the error with the given reason.
    pub fn with_reason(self, reason: impl Into<String>) -> PlannerError {
        PlannerError::InvalidInput {
            field: self.field,
            reason: reason.into(),
        }
    }
}

impl PlannerError {
    /// Creates a builder for database errors.
    pub fn database(message: impl Into<String>) -> DatabaseErrorBuilder {
        DatabaseErrorBuilder::new(message)
    }

    /// Creates a builder for input validation errors.
    pub fn invalid_input(field: impl Into<String>) -> InvalidInputBuilder {
        InvalidInputBuilder::new(field)
    }

    /// Creates a new database error with additional context (legacy method)
    pub fn database_error(message: &str, source: rusqlite::Error) -> Self {
        Self::database(message).with_source(source)
    }
}

/// Extension trait for Result to provide concise error mapping with
/// anyhow-style context.
pub trait ResultExt<T, E> {
    /// Add context to any error type, converting to PlannerError.
    fn with_context<C>(self, context: C) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static;

    /// Add lazy context to any error type, converting to PlannerError.
    fn with_context_lazy<C, F>(self, f: F) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C;
}

/// Specialized extension trait for database-related Results.
pub trait DatabaseResultExt<T> {
    /// Map database errors with a message.
    fn db_context(self, message: &str) -> Result<T>;
}

/// Specialized extension trait for configuration-related Results.  
pub trait ConfigResultExt<T> {
    /// Map configuration errors with a message.
    fn config_context(self, message: &str) -> Result<T>;
}

impl<T, E> ResultExt<T, E> for std::result::Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn with_context<C>(self, context: C) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
    {
        self.map_err(|e| PlannerError::Configuration {
            message: format!("{}: {}", context, e),
        })
    }

    fn with_context_lazy<C, F>(self, f: F) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.map_err(|e| PlannerError::Configuration {
            message: format!("{}: {}", f(), e),
        })
    }
}

impl<T> DatabaseResultExt<T> for std::result::Result<T, rusqlite::Error> {
    fn db_context(self, message: &str) -> Result<T> {
        self.map_err(|e| PlannerError::database(message).with_source(e))
    }
}

impl<T> ConfigResultExt<T> for std::result::Result<T, rusqlite::Error> {
    fn config_context(self, message: &str) -> Result<T> {
        self.map_err(|e| PlannerError::Configuration {
            message: format!("{}: {}", message, e),
        })
    }
}

/// Legacy extension trait for backwards compatibility with existing code.
pub trait ResultExtLegacy<T> {
    /// Map database errors with a message (legacy)
    fn db_err(self, message: &str) -> Result<T>;

    /// Map configuration errors with a message (legacy)  
    fn config_err(self, message: &str) -> Result<T>;
}

impl<T> ResultExtLegacy<T> for std::result::Result<T, rusqlite::Error> {
    fn db_err(self, message: &str) -> Result<T> {
        self.db_context(message)
    }

    fn config_err(self, message: &str) -> Result<T> {
        self.config_context(message)
    }
}

/// Result type alias for planner operations
pub type Result<T> = std::result::Result<T, PlannerError>;
