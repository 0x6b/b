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
use cli::{Cli, Commands, OutputFormat, PlanCommands, StepCommands, StepStatusArg};
use log::{debug, info};
use mcp::{run_stdio_server, BeaconMcpServer};
use renderer::TerminalRenderer;
use serde::Serialize;

/// Plan with progress information for JSON output
#[derive(Serialize)]
struct PlanWithProgress {
    id: u64,
    title: String,
    description: Option<String>,
    directory: Option<String>,
    created_at: jiff::Timestamp,
    updated_at: jiff::Timestamp,
    steps_completed: usize,
    steps_total: usize,
}

impl PlanWithProgress {
    fn from_plan(plan: beacon_core::Plan, steps_completed: usize, steps_total: usize) -> Self {
        Self {
            id: plan.id,
            title: plan.title,
            description: plan.description,
            directory: plan.directory,
            created_at: plan.created_at,
            updated_at: plan.updated_at,
            steps_completed,
            steps_total,
        }
    }
}

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
    match cli.command {
        Commands::Plan { command } => handle_plan_command(planner, command, &renderer).await,
        Commands::Step { command } => handle_step_command(planner, command, &renderer).await,
        Commands::Serve => handle_serve(planner).await,
    }
}

/// Handle plan subcommands
async fn handle_plan_command(
    planner: beacon_core::Planner,
    command: PlanCommands,
    renderer: &TerminalRenderer,
) -> Result<()> {
    match command {
        PlanCommands::Create {
            title,
            description,
            directory,
        } => {
            handle_plan_create(
                planner,
                &title,
                description.as_deref(),
                directory.as_deref(),
                renderer,
            )
            .await
        }
        PlanCommands::List { archived, format } => {
            handle_plan_list(planner, archived, format, renderer).await
        }
        PlanCommands::Show { id, format } => handle_plan_show(planner, id, format, renderer).await,
        PlanCommands::Archive { id } => handle_plan_archive(planner, id, renderer).await,
        PlanCommands::Unarchive { id } => handle_plan_unarchive(planner, id, renderer).await,
        PlanCommands::Search {
            directory,
            archived,
            format,
        } => handle_plan_search(planner, directory, archived, format, renderer).await,
    }
}

/// Handle step subcommands
async fn handle_step_command(
    planner: beacon_core::Planner,
    command: StepCommands,
    renderer: &TerminalRenderer,
) -> Result<()> {
    match command {
        StepCommands::Add {
            plan_id,
            title,
            description,
            acceptance_criteria,
            references,
        } => {
            handle_step_add(
                planner,
                plan_id,
                &title,
                description.as_deref(),
                acceptance_criteria.as_deref(),
                references,
                renderer,
            )
            .await
        }
        StepCommands::Insert {
            plan_id,
            position,
            title,
            description,
            acceptance_criteria,
            references,
        } => {
            handle_step_insert(
                planner,
                plan_id,
                position,
                &title,
                description.as_deref(),
                acceptance_criteria.as_deref(),
                references,
                renderer,
            )
            .await
        }
        StepCommands::Update {
            id,
            status,
            title,
            description,
            acceptance_criteria,
            references,
            result,
        } => {
            handle_step_update(
                planner,
                id,
                status,
                title,
                description,
                acceptance_criteria,
                references,
                result,
                renderer,
            )
            .await
        }
        StepCommands::Show { id, format } => handle_step_show(planner, id, format, renderer).await,
        StepCommands::Swap { step1, step2 } => {
            handle_step_swap(planner, step1, step2, renderer).await
        }
    }
}

/// Handle plan create command
async fn handle_plan_create(
    planner: beacon_core::Planner,
    title: &str,
    description: Option<&str>,
    directory: Option<&str>,
    renderer: &TerminalRenderer,
) -> Result<()> {
    let plan = planner
        .create_plan(title, description, directory)
        .await
        .context("Failed to create plan")?;

    let markdown = format!("# Plan Created\n\n{plan}");
    renderer.render(&markdown)?;

    Ok(())
}

/// Handle plan list command
async fn handle_plan_list(
    planner: beacon_core::Planner,
    archived: bool,
    format: OutputFormat,
    renderer: &TerminalRenderer,
) -> Result<()> {
    let filter = if archived {
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

    match format {
        OutputFormat::Text => {
            let markdown = if plans.is_empty() {
                if archived {
                    "# No archived plans found".to_string()
                } else {
                    "# No active plans found".to_string()
                }
            } else {
                let mut result = if archived {
                    "# Archived Plans\n\n".to_string()
                } else {
                    "# Active Plans\n\n".to_string()
                };

                for plan in plans {
                    let steps = planner
                        .get_steps(plan.id)
                        .await
                        .context("Failed to get steps for plan")?;

                    let completed_steps = steps
                        .iter()
                        .filter(|s| s.status == StepStatus::Done)
                        .count() as u32;
                    let total_steps = steps.len() as u32;

                    let plan_summary = beacon_core::PlanSummary {
                        id: plan.id,
                        title: plan.title,
                        description: plan.description,
                        status: plan.status,
                        directory: plan.directory,
                        created_at: plan.created_at,
                        updated_at: plan.updated_at,
                        total_steps,
                        completed_steps,
                        pending_steps: total_steps - completed_steps,
                    };

                    result.push_str(&format!("{plan_summary}"));
                }
                result
            };

            renderer.render(&markdown)?;
        }
        OutputFormat::Json => {
            let mut plans_with_progress = Vec::new();

            for plan in plans {
                let steps = planner
                    .get_steps(plan.id)
                    .await
                    .context("Failed to get steps for plan")?;

                let completed_steps = steps
                    .iter()
                    .filter(|s| s.status == StepStatus::Done)
                    .count();
                let total_steps = steps.len();

                plans_with_progress.push(PlanWithProgress::from_plan(
                    plan,
                    completed_steps,
                    total_steps,
                ));
            }

            println!("{}", serde_json::to_string_pretty(&plans_with_progress)?);
        }
    }

    Ok(())
}

/// Handle plan show command
async fn handle_plan_show(
    planner: beacon_core::Planner,
    id: u64,
    format: OutputFormat,
    renderer: &TerminalRenderer,
) -> Result<()> {
    let mut plan = planner
        .get_plan(id)
        .await
        .context("Failed to get plan")?
        .ok_or_else(|| anyhow::anyhow!("Plan with ID {} not found", id))?;

    plan.steps = planner.get_steps(id).await.context("Failed to get steps")?;

    match format {
        OutputFormat::Text => renderer.render(&format!("{plan}"))?,
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&plan)?),
    }

    Ok(())
}

/// Handle plan archive command
async fn handle_plan_archive(
    planner: beacon_core::Planner,
    id: u64,
    renderer: &TerminalRenderer,
) -> Result<()> {
    // Get plan details for confirmation
    let plan = planner
        .get_plan(id)
        .await
        .context("Failed to get plan")?
        .ok_or_else(|| anyhow::anyhow!("Plan with ID {} not found", id))?;

    let steps = planner.get_steps(id).await.context("Failed to get steps")?;

    // Show what will be archived
    let mut markdown = format!("Plan to archive: {} (ID: {})", plan.title, id);
    if !steps.is_empty() {
        markdown.push_str(&format!("\nThis plan has {} step(s).", steps.len()));
    }

    planner
        .archive_plan(id)
        .await
        .with_context(|| format!("Failed to archive plan {id}"))?;

    markdown.push_str(&format!("\n\nArchived plan: {} (ID: {})", plan.title, id));
    markdown.push_str(&format!(
        "\nUse 'beacon plan unarchive {id}' to restore this plan."
    ));

    renderer.render(&markdown)?;
    Ok(())
}

/// Handle plan unarchive command
async fn handle_plan_unarchive(
    planner: beacon_core::Planner,
    id: u64,
    renderer: &TerminalRenderer,
) -> Result<()> {
    planner
        .unarchive_plan(id)
        .await
        .with_context(|| format!("Failed to unarchive plan {id}"))?;

    let markdown = format!("Unarchived plan with ID: {id}");
    renderer.render(&markdown)?;
    Ok(())
}

/// Handle plan search command
async fn handle_plan_search(
    planner: beacon_core::Planner,
    directory: String,
    archived: bool,
    format: OutputFormat,
    renderer: &TerminalRenderer,
) -> Result<()> {
    let plans = if archived {
        // For archived plans, use list_plans with directory filter
        let filter = Some(beacon_core::PlanFilter {
            status: Some(beacon_core::PlanStatus::Archived),
            directory: Some(directory.clone()),
            ..Default::default()
        });
        planner
            .list_plans(filter)
            .await
            .context("Failed to search archived plans")?
    } else {
        planner
            .search_plans_by_directory(&directory)
            .await
            .context("Failed to search plans")?
    };

    match format {
        OutputFormat::Text => {
            let status_text = if archived { "archived" } else { "active" };
            let markdown = if plans.is_empty() {
                format!("# No {} plans found in directory: {directory}", status_text)
            } else {
                let mut result = format!(
                    "# {} plans in directory: {directory}\n\n",
                    status_text.to_uppercase()
                );

                for plan in plans {
                    let steps = planner
                        .get_steps(plan.id)
                        .await
                        .context("Failed to get steps for plan")?;

                    let completed_steps = steps
                        .iter()
                        .filter(|s| s.status == StepStatus::Done)
                        .count() as u32;
                    let total_steps = steps.len() as u32;

                    let plan_summary = beacon_core::PlanSummary {
                        id: plan.id,
                        title: plan.title,
                        description: plan.description,
                        status: plan.status,
                        directory: plan.directory,
                        created_at: plan.created_at,
                        updated_at: plan.updated_at,
                        total_steps,
                        completed_steps,
                        pending_steps: total_steps - completed_steps,
                    };

                    result.push_str(&format!("{plan_summary}"));
                }
                result
            };

            renderer.render(&markdown)?;
        }
        OutputFormat::Json => {
            let mut plans_with_progress = Vec::new();

            for plan in plans {
                let steps = planner
                    .get_steps(plan.id)
                    .await
                    .context("Failed to get steps for plan")?;

                let completed_steps = steps
                    .iter()
                    .filter(|s| s.status == StepStatus::Done)
                    .count();
                let total_steps = steps.len();

                plans_with_progress.push(PlanWithProgress::from_plan(
                    plan,
                    completed_steps,
                    total_steps,
                ));
            }

            println!("{}", serde_json::to_string_pretty(&plans_with_progress)?);
        }
    }

    Ok(())
}

/// Handle step add command
async fn handle_step_add(
    planner: beacon_core::Planner,
    plan_id: u64,
    title: &str,
    description: Option<&str>,
    acceptance_criteria: Option<&str>,
    references: Vec<String>,
    renderer: &TerminalRenderer,
) -> Result<()> {
    let step = planner
        .add_step(plan_id, title, description, acceptance_criteria, references)
        .await
        .with_context(|| format!("Failed to add step to plan {plan_id}"))?;

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
    plan_id: u64,
    position: u32,
    title: &str,
    description: Option<&str>,
    acceptance_criteria: Option<&str>,
    references: Vec<String>,
    renderer: &TerminalRenderer,
) -> Result<()> {
    let step = planner
        .insert_step(
            plan_id,
            position,
            title,
            description,
            acceptance_criteria,
            references,
        )
        .await
        .with_context(|| {
            format!("Failed to insert step into plan {plan_id} at position {position}")
        })?;

    let mut markdown = format!(
        "Inserted step: {} (ID: {})\nInserted at position: {} in plan: {}",
        step.title, step.id, step.order, step.plan_id
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
    id: u64,
    status: Option<StepStatusArg>,
    title: Option<String>,
    description: Option<String>,
    acceptance_criteria: Option<String>,
    references: Option<Vec<String>>,
    result: Option<String>,
    renderer: &TerminalRenderer,
) -> Result<()> {
    // Check if we have anything to update
    if status.is_none()
        && title.is_none()
        && description.is_none()
        && acceptance_criteria.is_none()
        && references.is_none()
        && result.is_none()
    {
        return Err(anyhow::anyhow!(
            "No updates specified. Use --status, --title, --description, --acceptance-criteria, --references, or --result"
        ));
    }

    // Parse status if provided
    let step_status = status.map(|s| match s {
        StepStatusArg::Todo => StepStatus::Todo,
        StepStatusArg::InProgress => StepStatus::InProgress,
        StepStatusArg::Done => StepStatus::Done,
    });

    // Validate result requirement for done status
    if let Some(StepStatus::Done) = step_status {
        if result.is_none() {
            return Err(anyhow::anyhow!(
                "Result description is required when marking a step as done. Use --result to describe what was accomplished."
            ));
        }
    }

    // Check what will be updated for the message
    let has_status = status.is_some();
    let has_title = title.is_some();
    let has_description = description.is_some();
    let has_criteria = acceptance_criteria.is_some();
    let has_references = references.is_some();

    // Update all fields in a single call
    planner
        .update_step(
            id,
            title,
            description,
            acceptance_criteria,
            references,
            step_status,
            result,
        )
        .await
        .with_context(|| format!("Failed to update step {id}"))?;

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

    let markdown = format!("Updated step {}: {}", id, updates.join(", "));
    renderer.render(&markdown)?;

    Ok(())
}

/// Handle step show command
async fn handle_step_show(
    planner: beacon_core::Planner,
    id: u64,
    format: OutputFormat,
    renderer: &TerminalRenderer,
) -> Result<()> {
    let step = planner
        .get_step(id)
        .await
        .context("Failed to get step")?
        .ok_or_else(|| anyhow::anyhow!("Step with ID {} not found", id))?;

    match format {
        OutputFormat::Text => {
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
                for reference in &step.references {
                    markdown.push_str(&format!("- {}\n", reference));
                }
            }

            markdown.push_str(&format!(
                "\nCreated: {}\nUpdated: {}",
                step.created_at, step.updated_at
            ));

            renderer.render(&markdown)?;
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&step)?);
        }
    }

    Ok(())
}

/// Handle step swap command
async fn handle_step_swap(
    planner: beacon_core::Planner,
    step1: u64,
    step2: u64,
    renderer: &TerminalRenderer,
) -> Result<()> {
    planner
        .swap_steps(step1, step2)
        .await
        .with_context(|| format!("Failed to swap steps {} and {}", step1, step2))?;

    let markdown = format!("Swapped order of steps {} and {}", step1, step2);
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
