//! Common business operations for the Beacon task planning system.
//!
//! This module contains shared business logic operations that are used across
//! different interfaces (CLI, MCP server, etc.). These operations extract
//! common patterns to reduce code duplication while maintaining consistency.

use std::str::FromStr;

use crate::{
    models::{PlanFilter, PlanStatus, StepStatus, UpdateStepRequest},
    params::ListPlans,
    PlannerError, Result,
};

/// Create a plan filter for listing plans based on archived status.
///
/// This operation standardizes the creation of plan filters for active
/// or archived plans, ensuring consistent behavior across interfaces.
///
/// # Arguments
///
/// * `params` - List plans parameters containing archived flag
///
/// # Returns
///
/// An optional PlanFilter configured for active or archived plans
///
/// # Examples
///
/// ```rust
/// # use beacon_core::{operations::create_plan_filter, params::ListPlans};
/// let params = ListPlans { archived: false };
/// let filter = create_plan_filter(&params);
/// assert!(filter.is_some());
/// ```
pub fn create_plan_filter(params: &ListPlans) -> Option<PlanFilter> {
    if params.archived {
        Some(PlanFilter {
            status: Some(PlanStatus::Archived),
            include_archived: true,
            ..Default::default()
        })
    } else {
        Some(PlanFilter {
            status: Some(PlanStatus::Active),
            include_archived: false,
            ..Default::default()
        })
    }
}

/// Create a directory-specific plan filter for search operations.
///
/// This operation creates a plan filter that combines directory filtering
/// with archived status filtering for search operations.
///
/// # Arguments
///
/// * `directory` - Directory path to filter by
/// * `archived` - Whether to include archived plans
///
/// # Returns
///
/// A PlanFilter configured for directory and status filtering
///
/// # Examples
///
/// ```rust
/// # use beacon_core::operations::create_directory_filter;
/// let filter = create_directory_filter("/path/to/project".to_string(), false);
/// // Filter configured for active plans in specific directory
/// ```
pub fn create_directory_filter(directory: String, archived: bool) -> PlanFilter {
    PlanFilter {
        status: Some(if archived {
            PlanStatus::Archived
        } else {
            PlanStatus::Active
        }),
        directory: Some(directory),
        include_archived: archived,
        ..Default::default()
    }
}

/// Validate and convert step update parameters with proper error handling.
///
/// This operation performs validation of step update requests, including
/// status parsing and result requirement validation for completed steps.
///
/// # Arguments
///
/// * `status_str` - Optional status string to validate and convert
/// * `result` - Optional result description for completed steps
///
/// # Returns
///
/// A Result containing the validated UpdateStepRequest with parsed status,
/// or an error if validation fails
///
/// # Errors
///
/// * `PlannerError::InvalidInput` - When status string is invalid
/// * `PlannerError::InvalidInput` - When result is missing for 'done' status
///
/// # Examples
///
/// ```rust
/// # use beacon_core::operations::validate_step_update;
/// // Valid update with status change
/// let request = validate_step_update(
///     Some("done".to_string()),
///     Some("Completed successfully".to_string()),
/// )?;
///
/// // Invalid - missing result for done status
/// let error = validate_step_update(Some("done".to_string()), None);
/// assert!(error.is_err());
/// # Result::<(), beacon_core::PlannerError>::Ok(())
/// ```
pub fn validate_step_update(
    status_str: Option<String>,
    result: Option<String>,
) -> Result<(Option<StepStatus>, Option<String>)> {
    let step_status = if let Some(status_str) = &status_str {
        Some(
            StepStatus::from_str(status_str).map_err(|_| PlannerError::InvalidInput {
                field: "status".to_string(),
                reason: format!(
                    "Invalid status: {}. Must be 'todo', 'inprogress', or 'done'",
                    status_str
                ),
            })?,
        )
    } else {
        None
    };

    // Validate result requirement for done status
    if let Some(StepStatus::Done) = step_status {
        if result.is_none() {
            return Err(PlannerError::InvalidInput {
                field: "result".to_string(),
                reason: "Result description is required when marking a step as done. Please provide a 'result' field describing what was accomplished.".to_string(),
            });
        }
    }

    Ok((step_status, result))
}

/// Create a complete UpdateStepRequest from individual parameters.
///
/// This operation combines multiple optional update parameters into a
/// single UpdateStepRequest structure for step updates.
///
/// # Arguments
///
/// * `title` - Optional new title
/// * `description` - Optional new description
/// * `acceptance_criteria` - Optional new acceptance criteria
/// * `references` - Optional new references list
/// * `status` - Optional validated status
/// * `result` - Optional result description
///
/// # Returns
///
/// A complete UpdateStepRequest with all provided parameters
///
/// # Examples
///
/// ```rust
/// # use beacon_core::{operations::create_update_request, models::StepStatus};
/// let request = create_update_request(
///     Some("New title".to_string()),
///     None,
///     None,
///     None,
///     Some(StepStatus::Done),
///     Some("Completed".to_string()),
/// );
/// ```
pub fn create_update_request(
    title: Option<String>,
    description: Option<String>,
    acceptance_criteria: Option<String>,
    references: Option<Vec<String>>,
    status: Option<StepStatus>,
    result: Option<String>,
) -> UpdateStepRequest {
    UpdateStepRequest {
        title,
        description,
        acceptance_criteria,
        references,
        status,
        result,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_plan_filter_active() {
        let params = ListPlans { archived: false };
        let filter = create_plan_filter(&params).unwrap();

        assert_eq!(filter.status, Some(PlanStatus::Active));
        assert!(!filter.include_archived);
    }

    #[test]
    fn test_create_plan_filter_archived() {
        let params = ListPlans { archived: true };
        let filter = create_plan_filter(&params).unwrap();

        assert_eq!(filter.status, Some(PlanStatus::Archived));
        assert!(filter.include_archived);
    }

    #[test]
    fn test_create_directory_filter_active() {
        let directory = "/path/to/project".to_string();
        let filter = create_directory_filter(directory.clone(), false);

        assert_eq!(filter.status, Some(PlanStatus::Active));
        assert_eq!(filter.directory, Some(directory));
        assert!(!filter.include_archived);
    }

    #[test]
    fn test_create_directory_filter_archived() {
        let directory = "/path/to/project".to_string();
        let filter = create_directory_filter(directory.clone(), true);

        assert_eq!(filter.status, Some(PlanStatus::Archived));
        assert_eq!(filter.directory, Some(directory));
        assert!(filter.include_archived);
    }

    #[test]
    fn test_validate_step_update_valid_todo() {
        let result = validate_step_update(Some("todo".to_string()), None);
        assert!(result.is_ok());

        let (status, result_desc) = result.unwrap();
        assert_eq!(status, Some(StepStatus::Todo));
        assert_eq!(result_desc, None);
    }

    #[test]
    fn test_validate_step_update_valid_inprogress() {
        let result = validate_step_update(Some("inprogress".to_string()), None);
        assert!(result.is_ok());

        let (status, result_desc) = result.unwrap();
        assert_eq!(status, Some(StepStatus::InProgress));
        assert_eq!(result_desc, None);
    }

    #[test]
    fn test_validate_step_update_valid_done_with_result() {
        let result = validate_step_update(
            Some("done".to_string()),
            Some("Successfully completed".to_string()),
        );
        assert!(result.is_ok());

        let (status, result_desc) = result.unwrap();
        assert_eq!(status, Some(StepStatus::Done));
        assert_eq!(result_desc, Some("Successfully completed".to_string()));
    }

    #[test]
    fn test_validate_step_update_done_missing_result() {
        let result = validate_step_update(Some("done".to_string()), None);
        assert!(result.is_err());

        match result.unwrap_err() {
            PlannerError::InvalidInput { field, reason } => {
                assert_eq!(field, "result");
                assert!(reason.contains("Result description is required"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[test]
    fn test_validate_step_update_invalid_status() {
        let result = validate_step_update(Some("invalid".to_string()), None);
        assert!(result.is_err());

        match result.unwrap_err() {
            PlannerError::InvalidInput { field, reason } => {
                assert_eq!(field, "status");
                assert!(reason.contains("Invalid status: invalid"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[test]
    fn test_validate_step_update_no_status() {
        let result = validate_step_update(None, Some("Some result".to_string()));
        assert!(result.is_ok());

        let (status, result_desc) = result.unwrap();
        assert_eq!(status, None);
        assert_eq!(result_desc, Some("Some result".to_string()));
    }

    #[test]
    fn test_create_update_request_all_fields() {
        let request = create_update_request(
            Some("New Title".to_string()),
            Some("New Description".to_string()),
            Some("New Acceptance".to_string()),
            Some(vec!["ref1.txt".to_string(), "ref2.txt".to_string()]),
            Some(StepStatus::Done),
            Some("Completed successfully".to_string()),
        );

        assert_eq!(request.title, Some("New Title".to_string()));
        assert_eq!(request.description, Some("New Description".to_string()));
        assert_eq!(
            request.acceptance_criteria,
            Some("New Acceptance".to_string())
        );
        assert_eq!(
            request.references,
            Some(vec!["ref1.txt".to_string(), "ref2.txt".to_string()])
        );
        assert_eq!(request.status, Some(StepStatus::Done));
        assert_eq!(request.result, Some("Completed successfully".to_string()));
    }

    #[test]
    fn test_create_update_request_minimal() {
        let request = create_update_request(None, None, None, None, None, None);

        assert_eq!(request.title, None);
        assert_eq!(request.description, None);
        assert_eq!(request.acceptance_criteria, None);
        assert_eq!(request.references, None);
        assert_eq!(request.status, None);
        assert_eq!(request.result, None);
    }
}
