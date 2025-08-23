//! MCP server implementation for Beacon
//!
//! This module implements the Model Context Protocol server for Beacon,
//! providing a standardized interface for AI models to interact with
//! the task planning system.

use std::{fmt::Write, future::Future, str::FromStr, sync::Arc};

use anyhow::Result;
use beacon_core::{
    display::{format_plan_list, CreateResult, OperationStatus},
    params as core, PlanFilter, PlanStatus, Planner, StepStatus, UpdateStepRequest,
};
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
struct McpParams<T>(T)
where
    T: JsonSchema;

impl<T> JsonSchema for McpParams<T>
where
    T: JsonSchema,
{
    fn schema_name() -> std::borrow::Cow<'static, str> {
        T::schema_name()
    }

    fn json_schema(gen: &mut schemars::SchemaGenerator) -> schemars::Schema {
        T::json_schema(gen)
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
type Id = McpParams<core::Id>;
type CreatePlan = McpParams<core::CreatePlan>;
type ListPlans = McpParams<core::ListPlans>;
type SearchPlans = McpParams<core::SearchPlans>;
type StepCreate = McpParams<core::StepCreate>;
type InsertStep = McpParams<core::InsertStep>;
type SwapSteps = McpParams<core::SwapSteps>;
type UpdateStep = McpParams<core::UpdateStep>;

/// Helper to convert planner errors to MCP errors
fn to_mcp_error(message: &str, error: beacon_core::PlannerError) -> ErrorData {
    ErrorData::internal_error(format!("{}: {}", message, error), None)
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
            description: "Create a structured action plan using Beacon's MCP tools".to_string(),
            template: r#"You are **Beacon Planner**, expert at creating well-structured task plans.

# Goal
{goal}

# Your Task
Create a comprehensive plan to achieve this goal using Beacon's MCP tools.

# Step 1: Check Existing Plans
First, use `search_plans` to check for existing plans in the current directory. If relevant plans exist, consider whether to build upon them or create fresh.

# Step 2: Create the Plan
Use `create_plan` with:
- **title**: Concise summary (5-7 words)
- **description**: Clear explanation of approach and expected outcome
- **directory**: (optional - defaults to current directory)

# Step 3: Define Steps
For each logical unit of work, use `add_step` with the plan_id.

## Step Structure Template
```
title: "[Action Verb] [Specific Target]"

description: |
  - **Context**: [Why this step is needed, current state]
  - **Approach**: [How to accomplish this]
  - **Scope**: [What's included/excluded]
  - **Tools/Commands**: [Specific tools or commands to use]
  - **Files**: [Key files/directories involved]

acceptance_criteria: |
  - [ ] [Specific measurable outcome]
  - [ ] [Test command and expected result]
  - [ ] [Quality metric to meet]
  - [ ] [Validation check]

references: ["file.rs", "docs/api.md", "tests/test.rs"]
```

## Step Types to Include

### Analysis Steps
- Understand current implementation
- Identify dependencies and constraints
- Document findings in step result

### Implementation Steps  
- Make specific code changes
- Include test coverage
- Follow project conventions

### Validation Steps
- Run tests and checks
- Verify acceptance criteria
- Ensure no regressions

### Integration Steps
- Connect components
- Verify system behavior
- Test rollback procedures

## Quality Guidelines

Each step should be:
- **Atomic**: Can be completed independently
- **Clear**: Self-contained with all context
- **Verifiable**: Has measurable acceptance criteria
- **Safe**: Includes rollback plan if risky

The complete plan should have:
- 5-10 well-defined steps
- Clear dependencies between steps
- Validation checkpoints
- Risk mitigation for complex operations

## Output
Create a plan that provides everything needed for successful execution. Each step should contain sufficient context that any agent can claim and complete it."#.to_string(),
            arguments: vec![
                PromptTemplateArg {
                    name: "goal".to_string(),
                    description: "The goal or outcome to create a plan for".to_string(),
                    required: true,
                },
            ],
        },
        PromptTemplate {
            name: "do".to_string(),
            description: "Execute a plan by launching focused subagents for each step".to_string(),
            template: r#"You are orchestrating the execution of a Beacon plan by launching focused subagents for each step.

# Plan to Execute
Plan ID: {plan_id}

# Execution Strategy
You will act as an orchestrator, launching specialized subagents, as parrallel if possible, to handle individual steps while maintaining overall progress tracking.

## Step 1: Locate the Plan
{plan_id ? "Use the provided plan_id" : "Use `search_plans` with the current directory to find the most recent active plan"}

## Step 2: Review the Plan
Call `show_plan(id: plan_id)` to understand:
- Overall goal and approach
- All steps and their current status
- Dependencies between steps
- Which steps can be parallelized

## Step 3: Execute Steps via Subagents

For each step with status "todo":

### 3.1 Claim the Step
```
claim_step(id: step_id)
```
This atomically reserves the step for your subagent.

### 3.2 Prepare Subagent Context
Call `show_step(id: step_id)` to gather:
- Step description with full context
- Acceptance criteria
- References and relevant files

### 3.3 Launch Focused Subagent

Create a subagent with a **focused, specific prompt**:

```
You are a specialized subagent tasked with completing a specific step.

## Your Mission
[Step title from show_step]

## Context
[Description from show_step, including Context, Approach, Scope, Tools, and Files sections]

## Success Criteria
[Acceptance criteria from show_step]
Each criterion must be verifiably met before considering the task complete.

## References
[List of relevant files/docs from show_step]

## Your Task
1. Execute the work described above
2. Stay focused on ONLY this specific step
3. Validate each acceptance criterion
4. Document what you accomplished

## Constraints
- Do not work on other steps
- Do not make changes outside the defined scope
- If blocked, document the specific issue
- Provide detailed evidence of success

## Deliverable
Upon completion, provide:
- Detailed description of what was accomplished
- Evidence that each acceptance criterion was met
- Any important findings or deviations
- Test results or validation output
```

### 3.4 Monitor Subagent Progress
While the subagent works:
- Let it focus on the specific task
- Avoid interrupting unless necessary
- Trust it to complete the defined scope

### 3.5 Capture Subagent Results
When the subagent completes, use its output to:
```
update_step(
  id: step_id,
  status: "done",
  result: "[Subagent's detailed report of what was accomplished, validation results, and evidence of success]"
)
```

### 3.6 Handle Subagent Blockers
If the subagent reports a blocker:
```
update_step(
  id: step_id,
  description: description + "\n\nBLOCKER: [Specific issue reported by subagent]",
  status: "inprogress"  // Keep claimed while resolving
)
```
Then either:
- Launch a new subagent with additional context
- Escalate for human intervention
- Try alternative approach

## Step 4: Orchestration Patterns

### Parallel Execution
When steps have no dependencies:
- Claim multiple steps simultaneously
- Launch multiple subagents in parallel
- Each subagent works independently
- Collect results as they complete

### Sequential Execution
When steps have dependencies:
- Wait for prerequisite steps to complete
- Pass relevant results to dependent step subagents
- Ensure outputs flow correctly between steps

### Complex Step Handling
If a step is too large for one subagent:
- Consider using `insert_step` to break it down
- Launch multiple specialized subagents for sub-tasks
- Coordinate their outputs into the final result

## Step 5: Progress Management

Periodically:
- Call `show_plan(id: plan_id)` to review overall progress
- Identify next steps ready for execution
- Check for any blocked steps needing attention
- Determine if additional subagents should be launched

## Subagent Launch Guidelines

### Keep Subagents Focused
- One step per subagent
- Clear, specific objectives
- Defined scope and constraints
- Explicit success criteria

### Provide Complete Context
Each subagent should receive:
- The full step description
- All acceptance criteria
- Relevant file references
- Any results from prerequisite steps

### Enable Independence
Subagents should be able to:
- Work without additional guidance
- Make decisions within their scope
- Validate their own success
- Report clear results

## Quality Assurance

Before marking any step done:
- Verify the subagent met ALL acceptance criteria
- Review the documented results
- Ensure no regressions were introduced
- Confirm the work aligns with the plan's goal

## Completion

When all steps show status "done":
- Review the complete plan with `show_plan`
- Verify the overall goal was achieved
- Consider archiving the plan if appropriate
- Document any lessons learned

Remember: You are the orchestrator. Your role is to launch focused subagents with clear missions, track progress, and ensure the plan succeeds through coordinated execution."#.to_string(),
            arguments: vec![
                PromptTemplateArg {
                    name: "plan_id".to_string(),
                    description: "The ID of the plan to execute (if not provided, will search for latest plan in current directory)".to_string(),
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
    async fn create_plan(&self, Parameters(params): Parameters<CreatePlan>) -> McpResult {
        debug!("create_plan: {:?}", params);

        let planner = self.planner.lock().await;
        let inner_params = params.as_ref();
        let plan = planner
            .create_plan(inner_params)
            .await
            .map_err(|e| to_mcp_error("Failed to create plan", e))?;

        let result = CreateResult::new(plan);
        Ok(CallToolResult::success(vec![Content::text(
            result.to_string(),
        )]))
    }

    #[tool(
        name = "list_plans",
        description = "List all task plans. Use archived=false (default) for active plans you're working on, or archived=true to see completed/hidden plans. Returns formatted list with IDs, titles, descriptions, and directories."
    )]
    async fn list_plans(&self, Parameters(params): Parameters<ListPlans>) -> McpResult {
        debug!("list_plans: {:?}", params);

        let planner = self.planner.lock().await;
        let inner_params = params.as_ref();
        let filter = if inner_params.archived {
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
            .map_err(|e| to_mcp_error("Failed to list plans", e))?;

        if plans.is_empty() {
            let title = if inner_params.archived {
                "No archived plans found"
            } else {
                "No active plans found"
            };
            let empty_plans: Vec<beacon_core::models::PlanSummary> = Vec::new();
            let result = format_plan_list(&empty_plans, Some(title));
            Ok(CallToolResult::success(vec![Content::text(result)]))
        } else {
            // Convert Plans to PlanSummary with step counts
            let mut plan_summaries = Vec::new();
            for plan in plans {
                let steps = planner
                    .get_steps(&core::Id { id: plan.id })
                    .await
                    .map_err(|e| {
                        ErrorData::internal_error(format!("Failed to get steps: {e}"), None)
                    })?;

                let completed_steps = steps
                    .iter()
                    .filter(|s| s.status == StepStatus::Done)
                    .count() as u32;
                let total_steps = steps.len() as u32;

                let summary =
                    beacon_core::models::PlanSummary::from_plan(plan, total_steps, completed_steps);
                plan_summaries.push(summary);
            }

            let title = if inner_params.archived {
                "Archived Plans"
            } else {
                "Active Plans"
            };
            let result = format_plan_list(&plan_summaries, Some(title));
            Ok(CallToolResult::success(vec![Content::text(result)]))
        }
    }

    #[tool(
        name = "show_plan",
        description = "Display complete details of a specific plan including all its steps, their status (todo/done), descriptions, and acceptance criteria. Use the plan ID to retrieve. Essential for understanding project scope and progress."
    )]
    async fn show_plan(&self, Parameters(params): Parameters<Id>) -> McpResult {
        debug!("show_plan: {:?}", params);

        let planner = self.planner.lock().await;
        let inner_params = params.as_ref();
        let mut plan = planner
            .get_plan(inner_params)
            .await
            .map_err(|e| to_mcp_error("Failed to get plan", e))?
            .ok_or_else(|| {
                ErrorData::internal_error(
                    format!("Plan with ID {} not found", inner_params.id),
                    None,
                )
            })?;

        plan.steps = planner
            .get_steps(inner_params)
            .await
            .map_err(|e| to_mcp_error("Failed to get steps", e))?;

        Ok(CallToolResult::success(vec![Content::text(
            plan.to_string(),
        )]))
    }

    #[tool(
        name = "archive_plan",
        description = "Archive a completed or inactive plan to hide it from the active list. Archived plans are preserved and can be restored later with unarchive_plan. Use when a project is finished or temporarily on hold."
    )]
    async fn archive_plan(&self, Parameters(params): Parameters<Id>) -> McpResult {
        debug!("archive_plan: {:?}", params);

        let planner = self.planner.lock().await;
        let inner_params = params.as_ref();
        planner
            .archive_plan(inner_params)
            .await
            .map_err(|e| ErrorData::internal_error(format!("Failed to archive plan: {e}"), None))?;

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
    async fn unarchive_plan(&self, Parameters(params): Parameters<Id>) -> McpResult {
        debug!("unarchive_plan: {:?}", params);

        let planner = self.planner.lock().await;
        let inner_params = params.as_ref();
        planner.unarchive_plan(inner_params).await.map_err(|e| {
            ErrorData::internal_error(format!("Failed to unarchive plan: {e}"), None)
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
        name = "search_plans",
        description = "Find all plans associated with a specific directory path. Use archived=false (default) for active plans you're working on, or archived=true to see completed/hidden plans for the directory. Useful for discovering existing plans in a project folder or organizing plans by location."
    )]
    async fn search_plans(&self, Parameters(params): Parameters<SearchPlans>) -> McpResult {
        debug!("search_plans: {:?}", params);

        let planner = self.planner.lock().await;
        let inner_params = params.as_ref();

        // Use the directory-specific search method which respects archived status
        let plans = if inner_params.archived {
            // For archived plans, search all plans and filter by directory
            let filter = Some(PlanFilter {
                status: Some(PlanStatus::Archived),
                directory: Some(inner_params.directory.clone()),
                ..Default::default()
            });
            planner.list_plans(filter).await.map_err(|e| {
                ErrorData::internal_error(format!("Failed to search plans: {e}"), None)
            })?
        } else {
            // For active plans, use the existing directory search
            planner
                .search_plans_by_directory(inner_params)
                .await
                .map_err(|e| {
                    ErrorData::internal_error(format!("Failed to search plans: {e}"), None)
                })?
        };

        let result = if plans.is_empty() {
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
            let mut result = String::new();
            let status_text = if inner_params.archived {
                "archived"
            } else {
                "active"
            };
            writeln!(
                result,
                "# {} plans in directory: {}\n",
                status_text.to_uppercase(),
                inner_params.directory
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
    async fn add_step(&self, Parameters(params): Parameters<StepCreate>) -> McpResult {
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
    async fn insert_step(&self, Parameters(params): Parameters<InsertStep>) -> McpResult {
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
    async fn swap_steps(&self, Parameters(params): Parameters<SwapSteps>) -> McpResult {
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
    async fn update_step(&self, Parameters(params): Parameters<UpdateStep>) -> McpResult {
        debug!("update_step: {:?}", params);

        let planner = self.planner.lock().await;
        let inner_params = params.as_ref();
        let mut messages = Vec::new();

        // Parse status if provided using FromStr
        let step_status = if let Some(status_str) = &inner_params.status {
            Some(StepStatus::from_str(status_str).map_err(|_| {
                ErrorData::internal_error(
                    format!(
                        "Invalid status: {status_str}. Must be 'todo', 'inprogress', or 'done'"
                    ),
                    None,
                )
            })?)
        } else {
            None
        };

        // Validate result requirement for done status
        if let Some(StepStatus::Done) = step_status {
            if inner_params.result.is_none() {
                return Err(ErrorData::internal_error(
                    "Result description is required when marking a step as done. Please provide a 'result' field describing what was accomplished.".to_string(),
                    None,
                ));
            }
        }

        // Update all fields in a single call
        planner
            .update_step(
                inner_params.id,
                UpdateStepRequest {
                    title: inner_params.title.clone(),
                    description: inner_params.description.clone(),
                    acceptance_criteria: inner_params.acceptance_criteria.clone(),
                    references: inner_params.references.clone(),
                    status: step_status,
                    result: inner_params.result.clone(),
                },
            )
            .await
            .map_err(|e| to_mcp_error("Failed to update step", e))?;

        // Build update messages
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
    async fn show_step(&self, Parameters(params): Parameters<Id>) -> McpResult {
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
    async fn claim_step(&self, Parameters(params): Parameters<Id>) -> McpResult {
        debug!("claim_step: {:?}", params);

        let planner = self.planner.lock().await;
        let inner_params = params.as_ref();

        match planner.claim_step(inner_params).await {
            Ok(true) => {
                let message = format!(
                    "Successfully claimed step {} - it is now marked as 'in progress'\n\n<system-reminder>\nLaunch a focused subagent for this step. Once completed, use `update_step` with the detailed results of what was accomplished.\n</system-reminder>",
                    inner_params.id
                );
                Ok(CallToolResult::success(vec![Content::text(message)]))
            }
            Ok(false) => {
                // Step was not in todo status, get current status
                let step = planner.get_step(inner_params).await.map_err(|e| {
                    ErrorData::internal_error(format!("Failed to get step: {e}"), None)
                })?;

                if let Some(step) = step {
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
    debug!(
        "Server created with {} tools",
        server.tool_router.list_all().len()
    );

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
