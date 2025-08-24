//! Command-line interface definitions using clap
//!
//! This module defines the complete CLI structure using clap's derive API,
//! implementing the parameter wrapper pattern for clean separation between
//! CLI framework concerns and core domain logic.
//!
//! ## Parameter Wrapper Pattern Implementation
//!
//! This module demonstrates the CLI side of the parameter wrapper pattern:
//!
//! ```text
//! User Input → CLI Args (clap) → Core Params → Business Logic
//! ```
//!
//! ### Design Benefits
//!
//! 1. **Framework Isolation**: Core parameter types remain free of
//!    clap-specific attributes and derives, enabling reuse across different
//!    interfaces.
//!
//! 2. **Validation Separation**: CLI-specific validation (argument parsing,
//!    help generation) is handled by clap derives, while business logic
//!    validation remains in the core domain.
//!
//! 3. **Interface Evolution**: CLI can evolve its argument structure (aliases,
//!    help text, validation) without affecting core parameter definitions.
//!
//! ### Implementation Pattern
//!
//! Each command follows this structure:
//!
//! ```rust
//! // CLI-specific argument structure with clap derives
//! #[derive(Args)]
//! pub struct OperationArgs {
//!     pub field: String,
//!     #[arg(short, long)] // CLI-specific attributes
//!     pub optional_field: Option<String>,
//! }
//!
//! // Conversion method to core parameters
//! impl OperationArgs {
//!     pub fn into_params(self) -> CoreOperationParams {
//!         CoreOperationParams {
//!             field: self.field,
//!             optional_field: self.optional_field,
//!         }
//!     }
//! }
//! ```
//!
//! This pattern ensures that:
//! - CLI concerns (help text, argument validation) stay in CLI layer
//! - Core types remain interface-agnostic
//! - Type conversion is explicit and verifiable at compile time

use std::path::PathBuf;

use beacon_core::params::*;
use clap::{Args, Parser, Subcommand, ValueEnum};

/// Main command-line interface for Beacon task management tool
///
/// Beacon is a hierarchical task planning and management system that helps
/// organize work into structured plans and steps. It provides a command-line
/// interface for creating, managing, and tracking tasks with support for both
/// local CLI operations and MCP (Model Context Protocol) server mode for
/// integration with AI assistants.
#[derive(Parser)]
#[command(version, about, name = "b")]
pub struct Cli {
    /// Path to the SQLite database file. Defaults to
    /// $XDG_DATA_HOME/beacon/beacon.db
    #[arg(long, global = true)]
    pub database_file: Option<PathBuf>,

    /// Disable colored output and use plain text
    #[arg(long, global = true)]
    pub no_color: bool,

    #[command(subcommand)]
    pub command: Commands,
}

/// Available commands for the Beacon CLI
///
/// The CLI is organized into three main command categories:
/// - `plan`: Operations for managing task plans (create, list, archive, etc.)
/// - `step`: Operations for managing individual steps within plans
/// - `serve`: Start the MCP server for AI assistant integration
#[derive(Subcommand)]
pub enum Commands {
    /// Manage plans
    #[command(alias = "p")]
    Plan {
        #[command(subcommand)]
        command: PlanCommands,
    },
    /// Manage steps within plans
    #[command(alias = "s")]
    Step {
        #[command(subcommand)]
        command: StepCommands,
    },
    /// Start the MCP server
    Serve,
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
#[derive(Args)]
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
#[derive(Args)]
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
#[derive(Args)]
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
#[derive(Args)]
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
#[derive(Args)]
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
#[derive(Args)]
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
#[derive(Args)]
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
#[derive(Args)]
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
#[derive(Args)]
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
#[derive(Args)]
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
#[derive(Args)]
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
#[derive(Args)]
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
