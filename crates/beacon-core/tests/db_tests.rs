use beacon_core::{Database, PlannerError, StepStatus, UpdateStepRequest};
use tempfile::NamedTempFile;

/// Helper function to create a temporary database for testing
fn create_test_db() -> (NamedTempFile, Database) {
    let temp_file = NamedTempFile::new().expect("Failed to create temporary file");
    let db = Database::new(temp_file.path()).expect("Failed to create test database");
    (temp_file, db)
}

#[test]
fn test_database_initialization() {
    let (_temp_file, _db) = create_test_db();

    // Database should be initialized and ready to use
    // This test passes if no panic occurs during creation
    assert!(_temp_file.path().exists());
}

#[test]
fn test_create_plan() {
    let (_temp_file, mut db) = create_test_db();

    let plan = db
        .create_plan("Test Title", Some("Test Description"), None)
        .expect("Failed to create plan");

    assert_eq!(plan.title, "Test Title");
    assert_eq!(plan.description, Some("Test Description".to_string()));
    assert!(plan.id > 0);
    assert!(plan.steps.is_empty());
}

#[test]
fn test_get_plan() {
    let (_temp_file, mut db) = create_test_db();

    let created_plan = db
        .create_plan("Get Title", None, None)
        .expect("Failed to create plan");

    let retrieved_plan = db
        .get_plan(created_plan.id)
        .expect("Failed to get plan")
        .expect("Plan should exist");

    assert_eq!(retrieved_plan.id, created_plan.id);
    assert_eq!(retrieved_plan.title, "Get Title");
    // Should have no steps initially (empty, but not a null/uninitialized vector)
    assert!(retrieved_plan.steps.is_empty());
}

#[test]
fn test_list_plans() {
    let (_temp_file, mut db) = create_test_db();

    db.create_plan("Title 1", None, None)
        .expect("Failed to create plan 1");
    db.create_plan("Title 2", None, None)
        .expect("Failed to create plan 2");
    db.create_plan("Title 3", None, None)
        .expect("Failed to create plan 3");

    let plans = db.list_plans(None).expect("Failed to list plans");
    assert_eq!(plans.len(), 3);
}

#[test]
fn test_add_step() {
    let (_temp_file, mut db) = create_test_db();

    let plan = db
        .create_plan("Step Plan", None, None)
        .expect("Failed to create plan");

    let step = db
        .add_step(plan.id, "First Step", None, None, Vec::new())
        .expect("Failed to add step");

    assert_eq!(step.plan_id, plan.id);
    assert_eq!(step.title, "First Step");
    assert_eq!(step.status, StepStatus::Todo);
    assert_eq!(step.order, 0);
}

#[test]
fn test_update_step_status() {
    let (_temp_file, mut db) = create_test_db();

    let plan = db
        .create_plan("Status Title", None, None)
        .expect("Failed to create plan");

    let step = db
        .add_step(plan.id, "Test Step", None, None, Vec::new())
        .expect("Failed to add step");

    // Test updating to InProgress
    db.update_step(
        step.id,
        UpdateStepRequest {
            status: Some(StepStatus::InProgress),
            ..Default::default()
        },
    )
    .expect("Failed to update status to InProgress");

    let steps = db.get_steps(plan.id).expect("Failed to get steps");
    assert_eq!(steps[0].status, StepStatus::InProgress);

    // Test updating to Done
    db.update_step(
        step.id,
        UpdateStepRequest {
            status: Some(StepStatus::Done),
            result: Some("Task completed successfully".to_string()),
            ..Default::default()
        },
    )
    .expect("Failed to update status to Done");

    let steps = db.get_steps(plan.id).expect("Failed to get steps");
    assert_eq!(steps[0].status, StepStatus::Done);
}

#[test]
fn test_claim_step() {
    let (_temp_file, mut db) = create_test_db();

    let plan = db
        .create_plan("Claim Title", None, None)
        .expect("Failed to create plan");

    let step = db
        .add_step(plan.id, "Test Step", None, None, Vec::new())
        .expect("Failed to add step");

    // Test claiming a todo step - should succeed
    let claimed = db.claim_step(step.id).expect("Failed to claim step");
    assert!(claimed, "Should successfully claim a todo step");

    // Verify the step is now in progress
    let steps = db.get_steps(plan.id).expect("Failed to get steps");
    assert_eq!(steps[0].status, StepStatus::InProgress);

    // Test claiming the same step again - should fail
    let claimed_again = db.claim_step(step.id).expect("Failed to claim step");
    assert!(
        !claimed_again,
        "Should not be able to claim an in-progress step"
    );

    // Test claiming a done step - should fail
    db.update_step(
        step.id,
        UpdateStepRequest {
            status: Some(StepStatus::Done),
            result: Some("Step marked as done for testing".to_string()),
            ..Default::default()
        },
    )
    .expect("Failed to update status");
    let claimed_done = db.claim_step(step.id).expect("Failed to claim step");
    assert!(!claimed_done, "Should not be able to claim a done step");

    // Test claiming non-existent step - should error
    let result = db.claim_step(999);
    assert!(
        result.is_err(),
        "Should error when claiming non-existent step"
    );
}

#[test]
fn test_get_steps() {
    let (_temp_file, mut db) = create_test_db();

    let plan = db
        .create_plan("Multi Title", None, None)
        .expect("Failed to create plan");

    db.add_step(plan.id, "Step 1", None, None, Vec::new())
        .expect("Failed to add step 1");
    db.add_step(plan.id, "Step 2", None, None, Vec::new())
        .expect("Failed to add step 2");
    db.add_step(plan.id, "Step 3", None, None, Vec::new())
        .expect("Failed to add step 3");

    let steps = db.get_steps(plan.id).expect("Failed to get steps");

    assert_eq!(steps.len(), 3);
    assert_eq!(steps[0].title, "Step 1");
    assert_eq!(steps[1].title, "Step 2");
    assert_eq!(steps[2].title, "Step 3");
    assert_eq!(steps[0].order, 0);
    assert_eq!(steps[1].order, 1);
    assert_eq!(steps[2].order, 2);
}

#[test]
fn test_remove_step() {
    let (_temp_file, mut db) = create_test_db();

    let plan = db
        .create_plan("Remove Title", None, None)
        .expect("Failed to create plan");

    let step1 = db
        .add_step(plan.id, "Keep this", None, None, Vec::new())
        .expect("Failed to add step");
    let step2 = db
        .add_step(plan.id, "Remove this", None, None, Vec::new())
        .expect("Failed to add step");
    let step3 = db
        .add_step(plan.id, "Keep this too", None, None, Vec::new())
        .expect("Failed to add step");

    db.remove_step(step2.id).expect("Failed to remove step");

    let steps = db.get_steps(plan.id).expect("Failed to get steps");
    assert_eq!(steps.len(), 2);
    assert!(steps.iter().all(|s| s.id != step2.id));
    assert!(steps.iter().any(|s| s.id == step1.id));
    assert!(steps.iter().any(|s| s.id == step3.id));
}

#[test]
fn test_insert_step_at_position() {
    let (_temp_file, mut db) = create_test_db();

    let plan = db
        .create_plan("Insert Test", None, None)
        .expect("Failed to create plan");

    // Add initial steps
    let step1 = db
        .add_step(plan.id, "Step 1", None, None, Vec::new())
        .expect("Failed to add step 1");
    let step2 = db
        .add_step(plan.id, "Step 2", None, None, Vec::new())
        .expect("Failed to add step 2");
    let step3 = db
        .add_step(plan.id, "Step 3", None, None, Vec::new())
        .expect("Failed to add step 3");

    // Insert a new step at position 1 (between Step 1 and Step 2)
    let inserted_step = db
        .insert_step(plan.id, 1, "Inserted Step", None, None, Vec::new())
        .expect("Failed to insert step");

    assert_eq!(inserted_step.order, 1);

    // Get all steps and verify their order
    let steps = db.get_steps(plan.id).expect("Failed to get steps");
    assert_eq!(steps.len(), 4);

    // Verify the order is correct
    assert_eq!(steps[0].id, step1.id);
    assert_eq!(steps[0].order, 0);
    assert_eq!(steps[1].id, inserted_step.id);
    assert_eq!(steps[1].order, 1);
    assert_eq!(steps[2].id, step2.id);
    assert_eq!(steps[2].order, 2);
    assert_eq!(steps[3].id, step3.id);
    assert_eq!(steps[3].order, 3);
}

#[test]
fn test_insert_step_at_beginning() {
    let (_temp_file, mut db) = create_test_db();

    let plan = db
        .create_plan("Insert Beginning Test", None, None)
        .expect("Failed to create plan");

    // Add initial steps
    let step1 = db
        .add_step(plan.id, "Step 1", None, None, Vec::new())
        .expect("Failed to add step 1");
    let step2 = db
        .add_step(plan.id, "Step 2", None, None, Vec::new())
        .expect("Failed to add step 2");

    // Insert a new step at position 0 (beginning)
    let inserted_step = db
        .insert_step(plan.id, 0, "First Step", None, None, Vec::new())
        .expect("Failed to insert step");

    assert_eq!(inserted_step.order, 0);

    // Get all steps and verify their order
    let steps = db.get_steps(plan.id).expect("Failed to get steps");
    assert_eq!(steps.len(), 3);

    assert_eq!(steps[0].id, inserted_step.id);
    assert_eq!(steps[0].order, 0);
    assert_eq!(steps[1].id, step1.id);
    assert_eq!(steps[1].order, 1);
    assert_eq!(steps[2].id, step2.id);
    assert_eq!(steps[2].order, 2);
}

#[test]
fn test_insert_step_at_end() {
    let (_temp_file, mut db) = create_test_db();

    let plan = db
        .create_plan("Insert End Test", None, None)
        .expect("Failed to create plan");

    // Add initial steps
    db.add_step(plan.id, "Step 1", None, None, Vec::new())
        .expect("Failed to add step 1");
    db.add_step(plan.id, "Step 2", None, None, Vec::new())
        .expect("Failed to add step 2");

    // Insert a new step at position 2 (end)
    let inserted_step = db
        .insert_step(plan.id, 2, "Last Step", None, None, Vec::new())
        .expect("Failed to insert step");

    assert_eq!(inserted_step.order, 2);

    let steps = db.get_steps(plan.id).expect("Failed to get steps");
    assert_eq!(steps.len(), 3);
    assert_eq!(steps[2].title, "Last Step");
}

#[test]
fn test_insert_step_out_of_range() {
    let (_temp_file, mut db) = create_test_db();

    let plan = db
        .create_plan("Insert Range Test", None, None)
        .expect("Failed to create plan");

    // Add two steps
    db.add_step(plan.id, "Step 1", None, None, Vec::new())
        .expect("Failed to add step 1");
    db.add_step(plan.id, "Step 2", None, None, Vec::new())
        .expect("Failed to add step 2");

    // Try to insert at position 3 (out of range, should fail)
    let result = db.insert_step(plan.id, 3, "Out of Range", None, None, Vec::new());
    assert!(result.is_err());
}

#[test]
fn test_insert_step_empty_plan() {
    let (_temp_file, mut db) = create_test_db();

    let plan = db
        .create_plan("Empty Plan", None, None)
        .expect("Failed to create plan");

    // Insert into empty plan at position 0
    let inserted_step = db
        .insert_step(plan.id, 0, "First Step", None, None, Vec::new())
        .expect("Failed to insert step");

    assert_eq!(inserted_step.order, 0);

    let steps = db.get_steps(plan.id).expect("Failed to get steps");
    assert_eq!(steps.len(), 1);
}

#[test]
fn test_transaction_rollback_on_error() {
    let (_temp_file, mut db) = create_test_db();

    // Try to add a step to a non-existent plan
    let result = db.add_step(999, "Invalid step", None, None, Vec::new());
    assert!(result.is_err());

    // The database should still be functional
    let plan = db
        .create_plan("Error Title", None, None)
        .expect("Should be able to create plan after error");
    assert!(plan.id > 0);
}

#[test]
fn test_duplicate_plan_titles_allowed() {
    let (_temp_file, mut db) = create_test_db();

    let plan1 = db
        .create_plan("Duplicate Title", None, None)
        .expect("Failed to create first plan");
    let plan2 = db
        .create_plan("Duplicate Title", None, None)
        .expect("Failed to create second plan");

    assert_ne!(plan1.id, plan2.id);
    assert_eq!(plan1.title, plan2.title);
}

#[test]
fn test_directory_path_conversion() {
    let (_temp_file, mut db) = create_test_db();

    // Test relative path conversion
    let relative_plan = db
        .create_plan("Relative Title", None, Some("projects/test"))
        .expect("Failed to create plan with relative path");

    // Directory should be converted to absolute path
    assert!(relative_plan.directory.as_ref().unwrap().starts_with('/'));
    assert!(relative_plan
        .directory
        .as_ref()
        .unwrap()
        .ends_with("projects/test"));

    // Test absolute path preservation
    let absolute_plan = db
        .create_plan("Absolute Title", None, Some("/tmp/beacon-test"))
        .expect("Failed to create plan with absolute path");

    assert_eq!(
        absolute_plan.directory.as_ref().unwrap(),
        "/tmp/beacon-test"
    );

    // Test default directory (current working directory)
    let default_plan = db
        .create_plan("Default Title", None, None)
        .expect("Failed to create plan with default directory");

    // Should have a directory and it should be absolute
    assert!(default_plan.directory.is_some());
    assert!(default_plan.directory.as_ref().unwrap().starts_with('/'));
}

#[test]
fn test_empty_relative_path_conversion() {
    let (_temp_file, mut db) = create_test_db();

    // Test empty string directory (should be treated as current directory)
    let empty_plan = db
        .create_plan("Empty Title", None, Some(""))
        .expect("Failed to create plan with empty path");

    // Empty path should be converted to current working directory
    assert!(empty_plan.directory.is_some());
    assert!(empty_plan.directory.as_ref().unwrap().starts_with('/'));

    // Test dot directory (current directory)
    let dot_plan = db
        .create_plan("Dot Title", None, Some("."))
        .expect("Failed to create plan with dot path");

    // Get the current working directory for comparison
    let expected_cwd = std::env::current_dir()
        .expect("Should be able to get current directory")
        .canonicalize()
        .unwrap()
        .to_str()
        .expect("Should be able to convert to string")
        .to_string();

    // Dot should be converted to current working directory
    assert!(dot_plan.directory.is_some());
    assert!(dot_plan.directory.as_ref().unwrap().starts_with('/'));
    assert_eq!(dot_plan.directory.as_ref().unwrap(), &expected_cwd);
}

#[test]
fn test_swap_steps_same_plan() {
    let (_temp_file, mut db) = create_test_db();

    let plan = db
        .create_plan("Swap Test Plan", None, None)
        .expect("Failed to create plan");

    // Add four steps
    let step1 = db
        .add_step(plan.id, "Step 1", None, None, Vec::new())
        .expect("Failed to add step 1");
    let step2 = db
        .add_step(plan.id, "Step 2", None, None, Vec::new())
        .expect("Failed to add step 2");
    let step3 = db
        .add_step(plan.id, "Step 3", None, None, Vec::new())
        .expect("Failed to add step 3");
    let step4 = db
        .add_step(plan.id, "Step 4", None, None, Vec::new())
        .expect("Failed to add step 4");

    // Initial order should be 0, 1, 2, 3
    assert_eq!(step1.order, 0);
    assert_eq!(step2.order, 1);
    assert_eq!(step3.order, 2);
    assert_eq!(step4.order, 3);

    // Swap step2 and step3
    db.swap_steps(step2.id, step3.id)
        .expect("Failed to swap steps");

    // Get updated steps
    let steps = db.get_steps(plan.id).expect("Failed to get steps");

    // Find the swapped steps
    let updated_step2 = steps.iter().find(|s| s.id == step2.id).unwrap();
    let updated_step3 = steps.iter().find(|s| s.id == step3.id).unwrap();

    // Check that orders have been swapped
    assert_eq!(updated_step2.order, 2);
    assert_eq!(updated_step3.order, 1);

    // Other steps should remain unchanged
    let updated_step1 = steps.iter().find(|s| s.id == step1.id).unwrap();
    let updated_step4 = steps.iter().find(|s| s.id == step4.id).unwrap();
    assert_eq!(updated_step1.order, 0);
    assert_eq!(updated_step4.order, 3);
}

#[test]
fn test_swap_steps_different_plans() {
    let (_temp_file, mut db) = create_test_db();

    let plan1 = db
        .create_plan("Plan 1", None, None)
        .expect("Failed to create plan 1");
    let plan2 = db
        .create_plan("Plan 2", None, None)
        .expect("Failed to create plan 2");

    let step1 = db
        .add_step(plan1.id, "Plan 1 Step", None, None, Vec::new())
        .expect("Failed to add step to plan 1");
    let step2 = db
        .add_step(plan2.id, "Plan 2 Step", None, None, Vec::new())
        .expect("Failed to add step to plan 2");

    // Attempting to swap steps from different plans should fail
    let result = db.swap_steps(step1.id, step2.id);
    assert!(result.is_err());

    match result.unwrap_err() {
        PlannerError::InvalidInput { field, reason } => {
            assert_eq!(field, "step_ids");
            assert!(reason.contains("same plan"));
        }
        _ => panic!("Expected InvalidInput error"),
    }
}

#[test]
fn test_swap_steps_with_self() {
    let (_temp_file, mut db) = create_test_db();

    let plan = db
        .create_plan("Self Swap Test", None, None)
        .expect("Failed to create plan");

    let step = db
        .add_step(plan.id, "Step", None, None, Vec::new())
        .expect("Failed to add step");

    // Swapping a step with itself should be a no-op (succeed without changes)
    db.swap_steps(step.id, step.id)
        .expect("Swapping with self should succeed");

    // Step order should remain unchanged
    let updated_step = db
        .get_step(step.id)
        .expect("Failed to get step")
        .expect("Step should exist");
    assert_eq!(updated_step.order, 0);
}

#[test]
fn test_swap_nonexistent_steps() {
    let (_temp_file, mut db) = create_test_db();

    let plan = db
        .create_plan("Test Plan", None, None)
        .expect("Failed to create plan");

    let step = db
        .add_step(plan.id, "Existing Step", None, None, Vec::new())
        .expect("Failed to add step");

    // Try to swap with a non-existent step
    let result = db.swap_steps(step.id, 99999);
    assert!(result.is_err());

    match result.unwrap_err() {
        PlannerError::StepNotFound { id } => {
            assert_eq!(id, 99999);
        }
        _ => panic!("Expected StepNotFound error"),
    }
}

#[test]
fn test_update_step_to_done_requires_result() {
    let (_temp_file, mut db) = create_test_db();

    let plan = db
        .create_plan("Test Plan", None, None)
        .expect("Failed to create plan");

    let step = db
        .add_step(plan.id, "Test Step", None, None, Vec::new())
        .expect("Failed to add step");

    // Try to mark step as done without result
    let result = db.update_step(
        step.id,
        UpdateStepRequest {
            status: Some(StepStatus::Done),
            result: None, // No result provided
            ..Default::default()
        },
    );

    assert!(result.is_err());
    match result.unwrap_err() {
        PlannerError::InvalidInput { field, reason } => {
            assert_eq!(field, "result");
            assert!(reason.contains("required"));
        }
        _ => panic!("Expected InvalidInput error for missing result"),
    }

    // Now mark as done with result
    let result = db.update_step(
        step.id,
        UpdateStepRequest {
            status: Some(StepStatus::Done),
            result: Some("Successfully completed the test step".to_string()),
            ..Default::default()
        },
    );
    assert!(result.is_ok());

    // Verify the step was updated with result
    let updated_step = db
        .get_step(step.id)
        .expect("Failed to get step")
        .expect("Step should exist");
    assert_eq!(updated_step.status, StepStatus::Done);
    assert_eq!(
        updated_step.result,
        Some("Successfully completed the test step".to_string())
    );
}

#[test]
fn test_update_step_result_ignored_for_non_done_status() {
    let (_temp_file, mut db) = create_test_db();

    let plan = db
        .create_plan("Test Plan", None, None)
        .expect("Failed to create plan");

    let step = db
        .add_step(plan.id, "Test Step", None, None, Vec::new())
        .expect("Failed to add step");

    // Update to in-progress with result (should be ignored)
    db.update_step(
        step.id,
        UpdateStepRequest {
            status: Some(StepStatus::InProgress),
            result: Some("This should be ignored".to_string()),
            ..Default::default()
        },
    )
    .expect("Failed to update step");

    let updated_step = db
        .get_step(step.id)
        .expect("Failed to get step")
        .expect("Step should exist");
    assert_eq!(updated_step.status, StepStatus::InProgress);
    assert_eq!(updated_step.result, None); // Result should be None

    // Mark as done with result
    db.update_step(
        step.id,
        UpdateStepRequest {
            status: Some(StepStatus::Done),
            result: Some("Completed successfully".to_string()),
            ..Default::default()
        },
    )
    .expect("Failed to update step");

    let updated_step = db
        .get_step(step.id)
        .expect("Failed to get step")
        .expect("Step should exist");
    assert_eq!(updated_step.status, StepStatus::Done);
    assert_eq!(
        updated_step.result,
        Some("Completed successfully".to_string())
    );

    // Change back to todo (result should be cleared)
    db.update_step(
        step.id,
        UpdateStepRequest {
            status: Some(StepStatus::Todo),
            result: Some("This should also be ignored".to_string()),
            ..Default::default()
        },
    )
    .expect("Failed to update step");

    let updated_step = db
        .get_step(step.id)
        .expect("Failed to get step")
        .expect("Step should exist");
    assert_eq!(updated_step.status, StepStatus::Todo);
    assert_eq!(updated_step.result, None); // Result should be cleared
}

#[test]
fn test_delete_plan() {
    let (_temp_file, mut db) = create_test_db();

    // Create a plan with some steps
    let plan = db
        .create_plan("Test Plan", Some("A plan to be deleted"), None)
        .expect("Failed to create plan");

    let step1 = db
        .add_step(plan.id, "Step 1", None, None, Vec::new())
        .expect("Failed to add step 1");
    let step2 = db
        .add_step(plan.id, "Step 2", None, None, Vec::new())
        .expect("Failed to add step 2");

    // Verify plan and steps exist
    assert!(db.get_plan(plan.id).expect("Failed to get plan").is_some());
    assert!(db.get_step(step1.id).expect("Failed to get step").is_some());
    assert!(db.get_step(step2.id).expect("Failed to get step").is_some());

    // Delete the plan
    db.delete_plan(plan.id).expect("Failed to delete plan");

    // Verify plan and steps are deleted
    assert!(db.get_plan(plan.id).expect("Failed to get plan").is_none());
    assert!(db.get_step(step1.id).expect("Failed to get step").is_none());
    assert!(db.get_step(step2.id).expect("Failed to get step").is_none());
}

#[test]
fn test_delete_nonexistent_plan() {
    let (_temp_file, mut db) = create_test_db();

    // Try to delete a plan that doesn't exist
    let result = db.delete_plan(999);
    assert!(result.is_err());

    match result.unwrap_err() {
        PlannerError::PlanNotFound { id } => {
            assert_eq!(id, 999);
        }
        _ => panic!("Expected PlanNotFound error"),
    }
}

#[test]
fn test_get_plan_with_eager_loaded_steps() {
    let (_temp_file, mut db) = create_test_db();

    // Create a plan
    let plan = db
        .create_plan("Test Plan with Steps", None, None)
        .expect("Failed to create plan");

    // Add some steps
    let step1 = db
        .add_step(plan.id, "First Step", None, None, Vec::new())
        .expect("Failed to add first step");
    let step2 = db
        .add_step(plan.id, "Second Step", None, None, Vec::new())
        .expect("Failed to add second step");

    // Get the plan - should have steps eagerly loaded
    let retrieved_plan = db
        .get_plan(plan.id)
        .expect("Failed to get plan")
        .expect("Plan should exist");

    // Verify steps are loaded
    assert_eq!(retrieved_plan.steps.len(), 2);
    assert_eq!(retrieved_plan.steps[0].id, step1.id);
    assert_eq!(retrieved_plan.steps[0].title, "First Step");
    assert_eq!(retrieved_plan.steps[1].id, step2.id);
    assert_eq!(retrieved_plan.steps[1].title, "Second Step");
}

#[test]
fn test_list_plans_with_eager_loaded_steps() {
    let (_temp_file, mut db) = create_test_db();

    // Create two plans
    let plan1 = db
        .create_plan("Plan One", None, None)
        .expect("Failed to create plan 1");
    let plan2 = db
        .create_plan("Plan Two", None, None)
        .expect("Failed to create plan 2");

    // Add steps to first plan
    db.add_step(plan1.id, "Plan 1 Step 1", None, None, Vec::new())
        .expect("Failed to add step to plan 1");
    db.add_step(plan1.id, "Plan 1 Step 2", None, None, Vec::new())
        .expect("Failed to add second step to plan 1");

    // Add one step to second plan
    db.add_step(plan2.id, "Plan 2 Step 1", None, None, Vec::new())
        .expect("Failed to add step to plan 2");

    // List plans - should have steps eagerly loaded
    let plans = db.list_plans(None).expect("Failed to list plans");

    assert_eq!(plans.len(), 2);

    // Find plans in the results (order may vary)
    let retrieved_plan1 = plans
        .iter()
        .find(|p| p.id == plan1.id)
        .expect("Plan 1 should be found");
    let retrieved_plan2 = plans
        .iter()
        .find(|p| p.id == plan2.id)
        .expect("Plan 2 should be found");

    // Verify steps are loaded for plan 1
    assert_eq!(retrieved_plan1.steps.len(), 2);
    assert_eq!(retrieved_plan1.steps[0].title, "Plan 1 Step 1");
    assert_eq!(retrieved_plan1.steps[1].title, "Plan 1 Step 2");

    // Verify steps are loaded for plan 2
    assert_eq!(retrieved_plan2.steps.len(), 1);
    assert_eq!(retrieved_plan2.steps[0].title, "Plan 2 Step 1");
}

#[test]
fn test_performance_eager_loading() {
    let (_temp_file, mut db) = create_test_db();

    // Create multiple plans with several steps each
    let mut plan_ids = Vec::new();
    for i in 1..=10 {
        let plan = db
            .create_plan(&format!("Performance Plan {}", i), None, None)
            .expect("Failed to create plan");
        plan_ids.push(plan.id);

        // Add 5 steps to each plan
        for j in 1..=5 {
            db.add_step(
                plan.id,
                &format!("Step {} for Plan {}", j, i),
                None,
                None,
                Vec::new(),
            )
            .expect("Failed to add step");
        }
    }

    let start = std::time::Instant::now();
    let plans = db.list_plans(None).expect("Failed to list plans");
    let duration = start.elapsed();

    // Verify all plans have their steps loaded
    assert_eq!(plans.len(), 10);
    for plan in &plans {
        assert_eq!(plan.steps.len(), 5);
    }

    // For personal use, this should be very fast even with eager loading
    // Using 100ms as a reasonable threshold for 10 plans with 5 steps each
    assert!(
        duration.as_millis() < 100,
        "Eager loading took too long: {:?}",
        duration
    );
    println!(
        "Eager loading performance: loaded {} plans with {} total steps in {:?}",
        plans.len(),
        plans.iter().map(|p| p.steps.len()).sum::<usize>(),
        duration
    );
}
