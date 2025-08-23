//! High-level planner API with async support.

use std::path::{Path, PathBuf};

use tokio::task;

use crate::{
    db::Database,
    error::{PlannerError, Result},
    models::{Plan, PlanFilter, Step, UpdateStepRequest},
    params::{CreatePlan, Id, InsertStep, SearchPlans, StepCreate, SwapSteps},
};

/// Main planner interface for managing plans and steps.
pub struct Planner {
    db_path: PathBuf,
}

impl Planner {
    /// Creates a new planner with the specified database path.
    fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }

    /// Creates a new plan with the given title, optional description, and
    /// optional directory. The directory path will always be stored as an
    /// absolute path. If a relative path is provided, it will be converted
    /// to absolute using the current working directory. If no directory is
    /// provided, the current working directory will be used.
    pub async fn create_plan(&self, params: &CreatePlan) -> Result<Plan> {
        let db_path = self.db_path.clone();
        let title = params.title.clone();
        let description = params.description.clone();
        let directory = params.directory.clone();

        task::spawn_blocking(move || {
            let mut db = Database::new(&db_path)?;
            db.create_plan(&title, description.as_deref(), directory.as_deref())
        })
        .await
        .map_err(|e| PlannerError::Configuration {
            message: format!("Task join error: {e}"),
        })?
    }

    /// Retrieves a plan by its ID.
    pub async fn get_plan(&self, params: &Id) -> Result<Option<Plan>> {
        let db_path = self.db_path.clone();
        let plan_id = params.id;

        task::spawn_blocking(move || {
            let db = Database::new(&db_path)?;
            db.get_plan(plan_id)
        })
        .await
        .map_err(|e| PlannerError::Configuration {
            message: format!("Task join error: {e}"),
        })?
    }

    /// Lists all plans with optional filtering.
    pub async fn list_plans(&self, filter: Option<PlanFilter>) -> Result<Vec<Plan>> {
        let db_path = self.db_path.clone();

        task::spawn_blocking(move || {
            let db = Database::new(&db_path)?;
            db.list_plans(filter.as_ref())
        })
        .await
        .map_err(|e| PlannerError::Configuration {
            message: format!("Task join error: {e}"),
        })?
    }

    /// Search for plans in a specific directory.
    /// The directory path can be relative or absolute.
    /// Returns all plans that have directories starting with the provided path.
    pub async fn search_plans_by_directory(&self, params: &SearchPlans) -> Result<Vec<Plan>> {
        let db_path = self.db_path.clone();
        let directory = params.directory.clone();

        // Canonicalize the directory path using the same logic as plan creation
        let canonicalized_directory = task::spawn_blocking(move || {
            let db = Database::new(&db_path)?;
            db.canonicalize_directory_for_search(&directory)
        })
        .await
        .map_err(|e| PlannerError::Configuration {
            message: format!("Task join error: {e}"),
        })??;

        let filter = PlanFilter {
            directory: Some(canonicalized_directory),
            ..Default::default()
        };
        self.list_plans(Some(filter)).await
    }

    /// Archives a plan (soft delete).
    pub async fn archive_plan(&self, params: &Id) -> Result<()> {
        let db_path = self.db_path.clone();
        let plan_id = params.id;

        task::spawn_blocking(move || {
            let mut db = Database::new(&db_path)?;
            db.archive_plan(plan_id)
        })
        .await
        .map_err(|e| PlannerError::Configuration {
            message: format!("Task join error: {e}"),
        })?
    }

    /// Unarchives a plan (restores from archive).
    pub async fn unarchive_plan(&self, params: &Id) -> Result<()> {
        let db_path = self.db_path.clone();
        let plan_id = params.id;

        task::spawn_blocking(move || {
            let mut db = Database::new(&db_path)?;
            db.unarchive_plan(plan_id)
        })
        .await
        .map_err(|e| PlannerError::Configuration {
            message: format!("Task join error: {e}"),
        })?
    }

    /// Permanently deletes a plan and all its associated steps.
    /// This operation cannot be undone.
    pub async fn delete_plan(&self, params: &Id) -> Result<()> {
        let db_path = self.db_path.clone();
        let plan_id = params.id;

        task::spawn_blocking(move || {
            let mut db = Database::new(&db_path)?;
            db.delete_plan(plan_id)
        })
        .await
        .map_err(|e| PlannerError::Configuration {
            message: format!("Task join error: {e}"),
        })?
    }

    /// Adds a new step to the specified plan with optional description,
    /// acceptance criteria and references.
    pub async fn add_step(&self, params: &StepCreate) -> Result<Step> {
        let db_path = self.db_path.clone();
        let title = params.title.clone();
        let description = params.description.clone();
        let acceptance_criteria = params.acceptance_criteria.clone();
        let references = params.references.clone();
        let plan_id = params.plan_id;

        task::spawn_blocking(move || {
            let mut db = Database::new(&db_path)?;
            db.add_step(
                plan_id,
                &title,
                description.as_deref(),
                acceptance_criteria.as_deref(),
                references,
            )
        })
        .await
        .map_err(|e| PlannerError::Configuration {
            message: format!("Task join error: {e}"),
        })?
    }

    /// Inserts a new step at a specific position in the plan's step order.
    pub async fn insert_step(&self, params: &InsertStep) -> Result<Step> {
        let db_path = self.db_path.clone();
        let title = params.step.title.clone();
        let description = params.step.description.clone();
        let acceptance_criteria = params.step.acceptance_criteria.clone();
        let references = params.step.references.clone();
        let plan_id = params.step.plan_id;
        let position = params.position;

        task::spawn_blocking(move || {
            let mut db = Database::new(&db_path)?;
            db.insert_step(
                plan_id,
                position,
                &title,
                description.as_deref(),
                acceptance_criteria.as_deref(),
                references,
            )
        })
        .await
        .map_err(|e| PlannerError::Configuration {
            message: format!("Task join error: {e}"),
        })?
    }

    /// Updates step details (title, description, acceptance criteria,
    /// references, and/or status).
    pub async fn update_step(&self, step_id: u64, request: UpdateStepRequest) -> Result<()> {
        let db_path = self.db_path.clone();

        task::spawn_blocking(move || {
            let mut db = Database::new(&db_path)?;
            db.update_step(step_id, request)
        })
        .await
        .map_err(|e| PlannerError::Configuration {
            message: format!("Task join error: {e}"),
        })?
    }

    /// Atomically claims a step for processing by transitioning it from Todo to
    /// InProgress. Returns Ok(true) if the step was successfully claimed,
    /// Ok(false) if the step was not in Todo status.
    pub async fn claim_step(&self, params: &Id) -> Result<bool> {
        let db_path = self.db_path.clone();
        let step_id = params.id;

        task::spawn_blocking(move || {
            let mut db = Database::new(&db_path)?;
            db.claim_step(step_id)
        })
        .await
        .map_err(|e| PlannerError::Configuration {
            message: format!("Task join error: {e}"),
        })?
    }

    /// Retrieves all steps for a given plan.
    pub async fn get_steps(&self, params: &Id) -> Result<Vec<Step>> {
        let db_path = self.db_path.clone();
        let plan_id = params.id;

        task::spawn_blocking(move || {
            let db = Database::new(&db_path)?;
            db.get_steps(plan_id)
        })
        .await
        .map_err(|e| PlannerError::Configuration {
            message: format!("Task join error: {e}"),
        })?
    }

    /// Retrieves a single step by its ID.
    pub async fn get_step(&self, params: &Id) -> Result<Option<Step>> {
        let db_path = self.db_path.clone();
        let step_id = params.id;

        task::spawn_blocking(move || {
            let db = Database::new(&db_path)?;
            db.get_step(step_id)
        })
        .await
        .map_err(|e| PlannerError::Configuration {
            message: format!("Task join error: {e}"),
        })?
    }

    /// Swaps the order of two steps within the same plan.
    pub async fn swap_steps(&self, params: &SwapSteps) -> Result<()> {
        let db_path = self.db_path.clone();
        let step1_id = params.step1_id;
        let step2_id = params.step2_id;

        task::spawn_blocking(move || {
            let mut db = Database::new(&db_path)?;
            db.swap_steps(step1_id, step2_id)
        })
        .await
        .map_err(|e| PlannerError::Configuration {
            message: format!("Task join error: {e}"),
        })?
    }

    /// Removes a step from a plan.
    pub async fn remove_step(&self, params: &Id) -> Result<()> {
        let db_path = self.db_path.clone();
        let step_id = params.id;

        task::spawn_blocking(move || {
            let mut db = Database::new(&db_path)?;
            db.remove_step(step_id)
        })
        .await
        .map_err(|e| PlannerError::Configuration {
            message: format!("Task join error: {e}"),
        })?
    }
}

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
