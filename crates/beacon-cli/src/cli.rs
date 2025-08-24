use std::str::FromStr;

use anyhow::{Context, Result};
use beacon_core::{
    params::*, CreateResult, Id, OperationStatus, Planner, StepStatus, UpdateResult,
};
use clap::{Parser, Subcommand, ValueEnum};

use crate::renderer::TerminalRenderer;

/// Handler implementations for the CLI
pub struct Cli {
    planner: Planner,
    renderer: TerminalRenderer,
}

impl Cli {
    pub fn new(planner: Planner, renderer: TerminalRenderer) -> Self {
        Self { planner, renderer }
    }

    /// Handle plan subcommands
    pub(crate) async fn handle_plan_command(&self, command: PlanCommands) -> Result<()> {
        use PlanCommands::*;
        match command {
            Create(args) => self.create_plan(&args.into()).await,
            List(args) => self.list_plans(&args.into()).await,
            Show(args) => self.show_plan(&args.into()).await,
            Archive(args) => self.archive_plan(&args.into()).await,
            Unarchive(args) => self.unarchive_plan(&args.into()).await,
            Delete(args) => self.delete_plan(&args.into()).await,
            Search(args) => self.search_plans(&args.into()).await,
        }
    }

    /// Handle step subcommands
    pub(crate) async fn handle_step_command(&self, command: StepCommands) -> Result<()> {
        use StepCommands::*;
        match command {
            Add(args) => self.add_step(&args.into()).await,
            Insert(args) => self.insert_step(&args.into()).await,
            Update(args) => self.update_step(&args.into()).await,
            Show(args) => self.show_step(&args.into()).await,
            Swap(args) => self.swap_step(&args.into()).await,
        }
    }

    /// Handle plan list command  
    pub async fn list_plans(&self, params: &ListPlans) -> Result<()> {
        let plan_summaries = self
            .planner
            .list_plans_summary(params)
            .await
            .context("Failed to list plans")?;

        let title = if params.archived {
            "Archived Plans"
        } else {
            "Active Plans"
        };

        self.renderer
            .render(format!("# {title}\n\n{plan_summaries}"));

        Ok(())
    }

    /// Handle plan create command
    async fn create_plan(&self, params: &CreatePlan) -> Result<()> {
        let plan = self
            .planner
            .create_plan(params)
            .await
            .context("Failed to create plan")?;

        self.renderer.render(CreateResult::new(plan));

        Ok(())
    }

    /// Handle plan show command
    async fn show_plan(&self, params: &Id) -> Result<()> {
        let plan = self
            .planner
            .get_plan(params)
            .await
            .context("Failed to get plan")?
            .ok_or_else(|| anyhow::anyhow!("Plan with ID {} not found", params.id))?;

        self.renderer.render(&plan);

        Ok(())
    }

    /// Handle plan archive command
    async fn archive_plan(&self, params: &Id) -> Result<()> {
        let plan = self
            .planner
            .archive_plan(params)
            .await
            .with_context(|| format!("Failed to archive plan {}", params.id))?
            .ok_or_else(|| anyhow::anyhow!("Plan with ID {} not found", params.id))?;

        let message = format!(
            "Archived plan '{}' (ID: {}). Use 'beacon plan unarchive {}' to restore.",
            plan.title, params.id, params.id
        );
        self.renderer.render(OperationStatus::success(message));
        Ok(())
    }

    /// Handle plan unarchive command
    async fn unarchive_plan(&self, params: &Id) -> Result<()> {
        let _plan = self
            .planner
            .unarchive_plan(params)
            .await
            .with_context(|| format!("Failed to unarchive plan {}", params.id))?;

        let message = format!("Unarchived plan with ID: {}", params.id);
        self.renderer.render(OperationStatus::success(message));
        Ok(())
    }

    /// Handle plan delete command
    async fn delete_plan(&self, args: &DeletePlan) -> Result<()> {
        let plan = self
            .planner
            .delete_plan(&args)
            .await
            .with_context(|| format!("Failed to delete plan {}", &args.id))?
            .ok_or_else(|| anyhow::anyhow!("Plan with ID {} not found", &args.id))?;

        let message = format!(
            "Permanently deleted plan '{}' (ID: {}). This action cannot be undone.",
            plan.title, plan.id
        );
        self.renderer.render(OperationStatus::success(message));
        Ok(())
    }

    /// Handle plan search command
    async fn search_plans(&self, params: &SearchPlans) -> Result<()> {
        let plan_summaries = self
            .planner
            .search_plans_summary(params)
            .await
            .context("Failed to search plans")?;

        let title = format!(
            "{} plans in directory: {}",
            if params.archived {
                "ARCHIVED"
            } else {
                "ACTIVE"
            },
            params.directory
        );

        self.renderer
            .render(format!("# {title}\n\n{plan_summaries}"));
        Ok(())
    }

    /// Handle step add command
    async fn add_step(&self, params: &StepCreate) -> Result<()> {
        let step = self
            .planner
            .add_step(params)
            .await
            .with_context(|| format!("Failed to add step to plan {}", params.plan_id))?;
        self.renderer.render(CreateResult::new(step));
        Ok(())
    }

    /// Handle step insert command
    async fn insert_step(&self, params: &InsertStep) -> Result<()> {
        let step = self.planner.insert_step(params).await.with_context(|| {
            format!(
                "Failed to insert step into plan {} at position {}",
                params.step.plan_id, params.position
            )
        })?;

        self.renderer.render(CreateResult::new(step));
        Ok(())
    }

    /// Handle step update command
    async fn update_step(&self, params: &UpdateStep) -> Result<()> {
        // Check if we have anything to update
        if params.status.is_none()
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

        let status =
            StepStatus::from_str(&params.status.as_ref().unwrap_or(&"".to_string())).unwrap();

        // Validate result requirement for done status
        if let StepStatus::Done = status {
            if params.result.is_none() {
                return Err(anyhow::anyhow!(
                    "Result description is required when marking a step as done. Use --result to describe what was accomplished."
                ));
            }
        }

        // Build list of changes made for display
        let mut changes = Vec::new();
        if params.status.is_some() {
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

        let updated_step = self
            .planner
            .update_step_validated(params)
            .await
            .with_context(|| format!("Failed to update step {}", params.id))?
            .ok_or_else(|| anyhow::anyhow!("Step with ID {} not found", params.id))?;

        let result = UpdateResult::with_changes(updated_step, changes);
        self.renderer.render(&result);

        Ok(())
    }

    /// Handle step show command
    async fn show_step(&self, params: &Id) -> Result<()> {
        let step = self
            .planner
            .get_step(params)
            .await
            .context("Failed to get step")?
            .ok_or_else(|| anyhow::anyhow!("Step with ID {} not found", params.id))?;

        self.renderer.render(&step);

        Ok(())
    }

    /// Handle step swap command
    async fn swap_step(&self, params: &SwapSteps) -> Result<()> {
        self.planner.swap_steps(params).await.with_context(|| {
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
        self.renderer.render(&status);

        Ok(())
    }
}

// ============================================================================
// CLI Argument Wrapper Implementations
// ============================================================================
//
// These structures implement the CLI side of the parameter wrapper pattern.
// Each wrapper:
// 1. Defines CLI-specific argument parsing with clap derives
// 2. Provides conversion methods to core parameter types
// 3. Isolates clap framework concerns from core domain logic
//
// The into_params() methods perform explicit type conversion, ensuring
// compile-time verification of parameter mapping between CLI and core layers.

/// Create a new plan
///
/// CLI wrapper for CreatePlan that adds clap-specific argument handling
/// including short/long flags, help text generation, and input validation.
#[derive(Parser)]
pub struct CreatePlanArgs {
    /// Title of the plan
    pub title: String,
    /// Optional description providing more context about the plan
    #[arg(
        short,
        long,
        help = "Optional description providing more context about the plan"
    )]
    pub description: Option<String>,
    /// Working directory to associate with this plan
    #[arg(long, help = "Working directory to associate with this plan")]
    pub directory: Option<String>,
}

impl From<CreatePlanArgs> for CreatePlan {
    /// Convert CLI arguments to core parameter structure
    ///
    /// This explicit conversion ensures type safety and makes the boundary
    /// between CLI concerns and core logic clear and verifiable.
    fn from(val: CreatePlanArgs) -> Self {
        CreatePlan {
            title: val.title,
            description: val.description,
            directory: val.directory,
        }
    }
}

/// List all plans
///
/// Display either active plans (default) or archived plans based on the
/// --archived flag. Active plans are those currently being worked on, while
/// archived plans are completed or temporarily inactive plans that have been
/// moved out of the main view.
#[derive(Parser)]
pub struct ListPlansArgs {
    /// Show archived plans instead of active plans
    #[arg(
        long,
        help = "Show archived (completed/inactive) plans instead of active ones"
    )]
    pub archived: bool,
}

impl From<ListPlansArgs> for ListPlans {
    fn from(val: ListPlansArgs) -> Self {
        ListPlans {
            archived: val.archived,
        }
    }
}

/// Show details of a specific plan
///
/// Display comprehensive information about a plan including its title,
/// description, directory, creation/modification timestamps, and all associated
/// steps with their current status and details.
#[derive(Parser)]
pub struct ShowPlanArgs {
    /// ID of the plan to display
    #[arg(help = "Unique identifier of the plan to show details for")]
    pub id: u64,
}

impl From<ShowPlanArgs> for Id {
    fn from(val: ShowPlanArgs) -> Self {
        Id { id: val.id }
    }
}

/// Archive a plan
///
/// Move a plan to the archived state, hiding it from the default plan list.
/// Archived plans are preserved and can be restored later with the unarchive
/// command. Use this for completed projects or plans that are temporarily on
/// hold.
#[derive(Parser)]
pub struct ArchivePlanArgs {
    /// ID of the plan to archive
    #[arg(help = "Unique identifier of the plan to move to archived state")]
    pub id: u64,
}

impl From<ArchivePlanArgs> for Id {
    fn from(val: ArchivePlanArgs) -> Self {
        Id { id: val.id }
    }
}

/// Unarchive a plan
///
/// Restore an archived plan back to the active list, making it visible in the
/// default plan listing. The plan and all its steps are preserved exactly as
/// they were when archived. Use this to resume work on previously archived
/// projects.
#[derive(Parser)]
pub struct UnarchivePlanArgs {
    /// ID of the plan to restore from archive
    #[arg(help = "Unique identifier of the archived plan to restore to active state")]
    pub id: u64,
}

impl From<UnarchivePlanArgs> for Id {
    fn from(val: UnarchivePlanArgs) -> Self {
        Id { id: val.id }
    }
}

/// Delete a plan permanently
#[derive(Parser)]
pub struct DeletePlanArgs {
    /// ID of the plan to delete
    #[arg(help = "Unique identifier of the plan to permanently delete")]
    pub id: u64,
    /// Confirm the deletion (required to prevent accidental deletion)
    #[arg(long)]
    pub confirm: bool,
}

impl From<DeletePlanArgs> for DeletePlan {
    fn from(val: DeletePlanArgs) -> Self {
        DeletePlan {
            id: val.id,
            confirmed: val.confirm,
        }
    }
}

/// Search for plans by directory
///
/// Find all plans associated with a specific directory path. Use --archived to
/// include archived plans in the search results. This is useful for discovering
/// existing plans in a project folder or organizing plans by location.
#[derive(Parser)]
pub struct SearchPlansArgs {
    /// Directory path to search for plans
    #[arg(help = "Directory path to search for plans in")]
    pub directory: String,
    /// Include archived plans in search results
    #[arg(
        long,
        help = "Include archived (completed/inactive) plans in search results"
    )]
    pub archived: bool,
}

impl From<SearchPlansArgs> for SearchPlans {
    fn from(val: SearchPlansArgs) -> Self {
        SearchPlans {
            directory: val.directory,
            archived: val.archived,
        }
    }
}

#[derive(Subcommand)]
pub enum PlanCommands {
    /// Create a new plan
    #[command(alias = "c")]
    Create(CreatePlanArgs),
    /// List all plans
    #[command(aliases = ["l", "ls"])]
    List(ListPlansArgs),
    /// Show details of a specific plan
    #[command(alias = "s")]
    Show(ShowPlanArgs),
    /// Archive a plan
    #[command(alias = "a")]
    Archive(ArchivePlanArgs),
    /// Unarchive a plan
    #[command(alias = "u")]
    Unarchive(UnarchivePlanArgs),
    /// Delete a plan permanently
    #[command(aliases = ["d", "rm"])]
    Delete(DeletePlanArgs),
    /// Search for plans by directory
    #[command(alias = "f")]
    Search(SearchPlansArgs),
}

/// Add a new step to a plan
///
/// Example of wrapper pattern with more complex parameter mapping, showing
/// how CLI-specific features (value_delimiter) can be added without affecting
/// the core parameter structure.
#[derive(Parser)]
pub struct AddStepArgs {
    /// ID of the plan to add the step to
    #[arg(help = "Unique identifier of the plan to add this step to")]
    pub plan_id: u64,
    /// Title of the step
    pub title: String,
    /// Optional detailed description of what needs to be done
    #[arg(
        short,
        long,
        help = "Optional detailed description of what needs to be done"
    )]
    pub description: Option<String>,
    /// Optional acceptance criteria defining when the step is complete
    #[arg(
        short,
        long,
        help = "Optional acceptance criteria defining when the step is complete"
    )]
    pub acceptance_criteria: Option<String>,
    /// References (file paths, URLs) - comma-separated list
    #[arg(
        short,
        long,
        value_delimiter = ',',
        help = "References (file paths, URLs) as comma-separated list"
    )]
    pub references: Vec<String>,
}

impl From<AddStepArgs> for StepCreate {
    /// Convert CLI arguments to core StepCreate
    ///
    /// Note how CLI-specific features (value_delimiter) are handled
    /// transparently by clap, while the core parameter structure remains
    /// simple and focused.
    fn from(val: AddStepArgs) -> Self {
        StepCreate {
            plan_id: val.plan_id,
            title: val.title,
            description: val.description,
            acceptance_criteria: val.acceptance_criteria,
            references: val.references,
        }
    }
}

/// Insert a new step at a specific position in a plan
///
/// This allows inserting a step at any position within the existing step order.
/// Position is 0-indexed (0 = first position). All existing steps at or after
/// this position will be shifted down to make room for the new step.
#[derive(Parser)]
pub struct InsertStepArgs {
    #[arg(help = "Unique identifier of the plan to insert this step into")]
    pub plan_id: u64,
    #[arg(help = "0-based position index where to insert the step (0 = first position)")]
    pub position: u32,
    /// Title of the step
    pub title: String,
    #[arg(
        short,
        long,
        help = "Optional detailed description of what needs to be done"
    )]
    pub description: Option<String>,
    #[arg(
        short,
        long,
        help = "Optional acceptance criteria defining when the step is complete"
    )]
    pub acceptance_criteria: Option<String>,
    #[arg(
        short,
        long,
        value_delimiter = ',',
        help = "References (file paths, URLs) as comma-separated list"
    )]
    pub references: Vec<String>,
}

impl From<InsertStepArgs> for InsertStep {
    fn from(val: InsertStepArgs) -> Self {
        InsertStep {
            step: StepCreate {
                plan_id: val.plan_id,
                title: val.title,
                description: val.description,
                acceptance_criteria: val.acceptance_criteria,
                references: val.references,
            },
            position: val.position,
        }
    }
}

/// Update a step's status or details
///
/// Allows modifying any aspect of an existing step including status, title,
/// description, acceptance criteria, and references. When changing status to
/// 'done', the result field should be provided to document what was
/// accomplished. The result field is required for completion tracking and is
/// ignored for other status changes.
#[derive(Parser)]
pub struct UpdateStepArgs {
    #[arg(help = "Unique identifier of the step to update")]
    pub id: u64,
    #[arg(short, long, help = "New status for the step (todo, inprogress, done)")]
    pub status: Option<StepStatusArg>,
    #[arg(short, long, help = "Updated title for the step")]
    pub title: Option<String>,
    #[arg(
        short,
        long,
        help = "Updated detailed description of what needs to be done"
    )]
    pub description: Option<String>,
    #[arg(
        short,
        long,
        help = "Updated acceptance criteria defining when the step is complete"
    )]
    pub acceptance_criteria: Option<String>,
    #[arg(
        short,
        long,
        value_delimiter = ',',
        help = "Updated references (file paths, URLs) as comma-separated list"
    )]
    pub references: Option<Vec<String>>,
    #[arg(
        long,
        help = "Description of what was accomplished - required when changing status to 'done'"
    )]
    pub result: Option<String>,
}

impl From<UpdateStepArgs> for UpdateStep {
    fn from(val: UpdateStepArgs) -> Self {
        UpdateStep {
            id: val.id,
            status: val.status.map(|s| s.to_string()),
            title: val.title,
            description: val.description,
            acceptance_criteria: val.acceptance_criteria,
            references: val.references,
            result: val.result,
        }
    }
}

/// Show details of a specific step
///
/// Displays comprehensive information about a single step including its status,
/// timestamps, description, acceptance criteria, references, and result (if
/// completed). Use when you need to focus on a single step's details rather
/// than the whole plan.
#[derive(Parser)]
pub struct ShowStepArgs {
    #[arg(help = "Unique identifier of the step to show details for")]
    pub id: u64,
}

impl From<ShowStepArgs> for Id {
    fn from(val: ShowStepArgs) -> Self {
        Id { id: val.id }
    }
}

/// Swap the order of two steps within the same plan
///
/// Reorders steps by swapping the positions of two existing steps. Both steps
/// must belong to the same plan. This operation preserves all step properties
/// and only changes their order in the plan's step sequence. Useful for
/// reorganizing workflow without deleting and recreating steps.
#[derive(Parser)]
pub struct SwapStepsArgs {
    #[arg(help = "Unique identifier of the first step to swap")]
    pub step1_id: u64,
    #[arg(help = "Unique identifier of the second step to swap")]
    pub step2_id: u64,
}

impl From<SwapStepsArgs> for SwapSteps {
    fn from(val: SwapStepsArgs) -> Self {
        SwapSteps {
            step1_id: val.step1_id,
            step2_id: val.step2_id,
        }
    }
}

#[derive(Subcommand)]
pub enum StepCommands {
    /// Add a new step to a plan
    #[command(alias = "a")]
    Add(AddStepArgs),
    /// Insert a new step at a specific position in a plan
    #[command(alias = "i")]
    Insert(InsertStepArgs),
    /// Update a step's status or details
    #[command(alias = "u")]
    Update(UpdateStepArgs),
    /// Show details of a specific step
    #[command(alias = "s")]
    Show(ShowStepArgs),
    /// Swap the order of two steps within the same plan
    #[command(alias = "sw")]
    Swap(SwapStepsArgs),
}

/// Command-line argument representation of step status values
///
/// This enum provides the CLI interface for step status transitions,
/// converting between user-friendly command arguments and internal status
/// strings. Used with the `--status` flag in step update commands.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum StepStatusArg {
    /// Mark step as todo
    Todo,
    /// Mark step as in progress
    InProgress,
    /// Mark step as done
    Done,
}

impl std::fmt::Display for StepStatusArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StepStatusArg::Todo => write!(f, "todo"),
            StepStatusArg::InProgress => write!(f, "inprogress"),
            StepStatusArg::Done => write!(f, "done"),
        }
    }
}
