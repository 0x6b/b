//! MCP server implementation for Beacon
//!
//! This module implements the Model Context Protocol server for Beacon,
//! providing a standardized interface for AI models to interact with
//! the task planning system.

use std::{fmt::Write, future::Future, sync::Arc};

use anyhow::Result;
use beacon_core::{PlanFilter, PlanStatus, Planner, StepStatus};
use rmcp::{
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::{
        CallToolResult, Content, GetPromptRequestParam, GetPromptResult, Implementation,
        ListPromptsResult, PaginatedRequestParam, Prompt, PromptArgument, PromptMessage,
        PromptMessageContent, PromptMessageRole, ProtocolVersion, ServerCapabilities, ServerInfo,
    },
    service::RequestContext,
    tool, tool_handler, tool_router, ErrorData, ErrorData as McpError, RoleServer, ServerHandler,
};
use schemars::JsonSchema;
use serde::Deserialize;
use tokio::sync::Mutex;
use tracing::{debug, info};

/// Parameters for creating a plan
#[derive(Debug, Deserialize, JsonSchema)]
struct CreatePlanParams {
    title: String,
    description: Option<String>,
    directory: Option<String>,
}

/// Parameters for listing plans
#[derive(Debug, Deserialize, JsonSchema)]
struct ListPlansParams {
    #[serde(default)]
    archived: bool,
}

/// Parameters for showing a plan
#[derive(Debug, Deserialize, JsonSchema)]
struct ShowPlanParams {
    id: u64,
}

/// Parameters for archiving/unarchiving a plan
#[derive(Debug, Deserialize, JsonSchema)]
struct PlanIdParams {
    id: u64,
}

/// Parameters for searching plans
#[derive(Debug, Deserialize, JsonSchema)]
struct SearchPlansParams {
    directory: String,
    #[serde(default)]
    archived: bool,
}

/// Parameters for adding a step
#[derive(Debug, Deserialize, JsonSchema)]
struct AddStepParams {
    plan_id: u64,
    title: String,
    description: Option<String>,
    acceptance_criteria: Option<String>,
    #[serde(default)]
    references: Vec<String>,
}

/// Parameters for inserting a step at a specific position
#[derive(Debug, Deserialize, JsonSchema)]
struct InsertStepParams {
    plan_id: u64,
    position: u32,
    title: String,
    description: Option<String>,
    acceptance_criteria: Option<String>,
    #[serde(default)]
    references: Vec<String>,
}

/// Parameters for swapping two steps
#[derive(Debug, Deserialize, JsonSchema)]
struct SwapStepsParams {
    step1_id: u64,
    step2_id: u64,
}

/// Parameters for updating a step
#[derive(Debug, Deserialize, JsonSchema)]
struct UpdateStepParams {
    /// Step ID to update
    id: u64,

    /// New status for the step ('todo', 'inprogress', or 'done')
    status: Option<String>,

    /// Title of the step
    title: Option<String>,

    /// Detailed description of the step
    description: Option<String>,

    /// Acceptance criteria for the step
    acceptance_criteria: Option<String>,

    /// References (URLs, file paths, etc.)
    references: Option<Vec<String>>,

    /// Result description - REQUIRED when changing status to 'done'.
    /// This field documents what was actually accomplished when completing the
    /// step. It will be IGNORED when:
    /// - Changing status to 'todo' or 'inprogress'
    /// - Updating other fields without changing status
    /// - Creating a new step (steps always start as 'todo')
    ///
    /// Example: "Implemented user authentication using JWT tokens with
    /// refresh token rotation. Added middleware for route protection and
    /// created login/logout endpoints. All tests passing."
    #[serde(default, skip_serializing_if = "Option::is_none")]
    result: Option<String>,
}

/// Parameters for showing a step
#[derive(Debug, Deserialize, JsonSchema)]
struct ShowStepParams {
    id: u64,
}

/// Parameters for claiming a step
#[derive(Debug, Deserialize, JsonSchema)]
struct ClaimStepParams {
    id: u64,
}

/// Definition of a prompt template
#[derive(Debug, Clone)]
struct PromptTemplate {
    name: String,
    description: String,
    template: String,
    arguments: Vec<PromptTemplateArg>,
}

/// Argument definition for a prompt template
#[derive(Debug, Clone)]
struct PromptTemplateArg {
    name: String,
    description: String,
    required: bool,
}

/// Get predefined prompt templates for task planning
fn get_prompt_templates() -> Vec<PromptTemplate> {
    vec![
        PromptTemplate {
            name: "plan".to_string(),
            description: "Create a plan from a goal".to_string(),
            template: r#"Create a detailed action plan for the following goal:

Goal: {goal}

Provide:

1. A clear plan title and description
2. Relevant number of actionable steps with clear title and description, logically ordered, with concrete and detailed acceptance criteria.
"#.to_string(),
            arguments: vec![
                PromptTemplateArg {
                    name: "goal".to_string(),
                    description: "The  goal to create a plan for".to_string(),
                    required: true,
                },
            ],
        },
        PromptTemplate {
            name: "do".to_string(),
            description: "Execute a plan by launching focused subagents for each step. If no plan_id provided, use search_plans tool to find the latest plan for the current directory.".to_string(),
            template: r#"Execute the plan by working through each step systematically.

For each step:
1. Launch a focused, small subagent to handle the specific task
2. Ensure the subagent has clear context and objectives
3. Monitor progress and results
4. Update step status as work progresses
5. Move to the next step only when the current one is complete

Approach each step with focused attention and ensure quality completion before proceeding.

Plan ID: {plan_id}"#.to_string(),
            arguments: vec![
                PromptTemplateArg {
                    name: "plan_id".to_string(),
                    description: "The ID of the plan to execute (if not provided, use search_plans tool to find latest plan for current directory)".to_string(),
                    required: false,
                },
            ],
        },
    ]
}

/// MCP server for Beacon
#[derive(Clone)]
pub struct BeaconMcpServer {
    planner: Arc<Mutex<Planner>>,
    tool_router: ToolRouter<Self>,
}

type McpResult = Result<CallToolResult, ErrorData>;

#[tool_router]
impl BeaconMcpServer {
    /// Create a new Beacon MCP server
    pub fn new(planner: Planner) -> Self {
        Self {
            planner: Arc::new(Mutex::new(planner)),
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        name = "create_plan",
        description = "Create a new task plan to organize work. Provide a clear title (required), optional detailed description for context, and optional directory to associate with specific project location. Returns the new plan ID for adding steps."
    )]
    async fn create_plan(&self, Parameters(params): Parameters<CreatePlanParams>) -> McpResult {
        debug!("create_plan: {:?}", params);

        let planner = self.planner.lock().await;
        let plan = planner
            .create_plan(
                &params.title,
                params.description.as_deref(),
                params.directory.as_deref(),
            )
            .await
            .map_err(|e| ErrorData::internal_error(format!("Failed to create plan: {e}"), None))?;

        let mut result = format!("Created plan: {} (ID: {})", plan.title, plan.id);
        if let Some(desc) = params.description {
            result.push_str(&format!("\nDescription: {desc}"));
        }
        if let Some(dir) = &plan.directory {
            result.push_str(&format!("\nDirectory: {dir}"));
        }

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(
        name = "list_plans",
        description = "List all task plans. Use archived=false (default) for active plans you're working on, or archived=true to see completed/hidden plans. Returns formatted list with IDs, titles, descriptions, and directories."
    )]
    async fn list_plans(&self, Parameters(params): Parameters<ListPlansParams>) -> McpResult {
        debug!("list_plans: {:?}", params);

        let planner = self.planner.lock().await;
        let filter = if params.archived {
            Some(PlanFilter {
                status: Some(PlanStatus::Archived),
                ..Default::default()
            })
        } else {
            Some(PlanFilter {
                status: Some(PlanStatus::Active),
                ..Default::default()
            })
        };

        let plans = planner
            .list_plans(filter)
            .await
            .map_err(|e| ErrorData::internal_error(format!("Failed to list plans: {e}"), None))?;

        let mut result = String::new();

        if plans.is_empty() {
            if params.archived {
                writeln!(result, "# No archived plans found").unwrap();
            } else {
                writeln!(result, "# No active plans found").unwrap();
            }
        } else {
            // Get step counts for each plan
            let mut plans_with_progress = Vec::new();
            for plan in plans {
                let steps = planner.get_steps(plan.id).await.map_err(|e| {
                    ErrorData::internal_error(format!("Failed to get steps: {e}"), None)
                })?;

                let completed_steps = steps
                    .iter()
                    .filter(|s| s.status == StepStatus::Done)
                    .count();
                let total_steps = steps.len();

                plans_with_progress.push((plan, completed_steps, total_steps));
            }

            if params.archived {
                writeln!(result, "# Archived Plans").unwrap();
            } else {
                writeln!(result, "# Active Plans").unwrap();
            }

            for (plan, completed, total) in plans_with_progress {
                writeln!(result).unwrap();

                let progress = if total > 0 {
                    format!(" ({}/{})", completed, total)
                } else {
                    String::new()
                };

                writeln!(result, "## {} (ID: {}){}", plan.title, plan.id, progress).unwrap();
                writeln!(result).unwrap();

                if let Some(desc) = &plan.description {
                    writeln!(result, "- Description: {}", desc).unwrap();
                }
                if let Some(dir) = &plan.directory {
                    writeln!(result, "- Directory: {}", dir).unwrap();
                }
                writeln!(
                    result,
                    "- Created: {}",
                    plan.created_at.strftime("%Y-%m-%d %H:%M:%S UTC")
                )
                .unwrap();
            }
        };

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(
        name = "show_plan",
        description = "Display complete details of a specific plan including all its steps, their status (todo/done), descriptions, and acceptance criteria. Use the plan ID to retrieve. Essential for understanding project scope and progress."
    )]
    async fn show_plan(&self, Parameters(params): Parameters<ShowPlanParams>) -> McpResult {
        debug!("show_plan: {:?}", params);

        let planner = self.planner.lock().await;
        let plan = planner
            .get_plan(params.id)
            .await
            .map_err(|e| ErrorData::internal_error(format!("Failed to get plan: {e}"), None))?
            .ok_or_else(|| {
                ErrorData::internal_error(format!("Plan with ID {} not found", params.id), None)
            })?;

        let steps = planner
            .get_steps(params.id)
            .await
            .map_err(|e| ErrorData::internal_error(format!("Failed to get steps: {e}"), None))?;

        let mut result = String::new();
        writeln!(result, "# {}. {}", plan.id, plan.title).unwrap();
        writeln!(result).unwrap();

        // Metadata section
        writeln!(result, "- Status: {}", plan.status.as_str()).unwrap();
        if let Some(dir) = &plan.directory {
            writeln!(result, "- Directory: {dir}").unwrap();
        }
        writeln!(
            result,
            "- Created: {}",
            plan.created_at.strftime("%Y-%m-%d %H:%M:%S UTC")
        )
        .unwrap();
        writeln!(
            result,
            "- Updated: {}",
            plan.updated_at.strftime("%Y-%m-%d %H:%M:%S UTC")
        )
        .unwrap();

        // Description as a paragraph
        if let Some(desc) = &plan.description {
            writeln!(result).unwrap();
            writeln!(result, "{desc}").unwrap();
        }

        if steps.is_empty() {
            writeln!(result).unwrap();
            writeln!(result, "No steps in this plan.").unwrap();
        } else {
            writeln!(result).unwrap();
            writeln!(result, "## Steps").unwrap();
            writeln!(result).unwrap();
            for (index, step) in steps.iter().enumerate() {
                let position = index + 1;
                let status_text = match step.status {
                    StepStatus::Done => "done",
                    StepStatus::InProgress => "in progress",
                    StepStatus::Todo => "todo",
                };
                writeln!(result, "### {}. {} ({})", position, step.title, status_text).unwrap();
                writeln!(result).unwrap();

                if let Some(desc) = &step.description {
                    writeln!(result, "{desc}").unwrap();
                    writeln!(result).unwrap();
                }

                if let Some(criteria) = &step.acceptance_criteria {
                    writeln!(result, "#### Acceptance").unwrap();
                    writeln!(result).unwrap();
                    writeln!(result, "{criteria}").unwrap();
                    writeln!(result).unwrap();
                }

                // Show result only for completed steps
                if step.status == StepStatus::Done {
                    if let Some(step_result) = &step.result {
                        writeln!(result, "#### Result").unwrap();
                        writeln!(result).unwrap();
                        writeln!(result, "{}", step_result).unwrap();
                        writeln!(result).unwrap();
                    }
                }

                if !step.references.is_empty() {
                    writeln!(result, "#### References").unwrap();
                    writeln!(result).unwrap();
                    for reference in &step.references {
                        writeln!(result, "- {reference}").unwrap();
                    }
                    writeln!(result).unwrap();
                }
            }
        }

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(
        name = "archive_plan",
        description = "Archive a completed or inactive plan to hide it from the active list. Archived plans are preserved and can be restored later with unarchive_plan. Use when a project is finished or temporarily on hold."
    )]
    async fn archive_plan(&self, Parameters(params): Parameters<PlanIdParams>) -> McpResult {
        debug!("archive_plan: {:?}", params);

        let planner = self.planner.lock().await;
        planner
            .archive_plan(params.id)
            .await
            .map_err(|e| ErrorData::internal_error(format!("Failed to archive plan: {e}"), None))?;

        let result = format!(
            "Archived plan with ID {}. Use 'unarchive_plan' to restore it.",
            params.id
        );
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(
        name = "unarchive_plan",
        description = "Restore an archived plan back to the active list. Use when resuming work on a previously archived project or when you need to reference completed work. The plan and all its steps are preserved exactly as they were."
    )]
    async fn unarchive_plan(&self, Parameters(params): Parameters<PlanIdParams>) -> McpResult {
        debug!("unarchive_plan: {:?}", params);

        let planner = self.planner.lock().await;
        planner.unarchive_plan(params.id).await.map_err(|e| {
            ErrorData::internal_error(format!("Failed to unarchive plan: {e}"), None)
        })?;

        let result = format!(
            "Unarchived plan with ID {}. Plan is now active again.",
            params.id
        );
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(
        name = "search_plans",
        description = "Find all plans associated with a specific directory path. Use archived=false (default) for active plans you're working on, or archived=true to see completed/hidden plans for the directory. Useful for discovering existing plans in a project folder or organizing plans by location."
    )]
    async fn search_plans(&self, Parameters(params): Parameters<SearchPlansParams>) -> McpResult {
        debug!("search_plans: {:?}", params);

        let planner = self.planner.lock().await;

        // Use the directory-specific search method which respects archived status
        let plans = if params.archived {
            // For archived plans, search all plans and filter by directory
            let filter = Some(PlanFilter {
                status: Some(PlanStatus::Archived),
                directory: Some(params.directory.clone()),
                ..Default::default()
            });
            planner.list_plans(filter).await.map_err(|e| {
                ErrorData::internal_error(format!("Failed to search plans: {e}"), None)
            })?
        } else {
            // For active plans, use the existing directory search
            planner
                .search_plans_by_directory(&params.directory)
                .await
                .map_err(|e| {
                    ErrorData::internal_error(format!("Failed to search plans: {e}"), None)
                })?
        };

        let result = if plans.is_empty() {
            let status_text = if params.archived {
                "archived"
            } else {
                "active"
            };
            format!(
                "No {} plans found in directory: {}",
                status_text, params.directory
            )
        } else {
            let mut result = String::new();
            let status_text = if params.archived {
                "archived"
            } else {
                "active"
            };
            writeln!(
                result,
                "# {} plans in directory: {}\n",
                status_text.to_uppercase(),
                params.directory
            )
            .unwrap();
            for plan in plans {
                writeln!(result, "- **{}** (ID: {})", plan.title, plan.id).unwrap();
                if let Some(desc) = &plan.description {
                    writeln!(result, "  Description: {desc}").unwrap();
                }
                if let Some(dir) = &plan.directory {
                    writeln!(result, "  Directory: {dir}").unwrap();
                }
                writeln!(result, "  Status: {}", plan.status.as_str()).unwrap();
            }
            result
        };

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(
        name = "add_step",
        description = "Add a new step to an existing plan. Requires plan_id and title. Optionally include: description (detailed info), acceptance_criteria (completion requirements), and references (URLs/files). Steps start with 'todo' status and are added at the end of the plan."
    )]
    async fn add_step(&self, Parameters(params): Parameters<AddStepParams>) -> McpResult {
        debug!("add_step: {:?}", params);

        let planner = self.planner.lock().await;
        let step = planner
            .add_step(
                params.plan_id,
                &params.title,
                params.description.as_deref(),
                params.acceptance_criteria.as_deref(),
                params.references,
            )
            .await
            .map_err(|e| ErrorData::internal_error(format!("Failed to add step: {e}"), None))?;

        let mut result = String::new();
        writeln!(
            result,
            "Added step: {} (ID: {}) to plan {}",
            step.title, step.id, params.plan_id
        )
        .unwrap();
        if let Some(desc) = &step.description {
            writeln!(result, "Description: {desc}").unwrap();
        }
        if let Some(criteria) = &step.acceptance_criteria {
            writeln!(result, "Acceptance criteria: {criteria}").unwrap();
        }
        if !step.references.is_empty() {
            writeln!(result, "References: {}", step.references.join(", ")).unwrap();
        }

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(
        name = "insert_step",
        description = "Insert a new step at a specific position in a plan's step order. Position is 0-indexed (0 = first position). All existing steps at or after this position will be shifted down. Useful for adding prerequisite tasks or reorganizing workflow."
    )]
    async fn insert_step(&self, Parameters(params): Parameters<InsertStepParams>) -> McpResult {
        debug!("insert_step: {:?}", params);

        let planner = self.planner.lock().await;
        let step = planner
            .insert_step(
                params.plan_id,
                params.position,
                &params.title,
                params.description.as_deref(),
                params.acceptance_criteria.as_deref(),
                params.references,
            )
            .await
            .map_err(|e| ErrorData::internal_error(format!("Failed to insert step: {e}"), None))?;

        let mut result = String::new();
        writeln!(
            result,
            "Inserted step: {} (ID: {}) at position {} in plan {}",
            step.title, step.id, params.position, params.plan_id
        )
        .unwrap();
        if let Some(desc) = &step.description {
            writeln!(result, "Description: {desc}").unwrap();
        }
        if let Some(criteria) = &step.acceptance_criteria {
            writeln!(result, "Acceptance criteria: {criteria}").unwrap();
        }
        if !step.references.is_empty() {
            writeln!(result, "References: {}", step.references.join(", ")).unwrap();
        }

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(
        name = "swap_steps",
        description = "Swap the order of two steps within the same plan. This is useful for reordering tasks without having to delete and recreate them. Both steps must belong to the same plan. The operation preserves all step properties and only changes their order."
    )]
    async fn swap_steps(&self, Parameters(params): Parameters<SwapStepsParams>) -> McpResult {
        debug!("swap_steps: {:?}", params);

        let planner = self.planner.lock().await;
        planner
            .swap_steps(params.step1_id, params.step2_id)
            .await
            .map_err(|e| ErrorData::internal_error(format!("Failed to swap steps: {e}"), None))?;

        let result = format!(
            "Successfully swapped the order of steps {} and {}",
            params.step1_id, params.step2_id
        );

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(
        name = "update_step",
        description = "Modify an existing step's properties. Use step ID to identify. 
        Can update: status ('todo', 'inprogress', or 'done'), title, description, 
        acceptance_criteria, and references. 
        
        IMPORTANT: When changing status to 'done', you MUST provide a 'result' field 
        describing what was actually accomplished. The result will be permanently recorded 
        and shown when viewing completed steps. The result field is ignored for all other 
        status values.
        
        Example for marking as done:
        {
          \"id\": 5,
          \"status\": \"done\",
          \"result\": \"Configured CI/CD pipeline with GitHub Actions. Added workflows for testing, 
                      linting, and deployment to staging. All checks passing on main branch.\"
        }"
    )]
    async fn update_step(&self, Parameters(params): Parameters<UpdateStepParams>) -> McpResult {
        debug!("update_step: {:?}", params);

        let planner = self.planner.lock().await;
        let mut messages = Vec::new();

        // Parse status if provided
        let step_status = if let Some(status_str) = &params.status {
            Some(match status_str.as_str() {
                "todo" => StepStatus::Todo,
                "inprogress" => StepStatus::InProgress,
                "done" => StepStatus::Done,
                _ => {
                    return Err(ErrorData::internal_error(
                        format!(
                            "Invalid status: {status_str}. Must be 'todo', 'inprogress', or 'done'"
                        ),
                        None,
                    ))
                }
            })
        } else {
            None
        };

        // Validate result requirement for done status
        if let Some(StepStatus::Done) = step_status {
            if params.result.is_none() {
                return Err(ErrorData::internal_error(
                    "Result description is required when marking a step as done. Please provide a 'result' field describing what was accomplished.".to_string(),
                    None,
                ));
            }
        }

        // Update all fields in a single call
        planner
            .update_step(
                params.id,
                params.title.clone(),
                params.description.clone(),
                params.acceptance_criteria.clone(),
                params.references.clone(),
                step_status,
                params.result.clone(),
            )
            .await
            .map_err(|e| ErrorData::internal_error(format!("Failed to update step: {e}"), None))?;

        // Build update messages
        if params.status.is_some() {
            messages.push(format!("Updated status to '{}'", params.status.unwrap()));
        }
        if params.title.is_some() {
            messages.push("Updated title".to_string());
        }
        if params.description.is_some() {
            messages.push("Updated description".to_string());
        }
        if params.acceptance_criteria.is_some() {
            messages.push("Updated acceptance criteria".to_string());
        }
        if params.references.is_some() {
            messages.push("Updated references".to_string());
        }

        let result = if messages.is_empty() {
            "No updates provided for step".to_string()
        } else {
            format!("Step {} updated: {}", params.id, messages.join(", "))
        };

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(
        name = "show_step",
        description = "View detailed information about a specific step including its status, timestamps, description, acceptance criteria, and references. Use when you need to focus on a single step's details rather than the whole plan."
    )]
    async fn show_step(&self, Parameters(params): Parameters<ShowStepParams>) -> McpResult {
        debug!("show_step: {:?}", params);

        let planner = self.planner.lock().await;
        let step = planner
            .get_step(params.id)
            .await
            .map_err(|e| ErrorData::internal_error(format!("Failed to get step: {e}"), None))?
            .ok_or_else(|| {
                ErrorData::internal_error(format!("Step with ID {} not found", params.id), None)
            })?;

        let mut result = String::new();
        writeln!(result, "# Step {} Details", step.id).unwrap();
        writeln!(result).unwrap();
        writeln!(result, "Title: {}", step.title).unwrap();

        let status_text = match step.status {
            StepStatus::Done => "✓ Done",
            StepStatus::InProgress => "➤ In Progress",
            StepStatus::Todo => "○ Todo",
        };
        writeln!(result, "Status: {}", status_text).unwrap();
        writeln!(result, "Plan ID: {}", step.plan_id).unwrap();

        if let Some(desc) = &step.description {
            writeln!(result).unwrap();
            writeln!(result, "## Description").unwrap();
            writeln!(result, "{}", desc).unwrap();
        }

        if let Some(criteria) = &step.acceptance_criteria {
            writeln!(result).unwrap();
            writeln!(result, "## Acceptance Criteria").unwrap();
            writeln!(result, "{}", criteria).unwrap();
        }

        // Show result only for completed steps
        if step.status == StepStatus::Done {
            if let Some(step_result) = &step.result {
                writeln!(result).unwrap();
                writeln!(result, "## Result").unwrap();
                writeln!(result, "{}", step_result).unwrap();
            }
        }

        if !step.references.is_empty() {
            writeln!(result).unwrap();
            writeln!(result, "## References").unwrap();
            for reference in &step.references {
                writeln!(result, "- {}", reference).unwrap();
            }
        }

        writeln!(result).unwrap();
        writeln!(result, "Created: {}", step.created_at).unwrap();
        writeln!(result, "Updated: {}", step.updated_at).unwrap();

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(
        name = "claim_step",
        description = "Atomically claim a step by transitioning it from 'todo' to 'inprogress' status. This prevents multiple agents from working on the same task simultaneously. Returns success if the step was claimed, or indicates if the step was already claimed or completed."
    )]
    async fn claim_step(&self, Parameters(params): Parameters<ClaimStepParams>) -> McpResult {
        debug!("claim_step: {:?}", params);

        let planner = self.planner.lock().await;

        match planner.claim_step(params.id).await {
            Ok(true) => {
                let message = format!(
                    "Successfully claimed step {} - it is now marked as 'in progress'\n\n<system-reminder>\nLaunch a focused subagent for this step. Once completed, use `update_step` with the detailed results of what was accomplished.\n</system-reminder>",
                    params.id
                );
                Ok(CallToolResult::success(vec![Content::text(message)]))
            }
            Ok(false) => {
                // Step was not in todo status, get current status
                let step = planner.get_step(params.id).await.map_err(|e| {
                    ErrorData::internal_error(format!("Failed to get step: {e}"), None)
                })?;

                if let Some(step) = step {
                    let status_str = match step.status {
                        StepStatus::InProgress => "already in progress",
                        StepStatus::Done => "already completed",
                        StepStatus::Todo => "in todo status but could not be claimed",
                    };
                    let message = format!("Cannot claim step {} - it is {}", params.id, status_str);
                    Ok(CallToolResult::success(vec![Content::text(message)]))
                } else {
                    Err(ErrorData::internal_error(
                        format!("Step with ID {} not found", params.id),
                        None,
                    ))
                }
            }
            Err(e) => Err(ErrorData::internal_error(
                format!("Failed to claim step: {e}"),
                None,
            )),
        }
    }

    /// List all available prompts
    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        debug!("list_prompts");

        let templates = get_prompt_templates();
        let prompts = templates
            .iter()
            .map(|template| {
                Prompt::new(
                    &template.name,
                    Some(&template.description),
                    Some(
                        template
                            .arguments
                            .iter()
                            .map(|arg| PromptArgument {
                                name: arg.name.clone(),
                                description: Some(arg.description.clone()),
                                required: Some(arg.required),
                            })
                            .collect(),
                    ),
                )
            })
            .collect();

        Ok(ListPromptsResult {
            next_cursor: None,
            prompts,
        })
    }

    /// Get a specific prompt by name and apply arguments
    async fn get_prompt(
        &self,
        request: GetPromptRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        debug!("get_prompt: {}", request.name);

        let templates = get_prompt_templates();
        let template = templates
            .iter()
            .find(|t| t.name == request.name)
            .ok_or_else(|| McpError::invalid_params("Prompt not found", None))?;

        let mut prompt_text = template.template.clone();

        // Apply argument substitution if arguments are provided
        if let Some(args) = &request.arguments {
            for arg_def in &template.arguments {
                if let Some(arg_value) = args.get(&arg_def.name) {
                    if let Some(arg_str) = arg_value.as_str() {
                        let placeholder = format!("{{{}}}", arg_def.name);
                        prompt_text = prompt_text.replace(&placeholder, arg_str);
                    } else if arg_def.required {
                        return Err(McpError::invalid_params(
                            format!("Argument '{}' must be a string", arg_def.name),
                            None,
                        ));
                    }
                } else if arg_def.required {
                    return Err(McpError::invalid_params(
                        format!("Required argument '{}' is missing", arg_def.name),
                        None,
                    ));
                }
            }
        } else {
            // Check if any required arguments are missing
            let required_args: Vec<_> = template
                .arguments
                .iter()
                .filter(|arg| arg.required)
                .map(|arg| arg.name.as_str())
                .collect();
            if !required_args.is_empty() {
                return Err(McpError::invalid_params(
                    format!("Required arguments missing: {}", required_args.join(", ")),
                    None,
                ));
            }
        }

        Ok(GetPromptResult {
            description: Some(template.description.clone()),
            messages: vec![PromptMessage {
                role: PromptMessageRole::User,
                content: PromptMessageContent::text(prompt_text),
            }],
        })
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for BeaconMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_prompts()
                .build(),
            server_info: Implementation {
                name: "beacon".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            instructions: Some(r#"Beacon is a task planning and management system that helps organize work into structured plans with actionable steps.

## Core Concepts
- **Plans**: High-level projects or goals with title, description, and optional working directory
- **Steps**: Individual tasks within a plan, each with status (todo/inprogress/done), descriptions, and acceptance criteria

## Workflow Examples

### Starting a New Project
1. Create a plan with `create_plan` - provide a clear title and optional description
2. Add steps with `add_step` - break down the work into manageable tasks
3. Use `show_plan` to review the complete project structure

### Tracking Progress
1. Use `list_plans` to see all active projects
2. Claim steps with `claim_step` to mark them as in progress (prevents conflicts when multiple agents work on the same plan)
3. Update step status with `update_step` as work progresses (todo → inprogress → done)
4. Archive finished plans with `archive_plan` to keep workspace organized

### Managing Multiple Projects
- Use directories to organize plans by project location
- Search plans by directory with `search_plans`
- View archived plans with `list_plans` (archived=true) for reference

## Best Practices
- Create clear, actionable step titles
- Use acceptance criteria to define 'done' for complex steps
- Add references (URLs, files) to steps for quick access to resources

## Tool Categories
- **Plan Management**: create_plan, list_plans, show_plan, archive_plan, unarchive_plan, search_plans
- **Step Management**: add_step, insert_step, update_step, show_step, claim_step, swap_steps

## Concurrency Support
The `claim_step` tool provides atomic step claiming, ensuring that multiple agents or LLMs can safely work on the same plan without conflicts. When a step is claimed, it transitions from 'todo' to 'inprogress' status, preventing other agents from claiming the same step."#.to_string()),
        }
    }

    async fn list_prompts(
        &self,
        request: Option<PaginatedRequestParam>,
        context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        self.list_prompts(request, context).await
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        self.get_prompt(request, context).await
    }
}

/// Run the MCP server with stdio transport
pub async fn run_stdio_server(server: BeaconMcpServer) -> Result<()> {
    use rmcp::{transport::stdio, ServiceExt};

    info!("Starting Beacon MCP server on stdio");

    let service = server.serve(stdio()).await.inspect_err(|e| {
        tracing::error!("serving error: {e:?}");
    })?;

    // Set up signal handlers for graceful shutdown
    let mut sigint = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())?;
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;

    tokio::select! {
        result = service.waiting() => {
            match result {
                Ok(_) => info!("MCP server stopped normally"),
                Err(e) => tracing::error!("MCP server error: {e:?}"),
            }
        }
        _ = sigint.recv() => {
            info!("Received SIGINT, shutting down gracefully...");
        }
        _ = sigterm.recv() => {
            info!("Received SIGTERM, shutting down gracefully...");
        }
    }

    info!("MCP server shutdown complete");
    Ok(())
}
