//! Builder for creating and configuring Planner instances.

use std::path::{Path, PathBuf};

use tokio::task;

use super::Planner;
use crate::{
    db::Database,
    error::{PlannerError, Result},
};

/// Builder for creating and configuring Planner instances.
#[derive(Debug, Clone)]
pub struct PlannerBuilder {
    database_path: Option<PathBuf>,
    connection_pool_size: usize,
    read_only: bool,
}

impl PlannerBuilder {
    /// Creates a new builder with default settings.
    pub fn new() -> Self {
        Self {
            database_path: None,
            connection_pool_size: 1,
            read_only: false,
        }
    }

    /// Sets a custom database file path.
    ///
    /// If not specified, uses XDG Base Directory specification:
    /// `$XDG_DATA_HOME/beacon/tasks.db` or `~/.local/share/beacon/tasks.db`
    pub fn with_database_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.database_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Sets the connection pool size for concurrent operations.
    ///
    /// Default is 1 (single connection). Higher values may improve
    /// performance for concurrent workloads but increase memory usage.
    pub fn with_connection_pool_size(mut self, size: usize) -> Self {
        self.connection_pool_size = size.max(1);
        self
    }

    /// Opens the database in read-only mode.
    ///
    /// Useful for querying operations where mutations are not required.
    /// Provides additional safety and may enable optimizations.
    pub fn read_only(mut self) -> Self {
        self.read_only = true;
        self
    }

    /// Builds the configured planner instance.
    ///
    /// # Errors
    ///
    /// Returns `PlannerError::FileSystem` if the database path is invalid
    /// Returns `PlannerError::Database` if database initialization fails
    pub async fn build(self) -> Result<Planner> {
        let db_path = match self.database_path {
            Some(path) => path,
            None => Self::default_database_path()?,
        };

        // Create parent directories if they don't exist
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| PlannerError::FileSystem {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }

        // Test database connection
        let db_path_clone = db_path.clone();
        task::spawn_blocking(move || {
            let _db = Database::new(&db_path_clone)?;
            Ok::<(), PlannerError>(())
        })
        .await
        .map_err(|e| PlannerError::Configuration {
            message: format!("Task join error: {e}"),
        })??;

        Ok(Planner::new(db_path))
    }

    /// Returns the default database path following XDG Base Directory
    /// specification.
    fn default_database_path() -> Result<PathBuf> {
        xdg::BaseDirectories::with_prefix("beacon")
            .place_data_file("beacon.db")
            .map_err(|e| PlannerError::XdgDirectory(e.to_string()))
    }
}

impl Default for PlannerBuilder {
    fn default() -> Self {
        Self::new()
    }
}
