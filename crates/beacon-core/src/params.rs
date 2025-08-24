//! Parameter structures for Beacon operations.
//!
//! This module contains shared parameter structures that can be used across
//! different interfaces (CLI, MCP, etc.) without framework-specific derives or
//! dependencies.

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Generic parameters for operations requiring just an ID.
///
/// Used for operations like show_plan, archive_plan, unarchive_plan, show_step,
/// claim_step.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct Id {
    /// The ID of the resource to operate on
    pub id: u64,
}

/// Parameters for creating a new plan.
///
/// Used to create a new task plan with a title, optional description, and
/// optional working directory association.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct CreatePlan {
    /// Title of the plan (required)
    pub title: String,
    /// Optional detailed description of the plan
    pub description: Option<String>,
    /// Optional working directory for the plan
    pub directory: Option<String>,
}

/// Parameters for listing plans.
///
/// Controls whether to show archived or active plans.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct ListPlans {
    /// Whether to show archived plans instead of active ones
    #[serde(default)]
    pub archived: bool,
}

/// Parameters for searching plans by directory.
///
/// Allows filtering plans by directory path and archived status.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct SearchPlans {
    /// Directory path to search for plans
    pub directory: String,
    /// Whether to include archived plans in search results
    #[serde(default)]
    pub archived: bool,
}

/// Parameters for deleting a plan.
///
/// Requires explicit confirmation to prevent accidental deletion of plans
/// and their associated steps. Deletion is permanent and cannot be undone.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct DeletePlan {
    /// The ID of the plan to delete
    pub id: u64,
    /// Confirmation flag required to prevent accidental deletion
    pub confirmed: bool,
}

/// Base parameters for step creation and modification.
///
/// Contains the common fields used when creating or modifying steps.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct StepCreate {
    /// ID of the plan to add the step to
    pub plan_id: u64,
    /// Title of the step (required)
    pub title: String,
    /// Optional detailed description of the step
    pub description: Option<String>,
    /// Optional acceptance criteria for the step
    pub acceptance_criteria: Option<String>,
    /// References (URLs, file paths, etc.)
    #[serde(default)]
    pub references: Vec<String>,
}

/// Parameters for inserting a step at a specific position.
///
/// Extends step creation parameters with position information for inserting
/// steps at specific locations within a plan's step order.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct InsertStep {
    /// Base step creation parameters
    #[serde(flatten)]
    pub step: StepCreate,
    /// Position to insert the step (0-indexed)
    pub position: u32,
}

/// Parameters for swapping the order of two steps.
///
/// Used to reorder steps within a plan by swapping their positions.
/// Both steps must belong to the same plan.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct SwapSteps {
    /// ID of the first step to swap
    pub step1_id: u64,
    /// ID of the second step to swap
    pub step2_id: u64,
}

/// Parameters for updating an existing step.
///
/// Allows partial updates to step properties. When changing status to 'done',
/// the result field should be provided to document what was accomplished.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct UpdateStep {
    /// Step ID to update (required)
    pub id: u64,
    /// New status for the step ('todo', 'inprogress', or 'done')
    pub status: Option<String>,
    /// Updated title of the step
    pub title: Option<String>,
    /// Updated detailed description of the step
    pub description: Option<String>,
    /// Updated acceptance criteria for the step
    pub acceptance_criteria: Option<String>,
    /// Updated references (URLs, file paths, etc.)
    pub references: Option<Vec<String>>,
    /// Result description - required when changing status to 'done'.
    ///
    /// This field documents what was actually accomplished when completing the
    /// step. It will be ignored when:
    /// - Changing status to 'todo' or 'inprogress'
    /// - Updating other fields without changing status
    /// - Creating a new step (steps always start as 'todo')
    ///
    /// Format using Markdown with **bold headers** and detailed bullet points:
    /// - What was created/modified (with file paths)
    /// - Technical implementation details
    /// - Preserved functionality and behavior  
    /// - Validation results (tests, builds, etc.)
    ///
    /// Example: "Successfully extracted watch functionality to watch.rs:
    ///
    /// **Created module:** `/path/to/watch.rs` containing:
    /// - Complete watch method implementation with file system monitoring
    /// - File event handling for Created, Modified, and Deleted events
    /// - Async task spawning for file re-indexing
    ///
    /// **Validation results:**
    /// - All 344 tests pass across entire codebase
    /// - No clippy warnings
    /// - Release build successful"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
}

impl UpdateStep {
    /// Validate step update parameters and return parsed status and result.
    ///
    /// This method performs validation of step update requests, including
    /// status parsing and result requirement validation for completed steps.
    ///
    /// # Returns
    ///
    /// A Result containing a tuple of (optional parsed StepStatus, optional
    /// result), or an error if validation fails.
    ///
    /// # Errors
    ///
    /// * `PlannerError::InvalidInput` - When status string is invalid
    /// * `PlannerError::InvalidInput` - When result is missing for 'done'
    ///   status
    pub fn validate(&self) -> crate::Result<(Option<crate::models::StepStatus>, Option<String>)> {
        use std::str::FromStr;

        use crate::models::StepStatus;

        let step_status = if let Some(status_str) = &self.status {
            Some(StepStatus::from_str(status_str).map_err(|_| {
                crate::PlannerError::InvalidInput {
                    field: "status".to_string(),
                    reason: format!(
                        "Invalid status: {}. Must be 'todo', 'inprogress', or 'done'",
                        status_str
                    ),
                }
            })?)
        } else {
            None
        };

        // Validate result requirement for done status
        if let Some(StepStatus::Done) = step_status
            && self.result.is_none()
        {
            return Err(crate::PlannerError::InvalidInput {
                    field: "result".to_string(),
                    reason: "Result description is required when marking a step as done. Please provide a 'result' field describing what was accomplished.".to_string(),
                });
        }

        Ok((step_status, self.result.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PlannerError, models::StepStatus};

    /// Helper function to create an UpdateStep with status and optional result
    fn update_with_status(status: Option<&str>, result: Option<&str>) -> UpdateStep {
        UpdateStep {
            id: 1,
            status: status.map(|s| s.to_string()),
            result: result.map(|r| r.to_string()),
            ..Default::default()
        }
    }

    /// Helper function to assert validation succeeds and returns expected
    /// values
    fn assert_validates_to(
        params: &UpdateStep,
        expected_status: Option<StepStatus>,
        expected_result: Option<&str>,
    ) {
        let result = params.validate();
        assert!(result.is_ok(), "Validation should succeed");

        let (status, result_desc) = result.unwrap();
        assert_eq!(status, expected_status);
        assert_eq!(result_desc, expected_result.map(|s| s.to_string()));
    }

    /// Helper function to assert validation fails with specific error details
    fn assert_validation_error(
        params: &UpdateStep,
        expected_field: &str,
        expected_reason_contains: &str,
    ) {
        let result = params.validate();
        assert!(result.is_err(), "Validation should fail");

        match result.unwrap_err() {
            PlannerError::InvalidInput { field, reason } => {
                assert_eq!(field, expected_field);
                assert!(
                    reason.contains(expected_reason_contains),
                    "Expected reason to contain '{}', got: {}",
                    expected_reason_contains,
                    reason
                );
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[test]
    fn test_valid_status_transitions() {
        // Test valid status changes without result
        assert_validates_to(
            &update_with_status(Some("todo"), None),
            Some(StepStatus::Todo),
            None,
        );
        assert_validates_to(
            &update_with_status(Some("inprogress"), None),
            Some(StepStatus::InProgress),
            None,
        );
        assert_validates_to(
            &update_with_status(Some("in_progress"), None),
            Some(StepStatus::InProgress),
            None,
        );

        // Test done status with result
        assert_validates_to(
            &update_with_status(Some("done"), Some("Successfully completed")),
            Some(StepStatus::Done),
            Some("Successfully completed"),
        );
    }

    #[test]
    fn test_no_status_changes() {
        // Test validation with no status change
        assert_validates_to(&UpdateStep::default(), None, None);
        assert_validates_to(
            &update_with_status(None, Some("Some result")),
            None,
            Some("Some result"),
        );
    }

    #[test]
    fn test_done_status_requires_result() {
        assert_validation_error(
            &update_with_status(Some("done"), None),
            "result",
            "Result description is required",
        );
    }
}
