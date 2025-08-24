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
}

impl PlannerBuilder {
    /// Creates a new builder with default settings.
    pub fn new() -> Self {
        Self {
            database_path: None,
        }
    }

    /// Sets a custom database file path.
    ///
    /// If not specified, uses XDG Base Directory specification:
    /// `$XDG_DATA_HOME/beacon/tasks.db` or `~/.local/share/beacon/tasks.db`
    pub fn with_database_path<P: AsRef<Path>>(mut self, path: Option<P>) -> Self {
        if let Some(path) = path {
            self.database_path = Some(path.as_ref().to_path_buf());
        }
        self
    }

    /// Builds the configured planner instance.
    ///
    /// # Errors
    ///
    /// Returns `PlannerError::FileSystem` if the database path is invalid
    /// Returns `PlannerError::Database` if database initialization fails
    pub async fn build(self) -> Result<Planner> {
        let db_path = if let Some(path) = self.database_path {
            path
        } else {
            Self::default_database_path()?
        };

        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| PlannerError::FileSystem {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }

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
