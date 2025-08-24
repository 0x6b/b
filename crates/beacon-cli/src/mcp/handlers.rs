//! MCP tool handlers implementation

use std::{future::Future, sync::Arc};

use beacon_core::{
    Planner,
    display::{CreateResult, OperationStatus},
    params as core,
};
use log::debug;
use rmcp::{
    ErrorData as McpError, RoleServer,
    handler::server::tool::Parameters,
    model::{
        CallToolResult, Content, GetPromptRequestParam, GetPromptResult, ListPromptsResult,
        PaginatedRequestParam, Prompt, PromptArgument, PromptMessage, PromptMessageContent,
        PromptMessageRole,
    },
    service::RequestContext,
    tool,
};
use schemars::JsonSchema;
use serde::Deserialize;
use tokio::sync::Mutex;

use super::{prompts::PROMPT_TEMPLATES, to_mcp_error};

// ============================================================================
// Generic Parameter Wrapper Implementation
// ============================================================================
//
// This generic wrapper struct implements the parameter wrapper pattern by:
// 1. Wrapping any core parameter type in a transparent serde container
// 2. Adding MCP-specific derives (Deserialize, JsonSchema) for JSON handling
// 3. Keeping the core types clean of framework dependencies
//
// The #[serde(transparent)] attribute ensures that
// serialization/deserialization passes through directly to the wrapped core
// type, maintaining API compatibility while adding the necessary trait
// implementations for MCP protocol handling.

/// Generic MCP wrapper for core parameter types with serde integration
///
/// Provides JSON deserialization and schema generation for any parameter type,
/// eliminating the need for individual wrapper structs while maintaining
/// the same functionality and type safety.
#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct McpParams<T>(T)
where
    T: JsonSchema;

impl<T> JsonSchema for McpParams<T>
where
    T: JsonSchema,
{
    fn schema_name() -> std::borrow::Cow<'static, str> {
        T::schema_name()
    }

    fn json_schema(g: &mut schemars::SchemaGenerator) -> schemars::Schema {
        T::json_schema(g)
    }
}

impl<T> AsRef<T> for McpParams<T>
where
    T: JsonSchema,
{
    fn as_ref(&self) -> &T {
        &self.0
    }
}

// Type aliases for cleaner usage in function signatures
pub type Id = McpParams<core::Id>;
pub type CreatePlan = McpParams<core::CreatePlan>;
pub type DeletePlan = McpParams<core::DeletePlan>;
pub type ListPlans = McpParams<core::ListPlans>;
pub type SearchPlans = McpParams<core::SearchPlans>;
pub type StepCreate = McpParams<core::StepCreate>;
pub type InsertStep = McpParams<core::InsertStep>;
pub type SwapSteps = McpParams<core::SwapSteps>;
pub type UpdateStep = McpParams<core::UpdateStep>;

pub type McpResult = Result<CallToolResult, ErrorData>;

/// Handler implementations for the MCP server
pub struct McpHandlers {
    planner: Arc<Mutex<Planner>>,
}

impl McpHandlers {
    pub fn new(planner: Arc<Mutex<Planner>>) -> Self {
        Self { planner }
    }

    #[tool(
        name = "create_plan",
        description = "Create a new task plan to organize work. Provide a clear title (required), optional detailed description for context, and optional directory to associate with specific project location. Returns the new plan ID for adding steps."
    )]
    pub async fn create_plan(&self, Parameters(params): Parameters<CreatePlan>) -> McpResult {
        debug!("create_plan: {:?}", params);

        let plan = self
            .planner
            .lock()
            .await
            .create_plan(params.as_ref())
            .await
            .map_err(|e| to_mcp_error("Failed to create plan", &e))?;

        let result = CreateResult::new(plan);
        Ok(CallToolResult::success(vec![Content::text(
            result.to_string(),
        )]))
    }

    #[tool(
        name = "list_plans",
        description = "List all task plans. Use archived=false (default) for active plans you're working on, or archived=true to see completed/hidden plans. Returns formatted list with IDs, titles, descriptions, and directories."
    )]
    pub async fn list_plans(&self, Parameters(params): Parameters<ListPlans>) -> McpResult {
        debug!("list_plans: {:?}", params);

        let planner = self.planner.lock().await;
        let inner_params = params.as_ref();
        let plan_summaries = planner
            .list_plans_summary(inner_params)
            .await
            .map_err(|e| to_mcp_error("Failed to list plans", &e))?;

        let title = if plan_summaries.is_empty() {
            if inner_params.archived {
                "No archived plans found"
            } else {
                "No active plans found"
            }
        } else if inner_params.archived {
            "Archived Plans"
        } else {
            "Active Plans"
        };

        let result = format!("# {}\n\n{}", title, plan_summaries);
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(
        name = "show_plan",
        description = "Display complete details of a specific plan including all its steps, their status (todo/done), descriptions, and acceptance criteria. Use the plan ID to retrieve. Essential for understanding project scope and progress."
    )]
    pub async fn show_plan(&self, Parameters(params): Parameters<Id>) -> McpResult {
        debug!("show_plan: {:?}", params);

        let plan = self
            .planner
            .lock()
            .await
            .get_plan(params.as_ref())
            .await
            .map_err(|e| to_mcp_error("Failed to get plan", &e))?
            .ok_or_else(|| {
                ErrorData::internal_error(
                    format!("Plan with ID {} not found", params.as_ref().id),
                    None,
                )
            })?;

        Ok(CallToolResult::success(vec![Content::text(
            plan.to_string(),
        )]))
    }

    #[tool(
        name = "archive_plan",
        description = "Archive a completed or inactive plan to hide it from the active list. Archived plans are preserved and can be restored later with unarchive_plan. Use when a project is finished or temporarily on hold."
    )]
    pub async fn archive_plan(&self, Parameters(params): Parameters<Id>) -> McpResult {
        debug!("archive_plan: {:?}", params);

        let planner = self.planner.lock().await;
        let inner_params = params.as_ref();
        let _archived_plan = planner
            .archive_plan(inner_params)
            .await
            .map_err(|e| ErrorData::internal_error(format!("Failed to archive plan: {e}"), None))?
            .ok_or_else(|| {
                ErrorData::internal_error(
                    format!("Plan with ID {} not found", inner_params.id),
                    None,
                )
            })?;

        let result = OperationStatus::success(format!(
            "Archived plan with ID {}. Use 'unarchive_plan' to restore it.",
            inner_params.id
        ));
        Ok(CallToolResult::success(vec![Content::text(
            result.to_string(),
        )]))
    }

    #[tool(
        name = "unarchive_plan",
        description = "Restore an archived plan back to the active list. Use when resuming work on a previously archived project or when you need to reference completed work. The plan and all its steps are preserved exactly as they were."
    )]
    pub async fn unarchive_plan(&self, Parameters(params): Parameters<Id>) -> McpResult {
        debug!("unarchive_plan: {:?}", params);

        let planner = self.planner.lock().await;
        let inner_params = params.as_ref();
        let _unarchived_plan = planner
            .unarchive_plan(inner_params)
            .await
            .map_err(|e| ErrorData::internal_error(format!("Failed to unarchive plan: {e}"), None))?
            .ok_or_else(|| {
                ErrorData::internal_error(
                    format!("Plan with ID {} not found", inner_params.id),
                    None,
                )
            })?;

        let result = OperationStatus::success(format!(
            "Unarchived plan with ID {}. Plan is now active again.",
            inner_params.id
        ));
        Ok(CallToolResult::success(vec![Content::text(
            result.to_string(),
        )]))
    }

    #[tool(
        name = "delete_plan",
        description = "Permanently delete a plan and all its associated steps from the database. This operation cannot be undone. Use with caution - consider archiving instead if you might need the plan later."
    )]
    pub async fn delete_plan(&self, Parameters(params): Parameters<DeletePlan>) -> McpResult {
        debug!("delete_plan: {:?}", params);
        let planner = self.planner.lock().await;
        let inner_params = params.as_ref();

        let deleted_plan = planner
            .delete_plan(inner_params)
            .await
            .map_err(|e| ErrorData::internal_error(format!("Failed to delete plan: {e}"), None))?
            .ok_or_else(|| {
                ErrorData::internal_error(
                    format!("Plan with ID {} not found", inner_params.id),
                    None,
                )
            })?;

        let result = OperationStatus::success(format!(
            "Permanently deleted plan '{}' (ID: {}). This action cannot be undone.",
            deleted_plan.title, inner_params.id
        ));
        Ok(CallToolResult::success(vec![Content::text(
            result.to_string(),
        )]))
    }

    #[tool(
        name = "search_plans",
        description = "Find all plans associated with a specific directory path. Use archived=false (default) for active plans you're working on, or archived=true to see completed/hidden plans for the directory. Useful for discovering existing plans in a project folder or organizing plans by location."
    )]
    pub async fn search_plans(&self, Parameters(params): Parameters<SearchPlans>) -> McpResult {
        debug!("search_plans: {:?}", params);

        let planner = self.planner.lock().await;
        let inner_params = params.as_ref();
        let plan_summaries = planner
            .search_plans_summary(inner_params)
            .await
            .map_err(|e| ErrorData::internal_error(format!("Failed to search plans: {e}"), None))?;

        let result = if plan_summaries.is_empty() {
            let status_text = if inner_params.archived {
                "archived"
            } else {
                "active"
            };
            format!(
                "No {} plans found in directory: {}",
                status_text, inner_params.directory
            )
        } else {
            let status_text = if inner_params.archived {
                "archived"
            } else {
                "active"
            };
            let title = format!(
                "{} plans in directory: {}",
                status_text.to_uppercase(),
                inner_params.directory
            );
            format!("# {}\n\n{}", title, plan_summaries)
        };

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(
        name = "add_step",
        description = "Add a new step to an existing plan. Requires plan_id and title. Optionally include: description (detailed info), acceptance_criteria (completion requirements), and references (URLs/files). Steps start with 'todo' status and are added at the end of the plan."
    )]
    pub async fn add_step(&self, Parameters(params): Parameters<StepCreate>) -> McpResult {
        debug!("add_step: {:?}", params);

        let planner = self.planner.lock().await;
        let inner_params = params.as_ref();
        let step = planner
            .add_step(inner_params)
            .await
            .map_err(|e| ErrorData::internal_error(format!("Failed to add step: {e}"), None))?;

        let result = CreateResult::new(step);
        Ok(CallToolResult::success(vec![Content::text(
            result.to_string(),
        )]))
    }

    #[tool(
        name = "insert_step",
        description = "Insert a new step at a specific position in a plan's step order. Position is 0-indexed (0 = first position). All existing steps at or after this position will be shifted down. Useful for adding prerequisite tasks or reorganizing workflow."
    )]
    pub async fn insert_step(&self, Parameters(params): Parameters<InsertStep>) -> McpResult {
        debug!("insert_step: {:?}", params);

        let planner = self.planner.lock().await;
        let inner_params = params.as_ref();
        let step = planner
            .insert_step(inner_params)
            .await
            .map_err(|e| ErrorData::internal_error(format!("Failed to insert step: {e}"), None))?;

        let result = CreateResult::new(step);
        Ok(CallToolResult::success(vec![Content::text(
            result.to_string(),
        )]))
    }

    #[tool(
        name = "swap_steps",
        description = "Swap the order of two steps within the same plan. This is useful for reordering tasks without having to delete and recreate them. Both steps must belong to the same plan. The operation preserves all step properties and only changes their order."
    )]
    pub async fn swap_steps(&self, Parameters(params): Parameters<SwapSteps>) -> McpResult {
        debug!("swap_steps: {:?}", params);

        let planner = self.planner.lock().await;
        let inner_params = params.as_ref();
        planner
            .swap_steps(inner_params)
            .await
            .map_err(|e| ErrorData::internal_error(format!("Failed to swap steps: {e}"), None))?;

        let result = OperationStatus::success(format!(
            "Successfully swapped the order of steps {} and {}",
            inner_params.step1_id, inner_params.step2_id
        ));

        Ok(CallToolResult::success(vec![Content::text(
            result.to_string(),
        )]))
    }

    #[tool(
        name = "update_step",
        description = "Modify an existing step's properties. Use step ID to identify. Can update: status ('todo', 'inprogress', or 'done'), title, description,        acceptance_criteria, and references.
        
        IMPORTANT: When changing status to 'done', you MUST provide a 'result' field describing what was actually accomplished, technically in detail. The result will be permanently recorded and shown when viewing completed steps. The result field is ignored for all other status values. Example for marking as done:
        {
          \"id\": 5,
          \"status\": \"done\",
          \"result\": \"Configured CI/CD pipeline with GitHub Actions. Added workflows for testing, 
                      linting, and deployment to staging. All checks passing on main branch.\"
        }"
    )]
    pub async fn update_step(&self, Parameters(params): Parameters<UpdateStep>) -> McpResult {
        debug!("update_step: {:?}", params);

        let planner = self.planner.lock().await;
        let inner_params = params.as_ref();
        let _updated_step = planner
            .update_step_validated(inner_params)
            .await
            .map_err(|e| to_mcp_error("Failed to update step", &e))?
            .ok_or_else(|| {
                ErrorData::internal_error(
                    format!("Step with ID {} not found", inner_params.id),
                    None,
                )
            })?;

        // Build update messages based on what was provided
        let mut messages = Vec::new();
        if inner_params.status.is_some() {
            messages.push(format!(
                "Updated status to '{}'",
                inner_params.status.as_ref().unwrap()
            ));
        }
        if inner_params.title.is_some() {
            messages.push("Updated title".to_string());
        }
        if inner_params.description.is_some() {
            messages.push("Updated description".to_string());
        }
        if inner_params.acceptance_criteria.is_some() {
            messages.push("Updated acceptance criteria".to_string());
        }
        if inner_params.references.is_some() {
            messages.push("Updated references".to_string());
        }

        let result = if messages.is_empty() {
            "No updates provided for step".to_string()
        } else {
            format!("Step {} updated: {}", inner_params.id, messages.join(", "))
        };

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(
        name = "show_step",
        description = "View detailed information about a specific step including its status, timestamps, description, acceptance criteria, and references. Use when you need to focus on a single step's details rather than the whole plan."
    )]
    pub async fn show_step(&self, Parameters(params): Parameters<Id>) -> McpResult {
        debug!("show_step: {:?}", params);

        let planner = self.planner.lock().await;
        let inner_params = params.as_ref();
        let step = planner
            .get_step(inner_params)
            .await
            .map_err(|e| ErrorData::internal_error(format!("Failed to get step: {e}"), None))?
            .ok_or_else(|| {
                ErrorData::internal_error(
                    format!("Step with ID {} not found", inner_params.id),
                    None,
                )
            })?;

        Ok(CallToolResult::success(vec![Content::text(
            step.to_string(),
        )]))
    }

    #[tool(
        name = "claim_step",
        description = "Atomically claim a step by transitioning it from 'todo' to 'inprogress' status. This prevents multiple agents from working on the same task simultaneously. Returns success if the step was claimed, or indicates if the step was already claimed or completed."
    )]
    pub async fn claim_step(&self, Parameters(params): Parameters<Id>) -> McpResult {
        debug!("claim_step: {:?}", params);

        let planner = self.planner.lock().await;
        let inner_params = params.as_ref();

        match planner.claim_step(inner_params).await {
            Ok(Some(_step)) => {
                let message = format!(
                    "Successfully claimed step {} - it is now marked as 'in progress'\n\n<system-reminder>\nLaunch a focused subagent for this step. Once completed, use `update_step` with the detailed results of what was accomplished.\n</system-reminder>",
                    inner_params.id
                );
                Ok(CallToolResult::success(vec![Content::text(message)]))
            }
            Ok(None) => {
                // Step was not found or not in todo status, get current status
                let step = planner.get_step(inner_params).await.map_err(|e| {
                    ErrorData::internal_error(format!("Failed to get step: {e}"), None)
                })?;

                if let Some(step) = step {
                    use beacon_core::models::StepStatus;
                    let status_description = match step.status {
                        StepStatus::InProgress => "already in progress",
                        StepStatus::Done => "already completed",
                        StepStatus::Todo => "in todo status but could not be claimed",
                    };
                    let message = format!(
                        "Cannot claim step {} - it is {}",
                        inner_params.id, status_description
                    );
                    Ok(CallToolResult::success(vec![Content::text(message)]))
                } else {
                    Err(ErrorData::internal_error(
                        format!("Step with ID {} not found", inner_params.id),
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
    pub async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        debug!("list_prompts");

        let templates = &PROMPT_TEMPLATES;
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
    pub async fn get_prompt(
        &self,
        request: GetPromptRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        debug!("get_prompt: {}", request.name);

        let templates = &PROMPT_TEMPLATES;
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

// Missing import - add ErrorData
use rmcp::ErrorData;
