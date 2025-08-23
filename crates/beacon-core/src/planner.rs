//! High-level planner API with async support.

use std::path::{Path, PathBuf};

use tokio::task;

use crate::{
    db::Database,
    error::{PlannerError, Result},
    models::{Plan, PlanFilter, PlanSummary, Step, UpdateStepRequest},
    params::{CreatePlan, Id, InsertStep, ListPlans, SearchPlans, StepCreate, SwapSteps, UpdateStep},
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

    // Handler methods moved from handlers.rs
    // These methods provide the business logic that was previously in handler functions
    
    /// Handle listing plans with optional archived filtering.
    ///
    /// Converts plans to summaries with step count information for consistent
    /// list display across interfaces.
    ///
    /// # Arguments
    ///
    /// * `params` - List parameters containing archived flag
    ///
    /// # Returns
    ///
    /// A vector of PlanSummary objects with step counts
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use beacon_core::{params::ListPlans, PlannerBuilder};
    /// # async {
    /// let planner = PlannerBuilder::new().build().await?;
    /// let params = ListPlans { archived: false };
    /// let summaries = planner.list_plans_summary(&params).await?;
    /// # Result::<(), beacon_core::PlannerError>::Ok(())
    /// # };
    /// ```
    pub async fn list_plans_summary(&self, params: &ListPlans) -> Result<Vec<PlanSummary>> {
        let filter = Some(PlanFilter::from(params));
        let plans = self.list_plans(filter).await?;
        Ok(plans.iter().map(Into::into).collect())
    }

    /// Handle showing a complete plan with all its steps.
    ///
    /// Retrieves a plan with its associated steps eagerly loaded.
    /// The returned Plan object includes all steps in the steps field.
    ///
    /// # Arguments
    ///
    /// * `params` - ID parameters specifying which plan to retrieve
    ///
    /// # Returns
    ///
    /// An optional Plan containing the plan with its steps loaded,
    /// or None if the plan doesn't exist
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use beacon_core::{params::Id, PlannerBuilder};
    /// # async {
    /// let planner = PlannerBuilder::new().build().await?;
    /// let params = Id { id: 1 };
    /// let plan = planner.show_plan_with_steps(&params).await?;
    /// # Result::<(), beacon_core::PlannerError>::Ok(())
    /// # };
    /// ```
    pub async fn show_plan_with_steps(&self, params: &Id) -> Result<Option<Plan>> {
        self.get_plan(params).await
    }

    /// Handle creating a new plan.
    ///
    /// Creates a new plan with the specified parameters and returns
    /// the created plan object for confirmation.
    ///
    /// # Arguments
    ///
    /// * `params` - Creation parameters containing title and optional fields
    ///
    /// # Returns
    ///
    /// The newly created Plan object
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use beacon_core::{params::CreatePlan, PlannerBuilder};
    /// # async {
    /// let planner = PlannerBuilder::new().build().await?;
    /// let params = CreatePlan {
    ///     title: "My Plan".to_string(),
    ///     description: Some("A test plan".to_string()),
    ///     directory: None,
    /// };
    /// let plan = planner.create_plan_result(&params).await?;
    /// # Result::<(), beacon_core::PlannerError>::Ok(())
    /// # };
    /// ```
    pub async fn create_plan_result(&self, params: &CreatePlan) -> Result<Plan> {
        self.create_plan(params).await
    }

    /// Handle archiving a plan with confirmation.
    ///
    /// Archives the specified plan, making it inactive but preserving
    /// all data for potential restoration. Uses get-before-delete pattern
    /// to return the plan details for confirmation.
    ///
    /// # Arguments
    ///
    /// * `params` - ID parameters specifying which plan to archive
    ///
    /// # Returns
    ///
    /// An optional Plan object if the plan was found and archived,
    /// or None if the plan doesn't exist
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use beacon_core::{params::Id, PlannerBuilder};
    /// # async {
    /// let planner = PlannerBuilder::new().build().await?;
    /// let params = Id { id: 1 };
    /// let archived_plan = planner.archive_plan_with_confirmation(&params).await?;
    /// # Result::<(), beacon_core::PlannerError>::Ok(())
    /// # };
    /// ```
    pub async fn archive_plan_with_confirmation(&self, params: &Id) -> Result<Option<Plan>> {
        // Get plan details before archiving for confirmation
        let plan = self.get_plan(params).await?;

        if plan.is_some() {
            self.archive_plan(params).await?;
        }

        Ok(plan)
    }

    /// Handle unarchiving a plan with confirmation.
    ///
    /// Restores an archived plan to active status, making it visible
    /// in regular plan listings. Uses get-before-delete pattern
    /// to return the plan details for confirmation.
    ///
    /// # Arguments
    ///
    /// * `params` - ID parameters specifying which plan to unarchive
    ///
    /// # Returns
    ///
    /// An optional Plan object if the plan was found and unarchived,
    /// or None if the plan doesn't exist
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use beacon_core::{params::Id, PlannerBuilder};
    /// # async {
    /// let planner = PlannerBuilder::new().build().await?;
    /// let params = Id { id: 1 };
    /// let unarchived_plan = planner.unarchive_plan_with_confirmation(&params).await?;
    /// # Result::<(), beacon_core::PlannerError>::Ok(())
    /// # };
    /// ```
    pub async fn unarchive_plan_with_confirmation(&self, params: &Id) -> Result<Option<Plan>> {
        // Get plan details before unarchiving for confirmation
        let plan = self.get_plan(params).await?;

        if plan.is_some() {
            self.unarchive_plan(params).await?;
        }

        Ok(plan)
    }

    /// Handle permanently deleting a plan with confirmation.
    ///
    /// Permanently removes a plan and all its associated steps from the database.
    /// This operation cannot be undone. Uses get-before-delete pattern
    /// to return the plan details for confirmation.
    ///
    /// # Arguments
    ///
    /// * `params` - ID parameters specifying which plan to delete
    ///
    /// # Returns
    ///
    /// Returns the plan details that were deleted for confirmation,
    /// or None if the plan doesn't exist
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use beacon_core::{params::Id, PlannerBuilder};
    /// # async {
    /// let planner = PlannerBuilder::new().build().await?;
    /// let params = Id { id: 1 };
    /// let deleted_plan = planner.delete_plan_with_confirmation(&params).await?;
    /// # Result::<(), beacon_core::PlannerError>::Ok(())
    /// # };
    /// ```
    pub async fn delete_plan_with_confirmation(&self, params: &Id) -> Result<Option<Plan>> {
        // Get plan details before deleting for confirmation
        let plan = self.get_plan(params).await?;

        if plan.is_some() {
            self.delete_plan(params).await?;
        }

        Ok(plan)
    }

    /// Handle searching for plans in a specific directory with summaries.
    ///
    /// Searches for plans associated with the specified directory path,
    /// with optional archived filtering, and returns them as summaries
    /// with step counts. Includes conditional logic for archived vs active plans.
    ///
    /// # Arguments
    ///
    /// * `params` - Search parameters containing directory and archived flag
    ///
    /// # Returns
    ///
    /// A vector of PlanSummary objects matching the search criteria
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use beacon_core::{params::SearchPlans, PlannerBuilder};
    /// # async {
    /// let planner = PlannerBuilder::new().build().await?;
    /// let params = SearchPlans {
    ///     directory: "/path/to/project".to_string(),
    ///     archived: false,
    /// };
    /// let summaries = planner.search_plans_summary(&params).await?;
    /// # Result::<(), beacon_core::PlannerError>::Ok(())
    /// # };
    /// ```
    pub async fn search_plans_summary(&self, params: &SearchPlans) -> Result<Vec<PlanSummary>> {
        let plans = if params.archived {
            // For archived plans, use list_plans with directory filter
            let filter = PlanFilter::for_directory(params.directory.clone(), true);
            self.list_plans(Some(filter)).await?
        } else {
            // For active plans, use the specialized search method
            self.search_plans_by_directory(params).await?
        };

        Ok(plans.iter().map(Into::into).collect())
    }

    /// Handle adding a step to a plan.
    ///
    /// Creates a new step with the specified parameters and returns
    /// the created step object for confirmation.
    ///
    /// # Arguments
    ///
    /// * `params` - Step creation parameters
    ///
    /// # Returns
    ///
    /// The newly created Step object
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use beacon_core::{params::StepCreate, PlannerBuilder};
    /// # async {
    /// let planner = PlannerBuilder::new().build().await?;
    /// let params = StepCreate {
    ///     plan_id: 1,
    ///     title: "Test Step".to_string(),
    ///     description: Some("A test step".to_string()),
    ///     acceptance_criteria: None,
    ///     references: vec![],
    /// };
    /// let step = planner.add_step_to_plan(&params).await?;
    /// # Result::<(), beacon_core::PlannerError>::Ok(())
    /// # };
    /// ```
    pub async fn add_step_to_plan(&self, params: &StepCreate) -> Result<Step> {
        self.add_step(params).await
    }

    /// Handle inserting a step at a specific position in a plan.
    ///
    /// Creates a new step and inserts it at the specified position,
    /// shifting other steps as needed.
    ///
    /// # Arguments
    ///
    /// * `params` - Step insertion parameters including position
    ///
    /// # Returns
    ///
    /// The newly created Step object
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use beacon_core::{params::{InsertStep, StepCreate}, PlannerBuilder};
    /// # async {
    /// let planner = PlannerBuilder::new().build().await?;
    /// let params = InsertStep {
    ///     step: StepCreate {
    ///         plan_id: 1,
    ///         title: "Test Step".to_string(),
    ///         description: Some("A test step".to_string()),
    ///         acceptance_criteria: None,
    ///         references: vec![],
    ///     },
    ///     position: 2,
    /// };
    /// let step = planner.insert_step_to_plan(&params).await?;
    /// # Result::<(), beacon_core::PlannerError>::Ok(())
    /// # };
    /// ```
    pub async fn insert_step_to_plan(&self, params: &InsertStep) -> Result<Step> {
        self.insert_step(params).await
    }

    /// Handle updating a step's properties with validation.
    ///
    /// Updates the specified step with new values, performing validation
    /// for status changes and result requirements using parameter validation.
    ///
    /// # Arguments
    ///
    /// * `params` - Update parameters containing step ID and new values
    ///
    /// # Returns
    ///
    /// An optional Step object if the step was found and updated,
    /// or None if the step doesn't exist
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use beacon_core::{params::UpdateStep, PlannerBuilder};
    /// # async {
    /// let planner = PlannerBuilder::new().build().await?;
    /// let params = UpdateStep {
    ///     id: 1,
    ///     status: Some("done".to_string()),
    ///     title: None,
    ///     description: None,
    ///     acceptance_criteria: None,
    ///     references: None,
    ///     result: Some("Completed successfully".to_string()),
    /// };
    /// let updated_step = planner.update_step_validated(&params).await?;
    /// # Result::<(), beacon_core::PlannerError>::Ok(())
    /// # };
    /// ```
    pub async fn update_step_validated(&self, params: &UpdateStep) -> Result<Option<Step>> {
        // Get step before update for confirmation
        let step = self.get_step(&Id { id: params.id }).await?;

        if step.is_some() {
            // Create validated update request using TryFrom trait
            let update_request = params.clone().try_into()?;

            // Perform the update
            self.update_step(params.id, update_request).await?;

            // Return the updated step
            self.get_step(&Id { id: params.id }).await
        } else {
            Ok(None)
        }
    }

    /// Handle showing a specific step.
    ///
    /// Retrieves detailed information about a single step.
    ///
    /// # Arguments
    ///
    /// * `params` - ID parameters specifying which step to retrieve
    ///
    /// # Returns
    ///
    /// An optional Step object if the step exists, or None if not found
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use beacon_core::{params::Id, PlannerBuilder};
    /// # async {
    /// let planner = PlannerBuilder::new().build().await?;
    /// let params = Id { id: 1 };
    /// let step = planner.show_step_details(&params).await?;
    /// # Result::<(), beacon_core::PlannerError>::Ok(())
    /// # };
    /// ```
    pub async fn show_step_details(&self, params: &Id) -> Result<Option<Step>> {
        self.get_step(params).await
    }

    /// Handle claiming a step for processing atomically.
    ///
    /// Atomically transitions a step from 'todo' to 'inprogress' status,
    /// preventing multiple agents from working on the same task.
    ///
    /// # Arguments
    ///
    /// * `params` - ID parameters specifying which step to claim
    ///
    /// # Returns
    ///
    /// A boolean indicating whether the step was successfully claimed
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use beacon_core::{params::Id, PlannerBuilder};
    /// # async {
    /// let planner = PlannerBuilder::new().build().await?;
    /// let params = Id { id: 1 };
    /// let claimed = planner.claim_step_atomically(&params).await?;
    /// # Result::<(), beacon_core::PlannerError>::Ok(())
    /// # };
    /// ```
    pub async fn claim_step_atomically(&self, params: &Id) -> Result<bool> {
        self.claim_step(params).await
    }

    /// Handle swapping the order of two steps.
    ///
    /// Reorders steps within a plan by swapping their positions,
    /// useful for task prioritization.
    ///
    /// # Arguments
    ///
    /// * `params` - Swap parameters containing both step IDs
    ///
    /// # Returns
    ///
    /// Unit result indicating success or failure of the operation
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use beacon_core::{params::SwapSteps, PlannerBuilder};
    /// # async {
    /// let planner = PlannerBuilder::new().build().await?;
    /// let params = SwapSteps {
    ///     step1_id: 1,
    ///     step2_id: 2,
    /// };
    /// planner.swap_step_positions(&params).await?;
    /// # Result::<(), beacon_core::PlannerError>::Ok(())
    /// # };
    /// ```
    pub async fn swap_step_positions(&self, params: &SwapSteps) -> Result<()> {
        self.swap_steps(params).await
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::{ListPlans, SearchPlans, UpdateStep};
    use tempfile::TempDir;

    /// Helper function to create a test planner
    async fn create_test_planner() -> (TempDir, Planner) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let planner = PlannerBuilder::new()
            .with_database_path(&db_path)
            .build()
            .await
            .expect("Failed to create planner");
        (temp_dir, planner)
    }

    #[tokio::test]
    async fn test_list_plans_summary_active() {
        let (_temp_dir, planner) = create_test_planner().await;

        // Create a plan
        let plan = planner
            .create_plan(&CreatePlan {
                title: "Test Plan".to_string(),
                description: Some("Test Description".to_string()),
                directory: None,
            })
            .await
            .expect("Failed to create plan");

        // Add a step
        planner
            .add_step(&StepCreate {
                plan_id: plan.id,
                title: "Test Step".to_string(),
                description: None,
                acceptance_criteria: None,
                references: vec![],
            })
            .await
            .expect("Failed to add step");

        // Test list_plans_summary for active plans
        let summaries = planner
            .list_plans_summary(&ListPlans { archived: false })
            .await
            .expect("Failed to list plan summaries");

        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].title, "Test Plan");
        assert_eq!(summaries[0].description, Some("Test Description".to_string()));
        assert_eq!(summaries[0].total_steps, 1);
        assert_eq!(summaries[0].completed_steps, 0);
    }

    #[tokio::test]
    async fn test_list_plans_summary_archived() {
        let (_temp_dir, planner) = create_test_planner().await;

        // Create and archive a plan
        let plan = planner
            .create_plan(&CreatePlan {
                title: "Archived Plan".to_string(),
                description: None,
                directory: None,
            })
            .await
            .expect("Failed to create plan");

        planner
            .archive_plan(&Id { id: plan.id })
            .await
            .expect("Failed to archive plan");

        // Test list_plans_summary for archived plans
        let summaries = planner
            .list_plans_summary(&ListPlans { archived: true })
            .await
            .expect("Failed to list archived plan summaries");

        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].title, "Archived Plan");

        // Verify active plans is empty
        let active_summaries = planner
            .list_plans_summary(&ListPlans { archived: false })
            .await
            .expect("Failed to list active plans");
        assert_eq!(active_summaries.len(), 0);
    }

    #[tokio::test]
    async fn test_show_plan_with_steps() {
        let (_temp_dir, planner) = create_test_planner().await;

        // Create a plan with steps
        let plan = planner
            .create_plan(&CreatePlan {
                title: "Plan with Steps".to_string(),
                description: None,
                directory: None,
            })
            .await
            .expect("Failed to create plan");

        planner
            .add_step(&StepCreate {
                plan_id: plan.id,
                title: "Step 1".to_string(),
                description: None,
                acceptance_criteria: None,
                references: vec![],
            })
            .await
            .expect("Failed to add step");

        planner
            .add_step(&StepCreate {
                plan_id: plan.id,
                title: "Step 2".to_string(),
                description: None,
                acceptance_criteria: None,
                references: vec![],
            })
            .await
            .expect("Failed to add step");

        // Test show_plan_with_steps
        let retrieved_plan = planner
            .show_plan_with_steps(&Id { id: plan.id })
            .await
            .expect("Failed to show plan with steps")
            .expect("Plan should exist");

        assert_eq!(retrieved_plan.title, "Plan with Steps");
        assert_eq!(retrieved_plan.steps.len(), 2);
        assert_eq!(retrieved_plan.steps[0].title, "Step 1");
        assert_eq!(retrieved_plan.steps[1].title, "Step 2");
    }

    #[tokio::test]
    async fn test_show_plan_with_steps_not_found() {
        let (_temp_dir, planner) = create_test_planner().await;

        // Test non-existent plan
        let result = planner
            .show_plan_with_steps(&Id { id: 999 })
            .await
            .expect("Should not fail on non-existent plan");

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_archive_plan_with_confirmation() {
        let (_temp_dir, planner) = create_test_planner().await;

        // Create a plan
        let plan = planner
            .create_plan(&CreatePlan {
                title: "To Archive".to_string(),
                description: None,
                directory: None,
            })
            .await
            .expect("Failed to create plan");

        // Test archive_plan_with_confirmation
        let archived_plan = planner
            .archive_plan_with_confirmation(&Id { id: plan.id })
            .await
            .expect("Failed to archive plan with confirmation")
            .expect("Plan should exist");

        assert_eq!(archived_plan.title, "To Archive");
        assert_eq!(archived_plan.id, plan.id);

        // Verify plan is actually archived by checking it's not in active list
        let active_plans = planner
            .list_plans(None)
            .await
            .expect("Failed to list plans");
        assert!(!active_plans.iter().any(|p| p.id == plan.id));
    }

    #[tokio::test]
    async fn test_archive_plan_with_confirmation_not_found() {
        let (_temp_dir, planner) = create_test_planner().await;

        // Test non-existent plan
        let result = planner
            .archive_plan_with_confirmation(&Id { id: 999 })
            .await
            .expect("Should not fail on non-existent plan");

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_unarchive_plan_with_confirmation() {
        let (_temp_dir, planner) = create_test_planner().await;

        // Create and archive a plan
        let plan = planner
            .create_plan(&CreatePlan {
                title: "To Unarchive".to_string(),
                description: None,
                directory: None,
            })
            .await
            .expect("Failed to create plan");

        planner
            .archive_plan(&Id { id: plan.id })
            .await
            .expect("Failed to archive plan");

        // Test unarchive_plan_with_confirmation
        let unarchived_plan = planner
            .unarchive_plan_with_confirmation(&Id { id: plan.id })
            .await
            .expect("Failed to unarchive plan with confirmation")
            .expect("Plan should exist");

        assert_eq!(unarchived_plan.title, "To Unarchive");
        assert_eq!(unarchived_plan.id, plan.id);

        // Verify plan is back in active list
        let active_plans = planner
            .list_plans(None)
            .await
            .expect("Failed to list plans");
        assert!(active_plans.iter().any(|p| p.id == plan.id));
    }

    #[tokio::test]
    async fn test_delete_plan_with_confirmation() {
        let (_temp_dir, planner) = create_test_planner().await;

        // Create a plan with steps
        let plan = planner
            .create_plan(&CreatePlan {
                title: "To Delete".to_string(),
                description: None,
                directory: None,
            })
            .await
            .expect("Failed to create plan");

        planner
            .add_step(&StepCreate {
                plan_id: plan.id,
                title: "Step to Delete".to_string(),
                description: None,
                acceptance_criteria: None,
                references: vec![],
            })
            .await
            .expect("Failed to add step");

        // Test delete_plan_with_confirmation
        let deleted_plan = planner
            .delete_plan_with_confirmation(&Id { id: plan.id })
            .await
            .expect("Failed to delete plan with confirmation")
            .expect("Plan should exist");

        assert_eq!(deleted_plan.title, "To Delete");
        assert_eq!(deleted_plan.id, plan.id);

        // Verify plan and steps are gone
        let result = planner
            .get_plan(&Id { id: plan.id })
            .await
            .expect("Should not fail on deleted plan");
        assert!(result.is_none());

        let steps = planner
            .get_steps(&Id { id: plan.id })
            .await
            .expect("Failed to get steps");
        assert_eq!(steps.len(), 0);
    }

    #[tokio::test]
    async fn test_search_plans_summary() {
        let (_temp_dir, planner) = create_test_planner().await;
        let test_dir = "/test/directory";

        // Create plans in different directories
        let plan1 = planner
            .create_plan(&CreatePlan {
                title: "Plan in Test Dir".to_string(),
                description: None,
                directory: Some(test_dir.to_string()),
            })
            .await
            .expect("Failed to create plan");

        planner
            .create_plan(&CreatePlan {
                title: "Plan in Other Dir".to_string(),
                description: None,
                directory: Some("/other/directory".to_string()),
            })
            .await
            .expect("Failed to create plan");

        // Add steps to the first plan
        planner
            .add_step(&StepCreate {
                plan_id: plan1.id,
                title: "Test Step".to_string(),
                description: None,
                acceptance_criteria: None,
                references: vec![],
            })
            .await
            .expect("Failed to add step");

        // Test search_plans_summary for active plans
        let summaries = planner
            .search_plans_summary(&SearchPlans {
                directory: test_dir.to_string(),
                archived: false,
            })
            .await
            .expect("Failed to search plans");

        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].title, "Plan in Test Dir");
        assert_eq!(summaries[0].total_steps, 1);
    }

    #[tokio::test]
    async fn test_search_plans_summary_archived() {
        let (_temp_dir, planner) = create_test_planner().await;
        let test_dir = "/test/directory";

        // Create and archive a plan
        let plan = planner
            .create_plan(&CreatePlan {
                title: "Archived Plan in Dir".to_string(),
                description: None,
                directory: Some(test_dir.to_string()),
            })
            .await
            .expect("Failed to create plan");

        planner
            .archive_plan(&Id { id: plan.id })
            .await
            .expect("Failed to archive plan");

        // Test search for archived plans
        let summaries = planner
            .search_plans_summary(&SearchPlans {
                directory: test_dir.to_string(),
                archived: true,
            })
            .await
            .expect("Failed to search archived plans");

        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].title, "Archived Plan in Dir");

        // Verify active search returns empty
        let active_summaries = planner
            .search_plans_summary(&SearchPlans {
                directory: test_dir.to_string(),
                archived: false,
            })
            .await
            .expect("Failed to search active plans");
        assert_eq!(active_summaries.len(), 0);
    }

    #[tokio::test]
    async fn test_update_step_validated() {
        let (_temp_dir, planner) = create_test_planner().await;

        // Create a plan and step
        let plan = planner
            .create_plan(&CreatePlan {
                title: "Update Test".to_string(),
                description: None,
                directory: None,
            })
            .await
            .expect("Failed to create plan");

        let step = planner
            .add_step(&StepCreate {
                plan_id: plan.id,
                title: "Step to Update".to_string(),
                description: None,
                acceptance_criteria: None,
                references: vec![],
            })
            .await
            .expect("Failed to add step");

        // Test update_step_validated
        let updated_step = planner
            .update_step_validated(&UpdateStep {
                id: step.id,
                status: Some("done".to_string()),
                title: Some("Updated Step Title".to_string()),
                description: Some("Updated description".to_string()),
                acceptance_criteria: None,
                references: None,
                result: Some("Step completed successfully".to_string()),
            })
            .await
            .expect("Failed to update step")
            .expect("Step should exist");

        assert_eq!(updated_step.title, "Updated Step Title");
        assert_eq!(updated_step.description, Some("Updated description".to_string()));
        assert_eq!(updated_step.result, Some("Step completed successfully".to_string()));
    }

    #[tokio::test]
    async fn test_update_step_validated_not_found() {
        let (_temp_dir, planner) = create_test_planner().await;

        // Test non-existent step
        let result = planner
            .update_step_validated(&UpdateStep {
                id: 999,
                status: Some("done".to_string()),
                title: None,
                description: None,
                acceptance_criteria: None,
                references: None,
                result: Some("Test result".to_string()),
            })
            .await
            .expect("Should not fail on non-existent step");

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_claim_step_atomically() {
        let (_temp_dir, planner) = create_test_planner().await;

        // Create a plan and step
        let plan = planner
            .create_plan(&CreatePlan {
                title: "Claim Test".to_string(),
                description: None,
                directory: None,
            })
            .await
            .expect("Failed to create plan");

        let step = planner
            .add_step(&StepCreate {
                plan_id: plan.id,
                title: "Step to Claim".to_string(),
                description: None,
                acceptance_criteria: None,
                references: vec![],
            })
            .await
            .expect("Failed to add step");

        // Test claim_step_atomically
        let claimed = planner
            .claim_step_atomically(&Id { id: step.id })
            .await
            .expect("Failed to claim step");

        assert!(claimed, "Step should be successfully claimed");

        // Verify step is in progress
        let retrieved_step = planner
            .get_step(&Id { id: step.id })
            .await
            .expect("Failed to get step")
            .expect("Step should exist");

        use crate::models::StepStatus;
        assert_eq!(retrieved_step.status, StepStatus::InProgress);

        // Test claiming already claimed step
        let claimed_again = planner
            .claim_step_atomically(&Id { id: step.id })
            .await
            .expect("Failed to attempt claiming again");

        assert!(!claimed_again, "Step should not be claimed again");
    }

    #[tokio::test]
    async fn test_add_step_to_plan() {
        let (_temp_dir, planner) = create_test_planner().await;

        // Create a plan
        let plan = planner
            .create_plan(&CreatePlan {
                title: "Add Step Test".to_string(),
                description: None,
                directory: None,
            })
            .await
            .expect("Failed to create plan");

        // Test add_step_to_plan
        let step = planner
            .add_step_to_plan(&StepCreate {
                plan_id: plan.id,
                title: "New Step".to_string(),
                description: Some("Step description".to_string()),
                acceptance_criteria: Some("Must be completed".to_string()),
                references: vec!["file1.rs".to_string(), "file2.rs".to_string()],
            })
            .await
            .expect("Failed to add step to plan");

        assert_eq!(step.title, "New Step");
        assert_eq!(step.description, Some("Step description".to_string()));
        assert_eq!(step.acceptance_criteria, Some("Must be completed".to_string()));
        assert_eq!(step.references, vec!["file1.rs".to_string(), "file2.rs".to_string()]);
    }

    #[tokio::test]
    async fn test_insert_step_to_plan() {
        let (_temp_dir, planner) = create_test_planner().await;

        // Create a plan with existing steps
        let plan = planner
            .create_plan(&CreatePlan {
                title: "Insert Step Test".to_string(),
                description: None,
                directory: None,
            })
            .await
            .expect("Failed to create plan");

        planner
            .add_step(&StepCreate {
                plan_id: plan.id,
                title: "First Step".to_string(),
                description: None,
                acceptance_criteria: None,
                references: vec![],
            })
            .await
            .expect("Failed to add first step");

        planner
            .add_step(&StepCreate {
                plan_id: plan.id,
                title: "Third Step".to_string(),
                description: None,
                acceptance_criteria: None,
                references: vec![],
            })
            .await
            .expect("Failed to add third step");

        // Test insert_step_to_plan at position 1 (between first and third)
        let inserted_step = planner
            .insert_step_to_plan(&InsertStep {
                step: StepCreate {
                    plan_id: plan.id,
                    title: "Second Step".to_string(),
                    description: None,
                    acceptance_criteria: None,
                    references: vec![],
                },
                position: 1,
            })
            .await
            .expect("Failed to insert step");

        assert_eq!(inserted_step.title, "Second Step");

        // Verify order
        let steps = planner
            .get_steps(&Id { id: plan.id })
            .await
            .expect("Failed to get steps");

        assert_eq!(steps.len(), 3);
        assert_eq!(steps[0].title, "First Step");
        assert_eq!(steps[1].title, "Second Step");
        assert_eq!(steps[2].title, "Third Step");
    }

    #[tokio::test]
    async fn test_show_step_details() {
        let (_temp_dir, planner) = create_test_planner().await;

        // Create a plan and step
        let plan = planner
            .create_plan(&CreatePlan {
                title: "Step Details Test".to_string(),
                description: None,
                directory: None,
            })
            .await
            .expect("Failed to create plan");

        let step = planner
            .add_step(&StepCreate {
                plan_id: plan.id,
                title: "Detailed Step".to_string(),
                description: Some("Detailed description".to_string()),
                acceptance_criteria: Some("Must pass all tests".to_string()),
                references: vec!["test.rs".to_string()],
            })
            .await
            .expect("Failed to add step");

        // Test show_step_details
        let retrieved_step = planner
            .show_step_details(&Id { id: step.id })
            .await
            .expect("Failed to show step details")
            .expect("Step should exist");

        assert_eq!(retrieved_step.title, "Detailed Step");
        assert_eq!(retrieved_step.description, Some("Detailed description".to_string()));
        assert_eq!(retrieved_step.acceptance_criteria, Some("Must pass all tests".to_string()));
        assert_eq!(retrieved_step.references, vec!["test.rs".to_string()]);
    }

    #[tokio::test]
    async fn test_swap_step_positions() {
        let (_temp_dir, planner) = create_test_planner().await;

        // Create a plan with multiple steps
        let plan = planner
            .create_plan(&CreatePlan {
                title: "Swap Test".to_string(),
                description: None,
                directory: None,
            })
            .await
            .expect("Failed to create plan");

        let step1 = planner
            .add_step(&StepCreate {
                plan_id: plan.id,
                title: "First Step".to_string(),
                description: None,
                acceptance_criteria: None,
                references: vec![],
            })
            .await
            .expect("Failed to add step 1");

        let _step2 = planner
            .add_step(&StepCreate {
                plan_id: plan.id,
                title: "Second Step".to_string(),
                description: None,
                acceptance_criteria: None,
                references: vec![],
            })
            .await
            .expect("Failed to add step 2");

        let step3 = planner
            .add_step(&StepCreate {
                plan_id: plan.id,
                title: "Third Step".to_string(),
                description: None,
                acceptance_criteria: None,
                references: vec![],
            })
            .await
            .expect("Failed to add step 3");

        // Test swap_step_positions
        planner
            .swap_step_positions(&SwapSteps {
                step1_id: step1.id,
                step2_id: step3.id,
            })
            .await
            .expect("Failed to swap steps");

        // Verify new order
        let steps = planner
            .get_steps(&Id { id: plan.id })
            .await
            .expect("Failed to get steps");

        assert_eq!(steps.len(), 3);
        assert_eq!(steps[0].title, "Third Step"); // step3 is now first
        assert_eq!(steps[1].title, "Second Step"); // step2 stays in middle
        assert_eq!(steps[2].title, "First Step"); // step1 is now last
    }
}
