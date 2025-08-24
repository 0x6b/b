#[cfg(test)]
mod model_tests {
    use jiff::Timestamp;

    use crate::{
        display::LocalDateTime,
        models::{Plan, PlanFilter, PlanStatus, PlanSummary, Step, StepStatus, UpdateStepRequest},
    };

    fn create_test_step(status: StepStatus) -> Step {
        Step {
            id: 123,
            plan_id: 456,
            title: "Test Step Title".to_string(),
            description: Some("This is a test step description".to_string()),
            acceptance_criteria: Some("Should pass all tests".to_string()),
            references: vec!["https://example.com".to_string(), "file.txt".to_string()],
            status,
            result: if status == StepStatus::Done {
                Some("Successfully completed the test".to_string())
            } else {
                None
            },
            order: 2,
            created_at: Timestamp::from_second(1640995200).unwrap(), // 2022-01-01 00:00:00 UTC
            updated_at: Timestamp::from_second(1641081600).unwrap(), // 2022-01-02 00:00:00 UTC
        }
    }

    fn create_test_plan() -> Plan {
        Plan {
            id: 789,
            title: "Test Plan Title".to_string(),
            description: Some("This is a test plan".to_string()),
            status: PlanStatus::Active,
            directory: Some("/test/path".to_string()),
            created_at: Timestamp::from_second(1640995200).unwrap(),
            updated_at: Timestamp::from_second(1641081600).unwrap(),
            steps: vec![
                create_test_step(StepStatus::Done),
                create_test_step(StepStatus::InProgress),
                create_test_step(StepStatus::Todo),
            ],
        }
    }

    fn create_test_plan_summary() -> PlanSummary {
        PlanSummary {
            id: 789,
            title: "Test Plan Summary".to_string(),
            description: Some("Summary description".to_string()),
            status: PlanStatus::Active,
            directory: Some("/test/summary".to_string()),
            created_at: Timestamp::from_second(1640995200).unwrap(),
            updated_at: Timestamp::from_second(1641081600).unwrap(),
            total_steps: 5,
            completed_steps: 2,
            pending_steps: 3,
        }
    }

    #[test]
    fn test_step_status_with_icon() {
        assert_eq!(StepStatus::Done.with_icon(), "✓ Done");
        assert_eq!(StepStatus::InProgress.with_icon(), "➤ In Progress");
        assert_eq!(StepStatus::Todo.with_icon(), "○ Todo");
    }

    #[test]
    fn test_step_display_independently_todo() {
        let step = create_test_step(StepStatus::Todo);
        let output = format!("{}", step);

        // Should contain step header with ID and status
        assert!(output.contains("### 123. Test Step Title (○ Todo)"));

        // Should contain description and acceptance criteria
        assert!(output.contains("This is a test step description"));
        assert!(output.contains("#### Acceptance"));
        assert!(output.contains("Should pass all tests"));

        // Should contain references
        assert!(output.contains("#### References"));
        assert!(output.contains("- https://example.com"));
        assert!(output.contains("- file.txt"));

        // Should NOT contain result section for todo steps
        assert!(!output.contains("#### Result"));
    }

    #[test]
    fn test_step_display_independently_in_progress() {
        let step = create_test_step(StepStatus::InProgress);
        let output = format!("{}", step);

        assert!(output.contains("### 123. Test Step Title (➤ In Progress)"));
        assert!(!output.contains("#### Result"));
    }

    #[test]
    fn test_step_display_independently_done() {
        let step = create_test_step(StepStatus::Done);
        let output = format!("{}", step);

        assert!(output.contains("### 123. Test Step Title (✓ Done)"));
        assert!(output.contains("#### Result"));
        assert!(output.contains("Successfully completed the test"));
    }

    #[test]
    fn test_step_display_within_plan_context() {
        let step = create_test_step(StepStatus::InProgress);
        let output = format!("{}", step);

        // Should use consistent formatting with step ID
        assert!(output.contains("### 123. Test Step Title (➤ In Progress)"));
        assert!(output.contains("#### Acceptance"));
        assert!(output.contains("#### References"));
    }

    #[test]
    fn test_plan_display_with_steps() {
        let plan = create_test_plan();
        let output = format!("{}", plan);

        // Should contain plan header
        assert!(output.contains("# 789. Test Plan Title"));

        // Should contain metadata
        assert!(output.contains("- Status: active"));
        assert!(output.contains("- Directory: /test/path"));
        assert!(output.contains("- Created: 2022-01-01"));
        assert!(output.contains("- Updated: 2022-01-02"));

        // Should contain description
        assert!(output.contains("This is a test plan"));

        // Should contain steps section
        assert!(output.contains("## Steps"));

        // Should contain step status icons in plan context
        assert!(output.contains("✓ Done"));
        assert!(output.contains("➤ In Progress"));
        assert!(output.contains("○ Todo"));
    }

    #[test]
    fn test_plan_display_empty_steps() {
        let mut plan = create_test_plan();
        plan.steps.clear();
        let output = format!("{}", plan);

        assert!(output.contains("No steps in this plan."));
        assert!(!output.contains("## Steps"));
    }

    #[test]
    fn test_plan_summary_display_with_progress() {
        let summary = create_test_plan_summary();
        let output = format!("{}", summary);

        // Should contain title with progress
        assert!(output.contains("## Test Plan Summary (ID: 789) (2/5)"));

        // Should contain metadata
        assert!(output.contains("- **Description**: Summary description"));
        assert!(output.contains("- **Directory**: /test/summary"));
        assert!(output.contains("- **Created**: 2022-01-01"));

        // Should have blank line at end
        assert!(output.ends_with("\n\n"));
    }

    #[test]
    fn test_plan_summary_display_no_steps() {
        let mut summary = create_test_plan_summary();
        summary.total_steps = 0;
        summary.completed_steps = 0;
        summary.pending_steps = 0;
        let output = format!("{}", summary);

        // Should not show progress when no steps
        assert!(output.contains("## Test Plan Summary (ID: 789)"));
        assert!(!output.contains("(0/0)"));
    }

    #[test]
    fn test_plan_summary_display_minimal_info() {
        let mut summary = create_test_plan_summary();
        summary.description = None;
        summary.directory = None;
        let output = format!("{}", summary);

        // Should still contain basic info
        assert!(output.contains("## Test Plan Summary (ID: 789) (2/5)"));
        assert!(output.contains("- **Created**: 2022-01-01"));

        // Should not contain optional fields
        assert!(!output.contains("- **Description**:"));
        assert!(!output.contains("- **Directory**:"));
    }

    #[test]
    fn test_step_status_display_consistency() {
        // Test that status icons are consistent across all display contexts
        let todo_step = create_test_step(StepStatus::Todo);
        let in_progress_step = create_test_step(StepStatus::InProgress);
        let done_step = create_test_step(StepStatus::Done);

        // Independent step display
        let todo_output = format!("{}", todo_step);
        let in_progress_output = format!("{}", in_progress_step);
        let done_output = format!("{}", done_step);

        assert!(todo_output.contains("○ Todo"));
        assert!(in_progress_output.contains("➤ In Progress"));
        assert!(done_output.contains("✓ Done"));

        // Plan context display
        let todo_pos_output = format!("{}", todo_step);
        let in_progress_pos_output = format!("{}", in_progress_step);
        let done_pos_output = format!("{}", done_step);

        assert!(todo_pos_output.contains("○ Todo"));
        assert!(in_progress_pos_output.contains("➤ In Progress"));
        assert!(done_pos_output.contains("✓ Done"));
    }

    #[test]
    fn test_plan_summary_from_plan_trait() {
        let plan = create_test_plan();
        let summary = PlanSummary::from(&plan);

        // Verify basic plan information is copied correctly
        assert_eq!(summary.id, plan.id);
        assert_eq!(summary.title, plan.title);
        assert_eq!(summary.description, plan.description);
        assert_eq!(summary.status, plan.status);
        assert_eq!(summary.directory, plan.directory);
        assert_eq!(summary.created_at, plan.created_at);
        assert_eq!(summary.updated_at, plan.updated_at);

        // Verify step counts are calculated correctly
        // The test plan has 3 steps: Done, InProgress, Todo
        assert_eq!(summary.total_steps, 3);
        assert_eq!(summary.completed_steps, 1); // Only the Done step
        assert_eq!(summary.pending_steps, 2); // InProgress + Todo steps
    }

    #[test]
    fn test_plan_summary_from_plan_trait_empty_steps() {
        let mut plan = create_test_plan();
        plan.steps.clear();
        let summary = PlanSummary::from(&plan);

        // Verify step counts for empty plan
        assert_eq!(summary.total_steps, 0);
        assert_eq!(summary.completed_steps, 0);
        assert_eq!(summary.pending_steps, 0);
    }

    #[test]
    fn test_plan_summary_from_plan_trait_all_completed() {
        let mut plan = create_test_plan();
        // Make all steps completed
        for step in &mut plan.steps {
            step.status = StepStatus::Done;
        }
        let summary = PlanSummary::from(&plan);

        // Verify step counts when all steps are completed
        assert_eq!(summary.total_steps, 3);
        assert_eq!(summary.completed_steps, 3);
        assert_eq!(summary.pending_steps, 0);
    }

    #[test]
    fn test_plan_filter_from_list_plans_active() {
        use crate::params::ListPlans;

        let params = ListPlans { archived: false };
        let filter: PlanFilter = (&params).into();

        assert_eq!(filter.status, Some(PlanStatus::Active));
        assert!(!filter.include_archived);
        assert_eq!(filter.title_contains, None);
        assert_eq!(filter.directory, None);
        assert_eq!(filter.created_after, None);
        assert_eq!(filter.created_before, None);
        assert_eq!(filter.completion_status, None);
    }

    #[test]
    fn test_plan_filter_from_list_plans_archived() {
        use crate::params::ListPlans;

        let params = ListPlans { archived: true };
        let filter: PlanFilter = (&params).into();

        assert_eq!(filter.status, Some(PlanStatus::Archived));
        assert!(filter.include_archived);
        assert_eq!(filter.title_contains, None);
        assert_eq!(filter.directory, None);
        assert_eq!(filter.created_after, None);
        assert_eq!(filter.created_before, None);
        assert_eq!(filter.completion_status, None);
    }

    #[test]
    fn test_plan_filter_for_directory_active() {
        let directory = "/path/to/project".to_string();
        let filter = PlanFilter::for_directory(directory.clone(), false);

        assert_eq!(filter.status, Some(PlanStatus::Active));
        assert_eq!(filter.directory, Some(directory));
        assert!(!filter.include_archived);
        // Verify other fields use defaults
        assert_eq!(filter.title_contains, None);
        assert_eq!(filter.created_after, None);
        assert_eq!(filter.created_before, None);
        assert_eq!(filter.completion_status, None);
    }

    #[test]
    fn test_plan_filter_for_directory_archived() {
        let directory = "/path/to/archived".to_string();
        let filter = PlanFilter::for_directory(directory.clone(), true);

        assert_eq!(filter.status, Some(PlanStatus::Archived));
        assert_eq!(filter.directory, Some(directory));
        assert!(filter.include_archived);
        // Verify other fields use defaults
        assert_eq!(filter.title_contains, None);
        assert_eq!(filter.created_after, None);
        assert_eq!(filter.created_before, None);
        assert_eq!(filter.completion_status, None);
    }

    #[test]
    fn test_update_step_request_new_constructor() {
        let request = UpdateStepRequest::new(
            Some("Test Title".to_string()),
            Some("Test Description".to_string()),
            Some("Test Acceptance".to_string()),
            Some(vec!["ref1.txt".to_string(), "ref2.txt".to_string()]),
            Some(StepStatus::Done),
            Some("Test Result".to_string()),
        );

        assert_eq!(request.title, Some("Test Title".to_string()));
        assert_eq!(request.description, Some("Test Description".to_string()));
        assert_eq!(
            request.acceptance_criteria,
            Some("Test Acceptance".to_string())
        );
        assert_eq!(
            request.references,
            Some(vec!["ref1.txt".to_string(), "ref2.txt".to_string()])
        );
        assert_eq!(request.status, Some(StepStatus::Done));
        assert_eq!(request.result, Some("Test Result".to_string()));
    }

    #[test]
    fn test_update_step_request_new_constructor_minimal() {
        let request = UpdateStepRequest::new(None, None, None, None, None, None);

        assert_eq!(request.title, None);
        assert_eq!(request.description, None);
        assert_eq!(request.acceptance_criteria, None);
        assert_eq!(request.references, None);
        assert_eq!(request.status, None);
        assert_eq!(request.result, None);
    }

    #[test]
    fn test_update_step_request_try_from_valid_todo() {
        use crate::params::UpdateStep;

        let params = UpdateStep {
            id: 1,
            status: Some("todo".to_string()),
            title: Some("Updated Title".to_string()),
            description: Some("Updated Description".to_string()),
            ..Default::default()
        };

        let result: Result<UpdateStepRequest, _> = params.try_into();
        assert!(result.is_ok());

        let request = result.unwrap();
        assert_eq!(request.title, Some("Updated Title".to_string()));
        assert_eq!(request.description, Some("Updated Description".to_string()));
        assert_eq!(request.status, Some(StepStatus::Todo));
        assert_eq!(request.result, None);
    }

    #[test]
    fn test_update_step_request_try_from_valid_done_with_result() {
        use crate::params::UpdateStep;

        let params = UpdateStep {
            id: 1,
            status: Some("done".to_string()),
            result: Some("Task completed successfully".to_string()),
            acceptance_criteria: Some("Must pass all tests".to_string()),
            references: Some(vec!["file.txt".to_string()]),
            ..Default::default()
        };

        let result: Result<UpdateStepRequest, _> = params.try_into();
        assert!(result.is_ok());

        let request = result.unwrap();
        assert_eq!(request.status, Some(StepStatus::Done));
        assert_eq!(
            request.result,
            Some("Task completed successfully".to_string())
        );
        assert_eq!(
            request.acceptance_criteria,
            Some("Must pass all tests".to_string())
        );
        assert_eq!(request.references, Some(vec!["file.txt".to_string()]));
    }

    #[test]
    fn test_update_step_request_try_from_done_missing_result() {
        use crate::params::UpdateStep;

        let params = UpdateStep {
            id: 1,
            status: Some("done".to_string()),
            result: None,
            ..Default::default()
        }; // Missing result for done status

        let result: Result<UpdateStepRequest, _> = params.try_into();
        assert!(result.is_err());

        match result.unwrap_err() {
            crate::PlannerError::InvalidInput { field, reason } => {
                assert_eq!(field, "result");
                assert!(reason.contains("Result description is required"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[test]
    fn test_update_step_request_try_from_invalid_status() {
        use crate::params::UpdateStep;

        let params = UpdateStep {
            id: 1,
            status: Some("invalid_status".to_string()),
            ..Default::default()
        };

        let result: Result<UpdateStepRequest, _> = params.try_into();
        assert!(result.is_err());

        match result.unwrap_err() {
            crate::PlannerError::InvalidInput { field, reason } => {
                assert_eq!(field, "status");
                assert!(reason.contains("Invalid status: invalid_status"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[test]
    fn test_update_step_request_try_from_no_changes() {
        use crate::params::UpdateStep;

        let params = UpdateStep::default(); // All fields None

        let result: Result<UpdateStepRequest, _> = params.try_into();
        assert!(result.is_ok());

        let request = result.unwrap();
        assert_eq!(request.title, None);
        assert_eq!(request.description, None);
        assert_eq!(request.acceptance_criteria, None);
        assert_eq!(request.references, None);
        assert_eq!(request.status, None);
        assert_eq!(request.result, None);
    }

    #[test]
    fn test_create_update_request_all_fields() {
        let request = UpdateStepRequest::new(
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
        let request = UpdateStepRequest::new(None, None, None, None, None, None);

        assert_eq!(request.title, None);
        assert_eq!(request.description, None);
        assert_eq!(request.acceptance_criteria, None);
        assert_eq!(request.references, None);
        assert_eq!(request.status, None);
        assert_eq!(request.result, None);
    }

    #[test]
    fn test_create_directory_filter_active() {
        let directory = "/path/to/project".to_string();
        let filter = PlanFilter::for_directory(directory.clone(), false);

        assert_eq!(filter.status, Some(PlanStatus::Active));
        assert_eq!(filter.directory, Some(directory));
        assert!(!filter.include_archived);
    }

    #[test]
    fn test_create_directory_filter_archived() {
        let directory = "/path/to/project".to_string();
        let filter = PlanFilter::for_directory(directory.clone(), true);

        assert_eq!(filter.status, Some(PlanStatus::Archived));
        assert_eq!(filter.directory, Some(directory));
        assert!(filter.include_archived);
    }

    #[test]
    fn test_local_date_time_new() {
        let timestamp = Timestamp::from_second(1640995200).unwrap(); // 2022-01-01 00:00:00 UTC
        let local_dt = LocalDateTime(&timestamp);

        // Verify the wrapper holds the correct timestamp
        assert_eq!(local_dt.0, &timestamp);
    }

    #[test]
    fn test_local_date_time_display_format() {
        let timestamp = Timestamp::from_second(1640995200).unwrap(); // 2022-01-01 00:00:00 UTC
        let local_dt = LocalDateTime(&timestamp);
        let output = format!("{}", local_dt);

        // Should contain date in YYYY-MM-DD format
        assert!(output.contains("2022-01-01"));
        // Should contain time components (exact time depends on system timezone)
        assert!(output.contains(":"));
        // Should contain timezone info
        let parts: Vec<&str> = output.split_whitespace().collect();
        assert_eq!(parts.len(), 3); // Date, Time, Timezone
        assert_eq!(parts[0], "2022-01-01");
        assert!(parts[1].contains(":")); // Time has colons
        assert!(!parts[2].is_empty()); // Timezone is non-empty
    }

    #[test]
    fn test_local_date_time_different_timestamps() {
        // Test with different timestamps to ensure formatting works consistently
        let timestamps = vec![
            Timestamp::from_second(1640995200).unwrap(), // 2022-01-01 00:00:00 UTC
            Timestamp::from_second(1672531200).unwrap(), // 2023-01-01 00:00:00 UTC
            Timestamp::from_second(1704067200).unwrap(), // 2024-01-01 00:00:00 UTC
        ];

        for timestamp in timestamps {
            let local_dt = LocalDateTime(&timestamp);
            let local_dt_output = format!("{}", local_dt);

            // Each should have the expected format structure
            let parts: Vec<&str> = local_dt_output.split_whitespace().collect();
            assert_eq!(parts.len(), 3); // Date, Time, Timezone
            assert!(parts[1].contains(":")); // Time component
            assert!(!local_dt_output.is_empty()); // Output should not be empty
        }
    }

    #[test]
    fn test_local_date_time_lifetime_safety() {
        // Test that LocalDateTime correctly holds lifetime to timestamp
        let timestamp = Timestamp::from_second(1640995200).unwrap();
        let local_dt = LocalDateTime(&timestamp);

        // Should be able to format multiple times
        let output1 = format!("{}", local_dt);
        let output2 = format!("{}", local_dt);

        assert_eq!(output1, output2);
        assert!(!output1.is_empty());
    }
}
