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
//! 1. **Framework Isolation**: Core parameter types remain free of clap-specific
//!    attributes and derives, enabling reuse across different interfaces.
//!
//! 2. **Validation Separation**: CLI-specific validation (argument parsing, help
//!    generation) is handled by clap derives, while business logic validation
//!    remains in the core domain.
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
//!     #[arg(short, long)]  // CLI-specific attributes
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

use clap::{Args, Parser, Subcommand, ValueEnum};
use beacon_core::params::*;

#[derive(Parser)]
#[command(version, about, name = "beacon")]
pub struct Cli {
    /// Path to the SQLite database file. Defaults to
    /// $XDG_DATA_HOME/beacon/beacon.db
    #[arg(long, global = true)]
    pub database_file: Option<PathBuf>,

    /// Disable colored output and use plain text
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Output format
    #[arg(short, long, default_value = "text", global = true)]
    pub format: OutputFormat,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Manage plans
    Plan {
        #[command(subcommand)]
        command: PlanCommands,
    },
    /// Manage steps within plans
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
/// CLI wrapper for CreatePlanParams that adds clap-specific argument handling
/// including short/long flags, help text generation, and input validation.
#[derive(Args)]
pub struct CreatePlanArgs {
    /// Title of the plan
    pub title: String,
    /// Optional description providing more context about the plan
    #[arg(short, long)]
    pub description: Option<String>,
    /// Working directory to associate with this plan
    #[arg(long)]
    pub directory: Option<String>,
}

impl From<CreatePlanArgs> for CreatePlanParams {
    /// Convert CLI arguments to core parameter structure
    ///
    /// This explicit conversion ensures type safety and makes the boundary
    /// between CLI concerns and core logic clear and verifiable.
    fn from(val: CreatePlanArgs) -> Self {
        CreatePlanParams {
            title: val.title,
            description: val.description,
            directory: val.directory,
        }
    }
}

/// List all plans  
#[derive(Args)]
pub struct ListPlansArgs {
    #[arg(long)]
    pub archived: bool,
}

impl From<ListPlansArgs> for ListPlansParams {
    fn from(val: ListPlansArgs) -> Self {
        ListPlansParams {
            archived: val.archived,
        }
    }
}

/// Show details of a specific plan
#[derive(Args)]
pub struct ShowPlanArgs {
    pub id: u64,
}

impl From<ShowPlanArgs> for IdParams {
    fn from(val: ShowPlanArgs) -> Self {
        IdParams { id: val.id }
    }
}

/// Archive a plan
#[derive(Args)]
pub struct ArchivePlanArgs {
    pub id: u64,
}

impl From<ArchivePlanArgs> for IdParams {
    fn from(val: ArchivePlanArgs) -> Self {
        IdParams { id: val.id }
    }
}

/// Unarchive a plan  
#[derive(Args)]
pub struct UnarchivePlanArgs {
    pub id: u64,
}

impl From<UnarchivePlanArgs> for IdParams {
    fn from(val: UnarchivePlanArgs) -> Self {
        IdParams { id: val.id }
    }
}

/// Search for plans by directory
#[derive(Args)]
pub struct SearchPlansArgs {
    pub directory: String,
    #[arg(long)]
    pub archived: bool,
}

impl From<SearchPlansArgs> for SearchPlansParams {
    fn from(val: SearchPlansArgs) -> Self {
        SearchPlansParams {
            directory: val.directory,
            archived: val.archived,
        }
    }
}

#[derive(Subcommand)]
pub enum PlanCommands {
    /// Create a new plan
    Create(CreatePlanArgs),
    /// List all plans
    List(ListPlansArgs),
    /// Show details of a specific plan
    Show(ShowPlanArgs),
    /// Archive a plan
    Archive(ArchivePlanArgs),
    /// Unarchive a plan
    Unarchive(UnarchivePlanArgs),
    /// Search for plans by directory
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
    pub plan_id: u64,
    /// Title of the step
    pub title: String,
    /// Optional detailed description of what needs to be done
    #[arg(short, long)]
    pub description: Option<String>,
    /// Optional acceptance criteria defining when the step is complete
    #[arg(short, long)]
    pub acceptance_criteria: Option<String>,
    /// References (file paths, URLs) - comma-separated list
    #[arg(short, long, value_delimiter = ',')]
    pub references: Vec<String>,
}

impl From<AddStepArgs> for StepCreateParams {
    /// Convert CLI arguments to core StepCreateParams
    ///
    /// Note how CLI-specific features (value_delimiter) are handled transparently
    /// by clap, while the core parameter structure remains simple and focused.
    fn from(val: AddStepArgs) -> Self {
        StepCreateParams {
            plan_id: val.plan_id,
            title: val.title,
            description: val.description,
            acceptance_criteria: val.acceptance_criteria,
            references: val.references,
        }
    }
}

/// Insert a new step at a specific position in a plan
#[derive(Args)]
pub struct InsertStepArgs {
    pub plan_id: u64,
    pub position: u32,
    pub title: String,
    #[arg(short, long)]
    pub description: Option<String>,
    #[arg(short, long)]
    pub acceptance_criteria: Option<String>,
    #[arg(short, long, value_delimiter = ',')]
    pub references: Vec<String>,
}

impl From<InsertStepArgs> for InsertStepParams {
    fn from(val: InsertStepArgs) -> Self {
        InsertStepParams {
            step: StepCreateParams {
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
#[derive(Args)]
pub struct UpdateStepArgs {
    pub id: u64,
    #[arg(short, long)]
    pub status: Option<StepStatusArg>,
    #[arg(short, long)]
    pub title: Option<String>,
    #[arg(short, long)]
    pub description: Option<String>,
    #[arg(short, long)]
    pub acceptance_criteria: Option<String>,
    #[arg(short, long, value_delimiter = ',')]
    pub references: Option<Vec<String>>,
    #[arg(long)]
    pub result: Option<String>,
}

impl From<UpdateStepArgs> for UpdateStepParams {
    fn from(val: UpdateStepArgs) -> Self {
        UpdateStepParams {
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
#[derive(Args)]
pub struct ShowStepArgs {
    pub id: u64,
}

impl From<ShowStepArgs> for IdParams {
    fn from(val: ShowStepArgs) -> Self {
        IdParams { id: val.id }
    }
}

/// Swap the order of two steps within the same plan
#[derive(Args)]
pub struct SwapStepsArgs {
    pub step1_id: u64,
    pub step2_id: u64,
}

impl From<SwapStepsArgs> for SwapStepsParams {
    fn from(val: SwapStepsArgs) -> Self {
        SwapStepsParams {
            step1_id: val.step1_id,
            step2_id: val.step2_id,
        }
    }
}

#[derive(Subcommand)]
pub enum StepCommands {
    /// Add a new step to a plan
    Add(AddStepArgs),
    /// Insert a new step at a specific position in a plan
    Insert(InsertStepArgs),
    /// Update a step's status or details
    Update(UpdateStepArgs),
    /// Show details of a specific step
    Show(ShowStepArgs),
    /// Swap the order of two steps within the same plan
    Swap(SwapStepsArgs),
}

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

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable text output
    Text,
    /// JSON output
    Json,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Text => write!(f, "text"),
            OutputFormat::Json => write!(f, "json"),
        }
    }
}
