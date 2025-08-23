//! Beacon CLI Application
//!
//! Command-line interface for the beacon task planning tool.
//! Following the m43 pattern for clean CLI implementation.

mod cli;
mod mcp;
mod renderer;

use anyhow::{Context, Result};
use beacon_core::{PlanFilter, PlanStatus, PlannerBuilder, StepStatus};
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
    planner: beacon_core::Planner,
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
        Search(args) => handle_plan_search(planner, &args.into(), renderer).await,
    }
}

/// Handle step subcommands
async fn handle_step_command(
    planner: beacon_core::Planner,
    command: StepCommands,
    renderer: &TerminalRenderer,
) -> Result<()> {
    use StepCommands::*;
    match command {
        Add(args) => handle_step_add(planner, &args.into(), renderer).await,
        Insert(args) => handle_step_insert(planner, &args.into(), renderer).await,
        Update(args) => {
            let params: beacon_core::params::UpdateStep = args.into();

            // Convert status string back to StepStatus enum for the handler
            let status = params.status.as_ref().map(|s| match s.as_str() {
                "todo" => StepStatus::Todo,
                "inprogress" => StepStatus::InProgress,
                "done" => StepStatus::Done,
                _ => StepStatus::Todo, // Default fallback
            });

            handle_step_update(planner, &params, status, renderer).await
        }
        Show(args) => handle_step_show(planner, &args.into(), renderer).await,
        Swap(args) => handle_step_swap(planner, &args.into(), renderer).await,
    }
}

/// Handle plan create command
async fn handle_plan_create(
    planner: beacon_core::Planner,
    params: &beacon_core::params::CreatePlan,
    renderer: &TerminalRenderer,
) -> Result<()> {
    let plan = planner
        .create_plan(params)
        .await
        .context("Failed to create plan")?;

    let markdown = format!("# Plan Created\n\n{plan}");
    renderer.render(&markdown)?;

    Ok(())
}

/// Handle plan list command
async fn handle_plan_list(
    planner: beacon_core::Planner,
    params: &beacon_core::params::ListPlans,
    renderer: &TerminalRenderer,
) -> Result<()> {
    let filter = if params.archived {
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
    };

    let plans = planner
        .list_plans(filter)
        .await
        .context("Failed to list plans")?;

    let markdown = if plans.is_empty() {
        if params.archived {
            "# No archived plans found".to_string()
        } else {
            "# No active plans found".to_string()
        }
    } else {
        let mut result = if params.archived {
            "# Archived Plans\n\n".to_string()
        } else {
            "# Active Plans\n\n".to_string()
        };

        for plan in plans {
            let steps = planner
                .get_steps(&beacon_core::params::Id { id: plan.id })
                .await
                .context("Failed to get steps for plan")?;

            let completed_steps = steps
                .iter()
                .filter(|s| s.status == StepStatus::Done)
                .count() as u32;
            let total_steps = steps.len() as u32;

            let plan_summary =
                beacon_core::PlanSummary::from_plan(plan, total_steps, completed_steps);

            result.push_str(&format!("{plan_summary}"));
        }
        result
    };

    renderer.render(&markdown)?;

    Ok(())
}

/// Handle plan show command
async fn handle_plan_show(
    planner: beacon_core::Planner,
    params: &beacon_core::params::Id,
    renderer: &TerminalRenderer,
) -> Result<()> {
    let mut plan = planner
        .get_plan(params)
        .await
        .context("Failed to get plan")?
        .ok_or_else(|| anyhow::anyhow!("Plan with ID {} not found", params.id))?;

    plan.steps = planner
        .get_steps(params)
        .await
        .context("Failed to get steps")?;

    renderer.render(&format!("{plan}"))?;

    Ok(())
}

/// Handle plan archive command
async fn handle_plan_archive(
    planner: beacon_core::Planner,
    params: &beacon_core::params::Id,
    renderer: &TerminalRenderer,
) -> Result<()> {
    // Get plan details for confirmation
    let plan = planner
        .get_plan(params)
        .await
        .context("Failed to get plan")?
        .ok_or_else(|| anyhow::anyhow!("Plan with ID {} not found", params.id))?;

    let steps = planner
        .get_steps(params)
        .await
        .context("Failed to get steps")?;

    // Show what will be archived
    let mut markdown = format!("Plan to archive: {} (ID: {})", plan.title, params.id);
    if !steps.is_empty() {
        markdown.push_str(&format!("\nThis plan has {} step(s).", steps.len()));
    }

    planner
        .archive_plan(params)
        .await
        .with_context(|| format!("Failed to archive plan {}", params.id))?;

    markdown.push_str(&format!(
        "\n\nArchived plan: {} (ID: {})",
        plan.title, params.id
    ));
    markdown.push_str(&format!(
        "\nUse 'beacon plan unarchive {}' to restore this plan.",
        params.id
    ));

    renderer.render(&markdown)?;
    Ok(())
}

/// Handle plan unarchive command
async fn handle_plan_unarchive(
    planner: beacon_core::Planner,
    params: &beacon_core::params::Id,
    renderer: &TerminalRenderer,
) -> Result<()> {
    planner
        .unarchive_plan(params)
        .await
        .with_context(|| format!("Failed to unarchive plan {}", params.id))?;

    let markdown = format!("Unarchived plan with ID: {}", params.id);
    renderer.render(&markdown)?;
    Ok(())
}

/// Handle plan search command
async fn handle_plan_search(
    planner: beacon_core::Planner,
    params: &beacon_core::params::SearchPlans,
    renderer: &TerminalRenderer,
) -> Result<()> {
    let plans = if params.archived {
        // For archived plans, use list_plans with directory filter
        let filter = Some(beacon_core::PlanFilter {
            status: Some(beacon_core::PlanStatus::Archived),
            directory: Some(params.directory.clone()),
            ..Default::default()
        });
        planner
            .list_plans(filter)
            .await
            .context("Failed to search archived plans")?
    } else {
        planner
            .search_plans_by_directory(params)
            .await
            .context("Failed to search plans")?
    };

    let status_text = if params.archived {
        "archived"
    } else {
        "active"
    };
    let markdown = if plans.is_empty() {
        format!(
            "# No {} plans found in directory: {}",
            status_text, params.directory
        )
    } else {
        let mut result = format!(
            "# {} plans in directory: {}\n\n",
            status_text.to_uppercase(),
            params.directory
        );

        for plan in plans {
            let steps = planner
                .get_steps(&beacon_core::params::Id { id: plan.id })
                .await
                .context("Failed to get steps for plan")?;

            let completed_steps = steps
                .iter()
                .filter(|s| s.status == StepStatus::Done)
                .count() as u32;
            let total_steps = steps.len() as u32;

            let plan_summary =
                beacon_core::PlanSummary::from_plan(plan, total_steps, completed_steps);

            result.push_str(&format!("{plan_summary}"));
        }
        result
    };

    renderer.render(&markdown)?;

    Ok(())
}

/// Handle step add command
async fn handle_step_add(
    planner: beacon_core::Planner,
    params: &beacon_core::params::StepCreate,
    renderer: &TerminalRenderer,
) -> Result<()> {
    let step = planner
        .add_step(params)
        .await
        .with_context(|| format!("Failed to add step to plan {}", params.plan_id))?;

    let mut markdown = format!(
        "Added step: {} (ID: {})\nAdded to plan: {}",
        step.title, step.id, step.plan_id
    );
    if let Some(desc) = &step.description {
        markdown.push_str(&format!("\nDescription: {desc}"));
    }
    if let Some(criteria) = &step.acceptance_criteria {
        markdown.push_str(&format!("\nAcceptance criteria: {criteria}"));
    }
    if !step.references.is_empty() {
        markdown.push_str(&format!("\nReferences: {}", step.references.join(", ")));
    }

    renderer.render(&markdown)?;

    Ok(())
}

/// Handle step insert command
async fn handle_step_insert(
    planner: beacon_core::Planner,
    params: &beacon_core::params::InsertStep,
    renderer: &TerminalRenderer,
) -> Result<()> {
    let step = planner.insert_step(params).await.with_context(|| {
        format!(
            "Failed to insert step into plan {} at position {}",
            params.step.plan_id, params.position
        )
    })?;

    let mut markdown = format!(
        "Inserted step: {} (ID: {})\nInserted at position: {} in plan: {}",
        step.title, step.id, step.order, params.step.plan_id
    );
    if let Some(desc) = &step.description {
        markdown.push_str(&format!("\nDescription: {desc}"));
    }
    if let Some(criteria) = &step.acceptance_criteria {
        markdown.push_str(&format!("\nAcceptance criteria: {criteria}"));
    }
    if !step.references.is_empty() {
        markdown.push_str(&format!("\nReferences: {}", step.references.join(", ")));
    }

    renderer.render(&markdown)?;

    Ok(())
}

/// Handle step update command
async fn handle_step_update(
    planner: beacon_core::Planner,
    params: &beacon_core::params::UpdateStep,
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

    // Use status directly since it's already StepStatus
    let step_status = status;

    // Validate result requirement for done status
    if let Some(StepStatus::Done) = step_status {
        if params.result.is_none() {
            return Err(anyhow::anyhow!(
                "Result description is required when marking a step as done. Use --result to describe what was accomplished."
            ));
        }
    }

    // Check what will be updated for the message
    let has_status = status.is_some();
    let has_title = params.title.is_some();
    let has_description = params.description.is_some();
    let has_criteria = params.acceptance_criteria.is_some();
    let has_references = params.references.is_some();

    // Update all fields in a single call
    planner
        .update_step(
            params.id,
            beacon_core::UpdateStepRequest {
                title: params.title.clone(),
                description: params.description.clone(),
                acceptance_criteria: params.acceptance_criteria.clone(),
                references: params.references.clone(),
                status: step_status,
                result: params.result.clone(),
            },
        )
        .await
        .with_context(|| format!("Failed to update step {}", params.id))?;

    // Build update message
    let mut updates = Vec::new();
    if has_status {
        updates.push("status");
    }
    if has_title {
        updates.push("title");
    }
    if has_description {
        updates.push("description");
    }
    if has_criteria {
        updates.push("acceptance criteria");
    }
    if has_references {
        updates.push("references");
    }

    let markdown = format!("Updated step {}: {}", params.id, updates.join(", "));
    renderer.render(&markdown)?;

    Ok(())
}

/// Handle step show command
async fn handle_step_show(
    planner: beacon_core::Planner,
    params: &beacon_core::params::Id,
    renderer: &TerminalRenderer,
) -> Result<()> {
    let step = planner
        .get_step(params)
        .await
        .context("Failed to get step")?
        .ok_or_else(|| anyhow::anyhow!("Step with ID {} not found", params.id))?;

    let mut markdown = format!("# Step {} Details\n\nTitle: {}\n", step.id, step.title);

    let status_icon = match step.status {
        StepStatus::Done => "✓ Done",
        StepStatus::InProgress => "➤ In Progress",
        StepStatus::Todo => "○ Todo",
    };
    markdown.push_str(&format!(
        "Status: {}\nPlan ID: {}\n",
        status_icon, step.plan_id
    ));

    if let Some(desc) = &step.description {
        markdown.push_str(&format!("\n## Description\n{}\n", desc));
    }

    if let Some(criteria) = &step.acceptance_criteria {
        markdown.push_str(&format!("\n## Acceptance Criteria\n{}\n", criteria));
    }

    // Show result only for completed steps
    if step.status == StepStatus::Done {
        if let Some(result) = &step.result {
            markdown.push_str(&format!("\n## Result\n{}\n", result));
        }
    }

    if !step.references.is_empty() {
        markdown.push_str("\n## References\n");
        markdown.push_str(
            &step
                .references
                .iter()
                .map(|reference| format!("- {}\n", reference))
                .collect::<String>(),
        );
    }

    markdown.push_str(&format!(
        "\nCreated: {}\nUpdated: {}",
        step.created_at, step.updated_at
    ));

    renderer.render(&markdown)?;

    Ok(())
}

/// Handle step swap command
async fn handle_step_swap(
    planner: beacon_core::Planner,
    params: &beacon_core::params::SwapSteps,
    renderer: &TerminalRenderer,
) -> Result<()> {
    planner.swap_steps(params).await.with_context(|| {
        format!(
            "Failed to swap steps {} and {}",
            params.step1_id, params.step2_id
        )
    })?;

    let markdown = format!(
        "Swapped order of steps {} and {}",
        params.step1_id, params.step2_id
    );
    renderer.render(&markdown)?;

    Ok(())
}

/// Handle serve command (MCP server)
async fn handle_serve(planner: beacon_core::Planner) -> Result<()> {
    info!("Starting Beacon MCP server");

    let server = BeaconMcpServer::new(planner);
    run_stdio_server(server)
        .await
        .context("MCP server failed")?;

    Ok(())
}
