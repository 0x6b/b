//! Tests for the planner module.

use super::*;
use crate::params::{CreatePlan, Id, InsertStep, ListPlans, SearchPlans, StepCreate, SwapSteps, UpdateStep};
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

    assert_eq!(summaries.0.len(), 1);
    assert_eq!(summaries.0[0].title, "Test Plan");
    assert_eq!(summaries.0[0].description, Some("Test Description".to_string()));
    assert_eq!(summaries.0[0].total_steps, 1);
    assert_eq!(summaries.0[0].completed_steps, 0);
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

    assert_eq!(summaries.0.len(), 1);
    assert_eq!(summaries.0[0].title, "Archived Plan");

    // Verify active plans is empty
    let active_summaries = planner
        .list_plans_summary(&ListPlans { archived: false })
        .await
        .expect("Failed to list active plans");
    assert_eq!(active_summaries.0.len(), 0);
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

    assert_eq!(summaries.0.len(), 1);
    assert_eq!(summaries.0[0].title, "Plan in Test Dir");
    assert_eq!(summaries.0[0].total_steps, 1);
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

    assert_eq!(summaries.0.len(), 1);
    assert_eq!(summaries.0[0].title, "Archived Plan in Dir");

    // Verify active search returns empty
    let active_summaries = planner
        .search_plans_summary(&SearchPlans {
            directory: test_dir.to_string(),
            archived: false,
        })
        .await
        .expect("Failed to search active plans");
    assert_eq!(active_summaries.0.len(), 0);
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