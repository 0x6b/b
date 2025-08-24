//! MCP server implementation for Beacon
//!
//! This module implements the Model Context Protocol server for Beacon,
//! providing a standardized interface for AI models to interact with
//! the task planning system.

use std::{future::Future, sync::Arc};

use anyhow::Result;
use beacon_core::Planner;
use log::{debug, error, info};
use rmcp::{
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::{
        GetPromptRequestParam, GetPromptResult, Implementation, ListPromptsResult,
        PaginatedRequestParam, ProtocolVersion, ServerCapabilities, ServerInfo,
    },
    service::RequestContext,
    tool, tool_handler, tool_router, ErrorData as McpError, RoleServer, ServerHandler,
};
use tokio::{
    signal::unix::{signal, SignalKind},
    sync::Mutex,
};

pub mod errors;
pub mod handlers;
pub mod prompts;

// Re-export parameter types and result type from handlers for external use
pub use handlers::{
    CreatePlan, DeletePlan, Id, InsertStep, ListPlans, McpResult, SearchPlans, StepCreate,
    SwapSteps, UpdateStep,
};

/// MCP server for Beacon
#[derive(Clone)]
pub struct BeaconMcpServer {
    planner: Arc<Mutex<Planner>>,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl BeaconMcpServer {
    /// Create a new Beacon MCP server
    pub fn new(planner: Planner) -> Self {
        Self {
            planner: Arc::new(Mutex::new(planner)),
            tool_router: Self::tool_router(),
        }
    }

    // Tool methods that delegate to handlers::McpHandlers methods
    #[tool(
        name = "create_plan",
        description = "Create a new task plan to organize work. Provide a clear title (required), optional detailed description for context, and optional directory to associate with specific project location. Returns the new plan ID for adding steps."
    )]
    async fn create_plan(&self, params: Parameters<CreatePlan>) -> McpResult {
        let handlers = handlers::McpHandlers::new(self.planner.clone());
        handlers.create_plan(params).await
    }

    #[tool(
        name = "list_plans",
        description = "List all task plans. Use archived=false (default) for active plans you're working on, or archived=true to see completed/hidden plans. Returns formatted list with IDs, titles, descriptions, and directories."
    )]
    async fn list_plans(&self, params: Parameters<ListPlans>) -> McpResult {
        let handlers = handlers::McpHandlers::new(self.planner.clone());
        handlers.list_plans(params).await
    }

    #[tool(
        name = "show_plan",
        description = "Display complete details of a specific plan including all its steps, their status (todo/done), descriptions, and acceptance criteria. Use the plan ID to retrieve. Essential for understanding project scope and progress."
    )]
    async fn show_plan(&self, params: Parameters<Id>) -> McpResult {
        let handlers = handlers::McpHandlers::new(self.planner.clone());
        handlers.show_plan(params).await
    }

    #[tool(
        name = "archive_plan",
        description = "Archive a completed or inactive plan to hide it from the active list. Archived plans are preserved and can be restored later with unarchive_plan. Use when a project is finished or temporarily on hold."
    )]
    async fn archive_plan(&self, params: Parameters<Id>) -> McpResult {
        let handlers = handlers::McpHandlers::new(self.planner.clone());
        handlers.archive_plan(params).await
    }

    #[tool(
        name = "unarchive_plan",
        description = "Restore an archived plan back to the active list. Use when resuming work on a previously archived project or when you need to reference completed work. The plan and all its steps are preserved exactly as they were."
    )]
    async fn unarchive_plan(&self, params: Parameters<Id>) -> McpResult {
        let handlers = handlers::McpHandlers::new(self.planner.clone());
        handlers.unarchive_plan(params).await
    }

    #[tool(
        name = "delete_plan",
        description = "Permanently delete a plan and all its associated steps from the database. This operation cannot be undone. Use with caution - consider archiving instead if you might need the plan later."
    )]
    async fn delete_plan(&self, params: Parameters<DeletePlan>) -> McpResult {
        let handlers = handlers::McpHandlers::new(self.planner.clone());
        handlers.delete_plan(params).await
    }

    #[tool(
        name = "search_plans",
        description = "Find all plans associated with a specific directory path. Use archived=false (default) for active plans you're working on, or archived=true to see completed/hidden plans for the directory. Useful for discovering existing plans in a project folder or organizing plans by location."
    )]
    async fn search_plans(&self, params: Parameters<SearchPlans>) -> McpResult {
        let handlers = handlers::McpHandlers::new(self.planner.clone());
        handlers.search_plans(params).await
    }

    #[tool(
        name = "add_step",
        description = "Add a new step to an existing plan. Requires plan_id and title. Optionally include: description (detailed info), acceptance_criteria (completion requirements), and references (URLs/files). Steps start with 'todo' status and are added at the end of the plan."
    )]
    async fn add_step(&self, params: Parameters<StepCreate>) -> McpResult {
        let handlers = handlers::McpHandlers::new(self.planner.clone());
        handlers.add_step(params).await
    }

    #[tool(
        name = "insert_step",
        description = "Insert a new step at a specific position in a plan's step order. Position is 0-indexed (0 = first position). All existing steps at or after this position will be shifted down. Useful for adding prerequisite tasks or reorganizing workflow."
    )]
    async fn insert_step(&self, params: Parameters<InsertStep>) -> McpResult {
        let handlers = handlers::McpHandlers::new(self.planner.clone());
        handlers.insert_step(params).await
    }

    #[tool(
        name = "swap_steps",
        description = "Swap the order of two steps within the same plan. This is useful for reordering tasks without having to delete and recreate them. Both steps must belong to the same plan. The operation preserves all step properties and only changes their order."
    )]
    async fn swap_steps(&self, params: Parameters<SwapSteps>) -> McpResult {
        let handlers = handlers::McpHandlers::new(self.planner.clone());
        handlers.swap_steps(params).await
    }

    #[tool(
        name = "update_step",
        description = "Modify an existing step's properties. Use step ID to identify. Can update: status ('todo', 'inprogress', or 'done'), title, description, acceptance_criteria, and references.
        
        IMPORTANT: When changing status to 'done', you MUST provide a 'result' field describing what was actually accomplished, technically in detail, with proper Markdown format. The result will be permanently recorded and shown when viewing completed steps. The result field is ignored for all other status values. Example for marking as done:
        {
          \"id\": 5,
          \"status\": \"done\",
          \"result\": \"Configured CI/CD pipeline with GitHub Actions. Added workflows for testing, 
                      linting, and deployment to staging. All checks passing on main branch.\"
        }"
    )]
    async fn update_step(&self, params: Parameters<UpdateStep>) -> McpResult {
        let handlers = handlers::McpHandlers::new(self.planner.clone());
        handlers.update_step(params).await
    }

    #[tool(
        name = "show_step",
        description = "View detailed information about a specific step including its status, timestamps, description, acceptance criteria, and references. Use when you need to focus on a single step's details rather than the whole plan."
    )]
    async fn show_step(&self, params: Parameters<Id>) -> McpResult {
        let handlers = handlers::McpHandlers::new(self.planner.clone());
        handlers.show_step(params).await
    }

    #[tool(
        name = "claim_step",
        description = "Atomically claim a step by transitioning it from 'todo' to 'inprogress' status. This prevents multiple agents from working on the same task simultaneously. Returns success if the step was claimed, or indicates if the step was already claimed or completed."
    )]
    async fn claim_step(&self, params: Parameters<Id>) -> McpResult {
        let handlers = handlers::McpHandlers::new(self.planner.clone());
        handlers.claim_step(params).await
    }

    /// List all available prompts
    async fn list_prompts(
        &self,
        request: Option<PaginatedRequestParam>,
        context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        let handlers = handlers::McpHandlers::new(self.planner.clone());
        handlers.list_prompts(request, context).await
    }

    /// Get a specific prompt by name and apply arguments
    async fn get_prompt(
        &self,
        request: GetPromptRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        let handlers = handlers::McpHandlers::new(self.planner.clone());
        handlers.get_prompt(request, context).await
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
- **Plan Management**: create_plan, list_plans, show_plan, archive_plan, unarchive_plan, delete_plan, search_plans
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
        error!("serving error: {e:?}");
    })?;

    // Set up signal handlers for graceful shutdown
    let mut sigint = signal(SignalKind::interrupt())?;
    let mut sigterm = signal(SignalKind::terminate())?;

    tokio::select! {
        result = service.waiting() => {
            match result {
                Ok(_) => info!("MCP server stopped normally"),
                Err(e) => error!("MCP server error: {e:?}"),
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
