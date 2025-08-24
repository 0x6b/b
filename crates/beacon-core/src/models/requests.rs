//! Request types for updating models.

use super::StepStatus;

/// Parameters for updating a step to reduce function argument count
#[derive(Debug, Default)]
pub struct UpdateStepRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub acceptance_criteria: Option<String>,
    pub references: Option<Vec<String>>,
    pub status: Option<StepStatus>,
    pub result: Option<String>,
}

impl UpdateStepRequest {
    /// Create an UpdateStepRequest from individual validated parameters.
    ///
    /// This constructor method creates an UpdateStepRequest from pre-validated
    /// components, typically used when validation has already been performed
    /// elsewhere in the system.
    ///
    /// # Arguments
    ///
    /// * `title` - Optional new title for the step
    /// * `description` - Optional new description for the step
    /// * `acceptance_criteria` - Optional new acceptance criteria for the step
    /// * `references` - Optional new references list for the step
    /// * `status` - Optional validated StepStatus (already parsed and
    ///   validated)
    /// * `result` - Optional result description for the step
    ///
    /// # Returns
    ///
    /// A new UpdateStepRequest with all provided parameters set
    ///
    /// # Examples
    ///
    /// ```rust
    /// use beacon_core::models::{StepStatus, UpdateStepRequest};
    ///
    /// let request = UpdateStepRequest::new(
    ///     Some("Updated title".to_string()),
    ///     None,
    ///     None,
    ///     None,
    ///     Some(StepStatus::Done),
    ///     Some("Task completed successfully".to_string()),
    /// );
    ///
    /// assert_eq!(request.title, Some("Updated title".to_string()));
    /// assert_eq!(request.status, Some(StepStatus::Done));
    /// assert_eq!(
    ///     request.result,
    ///     Some("Task completed successfully".to_string())
    /// );
    /// ```
    pub fn new(
        title: Option<String>,
        description: Option<String>,
        acceptance_criteria: Option<String>,
        references: Option<Vec<String>>,
        status: Option<StepStatus>,
        result: Option<String>,
    ) -> Self {
        Self {
            title,
            description,
            acceptance_criteria,
            references,
            status,
            result,
        }
    }
}

impl TryFrom<crate::params::UpdateStep> for UpdateStepRequest {
    type Error = crate::PlannerError;

    /// Convert an UpdateStep parameter into a validated UpdateStepRequest.
    ///
    /// This trait implementation replaces the `create_update_request` function
    /// with an idiomatic Rust conversion. It performs validation of the status
    /// field and ensures result requirements are met for 'done' status.
    ///
    /// # Arguments
    ///
    /// * `params` - UpdateStep parameters from the params module
    ///
    /// # Returns
    ///
    /// A Result containing the validated UpdateStepRequest, or a PlannerError
    /// if validation fails
    ///
    /// # Errors
    ///
    /// * `PlannerError::InvalidInput` - When status string is invalid
    /// * `PlannerError::InvalidInput` - When result is missing for 'done'
    ///   status
    ///
    /// # Examples
    ///
    /// ```rust
    /// use beacon_core::{models::UpdateStepRequest, params::UpdateStep};
    ///
    /// // Valid conversion with status change
    /// let mut params = UpdateStep::default();
    /// params.id = 1;
    /// params.status = Some("done".to_string());
    /// params.result = Some("Completed successfully".to_string());
    /// params.title = Some("New title".to_string());
    ///
    /// let request: UpdateStepRequest = params.try_into()?;
    /// assert_eq!(request.title, Some("New title".to_string()));
    /// # use beacon_core::Result;
    /// # Result::<()>::Ok(())
    /// ```
    fn try_from(params: crate::params::UpdateStep) -> Result<Self, Self::Error> {
        // Use the existing validation method from UpdateStep
        let (validated_status, validated_result) = params.validate()?;

        Ok(Self {
            title: params.title,
            description: params.description,
            acceptance_criteria: params.acceptance_criteria,
            references: params.references,
            status: validated_status,
            result: validated_result,
        })
    }
}
