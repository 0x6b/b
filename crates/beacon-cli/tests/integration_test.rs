//! Integration tests comparing CLI and direct Display implementations
//!
//! This test suite verifies that CLI output uses the same Display traits
//! that would be used in MCP after the Display trait refactoring.

use std::process::Command;

use beacon_core::{PlanFilter, PlanStatus, Planner, PlannerBuilder};
use tempfile::TempDir;

/// Helper function to create a test planner with temporary database
async fn create_test_planner() -> (Planner, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let db_path = temp_dir.path().join("test.db");

    let planner = PlannerBuilder::new()
        .with_database_path(db_path)
        .build()
        .await
        .expect("Failed to create planner");

    (planner, temp_dir)
}

/// Run a CLI command and capture its output
fn run_cli_command(db_path: &str, args: &[&str]) -> String {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_beacon"));
    cmd.arg("--no-color").arg("--database-file").arg(db_path);

    for arg in args {
        cmd.arg(arg);
    }

    let output = cmd.output().expect("Failed to run CLI command");
    String::from_utf8(output.stdout).expect("Invalid UTF-8 in CLI output")
}

/// Test that plan creation has consistent output between CLI and direct Display
/// impl
#[tokio::test]
async fn test_plan_display_consistency() {
    let (planner, temp_dir) = create_test_planner().await;
    let db_path = temp_dir.path().join("test.db");
    let db_str = db_path.to_str().unwrap();

    // Create plan via CLI
    let cli_output = run_cli_command(
        db_str,
        &[
            "plan",
            "create",
            "Integration Test Plan",
            "--description",
            "Test plan for integration testing",
        ],
    );

    // Create plan via direct planner call
    let params = beacon_core::params::CreatePlan {
        title: "Integration Test Plan Direct".to_string(),
        description: Some("Test plan for integration testing".to_string()),
        directory: None,
    };

    let plan = planner
        .create_plan(&params)
        .await
        .expect("Failed to create plan");
    let result = beacon_core::display::CreateResult::new(plan);
    let direct_output = result.to_string();

    // Both outputs should contain the same structure (ignoring specific IDs and
    // timestamps)
    assert!(cli_output.contains("Created plan with ID:"));
    assert!(direct_output.contains("Created plan with ID:"));
    assert!(cli_output.contains("Integration Test Plan"));
    assert!(direct_output.contains("Integration Test Plan Direct"));
    assert!(cli_output.contains("Test plan for integration testing"));
    assert!(direct_output.contains("Test plan for integration testing"));
}

/// Test that step creation has consistent output format
#[tokio::test]
async fn test_step_display_consistency() {
    let (planner, temp_dir) = create_test_planner().await;
    let db_path = temp_dir.path().join("test.db");
    let db_str = db_path.to_str().unwrap();

    // Create a plan first via CLI
    let _plan_output = run_cli_command(db_str, &["plan", "create", "Step Test Plan"]);

    // Add step via CLI
    let cli_output = run_cli_command(
        db_str,
        &[
            "step",
            "add",
            "1",
            "Test Step",
            "--description",
            "Step added via CLI",
        ],
    );

    // Create plan and step via direct planner call
    let plan_params = beacon_core::params::CreatePlan {
        title: "Direct Step Test Plan".to_string(),
        description: None,
        directory: None,
    };

    let plan = planner
        .create_plan(&plan_params)
        .await
        .expect("Failed to create plan");

    let step_params = beacon_core::params::StepCreate {
        plan_id: plan.id,
        title: "Test Step".to_string(),
        description: Some("Step added via direct call".to_string()),
        acceptance_criteria: None,
        references: vec![],
    };

    let step = planner
        .add_step(&step_params)
        .await
        .expect("Failed to add step");
    let result = beacon_core::display::CreateResult::new(step);
    let direct_output = result.to_string();

    // Both outputs should have the same structure
    assert!(cli_output.contains("Created step with ID:"));
    assert!(direct_output.contains("Created step with ID:"));
    assert!(cli_output.contains("# Step"));
    assert!(direct_output.contains("# Step"));
    assert!(cli_output.contains("Test Step"));
    assert!(direct_output.contains("Test Step"));
}

/// Test list plan output consistency
#[tokio::test]
async fn test_list_plans_consistency() {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let db_path = temp_dir.path().join("test.db");
    let db_str = db_path.to_str().unwrap();

    // Create some plans via CLI
    let _output1 = run_cli_command(db_str, &["plan", "create", "List Test Plan 1"]);
    let _output2 = run_cli_command(
        db_str,
        &[
            "plan",
            "create",
            "List Test Plan 2",
            "--description",
            "Second plan",
        ],
    );

    // List plans via CLI
    let cli_output = run_cli_command(db_str, &["plan", "list"]);

    // Create planner and list plans directly
    let (planner, _temp_dir2) = create_test_planner().await;

    // Create plans directly
    let plan_params1 = beacon_core::params::CreatePlan {
        title: "Direct List Test Plan 1".to_string(),
        description: None,
        directory: None,
    };
    let plan_params2 = beacon_core::params::CreatePlan {
        title: "Direct List Test Plan 2".to_string(),
        description: Some("Second plan".to_string()),
        directory: None,
    };

    let plan1 = planner
        .create_plan(&plan_params1)
        .await
        .expect("Failed to create plan");
    let plan2 = planner
        .create_plan(&plan_params2)
        .await
        .expect("Failed to create plan");

    // Get plan summaries
    let summary1 = beacon_core::PlanSummary::from_plan(plan1, 0, 0);
    let summary2 = beacon_core::PlanSummary::from_plan(plan2, 0, 0);

    let summaries = vec![summary1, summary2];
    let direct_output = beacon_core::display::format_plan_list(&summaries, Some("Active Plans"));

    // Both should have similar structure
    assert!(cli_output.contains("# Active Plans"));
    assert!(direct_output.contains("# Active Plans"));
    assert!(cli_output.contains("List Test Plan"));
    assert!(direct_output.contains("Direct List Test Plan"));
    assert!(cli_output.contains("ID:"));
    assert!(direct_output.contains("ID:"));
}

/// Test empty list output consistency
#[tokio::test]
async fn test_empty_list_consistency() {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let db_path = temp_dir.path().join("test.db");
    let db_str = db_path.to_str().unwrap();

    // List empty plans via CLI
    let cli_output = run_cli_command(db_str, &["plan", "list"]);

    // Create empty list directly
    let summaries: Vec<beacon_core::PlanSummary> = vec![];
    let direct_output = beacon_core::display::format_plan_list(&summaries, Some("Active Plans"));

    // Both should have similar empty structure
    assert!(cli_output.contains("# Active Plans"));
    assert!(direct_output.contains("# Active Plans"));
    assert!(cli_output.contains("No plans found."));
    assert!(direct_output.contains("No plans found."));
}

/// Test show plan output consistency
#[tokio::test]
async fn test_show_plan_consistency() {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let db_path = temp_dir.path().join("test.db");
    let db_str = db_path.to_str().unwrap();

    // Create plan with step via CLI
    let _plan_output = run_cli_command(
        db_str,
        &[
            "plan",
            "create",
            "Show Test Plan",
            "--description",
            "Plan for show testing",
        ],
    );

    let _step_output = run_cli_command(
        db_str,
        &[
            "step",
            "add",
            "1",
            "Test Step",
            "--description",
            "Step for testing",
        ],
    );

    // Show plan via CLI
    let cli_output = run_cli_command(db_str, &["plan", "show", "1"]);

    // Create plan and step directly
    let (planner, _temp_dir2) = create_test_planner().await;

    let plan_params = beacon_core::params::CreatePlan {
        title: "Show Test Plan".to_string(),
        description: Some("Plan for show testing".to_string()),
        directory: None,
    };

    let plan = planner
        .create_plan(&plan_params)
        .await
        .expect("Failed to create plan");

    let step_params = beacon_core::params::StepCreate {
        plan_id: plan.id,
        title: "Test Step".to_string(),
        description: Some("Step for testing".to_string()),
        acceptance_criteria: None,
        references: vec![],
    };

    let _step = planner
        .add_step(&step_params)
        .await
        .expect("Failed to add step");

    // Get plan with steps
    let params = beacon_core::params::Id { id: plan.id };
    let mut full_plan = planner
        .get_plan(&params)
        .await
        .expect("Failed to get plan")
        .expect("Plan not found");
    full_plan.steps = planner
        .get_steps(&params)
        .await
        .expect("Failed to get steps");

    let direct_output = full_plan.to_string();

    // Both should have similar structure
    assert!(cli_output.contains("# 1. Show Test Plan"));
    assert!(direct_output.contains("Show Test Plan"));
    assert!(cli_output.contains("Plan for show testing"));
    assert!(direct_output.contains("Plan for show testing"));
    assert!(cli_output.contains("## Steps"));
    assert!(direct_output.contains("## Steps"));
    assert!(cli_output.contains("Test Step"));
    assert!(direct_output.contains("Test Step"));
}

/// Test show step output consistency
#[tokio::test]
async fn test_show_step_consistency() {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let db_path = temp_dir.path().join("test.db");
    let db_str = db_path.to_str().unwrap();

    // Create plan and step via CLI
    let _plan_output = run_cli_command(db_str, &["plan", "create", "Step Show Test Plan"]);

    let _step_output = run_cli_command(
        db_str,
        &[
            "step",
            "add",
            "1",
            "Show Step Test",
            "--description",
            "Detailed step description",
            "--acceptance-criteria",
            "Should show all fields correctly",
        ],
    );

    // Show step via CLI
    let cli_output = run_cli_command(db_str, &["step", "show", "1"]);

    // Create step directly
    let (planner, _temp_dir2) = create_test_planner().await;

    let plan_params = beacon_core::params::CreatePlan {
        title: "Direct Step Show Test Plan".to_string(),
        description: None,
        directory: None,
    };

    let plan = planner
        .create_plan(&plan_params)
        .await
        .expect("Failed to create plan");

    let step_params = beacon_core::params::StepCreate {
        plan_id: plan.id,
        title: "Show Step Test".to_string(),
        description: Some("Detailed step description".to_string()),
        acceptance_criteria: Some("Should show all fields correctly".to_string()),
        references: vec![],
    };

    let step = planner
        .add_step(&step_params)
        .await
        .expect("Failed to add step");
    let direct_output = step.to_string();

    // Both should have similar structure
    assert!(cli_output.contains("# Step"));
    assert!(direct_output.contains("# Step"));
    assert!(cli_output.contains("Show Step Test"));
    assert!(direct_output.contains("Show Step Test"));
    assert!(cli_output.contains("Detailed step description"));
    assert!(direct_output.contains("Detailed step description"));
    assert!(cli_output.contains("Should show all fields correctly"));
    assert!(direct_output.contains("Should show all fields correctly"));
}

/// Test CLI vs MCP-style list output (simulating what the MCP server would
/// return)
#[tokio::test]
async fn test_cli_vs_mcp_list_output() {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let db_path = temp_dir.path().join("test.db");
    let db_str = db_path.to_str().unwrap();

    // Test empty list first
    let cli_empty = run_cli_command(db_str, &["plan", "list"]);

    // Simulate MCP-style empty list output
    let empty_plans: Vec<beacon_core::PlanSummary> = vec![];
    let mcp_empty_str = beacon_core::display::format_plan_list(&empty_plans, Some("Active Plans"));

    // Both should produce the same output for empty lists
    assert_eq!(cli_empty.trim(), mcp_empty_str.trim());

    // Create some plans via CLI
    let _output1 = run_cli_command(
        db_str,
        &[
            "plan",
            "create",
            "MCP Test Plan 1",
            "--description",
            "First test plan",
        ],
    );
    let _output2 = run_cli_command(db_str, &["plan", "create", "MCP Test Plan 2"]);

    // Get CLI list output
    let cli_list = run_cli_command(db_str, &["plan", "list"]);

    // Simulate MCP server behavior - get plans and format them
    let planner = PlannerBuilder::new()
        .with_database_path(&db_path)
        .build()
        .await
        .expect("Failed to create planner");

    let filter = Some(PlanFilter {
        status: Some(PlanStatus::Active),
        ..Default::default()
    });

    let plans = planner
        .list_plans(filter)
        .await
        .expect("Failed to list plans");

    // Convert to summaries as the MCP server would
    let mut plan_summaries = Vec::new();
    for plan in plans {
        let steps = planner
            .get_steps(&beacon_core::params::Id { id: plan.id })
            .await
            .expect("Failed to get steps");

        let completed_steps = steps
            .iter()
            .filter(|s| s.status == beacon_core::StepStatus::Done)
            .count() as u32;
        let total_steps = steps.len() as u32;

        let summary = beacon_core::PlanSummary::from_plan(plan, total_steps, completed_steps);
        plan_summaries.push(summary);
    }

    let mcp_list_str =
        beacon_core::display::format_plan_list(&plan_summaries, Some("Active Plans"));

    // Both outputs should have the same structure
    assert!(cli_list.contains("# Active Plans"));
    assert!(mcp_list_str.contains("# Active Plans"));
    assert!(cli_list.contains("MCP Test Plan 1"));
    assert!(mcp_list_str.contains("MCP Test Plan 1"));
    assert!(cli_list.contains("MCP Test Plan 2"));
    assert!(mcp_list_str.contains("MCP Test Plan 2"));
    assert!(cli_list.contains("First test plan"));
    assert!(mcp_list_str.contains("First test plan"));

    // The core formatting should be identical (ignoring potential whitespace
    // differences)
    let cli_lines: Vec<&str> = cli_list.lines().map(|l| l.trim()).collect();
    let mcp_lines: Vec<&str> = mcp_list_str.lines().map(|l| l.trim()).collect();

    assert_eq!(
        cli_lines.len(),
        mcp_lines.len(),
        "Different number of output lines"
    );

    // Check that key structural elements match
    assert!(cli_lines.iter().any(|line| line.contains("# Active Plans")));
    assert!(mcp_lines.iter().any(|line| line.contains("# Active Plans")));
}

/// Test CLI vs MCP-style show plan output
#[tokio::test]
async fn test_cli_vs_mcp_show_plan_output() {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let db_path = temp_dir.path().join("test.db");
    let db_str = db_path.to_str().unwrap();

    // Create plan with steps via CLI
    let _plan_output = run_cli_command(
        db_str,
        &[
            "plan",
            "create",
            "MCP Show Plan",
            "--description",
            "Plan for MCP comparison testing",
        ],
    );

    let _step1_output = run_cli_command(
        db_str,
        &[
            "step",
            "add",
            "1",
            "First Step",
            "--description",
            "First step description",
        ],
    );

    let _step2_output = run_cli_command(
        db_str,
        &[
            "step",
            "add",
            "1",
            "Second Step",
            "--acceptance-criteria",
            "Should complete successfully",
        ],
    );

    // Get CLI show output
    let cli_show = run_cli_command(db_str, &["plan", "show", "1"]);

    // Simulate MCP server show_plan behavior
    let planner = PlannerBuilder::new()
        .with_database_path(&db_path)
        .build()
        .await
        .expect("Failed to create planner");

    let params = beacon_core::params::Id { id: 1 };
    let mut plan = planner
        .get_plan(&params)
        .await
        .expect("Failed to get plan")
        .expect("Plan not found");

    plan.steps = planner
        .get_steps(&params)
        .await
        .expect("Failed to get steps");

    let mcp_show = plan.to_string();

    // Both outputs should be identical since they use the same Display impl
    assert_eq!(cli_show.trim(), mcp_show.trim());
}

/// Test CLI vs MCP-style show step output
#[tokio::test]
async fn test_cli_vs_mcp_show_step_output() {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let db_path = temp_dir.path().join("test.db");
    let db_str = db_path.to_str().unwrap();

    // Create plan and step via CLI
    let _plan_output = run_cli_command(db_str, &["plan", "create", "Step Comparison Plan"]);

    let _step_output = run_cli_command(
        db_str,
        &[
            "step",
            "add",
            "1",
            "Comparison Step",
            "--description",
            "Step for CLI vs MCP comparison",
            "--acceptance-criteria",
            "Both outputs should match exactly",
            "--references",
            "doc1.md,https://example.com",
        ],
    );

    // Get CLI show step output
    let cli_step = run_cli_command(db_str, &["step", "show", "1"]);

    // Simulate MCP server show_step behavior
    let planner = PlannerBuilder::new()
        .with_database_path(&db_path)
        .build()
        .await
        .expect("Failed to create planner");

    let params = beacon_core::params::Id { id: 1 };
    let step = planner
        .get_step(&params)
        .await
        .expect("Failed to get step")
        .expect("Step not found");

    let mcp_step = step.to_string();

    // Both outputs should be identical since they use the same Display impl
    assert_eq!(cli_step.trim(), mcp_step.trim());
}
