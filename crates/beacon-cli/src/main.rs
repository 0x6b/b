//! Beacon CLI Application
//!
//! Command-line interface for the beacon task planning tool.
//! Following the m43 pattern for clean CLI implementation.

mod cli;
mod mcp;
mod renderer;

use std::str::FromStr;

use anyhow::{Context, Result};
use beacon_core::{
    format_plan_list, CreatePlan, CreateResult, Id,
    InsertStep, ListPlans, OperationStatus, Planner, PlannerBuilder, SearchPlans, StepCreate,
    StepStatus, SwapSteps, UpdateResult, UpdateStep,
};
use clap::Parser;
use cli::{Cli, Commands, PlanCommands, StepCommands};
use log::{debug, info};
use mcp::{run_stdio_server, BeaconMcpServer};
use renderer::TerminalRenderer;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();

    // Parse command line arguments
    let cli = Cli::parse();

    // Create terminal renderer based on CLI flags
    let renderer = TerminalRenderer::new(!cli.no_color);

    // Create planner with optional database path from CLI
    let mut planner_builder = PlannerBuilder::new();
    if let Some(path) = cli.database_file {
        debug!("Using database path from CLI: {}", path.display());
        planner_builder = planner_builder.with_database_path(path);
    } else {
        debug!("Using default XDG database path");
    }

    let planner = planner_builder
        .build()
        .await
        .context("Failed to initialize planner")?;

    info!("Beacon started");

    // Dispatch to command handlers
    use Commands::*;
    match cli.command {
        Plan { command } => handle_plan_command(planner, command, &renderer).await,
        Step { command } => handle_step_command(planner, command, &renderer).await,
        Serve => handle_serve(planner).await,
    }
}

/// Handle plan subcommands
async fn handle_plan_command(
    planner: Planner,
    command: PlanCommands,
    renderer: &TerminalRenderer,
) -> Result<()> {
    use PlanCommands::*;
    match command {
        Create(args) => handle_plan_create(planner, &args.into(), renderer).await,
        List(args) => handle_plan_list(planner, &args.into(), renderer).await,
        Show(args) => handle_plan_show(planner, &args.into(), renderer).await,
        Archive(args) => handle_plan_archive(planner, &args.into(), renderer).await,
        Unarchive(args) => handle_plan_unarchive(planner, &args.into(), renderer).await,
        Delete(args) => handle_plan_delete(planner, args, renderer).await,
        Search(args) => handle_plan_search(planner, &args.into(), renderer).await,
    }
}

/// Handle step subcommands
async fn handle_step_command(
    planner: Planner,
    command: StepCommands,
    renderer: &TerminalRenderer,
) -> Result<()> {
    use StepCommands::*;
    match command {
        Add(args) => handle_step_add(planner, &args.into(), renderer).await,
        Insert(args) => handle_step_insert(planner, &args.into(), renderer).await,
        Update(args) => {
            let params: UpdateStep = args.into();

            // Parse status using FromStr implementation
            let status = params.status.as_ref().map(|s| {
                StepStatus::from_str(s).unwrap_or_else(|_| {
                    eprintln!("Warning: Invalid status '{}', defaulting to 'todo'", s);
                    StepStatus::Todo
                })
            });

            handle_step_update(planner, &params, status, renderer).await
        }
        Show(args) => handle_step_show(planner, &args.into(), renderer).await,
        Swap(args) => handle_step_swap(planner, &args.into(), renderer).await,
    }
}

/// Handle plan create command
async fn handle_plan_create(
    planner: Planner,
    params: &CreatePlan,
    renderer: &TerminalRenderer,
) -> Result<()> {
    let plan = planner.create_plan_result(params)
        .await
        .context("Failed to create plan")?;

    let result = CreateResult::new(plan);
    renderer.render(&result.to_string())?;

    Ok(())
}

/// Handle plan list command
async fn handle_plan_list(
    planner: Planner,
    params: &ListPlans,
    renderer: &TerminalRenderer,
) -> Result<()> {
    let plan_summaries = planner.list_plans_summary(params)
        .await
        .context("Failed to list plans")?;

    let title = if params.archived {
        "Archived Plans"
    } else {
        "Active Plans"
    };

    let formatted_output = format_plan_list(&plan_summaries, Some(title));
    renderer.render(&formatted_output)?;

    Ok(())
}

/// Handle plan show command
async fn handle_plan_show(
    planner: Planner,
    params: &Id,
    renderer: &TerminalRenderer,
) -> Result<()> {
    let plan = planner.show_plan_with_steps(params)
        .await
        .context("Failed to get plan")?
        .ok_or_else(|| anyhow::anyhow!("Plan with ID {} not found", params.id))?;

    renderer.render(&plan.to_string())?;

    Ok(())
}

/// Handle plan archive command
async fn handle_plan_archive(
    planner: Planner,
    params: &Id,
    renderer: &TerminalRenderer,
) -> Result<()> {
    let plan = planner.archive_plan_with_confirmation(params)
        .await
        .with_context(|| format!("Failed to archive plan {}", params.id))?
        .ok_or_else(|| anyhow::anyhow!("Plan with ID {} not found", params.id))?;

    let steps = planner
        .get_steps(params)
        .await
        .context("Failed to get steps")?;

    let step_info = if !steps.is_empty() {
        format!(" with {} step(s)", steps.len())
    } else {
        String::new()
    };

    let message = format!(
        "Archived plan '{}' (ID: {}){step_info}. Use 'beacon plan unarchive {}' to restore.",
        plan.title, params.id, params.id
    );

    let status = OperationStatus::success(message);
    renderer.render(&status.to_string())?;
    Ok(())
}

/// Handle plan unarchive command
async fn handle_plan_unarchive(
    planner: Planner,
    params: &Id,
    renderer: &TerminalRenderer,
) -> Result<()> {
    let _plan = planner.unarchive_plan_with_confirmation(params)
        .await
        .with_context(|| format!("Failed to unarchive plan {}", params.id))?;

    let message = format!("Unarchived plan with ID: {}", params.id);
    let status = OperationStatus::success(message);
    renderer.render(&status.to_string())?;
    Ok(())
}

/// Handle plan delete command
async fn handle_plan_delete(
    planner: Planner,
    args: cli::DeletePlanArgs,
    renderer: &TerminalRenderer,
) -> Result<()> {
    // Check if --confirm flag was provided
    if !args.confirm {
        let message = format!(
            "Plan deletion requires confirmation. Use 'beacon plan delete {} --confirm' to permanently delete the plan.",
            args.id
        );
        let status = OperationStatus::failure(message);
        renderer.render(&status.to_string())?;
        return Ok(());
    }

    let params: Id = args.into();

    // Get step count before deletion for informative message
    let steps = planner
        .get_steps(&params)
        .await
        .with_context(|| format!("Failed to get steps for plan {}", params.id))?;

    let plan = planner.delete_plan_with_confirmation(&params)
        .await
        .with_context(|| format!("Failed to delete plan {}", params.id))?
        .ok_or_else(|| anyhow::anyhow!("Plan with ID {} not found", params.id))?;

    let step_info = if steps.is_empty() {
        String::new()
    } else {
        format!(
            " and {} step{}",
            steps.len(),
            if steps.len() == 1 { "" } else { "s" }
        )
    };

    let message = format!(
        "Permanently deleted plan '{}' (ID: {}){step_info}. This action cannot be undone.",
        plan.title, plan.id
    );
    let status = OperationStatus::success(message);
    renderer.render(&status.to_string())?;
    Ok(())
}

/// Handle plan search command
async fn handle_plan_search(
    planner: Planner,
    params: &SearchPlans,
    renderer: &TerminalRenderer,
) -> Result<()> {
    let plan_summaries = planner.search_plans_summary(params)
        .await
        .context("Failed to search plans")?;

    let status_text = if params.archived {
        "ARCHIVED"
    } else {
        "ACTIVE"
    };
    let title = format!("{status_text} plans in directory: {}", params.directory);

    let formatted_output = format_plan_list(&plan_summaries, Some(&title));
    renderer.render(&formatted_output)?;

    Ok(())
}

/// Handle step add command
async fn handle_step_add(
    planner: Planner,
    params: &StepCreate,
    renderer: &TerminalRenderer,
) -> Result<()> {
    let step = planner.add_step_to_plan(params)
        .await
        .with_context(|| format!("Failed to add step to plan {}", params.plan_id))?;

    let result = CreateResult::new(step);
    renderer.render(&result.to_string())?;

    Ok(())
}

/// Handle step insert command
async fn handle_step_insert(
    planner: Planner,
    params: &InsertStep,
    renderer: &TerminalRenderer,
) -> Result<()> {
    let step = planner.insert_step_to_plan(params)
        .await
        .with_context(|| {
            format!(
                "Failed to insert step into plan {} at position {}",
                params.step.plan_id, params.position
            )
        })?;

    let result = CreateResult::new(step);
    renderer.render(&result.to_string())?;

    Ok(())
}

/// Handle step update command
async fn handle_step_update(
    planner: Planner,
    params: &UpdateStep,
    status: Option<StepStatus>,
    renderer: &TerminalRenderer,
) -> Result<()> {
    // Check if we have anything to update
    if status.is_none()
        && params.title.is_none()
        && params.description.is_none()
        && params.acceptance_criteria.is_none()
        && params.references.is_none()
        && params.result.is_none()
    {
        return Err(anyhow::anyhow!(
            "No updates specified. Use --status, --title, --description, --acceptance-criteria, --references, or --result"
        ));
    }

    // Validate result requirement for done status
    if let Some(StepStatus::Done) = status {
        if params.result.is_none() {
            return Err(anyhow::anyhow!(
                "Result description is required when marking a step as done. Use --result to describe what was accomplished."
            ));
        }
    }

    // Build list of changes made for display
    let mut changes = Vec::new();
    if status.is_some() {
        changes.push("status".to_string());
    }
    if params.title.is_some() {
        changes.push("title".to_string());
    }
    if params.description.is_some() {
        changes.push("description".to_string());
    }
    if params.acceptance_criteria.is_some() {
        changes.push("acceptance criteria".to_string());
    }
    if params.references.is_some() {
        changes.push("references".to_string());
    }

    let updated_step = planner.update_step_validated(params)
        .await
        .with_context(|| format!("Failed to update step {}", params.id))?
        .ok_or_else(|| anyhow::anyhow!("Step with ID {} not found", params.id))?;

    let result = UpdateResult::with_changes(updated_step, changes);
    renderer.render(&result.to_string())?;

    Ok(())
}

/// Handle step show command
async fn handle_step_show(
    planner: Planner,
    params: &Id,
    renderer: &TerminalRenderer,
) -> Result<()> {
    let step = planner.show_step_details(params)
        .await
        .context("Failed to get step")?
        .ok_or_else(|| anyhow::anyhow!("Step with ID {} not found", params.id))?;

    renderer.render(&step.to_string())?;

    Ok(())
}

/// Handle step swap command
async fn handle_step_swap(
    planner: Planner,
    params: &SwapSteps,
    renderer: &TerminalRenderer,
) -> Result<()> {
    planner.swap_step_positions(params).await.with_context(|| {
        format!(
            "Failed to swap steps {} and {}",
            params.step1_id, params.step2_id
        )
    })?;

    let message = format!(
        "Swapped order of steps {} and {}",
        params.step1_id, params.step2_id
    );
    let status = OperationStatus::success(message);
    renderer.render(&status.to_string())?;

    Ok(())
}

/// Handle serve command (MCP server)
async fn handle_serve(planner: Planner) -> Result<()> {
    info!("Starting Beacon MCP server");

    let server = BeaconMcpServer::new(planner);
    run_stdio_server(server)
        .await
        .context("MCP server failed")?;

    Ok(())
}
