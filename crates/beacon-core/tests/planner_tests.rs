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
#[allow(clippy::too_many_lines)]
async fn test_complete_plan_workflow() {
    let (_temp_dir, db_path) = create_test_environment();

    let planner = PlannerBuilder::new()
        .with_database_path(Some(db_path))
        .build()
        .await
        .expect("Failed to create planner");

    // Create a plan
    let plan = planner
        .create_plan(&beacon_core::params::CreatePlan {
            title: "Integration Test".to_string(),
            description: Some("Testing complete workflow".to_string()),
            directory: None,
        })
        .await
        .expect("Failed to create plan");

    // Add multiple steps
    let step1 = planner
        .add_step(&beacon_core::params::StepCreate {
            plan_id: plan.id,
            title: "First step".to_string(),
            description: None,
            acceptance_criteria: None,
            references: Vec::new(),
        })
        .await
        .expect("Failed to add step");
    let step2 = planner
        .add_step(&beacon_core::params::StepCreate {
            plan_id: plan.id,
            title: "Second step".to_string(),
            description: None,
            acceptance_criteria: None,
            references: Vec::new(),
        })
        .await
        .expect("Failed to add step");
    let step3 = planner
        .add_step(&beacon_core::params::StepCreate {
            plan_id: plan.id,
            title: "Third step".to_string(),
            description: None,
            acceptance_criteria: None,
            references: Vec::new(),
        })
        .await
        .expect("Failed to add step");

    // Verify step ordering
    let steps = planner
        .get_steps(&beacon_core::params::Id { id: plan.id })
        .await
        .expect("Failed to get steps");
    assert_eq!(steps.len(), 3);
    assert_eq!(steps[0].order, 0);
    assert_eq!(steps[1].order, 1);
    assert_eq!(steps[2].order, 2);

    // Test claiming a step
    let claimed = planner
        .claim_step(&beacon_core::params::Id { id: step2.id })
        .await
        .expect("Failed to claim step");
    assert!(claimed.is_some(), "Should successfully claim step2");

    // Verify step is in progress
    let steps_after_claim = planner
        .get_steps(&beacon_core::params::Id { id: plan.id })
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
        .get_steps(&beacon_core::params::Id { id: plan.id })
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
            .with_database_path(Some(db_path.clone()))
            .build()
            .await
            .expect("Failed to create first planner");

        let plan = planner
            .create_plan(&beacon_core::params::CreatePlan {
                title: "Test Plan".to_string(),
                description: None,
                directory: None,
            })
            .await
            .expect("Failed to create plan");

        planner
            .add_step(&beacon_core::params::StepCreate {
                plan_id: plan.id,
                title: "Test step".to_string(),
                description: None,
                acceptance_criteria: None,
                references: Vec::new(),
            })
            .await
            .expect("Failed to add step");

        plan.id
    };

    // Create new planner instance (simulating app restart)
    let planner = PlannerBuilder::new()
        .with_database_path(Some(db_path))
        .build()
        .await
        .expect("Failed to create second planner");

    // Verify data persisted
    let retrieved_plan = planner
        .get_plan(&beacon_core::params::Id { id: plan_id })
        .await
        .expect("Failed to retrieve plan")
        .expect("Plan should exist");

    assert_eq!(retrieved_plan.title, "Test Plan");

    let steps = planner
        .get_steps(&beacon_core::params::Id { id: plan_id })
        .await
        .expect("Failed to get steps");
    assert_eq!(steps.len(), 1);
    assert_eq!(steps[0].title, "Test step");
}

#[tokio::test]
async fn test_error_handling_invalid_operations() {
    let (_temp_dir, db_path) = create_test_environment();

    let planner = PlannerBuilder::new()
        .with_database_path(Some(db_path))
        .build()
        .await
        .expect("Failed to create planner");

    // Test operations on non-existent plan
    let result = planner
        .get_plan(&beacon_core::params::Id { id: 999 })
        .await
        .expect("Failed to query non-existent plan");
    assert!(result.is_none());

    let result = planner
        .add_step(&beacon_core::params::StepCreate {
            plan_id: 999,
            title: "Invalid step".to_string(),
            description: None,
            acceptance_criteria: None,
            references: Vec::new(),
        })
        .await;
    assert!(result.is_err());

    let result = planner
        .archive_plan(&beacon_core::params::Id { id: 999 })
        .await
        .expect("archive_plan should not error even for non-existent plans");
    assert!(result.is_none(), "Should return None for non-existent plan");

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

    let result = planner
        .remove_step(&beacon_core::params::Id { id: 999 })
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_plan_with_steps_retrieval() {
    let (_temp_dir, db_path) = create_test_environment();

    let planner = PlannerBuilder::new()
        .with_database_path(Some(db_path))
        .build()
        .await
        .expect("Failed to create planner");

    // Create a plan with steps
    let plan = planner
        .create_plan(&beacon_core::params::CreatePlan {
            title: "Test Plan".to_string(),
            description: Some("Testing step retrieval".to_string()),
            directory: None,
        })
        .await
        .expect("Failed to create plan");

    planner
        .add_step(&beacon_core::params::StepCreate {
            plan_id: plan.id,
            title: "Step 1".to_string(),
            description: None,
            acceptance_criteria: None,
            references: Vec::new(),
        })
        .await
        .expect("Failed to add step 1");
    planner
        .add_step(&beacon_core::params::StepCreate {
            plan_id: plan.id,
            title: "Step 2".to_string(),
            description: None,
            acceptance_criteria: None,
            references: Vec::new(),
        })
        .await
        .expect("Failed to add step 2");

    // Retrieve plan with steps (now eagerly loaded)
    let plan_with_steps = planner
        .get_plan(&beacon_core::params::Id { id: plan.id })
        .await
        .expect("Failed to get plan")
        .expect("Plan should exist");

    assert_eq!(plan_with_steps.steps.len(), 2);
    assert_eq!(plan_with_steps.steps[0].title, "Step 1");
    assert_eq!(plan_with_steps.steps[1].title, "Step 2");
}

#[tokio::test]
async fn test_step_removal() {
    let (_temp_dir, db_path) = create_test_environment();

    let planner = PlannerBuilder::new()
        .with_database_path(Some(db_path))
        .build()
        .await
        .expect("Failed to create planner");

    let plan = planner
        .create_plan(&beacon_core::params::CreatePlan {
            title: "Step Test".to_string(),
            description: None,
            directory: None,
        })
        .await
        .expect("Failed to create plan");

    let step1 = planner
        .add_step(&beacon_core::params::StepCreate {
            plan_id: plan.id,
            title: "Step to keep".to_string(),
            description: None,
            acceptance_criteria: None,
            references: Vec::new(),
        })
        .await
        .expect("Failed to add step");
    let step2 = planner
        .add_step(&beacon_core::params::StepCreate {
            plan_id: plan.id,
            title: "Step to remove".to_string(),
            description: None,
            acceptance_criteria: None,
            references: Vec::new(),
        })
        .await
        .expect("Failed to add step");
    let step3 = planner
        .add_step(&beacon_core::params::StepCreate {
            plan_id: plan.id,
            title: "Another step to keep".to_string(),
            description: None,
            acceptance_criteria: None,
            references: Vec::new(),
        })
        .await
        .expect("Failed to add step");

    // Remove the middle step
    planner
        .remove_step(&beacon_core::params::Id { id: step2.id })
        .await
        .expect("Failed to remove step");

    // Verify remaining steps
    let steps = planner
        .get_steps(&beacon_core::params::Id { id: plan.id })
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
        .with_database_path(Some(db_path))
        .build()
        .await
        .expect("Failed to create planner");

    let plan = planner
        .create_plan(&beacon_core::params::CreatePlan {
            title: "Archive Test".to_string(),
            description: None,
            directory: None,
        })
        .await
        .expect("Failed to create plan");

    // Add steps
    planner
        .add_step(&beacon_core::params::StepCreate {
            plan_id: plan.id,
            title: "Step 1".to_string(),
            description: None,
            acceptance_criteria: None,
            references: Vec::new(),
        })
        .await
        .expect("Failed to add step");
    planner
        .add_step(&beacon_core::params::StepCreate {
            plan_id: plan.id,
            title: "Step 2".to_string(),
            description: None,
            acceptance_criteria: None,
            references: Vec::new(),
        })
        .await
        .expect("Failed to add step");

    // Archive the plan
    let archived_plan = planner
        .archive_plan(&beacon_core::params::Id { id: plan.id })
        .await
        .expect("Failed to archive plan")
        .expect("Plan should exist");
    assert_eq!(archived_plan.id, plan.id);

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
        .get_steps(&beacon_core::params::Id { id: plan.id })
        .await
        .expect("Query should succeed");
    assert_eq!(steps.len(), 2);
}
