//! Step handler operations that return formatted wrapper types for the Planner.

use crate::{
    error::Result,
    models::Step,
    params::{Id, InsertStep, StepCreate, SwapSteps, UpdateStep},
};

use super::Planner;

impl Planner {
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