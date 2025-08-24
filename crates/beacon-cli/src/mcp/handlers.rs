//! MCP tool handlers implementation

use std::sync::Arc;

use beacon_core::{
    Planner,
    display::{CreateResult, OperationStatus},
    params as core,
};
use log::debug;
use rmcp::{
    ErrorData, ErrorData as McpError, RoleServer,
    handler::server::tool::Parameters,
    model::{
        CallToolResult, Content, GetPromptRequestParam, GetPromptResult, ListPromptsResult,
        PaginatedRequestParam, Prompt, PromptArgument, PromptMessage, PromptMessageContent,
        PromptMessageRole,
    },
    service::RequestContext,
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
