//! Plan operations for the Planner.

use tokio::task;

use super::Planner;
use crate::{
    db::Database,
    error::{PlannerError, Result},
    models::{Plan, PlanFilter},
    params::{CreatePlan, Id, SearchPlans},
};

impl Planner {
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
}
