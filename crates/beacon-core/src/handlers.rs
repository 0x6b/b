//! Core handler functions for unified business logic.
//!
//! This module provides a unified interface for all business operations,
//! consolidating logic that was previously duplicated between CLI and MCP
//! interfaces. Each handler function encapsulates complete business workflows
//! and returns structured data that can be formatted by different interfaces.
//!
//! ## Architecture
//!
//! The handlers follow a consistent pattern:
//!
//! ```text
//! Interface → Handler → Operations + Planner → Models
//! ```
//!
//! - **Handlers**: High-level business workflows (this module)
//! - **Parameters**: Request parameters and validation ([`crate::params`])
//! - **Planner**: Low-level data operations ([`crate::planner`])
//! - **Models**: Domain objects ([`crate::models`])
//!
//! ## Benefits
//!
//! 1. **Code Reuse**: Eliminates duplication between interfaces
//! 2. **Consistency**: Ensures identical behavior across all interfaces
//! 3. **Testability**: Business logic can be tested independently
//! 4. **Maintainability**: Single source of truth for business operations
//!
//! ## Handler Patterns
//!
//! ### Query Handlers
//! Return domain objects or collections directly:
//! ```text
//! pub async fn handle_show_plan(planner: &Planner, params: &Id) -> Result<Option<Plan>>
//! ```
//!
//! ### Command Handlers  
//! Return created/modified objects for confirmation:
//! ```text
//! pub async fn handle_create_plan(planner: &Planner, params: &CreatePlan) -> Result<Plan>
//! ```
//!
//! ### List Handlers
//! Return collections with metadata for consistent formatting:
//! ```text
//! pub async fn handle_list_plans(planner: &Planner, params: &ListPlans) -> Result<Vec<PlanSummary>>
//! ```
//!
//! ## Usage Examples
//!
//! ### CLI Integration
//! ```rust,no_run
//! # use beacon_core::{handle_list_plans, format_plan_list, params::ListPlans, PlannerBuilder};
//! # async {
//! # let planner = PlannerBuilder::new().build().await?;
//! # let params = ListPlans { archived: false };
//! let plans = handle_list_plans(&planner, &params).await?;
//! let output = format_plan_list(&plans, Some("Active Plans"));
//! // renderer.render(&output)?;
//! # Result::<(), beacon_core::PlannerError>::Ok(())
//! # };
//! ```
//!
//! ### MCP Integration
//! ```rust,no_run
//! # use beacon_core::{handle_list_plans, format_plan_list, params::ListPlans, PlannerBuilder};
//! # async {
//! # let planner = PlannerBuilder::new().build().await?;
//! # let params = ListPlans { archived: false };
//! let plans = handle_list_plans(&planner, &params).await?;
//! let output = format_plan_list(&plans, None);
//! // Ok(McpResult::create_result(&output))
//! # Result::<(), beacon_core::PlannerError>::Ok(())
//! # };
//! ```

use crate::{
    models::{Plan, PlanFilter, PlanSummary, Step},
    params::{
        CreatePlan, Id, InsertStep, ListPlans, SearchPlans, StepCreate, SwapSteps, UpdateStep,
    },
    Planner, Result,
};

/// Handle listing plans with optional archived filtering.
///
/// Converts plans to summaries with step count information for consistent
/// list display across interfaces.
///
/// # Arguments
///
/// * `planner` - The planner instance for data operations
/// * `params` - List parameters containing archived flag
///
/// # Returns
///
/// A vector of PlanSummary objects with step counts
///
/// # Examples
///
/// ```rust,no_run
/// # use beacon_core::{handlers::handle_list_plans, params::ListPlans, PlannerBuilder};
/// # async {
/// let planner = PlannerBuilder::new().build().await?;
/// let params = ListPlans { archived: false };
/// let summaries = handle_list_plans(&planner, &params).await?;
/// # Result::<(), beacon_core::PlannerError>::Ok(())
/// # };
/// ```
pub async fn handle_list_plans(planner: &Planner, params: &ListPlans) -> Result<Vec<PlanSummary>> {
    let filter = Some(PlanFilter::from(params));
    let plans = planner.list_plans(filter).await?;
    Ok(plans.iter().map(Into::into).collect())
}

/// Handle showing a complete plan with all its steps.
///
/// Retrieves a plan with its associated steps eagerly loaded.
/// The returned Plan object includes all steps in the steps field.
///
/// # Arguments
///
/// * `planner` - The planner instance for data operations
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
/// # use beacon_core::{handlers::handle_show_plan, params::Id, PlannerBuilder};
/// # async {
/// let planner = PlannerBuilder::new().build().await?;
/// let params = Id { id: 1 };
/// let plan = handle_show_plan(&planner, &params).await?;
/// # Result::<(), beacon_core::PlannerError>::Ok(())
/// # };
/// ```
pub async fn handle_show_plan(planner: &Planner, params: &Id) -> Result<Option<Plan>> {
    planner.get_plan(params).await
}

/// Handle creating a new plan.
///
/// Creates a new plan with the specified parameters and returns
/// the created plan object for confirmation.
///
/// # Arguments
///
/// * `planner` - The planner instance for data operations
/// * `params` - Creation parameters containing title and optional fields
///
/// # Returns
///
/// The newly created Plan object
///
/// # Examples
///
/// ```rust,no_run
/// # use beacon_core::{handlers::handle_create_plan, params::CreatePlan, PlannerBuilder};
/// # async {
/// let planner = PlannerBuilder::new().build().await?;
/// let params = CreatePlan {
///     title: "My Plan".to_string(),
///     description: Some("A test plan".to_string()),
///     directory: None,
/// };
/// let plan = handle_create_plan(&planner, &params).await?;
/// # Result::<(), beacon_core::PlannerError>::Ok(())
/// # };
/// ```
pub async fn handle_create_plan(planner: &Planner, params: &CreatePlan) -> Result<Plan> {
    planner.create_plan(params).await
}

/// Handle archiving a plan.
///
/// Archives the specified plan, making it inactive but preserving
/// all data for potential restoration.
///
/// # Arguments
///
/// * `planner` - The planner instance for data operations
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
/// # use beacon_core::{handlers::handle_archive_plan, params::Id, PlannerBuilder};
/// # async {
/// let planner = PlannerBuilder::new().build().await?;
/// let params = Id { id: 1 };
/// let archived_plan = handle_archive_plan(&planner, &params).await?;
/// # Result::<(), beacon_core::PlannerError>::Ok(())
/// # };
/// ```
pub async fn handle_archive_plan(planner: &Planner, params: &Id) -> Result<Option<Plan>> {
    // Get plan details before archiving for confirmation
    let plan = planner.get_plan(params).await?;

    if plan.is_some() {
        planner.archive_plan(params).await?;
    }

    Ok(plan)
}

/// Handle unarchiving a plan.
///
/// Restores an archived plan to active status, making it visible
/// in regular plan listings.
///
/// # Arguments
///
/// * `planner` - The planner instance for data operations
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
/// # use beacon_core::{handlers::handle_unarchive_plan, params::Id, PlannerBuilder};
/// # async {
/// let planner = PlannerBuilder::new().build().await?;
/// let params = Id { id: 1 };
/// let unarchived_plan = handle_unarchive_plan(&planner, &params).await?;
/// # Result::<(), beacon_core::PlannerError>::Ok(())
/// # };
/// ```
pub async fn handle_unarchive_plan(planner: &Planner, params: &Id) -> Result<Option<Plan>> {
    // Get plan details before unarchiving for confirmation
    let plan = planner.get_plan(params).await?;

    if plan.is_some() {
        planner.unarchive_plan(params).await?;
    }

    Ok(plan)
}

/// Handle permanently deleting a plan.
///
/// Permanently removes a plan and all its associated steps from the database.
/// This operation cannot be undone.
///
/// # Arguments
///
/// * `planner` - The planner instance for data operations
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
/// # use beacon_core::{handlers::handle_delete_plan, params::Id, PlannerBuilder};
/// # async {
/// let planner = PlannerBuilder::new().build().await?;
/// let params = Id { id: 1 };
/// let deleted_plan = handle_delete_plan(&planner, &params).await?;
/// # Result::<(), beacon_core::PlannerError>::Ok(())
/// # };
/// ```
pub async fn handle_delete_plan(planner: &Planner, params: &Id) -> Result<Option<Plan>> {
    // Get plan details before deleting for confirmation
    let plan = planner.get_plan(params).await?;

    if plan.is_some() {
        planner.delete_plan(params).await?;
    }

    Ok(plan)
}

/// Handle searching for plans in a specific directory.
///
/// Searches for plans associated with the specified directory path,
/// with optional archived filtering, and returns them as summaries
/// with step counts.
///
/// # Arguments
///
/// * `planner` - The planner instance for data operations
/// * `params` - Search parameters containing directory and archived flag
///
/// # Returns
///
/// A vector of PlanSummary objects matching the search criteria
///
/// # Examples
///
/// ```rust,no_run
/// # use beacon_core::{handlers::handle_search_plans, params::SearchPlans, PlannerBuilder};
/// # async {
/// let planner = PlannerBuilder::new().build().await?;
/// let params = SearchPlans {
///     directory: "/path/to/project".to_string(),
///     archived: false,
/// };
/// let summaries = handle_search_plans(&planner, &params).await?;
/// # Result::<(), beacon_core::PlannerError>::Ok(())
/// # };
/// ```
pub async fn handle_search_plans(
    planner: &Planner,
    params: &SearchPlans,
) -> Result<Vec<PlanSummary>> {
    let plans = if params.archived {
        // For archived plans, use list_plans with directory filter
        let filter = PlanFilter::for_directory(params.directory.clone(), true);
        planner.list_plans(Some(filter)).await?
    } else {
        // For active plans, use the specialized search method
        planner.search_plans_by_directory(params).await?
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
/// * `planner` - The planner instance for data operations
/// * `params` - Step creation parameters
///
/// # Returns
///
/// The newly created Step object
///
/// # Examples
///
/// ```rust,no_run
/// # use beacon_core::{handlers::handle_add_step, params::StepCreate, PlannerBuilder};
/// # async {
/// let planner = PlannerBuilder::new().build().await?;
/// let params = StepCreate {
///     plan_id: 1,
///     title: "Test Step".to_string(),
///     description: Some("A test step".to_string()),
///     acceptance_criteria: None,
///     references: vec![],
/// };
/// let step = handle_add_step(&planner, &params).await?;
/// # Result::<(), beacon_core::PlannerError>::Ok(())
/// # };
/// ```
pub async fn handle_add_step(planner: &Planner, params: &StepCreate) -> Result<Step> {
    planner.add_step(params).await
}

/// Handle inserting a step at a specific position in a plan.
///
/// Creates a new step and inserts it at the specified position,
/// shifting other steps as needed.
///
/// # Arguments
///
/// * `planner` - The planner instance for data operations
/// * `params` - Step insertion parameters including position
///
/// # Returns
///
/// The newly created Step object
///
/// # Examples
///
/// ```rust,no_run
/// # use beacon_core::{handlers::handle_insert_step, params::{InsertStep, StepCreate}, PlannerBuilder};
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
/// let step = handle_insert_step(&planner, &params).await?;
/// # Result::<(), beacon_core::PlannerError>::Ok(())
/// # };
/// ```
pub async fn handle_insert_step(planner: &Planner, params: &InsertStep) -> Result<Step> {
    planner.insert_step(params).await
}

/// Handle updating a step's properties.
///
/// Updates the specified step with new values, performing validation
/// for status changes and result requirements.
///
/// # Arguments
///
/// * `planner` - The planner instance for data operations
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
/// # use beacon_core::{handlers::handle_update_step, params::UpdateStep, PlannerBuilder};
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
/// let updated_step = handle_update_step(&planner, &params).await?;
/// # Result::<(), beacon_core::PlannerError>::Ok(())
/// # };
/// ```
pub async fn handle_update_step(planner: &Planner, params: &UpdateStep) -> Result<Option<Step>> {
    // Get step before update for confirmation
    let step = planner.get_step(&Id { id: params.id }).await?;

    if step.is_some() {
        // Create validated update request using TryFrom trait
        let update_request = params.clone().try_into()?;

        // Perform the update
        planner.update_step(params.id, update_request).await?;

        // Return the updated step
        planner.get_step(&Id { id: params.id }).await
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
/// * `planner` - The planner instance for data operations
/// * `params` - ID parameters specifying which step to retrieve
///
/// # Returns
///
/// An optional Step object if the step exists, or None if not found
///
/// # Examples
///
/// ```rust,no_run
/// # use beacon_core::{handlers::handle_show_step, params::Id, PlannerBuilder};
/// # async {
/// let planner = PlannerBuilder::new().build().await?;
/// let params = Id { id: 1 };
/// let step = handle_show_step(&planner, &params).await?;
/// # Result::<(), beacon_core::PlannerError>::Ok(())
/// # };
/// ```
pub async fn handle_show_step(planner: &Planner, params: &Id) -> Result<Option<Step>> {
    planner.get_step(params).await
}

/// Handle claiming a step for processing.
///
/// Atomically transitions a step from 'todo' to 'inprogress' status,
/// preventing multiple agents from working on the same task.
///
/// # Arguments
///
/// * `planner` - The planner instance for data operations
/// * `params` - ID parameters specifying which step to claim
///
/// # Returns
///
/// A boolean indicating whether the step was successfully claimed
///
/// # Examples
///
/// ```rust,no_run
/// # use beacon_core::{handlers::handle_claim_step, params::Id, PlannerBuilder};
/// # async {
/// let planner = PlannerBuilder::new().build().await?;
/// let params = Id { id: 1 };
/// let claimed = handle_claim_step(&planner, &params).await?;
/// # Result::<(), beacon_core::PlannerError>::Ok(())
/// # };
/// ```
pub async fn handle_claim_step(planner: &Planner, params: &Id) -> Result<bool> {
    planner.claim_step(params).await
}

/// Handle swapping the order of two steps.
///
/// Reorders steps within a plan by swapping their positions,
/// useful for task prioritization.
///
/// # Arguments
///
/// * `planner` - The planner instance for data operations
/// * `params` - Swap parameters containing both step IDs
///
/// # Returns
///
/// Unit result indicating success or failure of the operation
///
/// # Examples
///
/// ```rust,no_run
/// # use beacon_core::{handlers::handle_swap_steps, params::SwapSteps, PlannerBuilder};
/// # async {
/// let planner = PlannerBuilder::new().build().await?;
/// let params = SwapSteps {
///     step1_id: 1,
///     step2_id: 2,
/// };
/// handle_swap_steps(&planner, &params).await?;
/// # Result::<(), beacon_core::PlannerError>::Ok(())
/// # };
/// ```
pub async fn handle_swap_steps(planner: &Planner, params: &SwapSteps) -> Result<()> {
    planner.swap_steps(params).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{models::StepStatus, PlannerBuilder};

    #[tokio::test]
    async fn test_handle_create_plan() {
        let planner = PlannerBuilder::new().build().await.unwrap();
        let params = CreatePlan {
            title: "Test Plan".to_string(),
            description: Some("A test plan".to_string()),
            directory: None,
        };

        let result = handle_create_plan(&planner, &params).await;
        assert!(result.is_ok());

        let plan = result.unwrap();
        assert_eq!(plan.title, "Test Plan");
        assert_eq!(plan.description, Some("A test plan".to_string()));
    }

    #[tokio::test]
    async fn test_handle_list_plans() {
        let planner = PlannerBuilder::new().build().await.unwrap();

        // Create a test plan first
        let create_params = CreatePlan {
            title: "Test Plan".to_string(),
            description: None,
            directory: None,
        };
        handle_create_plan(&planner, &create_params).await.unwrap();

        // List active plans
        let list_params = ListPlans { archived: false };
        let result = handle_list_plans(&planner, &list_params).await;
        assert!(result.is_ok());

        let summaries = result.unwrap();
        assert!(!summaries.is_empty());
        assert_eq!(summaries[0].title, "Test Plan");
    }

    #[tokio::test]
    async fn test_handle_show_plan() {
        let planner = PlannerBuilder::new().build().await.unwrap();

        // Create a test plan first
        let create_params = CreatePlan {
            title: "Test Plan".to_string(),
            description: Some("A test plan".to_string()),
            directory: None,
        };
        let plan = handle_create_plan(&planner, &create_params).await.unwrap();

        // Show the plan
        let show_params = Id { id: plan.id };
        let result = handle_show_plan(&planner, &show_params).await;
        assert!(result.is_ok());

        let plan = result.unwrap();
        assert!(plan.is_some());

        let plan = plan.unwrap();
        assert_eq!(plan.title, "Test Plan");
        assert_eq!(plan.steps.len(), 0);
    }

    #[tokio::test]
    async fn test_handle_show_plan_nonexistent() {
        let planner = PlannerBuilder::new().build().await.unwrap();

        // Try to show a non-existent plan
        let show_params = Id { id: 999 };
        let result = handle_show_plan(&planner, &show_params).await;
        assert!(result.is_ok());

        let plan = result.unwrap();
        assert!(plan.is_none());
    }

    #[tokio::test]
    async fn test_handle_add_step() {
        let planner = PlannerBuilder::new().build().await.unwrap();

        // Create a test plan first
        let create_params = CreatePlan {
            title: "Test Plan".to_string(),
            description: None,
            directory: None,
        };
        let plan = handle_create_plan(&planner, &create_params).await.unwrap();

        // Add a step
        let step_params = StepCreate {
            plan_id: plan.id,
            title: "Test Step".to_string(),
            description: Some("A test step".to_string()),
            acceptance_criteria: Some("Should work".to_string()),
            references: vec!["ref1.txt".to_string()],
        };

        let result = handle_add_step(&planner, &step_params).await;
        assert!(result.is_ok());

        let step = result.unwrap();
        assert_eq!(step.title, "Test Step");
        assert_eq!(step.description, Some("A test step".to_string()));
        assert_eq!(step.acceptance_criteria, Some("Should work".to_string()));
        assert_eq!(step.references, vec!["ref1.txt".to_string()]);
        assert_eq!(step.status, StepStatus::Todo);
    }

    #[tokio::test]
    async fn test_handle_update_step() {
        let planner = PlannerBuilder::new().build().await.unwrap();

        // Create a test plan and step
        let create_params = CreatePlan {
            title: "Test Plan".to_string(),
            description: None,
            directory: None,
        };
        let plan = handle_create_plan(&planner, &create_params).await.unwrap();

        let step_params = StepCreate {
            plan_id: plan.id,
            title: "Test Step".to_string(),
            description: None,
            acceptance_criteria: None,
            references: vec![],
        };
        let step = handle_add_step(&planner, &step_params).await.unwrap();

        // Update the step
        let update_params = UpdateStep {
            id: step.id,
            status: Some("done".to_string()),
            title: Some("Updated Step".to_string()),
            description: Some("Updated description".to_string()),
            acceptance_criteria: None,
            references: None,
            result: Some("Completed successfully".to_string()),
        };

        let result = handle_update_step(&planner, &update_params).await;
        assert!(result.is_ok());

        let updated_step = result.unwrap();
        assert!(updated_step.is_some());

        let updated_step = updated_step.unwrap();
        assert_eq!(updated_step.title, "Updated Step");
        assert_eq!(
            updated_step.description,
            Some("Updated description".to_string())
        );
        assert_eq!(updated_step.status, StepStatus::Done);
        assert_eq!(
            updated_step.result,
            Some("Completed successfully".to_string())
        );
    }

    #[tokio::test]
    async fn test_handle_claim_step() {
        let planner = PlannerBuilder::new().build().await.unwrap();

        // Create a test plan and step
        let create_params = CreatePlan {
            title: "Test Plan".to_string(),
            description: None,
            directory: None,
        };
        let plan = handle_create_plan(&planner, &create_params).await.unwrap();

        let step_params = StepCreate {
            plan_id: plan.id,
            title: "Test Step".to_string(),
            description: None,
            acceptance_criteria: None,
            references: vec![],
        };
        let step = handle_add_step(&planner, &step_params).await.unwrap();

        // Claim the step
        let claim_params = Id { id: step.id };
        let result = handle_claim_step(&planner, &claim_params).await;
        assert!(result.is_ok());

        let claimed = result.unwrap();
        assert!(claimed);

        // Verify the step is now in progress
        let updated_step = handle_show_step(&planner, &claim_params)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated_step.status, StepStatus::InProgress);
    }

    #[tokio::test]
    async fn test_handle_archive_unarchive_plan() {
        let planner = PlannerBuilder::new().build().await.unwrap();

        // Create a test plan
        let create_params = CreatePlan {
            title: "Test Plan".to_string(),
            description: None,
            directory: None,
        };
        let plan = handle_create_plan(&planner, &create_params).await.unwrap();

        // Archive the plan
        let archive_params = Id { id: plan.id };
        let result = handle_archive_plan(&planner, &archive_params).await;
        assert!(result.is_ok());

        let archived_plan = result.unwrap();
        assert!(archived_plan.is_some());

        // Unarchive the plan
        let result = handle_unarchive_plan(&planner, &archive_params).await;
        assert!(result.is_ok());

        let unarchived_plan = result.unwrap();
        assert!(unarchived_plan.is_some());
    }

    #[tokio::test]
    async fn test_handle_delete_plan() {
        let planner = PlannerBuilder::new().build().await.unwrap();

        // Create a test plan with steps
        let create_params = CreatePlan {
            title: "Test Plan to Delete".to_string(),
            description: Some("This plan will be deleted".to_string()),
            directory: None,
        };
        let plan = handle_create_plan(&planner, &create_params).await.unwrap();

        // Add some steps to the plan
        let step_params = StepCreate {
            plan_id: plan.id,
            title: "Test Step".to_string(),
            description: None,
            acceptance_criteria: None,
            references: vec![],
        };
        let _step = handle_add_step(&planner, &step_params).await.unwrap();

        // Verify the plan exists before deletion
        let show_params = Id { id: plan.id };
        let existing_plan = handle_show_plan(&planner, &show_params).await.unwrap();
        assert!(existing_plan.is_some());

        // Delete the plan
        let delete_params = Id { id: plan.id };
        let result = handle_delete_plan(&planner, &delete_params).await;
        assert!(result.is_ok());

        let deleted_plan = result.unwrap();
        assert!(deleted_plan.is_some());
        assert_eq!(deleted_plan.unwrap().title, "Test Plan to Delete");

        // Verify the plan no longer exists
        let missing_plan = handle_show_plan(&planner, &show_params).await.unwrap();
        assert!(missing_plan.is_none());
    }

    #[tokio::test]
    async fn test_handle_delete_nonexistent_plan() {
        let planner = PlannerBuilder::new().build().await.unwrap();

        // Try to delete a plan that doesn't exist
        let delete_params = Id { id: 999 };
        let result = handle_delete_plan(&planner, &delete_params).await;
        assert!(result.is_ok());

        let deleted_plan = result.unwrap();
        assert!(deleted_plan.is_none()); // Should return None for non-existent
                                         // plan
    }
}
