//! Step operations for the Planner.

use tokio::task;

use super::Planner;
use crate::{
    db::Database,
    error::{PlannerError, Result},
    models::{Step, UpdateStepRequest},
    params::{Id, InsertStep, StepCreate, SwapSteps},
};

impl Planner {
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
    /// InProgress. Returns the step details if successfully claimed, None if
    /// the step doesn't exist or cannot be claimed.
    pub async fn claim_step(&self, params: &Id) -> Result<Option<Step>> {
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
    pub async fn get_steps(&self, params: &Id) -> Result<crate::display::Steps> {
        let db_path = self.db_path.clone();
        let plan_id = params.id;

        let steps = task::spawn_blocking(move || {
            let db = Database::new(&db_path)?;
            db.get_steps(plan_id)
        })
        .await
        .map_err(|e| PlannerError::Configuration {
            message: format!("Task join error: {e}"),
        })??;

        Ok(crate::display::Steps(steps))
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
