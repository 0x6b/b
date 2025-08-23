use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

/// Helper function to create a temporary directory for CLI tests
fn create_cli_test_environment() -> TempDir {
    TempDir::new().expect("Failed to create temporary directory")
}

/// Helper function to create a Command with --no-color flag for testing
fn beacon_cmd() -> Command {
    let mut cmd = Command::cargo_bin("beacon").expect("Failed to find beacon binary");
    cmd.arg("--no-color");
    cmd
}

#[test]
fn test_cli_create_plan_success() {
    let temp_dir = create_cli_test_environment();
    let db_path = temp_dir.path().join("cli_test.db");

    beacon_cmd()
        .args([
            "--database-file",
            db_path.to_str().unwrap(),
            "plan",
            "create",
            "Test Title",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Test Title"))
        .stdout(predicate::str::contains("# 1."));
}

#[test]
fn test_cli_create_plan_with_description() {
    let temp_dir = create_cli_test_environment();
    let db_path = temp_dir.path().join("cli_test.db");

    beacon_cmd()
        .args([
            "--database-file",
            db_path.to_str().unwrap(),
            "plan",
            "create",
            "Test Title With Description",
            "--description",
            "A detailed description",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Test Title With Description"))
        .stdout(predicate::str::contains("A detailed description"));
}

#[test]
fn test_cli_list_empty_plans() {
    let temp_dir = create_cli_test_environment();
    let db_path = temp_dir.path().join("cli_test.db");

    beacon_cmd()
        .args(["--database-file", db_path.to_str().unwrap(), "plan", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No plans found."));
}

#[test]
fn test_cli_list_plans_text_format() {
    let temp_dir = create_cli_test_environment();
    let db_path = temp_dir.path().join("cli_test.db");
    let db_arg = db_path.to_str().unwrap();

    // Create a plan first
    beacon_cmd()
        .args(["--database-file", db_arg, "plan", "create", "List Title"])
        .assert()
        .success();

    // List plans
    beacon_cmd()
        .args(["--database-file", db_arg, "plan", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("# Active Plans"))
        .stdout(predicate::str::contains("List Title"));
}

#[test]
fn test_cli_show_plan() {
    let temp_dir = create_cli_test_environment();
    let db_path = temp_dir.path().join("cli_test.db");
    let db_arg = db_path.to_str().unwrap();

    // Create a plan and extract ID
    let output = beacon_cmd()
        .args([
            "--database-file",
            db_arg,
            "plan",
            "create",
            "Show Title",
            "--description",
            "Test Description",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).expect("Invalid UTF-8");
    let plan_id = extract_id_from_output(&output_str);

    // Show the plan
    beacon_cmd()
        .args(["--database-file", db_arg, "plan", "show", &plan_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Show Title"))
        .stdout(predicate::str::contains("Test Description"));
}

#[test]
fn test_cli_add_step() {
    let temp_dir = create_cli_test_environment();
    let db_path = temp_dir.path().join("cli_test.db");
    let db_arg = db_path.to_str().unwrap();

    // Create a plan
    let output = beacon_cmd()
        .args(["--database-file", db_arg, "plan", "create", "Step Title"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).expect("Invalid UTF-8");
    let plan_id = extract_id_from_output(&output_str);

    // Add a step
    beacon_cmd()
        .args([
            "--database-file",
            db_arg,
            "step",
            "add",
            &plan_id,
            "Test Step",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created step with ID:"));
}

#[test]
fn test_cli_update_step_status_to_done() {
    let temp_dir = create_cli_test_environment();
    let db_path = temp_dir.path().join("cli_test.db");
    let db_arg = db_path.to_str().unwrap();

    // Create a plan with a step
    let output = beacon_cmd()
        .args(["--database-file", db_arg, "plan", "create", "Done Title"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).expect("Invalid UTF-8");
    let plan_id = extract_id_from_output(&output_str);

    let output = beacon_cmd()
        .args([
            "--database-file",
            db_arg,
            "step",
            "add",
            &plan_id,
            "Mark as Done",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).expect("Invalid UTF-8");
    let step_id = extract_id_from_output(&output_str);

    // Update step status to done
    beacon_cmd()
        .args([
            "--database-file",
            db_arg,
            "step",
            "update",
            &step_id,
            "--status",
            "done",
            "--result",
            "Successfully completed the test step with all requirements met",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated step"));
}

#[test]
fn test_cli_update_step_status_to_in_progress() {
    let temp_dir = create_cli_test_environment();
    let db_path = temp_dir.path().join("cli_test.db");
    let db_arg = db_path.to_str().unwrap();

    // Create a plan with a step
    let output = beacon_cmd()
        .args([
            "--database-file",
            db_arg,
            "plan",
            "create",
            "Progress Title",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).expect("Invalid UTF-8");
    let plan_id = extract_id_from_output(&output_str);

    let output = beacon_cmd()
        .args([
            "--database-file",
            db_arg,
            "step",
            "add",
            &plan_id,
            "Work in Progress",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).expect("Invalid UTF-8");
    let step_id = extract_id_from_output(&output_str);

    // Update step status to in-progress
    beacon_cmd()
        .args([
            "--database-file",
            db_arg,
            "step",
            "update",
            &step_id,
            "--status",
            "in-progress",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated step"));
}

#[test]
fn test_cli_update_step_details() {
    let temp_dir = create_cli_test_environment();
    let db_path = temp_dir.path().join("cli_test.db");
    let db_arg = db_path.to_str().unwrap();

    // Create a plan with a step
    let output = beacon_cmd()
        .args(["--database-file", db_arg, "plan", "create", "Update Title"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).expect("Invalid UTF-8");
    let plan_id = extract_id_from_output(&output_str);

    let output = beacon_cmd()
        .args([
            "--database-file",
            db_arg,
            "step",
            "add",
            &plan_id,
            "Original Step",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).expect("Invalid UTF-8");
    let step_id = extract_id_from_output(&output_str);

    // Update step description and references
    beacon_cmd()
        .args([
            "--database-file",
            db_arg,
            "step",
            "update",
            &step_id,
            "--description",
            "Updated Step",
            "--references",
            "ref1.txt,ref2.md",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated step"))
        .stdout(predicate::str::contains("description"));
}

#[test]
fn test_cli_archive_plan() {
    let temp_dir = create_cli_test_environment();
    let db_path = temp_dir.path().join("cli_test.db");
    let db_arg = db_path.to_str().unwrap();

    // Create a plan
    let output = beacon_cmd()
        .args(["--database-file", db_arg, "plan", "create", "Archive Title"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).expect("Invalid UTF-8");
    let plan_id = extract_id_from_output(&output_str);

    // Archive the plan
    beacon_cmd()
        .args(["--database-file", db_arg, "plan", "archive", &plan_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Archived plan"));

    // Verify it's not in active list
    beacon_cmd()
        .args(["--database-file", db_arg, "plan", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No plans found."));

    // Verify it's in archived list
    beacon_cmd()
        .args(["--database-file", db_arg, "plan", "list", "--archived"])
        .assert()
        .success()
        .stdout(predicate::str::contains("# Archived Plans"))
        .stdout(predicate::str::contains("Archive Title"));
}

#[test]
fn test_cli_unarchive_plan() {
    let temp_dir = create_cli_test_environment();
    let db_path = temp_dir.path().join("cli_test.db");
    let db_arg = db_path.to_str().unwrap();

    // Create and archive a plan
    let output = beacon_cmd()
        .args([
            "--database-file",
            db_arg,
            "plan",
            "create",
            "Unarchive Title",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).expect("Invalid UTF-8");
    let plan_id = extract_id_from_output(&output_str);

    beacon_cmd()
        .args(["--database-file", db_arg, "plan", "archive", &plan_id])
        .assert()
        .success();

    // Unarchive the plan
    beacon_cmd()
        .args(["--database-file", db_arg, "plan", "unarchive", &plan_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Unarchived plan"));

    // Verify it's back in active list
    beacon_cmd()
        .args(["--database-file", db_arg, "plan", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Unarchive Title"));
}

#[test]
fn test_cli_help_output() {
    beacon_cmd()
        .args(["--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("A simple task planning tool"))
        .stdout(predicate::str::contains("Commands:"))
        .stdout(predicate::str::contains("plan"))
        .stdout(predicate::str::contains("step"))
        .stdout(predicate::str::contains("serve"));
}

#[test]
fn test_cli_plan_help() {
    beacon_cmd()
        .args(["plan", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Manage plans"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("show"))
        .stdout(predicate::str::contains("archive"))
        .stdout(predicate::str::contains("unarchive"))
        .stdout(predicate::str::contains("search"));
}

#[test]
fn test_cli_step_help() {
    beacon_cmd()
        .args(["step", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Manage steps"))
        .stdout(predicate::str::contains("add"))
        .stdout(predicate::str::contains("update"));
}

#[test]
fn test_cli_version_output() {
    beacon_cmd()
        .args(["--version"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("beacon "));
}

#[test]
fn test_cli_search_plans_empty() {
    let temp_dir = create_cli_test_environment();
    let db_path = temp_dir.path().join("cli_test.db");

    beacon_cmd()
        .args([
            "--database-file",
            db_path.to_str().unwrap(),
            "plan",
            "search",
            "/some/directory",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("No plans found."));
}

#[test]
fn test_cli_search_plans_with_results() {
    let temp_dir = create_cli_test_environment();
    let db_path = temp_dir.path().join("cli_test.db");
    let db_arg = db_path.to_str().unwrap();

    // Create a plan first
    beacon_cmd()
        .args([
            "--database-file",
            db_arg,
            "plan",
            "create",
            "Search Title",
            "--directory",
            "/workspace/crates",
        ])
        .assert()
        .success();

    // Search for plans in /workspace directory (should find the plan)
    beacon_cmd()
        .args(["--database-file", db_arg, "plan", "search", "/workspace"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "# ACTIVE plans in directory: /workspace",
        ))
        .stdout(predicate::str::contains("Search Title"));
}

#[test]
fn test_cli_show_step() {
    let temp_dir = create_cli_test_environment();
    let db_path = temp_dir.path().join("cli_test.db");
    let db_arg = db_path.to_str().unwrap();

    // Create a plan with a step
    let output = beacon_cmd()
        .args([
            "--database-file",
            db_arg,
            "plan",
            "create",
            "Step Show Title",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).expect("Invalid UTF-8");
    let plan_id = extract_id_from_output(&output_str);

    let output = beacon_cmd()
        .args([
            "--database-file",
            db_arg,
            "step",
            "add",
            &plan_id,
            "Show Step",
            "--description",
            "Step to be shown",
            "--acceptance-criteria",
            "Must display correctly",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).expect("Invalid UTF-8");
    let step_id = extract_id_from_output(&output_str);

    // Show the step
    beacon_cmd()
        .args(["--database-file", db_arg, "step", "show", &step_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Show Step"))
        .stdout(predicate::str::contains("Step to be shown"))
        .stdout(predicate::str::contains("Must display correctly"));
}

#[test]
fn test_cli_invalid_plan_id() {
    let temp_dir = create_cli_test_environment();
    let db_path = temp_dir.path().join("cli_test.db");

    beacon_cmd()
        .args([
            "--database-file",
            db_path.to_str().unwrap(),
            "plan",
            "show",
            "99999",
        ])
        .assert()
        .failure();
}

#[test]
fn test_cli_invalid_step_id() {
    let temp_dir = create_cli_test_environment();
    let db_path = temp_dir.path().join("cli_test.db");

    beacon_cmd()
        .args([
            "--database-file",
            db_path.to_str().unwrap(),
            "step",
            "show",
            "99999",
        ])
        .assert()
        .failure();
}

#[test]
fn test_cli_add_step_with_all_fields() {
    let temp_dir = create_cli_test_environment();
    let db_path = temp_dir.path().join("cli_test.db");
    let db_arg = db_path.to_str().unwrap();

    // Create a plan
    let output = beacon_cmd()
        .args([
            "--database-file",
            db_arg,
            "plan",
            "create",
            "Full Step Title",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).expect("Invalid UTF-8");
    let plan_id = extract_id_from_output(&output_str);

    // Add a step with all fields
    beacon_cmd()
        .args([
            "--database-file",
            db_arg,
            "step",
            "add",
            &plan_id,
            "Complete Step",
            "--description",
            "A complete step with all fields",
            "--acceptance-criteria",
            "Should have all fields populated",
            "--references",
            "doc1.md,https://example.com",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created step with ID:"));
}

#[test]
fn test_cli_update_step_title() {
    let temp_dir = create_cli_test_environment();
    let db_path = temp_dir.path().join("cli_test.db");
    let db_arg = db_path.to_str().unwrap();

    // Create a plan with a step
    let output = beacon_cmd()
        .args(["--database-file", db_arg, "plan", "create", "Title Update"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).expect("Invalid UTF-8");
    let plan_id = extract_id_from_output(&output_str);

    let output = beacon_cmd()
        .args([
            "--database-file",
            db_arg,
            "step",
            "add",
            &plan_id,
            "Original Title",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).expect("Invalid UTF-8");
    let step_id = extract_id_from_output(&output_str);

    // Update step title
    beacon_cmd()
        .args([
            "--database-file",
            db_arg,
            "step",
            "update",
            &step_id,
            "--title",
            "Updated Title",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated step"));
}

/// Helper function to extract ID from command output
fn extract_id_from_output(output: &str) -> String {
    // Try new format: look for "# <number>. " pattern
    // Skip "# Plan Created" and similar headers
    for line in output.lines() {
        if let Some(stripped) = line.strip_prefix("# ") {
            let after_hash = &stripped.trim();
            // Check if this line starts with a number followed by a dot
            if let Some(dot_pos) = after_hash.find('.') {
                let potential_id = &after_hash[..dot_pos];
                if !potential_id.is_empty() && potential_id.chars().all(|c| c.is_numeric()) {
                    return potential_id.to_string();
                }
            }
        }
    }

    // Fall back to old format: "ID: <number>"
    if let Some(start) = output.find("ID: ") {
        let id_str = &output[start + 4..];
        if let Some(end) = id_str.find(|c: char| !c.is_numeric()) {
            return id_str[..end].to_string();
        }
    }

    panic!("Could not extract ID from output: {output}");
}
