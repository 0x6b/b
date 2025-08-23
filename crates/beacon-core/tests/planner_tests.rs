use std::path::PathBuf;

use beacon_core::{CompletionFilter, PlanFilter, PlannerBuilder, StepStatus, UpdateStepRequest};
use tempfile::TempDir;

/// Helper function to create a temporary directory and database path
fn create_test_environment() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let db_path = temp_dir.path().join("test_tasks.db");
    (temp_dir, db_path)
}

#[tokio::test]
async fn test_complete_plan_workflow() {
    let (_temp_dir, db_path) = create_test_environment();

    let planner = PlannerBuilder::new()
        .with_database_path(&db_path)
        .build()
        .await
        .expect("Failed to create planner");

    // Create a plan
    let plan = planner
        .create_plan("Integration Test", Some("Testing complete workflow"), None)
        .await
        .expect("Failed to create plan");

    // Add multiple steps
    let step1 = planner
        .add_step(plan.id, "First step", None, None, Vec::new())
        .await
        .expect("Failed to add step");
    let step2 = planner
        .add_step(plan.id, "Second step", None, None, Vec::new())
        .await
        .expect("Failed to add step");
    let step3 = planner
        .add_step(plan.id, "Third step", None, None, Vec::new())
        .await
        .expect("Failed to add step");

    // Verify step ordering
    let steps = planner
        .get_steps(plan.id)
        .await
        .expect("Failed to get steps");
    assert_eq!(steps.len(), 3);
    assert_eq!(steps[0].order, 0);
    assert_eq!(steps[1].order, 1);
    assert_eq!(steps[2].order, 2);

    // Test claiming a step
    let claimed = planner
        .claim_step(step2.id)
        .await
        .expect("Failed to claim step");
    assert!(claimed, "Should successfully claim step2");

    // Verify step is in progress
    let steps_after_claim = planner
        .get_steps(plan.id)
        .await
        .expect("Failed to get steps after claim");
    assert_eq!(steps_after_claim[1].status, StepStatus::InProgress);

    // Complete some steps
    planner
        .update_step(
            step1.id,
            UpdateStepRequest {
                status: Some(StepStatus::Done),
                result: Some("First step completed".to_string()),
                ..Default::default()
            },
        )
        .await
        .expect("Failed to update step");
    planner
        .update_step(
            step3.id,
            UpdateStepRequest {
                status: Some(StepStatus::Done),
                result: Some("Third step completed".to_string()),
                ..Default::default()
            },
        )
        .await
        .expect("Failed to update step");

    // Verify completion status
    let updated_steps = planner
        .get_steps(plan.id)
        .await
        .expect("Failed to get updated steps");
    assert_eq!(updated_steps[0].status, StepStatus::Done);
    assert_eq!(updated_steps[1].status, StepStatus::InProgress);
    assert_eq!(updated_steps[2].status, StepStatus::Done);

    // Test filtering
    let incomplete_filter = PlanFilter {
        completion_status: Some(CompletionFilter::Incomplete),
        ..Default::default()
    };

    let filtered_plans = planner
        .list_plans(Some(incomplete_filter))
        .await
        .expect("Failed to filter plans");
    assert_eq!(filtered_plans.len(), 1);
    assert_eq!(filtered_plans[0].id, plan.id);
}

#[tokio::test]
async fn test_database_persistence_across_connections() {
    let (_temp_dir, db_path) = create_test_environment();

    let plan_id = {
        // Create planner and plan in first connection
        let planner = PlannerBuilder::new()
            .with_database_path(&db_path)
            .build()
            .await
            .expect("Failed to create first planner");

        let plan = planner
            .create_plan("Test Plan", None, None)
            .await
            .expect("Failed to create plan");

        planner
            .add_step(plan.id, "Test step", None, None, Vec::new())
            .await
            .expect("Failed to add step");

        plan.id
    };

    // Create new planner instance (simulating app restart)
    let planner = PlannerBuilder::new()
        .with_database_path(&db_path)
        .build()
        .await
        .expect("Failed to create second planner");

    // Verify data persisted
    let retrieved_plan = planner
        .get_plan(plan_id)
        .await
        .expect("Failed to retrieve plan")
        .expect("Plan should exist");

    assert_eq!(retrieved_plan.title, "Test Plan");

    let steps = planner
        .get_steps(plan_id)
        .await
        .expect("Failed to get steps");
    assert_eq!(steps.len(), 1);
    assert_eq!(steps[0].title, "Test step");
}

#[tokio::test]
async fn test_error_handling_invalid_operations() {
    let (_temp_dir, db_path) = create_test_environment();

    let planner = PlannerBuilder::new()
        .with_database_path(&db_path)
        .build()
        .await
        .expect("Failed to create planner");

    // Test operations on non-existent plan
    let result = planner
        .get_plan(999)
        .await
        .expect("Failed to query non-existent plan");
    assert!(result.is_none());

    let result = planner
        .add_step(999, "Invalid step", None, None, Vec::new())
        .await;
    assert!(result.is_err());

    let result = planner.archive_plan(999).await;
    assert!(result.is_err());

    // Test operations on non-existent step
    let result = planner
        .update_step(
            999,
            UpdateStepRequest {
                status: Some(StepStatus::Done),
                result: Some("Test result".to_string()),
                ..Default::default()
            },
        )
        .await;
    assert!(result.is_err());

    let result = planner.remove_step(999).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_plan_with_steps_retrieval() {
    let (_temp_dir, db_path) = create_test_environment();

    let planner = PlannerBuilder::new()
        .with_database_path(&db_path)
        .build()
        .await
        .expect("Failed to create planner");

    // Create a plan with steps
    let plan = planner
        .create_plan("Test Plan", Some("Testing step retrieval"), None)
        .await
        .expect("Failed to create plan");

    planner
        .add_step(plan.id, "Step 1", None, None, Vec::new())
        .await
        .expect("Failed to add step 1");
    planner
        .add_step(plan.id, "Step 2", None, None, Vec::new())
        .await
        .expect("Failed to add step 2");

    // Retrieve plan with steps
    let plan_with_steps = planner
        .get_plan_with_steps(plan.id)
        .await
        .expect("Failed to get plan with steps")
        .expect("Plan should exist");

    assert_eq!(plan_with_steps.steps.len(), 2);
    assert_eq!(plan_with_steps.steps[0].title, "Step 1");
    assert_eq!(plan_with_steps.steps[1].title, "Step 2");
}

#[tokio::test]
async fn test_step_removal() {
    let (_temp_dir, db_path) = create_test_environment();

    let planner = PlannerBuilder::new()
        .with_database_path(&db_path)
        .build()
        .await
        .expect("Failed to create planner");

    let plan = planner
        .create_plan("Step Test", None, None)
        .await
        .expect("Failed to create plan");

    let step1 = planner
        .add_step(plan.id, "Step to keep", None, None, Vec::new())
        .await
        .expect("Failed to add step");
    let step2 = planner
        .add_step(plan.id, "Step to remove", None, None, Vec::new())
        .await
        .expect("Failed to add step");
    let step3 = planner
        .add_step(plan.id, "Another step to keep", None, None, Vec::new())
        .await
        .expect("Failed to add step");

    // Remove the middle step
    planner
        .remove_step(step2.id)
        .await
        .expect("Failed to remove step");

    // Verify remaining steps
    let steps = planner
        .get_steps(plan.id)
        .await
        .expect("Failed to get steps");
    assert_eq!(steps.len(), 2);
    assert_eq!(steps[0].id, step1.id);
    assert_eq!(steps[1].id, step3.id);
}

#[tokio::test]
async fn test_plan_archiving() {
    let (_temp_dir, db_path) = create_test_environment();

    let planner = PlannerBuilder::new()
        .with_database_path(&db_path)
        .build()
        .await
        .expect("Failed to create planner");

    let plan = planner
        .create_plan("Archive Test", None, None)
        .await
        .expect("Failed to create plan");

    // Add steps
    planner
        .add_step(plan.id, "Step 1", None, None, Vec::new())
        .await
        .expect("Failed to add step");
    planner
        .add_step(plan.id, "Step 2", None, None, Vec::new())
        .await
        .expect("Failed to add step");

    // Archive the plan
    planner
        .archive_plan(plan.id)
        .await
        .expect("Failed to archive plan");

    // Verify plan is not visible in normal list
    let active_plans = planner
        .list_plans(None)
        .await
        .expect("Failed to list plans");
    assert!(!active_plans.iter().any(|p| p.id == plan.id));

    // Verify plan is visible when including archived
    let filter = PlanFilter {
        include_archived: true,
        ..Default::default()
    };
    let all_plans = planner
        .list_plans(Some(filter))
        .await
        .expect("Failed to list all plans");
    assert!(all_plans.iter().any(|p| p.id == plan.id));

    // Verify steps are still there
    let steps = planner
        .get_steps(plan.id)
        .await
        .expect("Query should succeed");
    assert_eq!(steps.len(), 2);
}
