//! Step handler operations that return formatted wrapper types for the Planner.

use super::Planner;
use crate::{
    error::Result,
    models::Step,
    params::{Id, UpdateStep},
};

impl Planner {
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
        let step = self.get_step(&Id { id: params.id }).await?;

        if step.is_some() {
            let update_request = params.clone().try_into()?;

            self.update_step(params.id, update_request).await?;

            self.get_step(&Id { id: params.id }).await
        } else {
            Ok(None)
        }
    }
}
