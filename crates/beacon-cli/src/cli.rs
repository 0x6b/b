//! Command-line interface definitions using clap
//!
//! This module defines the complete CLI structure using clap's derive API,
//! following the m43 pattern for clean command definition.

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

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

#[derive(Subcommand)]
pub enum PlanCommands {
    /// Create a new plan
    Create {
        /// Title of the plan
        title: String,
        /// Optional detailed description of the plan
        #[arg(short, long)]
        description: Option<String>,
        /// Working directory for the plan (always stored as absolute path;
        /// relative paths will be converted to absolute using current working
        /// directory)
        #[arg(long)]
        directory: Option<String>,
    },
    /// List all plans
    List {
        /// Show archived plans instead of active ones
        #[arg(long)]
        archived: bool,
        /// Output format
        #[arg(short, long, default_value = "text")]
        format: OutputFormat,
    },
    /// Show details of a specific plan
    Show {
        /// Plan ID to show
        id: u64,
        /// Output format
        #[arg(short, long, default_value = "text")]
        format: OutputFormat,
    },
    /// Archive a plan
    Archive {
        /// Plan ID to archive
        id: u64,
    },
    /// Unarchive a plan
    Unarchive {
        /// Plan ID to unarchive
        id: u64,
    },
    /// Search for plans by directory
    Search {
        /// Directory path to search for plans (searches for plans with
        /// directories starting with this path)
        directory: String,
        /// Include archived plans in search results
        #[arg(long)]
        archived: bool,
        /// Output format
        #[arg(short, long, default_value = "text")]
        format: OutputFormat,
    },
}

#[derive(Subcommand)]
pub enum StepCommands {
    /// Add a new step to a plan
    Add {
        /// Plan ID to add step to
        plan_id: u64,
        /// Title of the step
        title: String,
        /// Optional detailed description of the step
        #[arg(short, long)]
        description: Option<String>,
        /// Acceptance criteria for the step
        #[arg(short, long)]
        acceptance_criteria: Option<String>,
        /// References (comma-separated URLs or file paths)
        #[arg(short, long, value_delimiter = ',')]
        references: Vec<String>,
    },
    /// Insert a new step at a specific position in a plan
    Insert {
        /// Plan ID to insert step into
        plan_id: u64,
        /// Position to insert the step (0-indexed)
        position: u32,
        /// Title of the step
        title: String,
        /// Optional detailed description of the step
        #[arg(short, long)]
        description: Option<String>,
        /// Acceptance criteria for the step
        #[arg(short, long)]
        acceptance_criteria: Option<String>,
        /// References (comma-separated URLs or file paths)
        #[arg(short, long, value_delimiter = ',')]
        references: Vec<String>,
    },
    /// Update a step's status or details
    Update {
        /// Step ID to update
        id: u64,
        /// New status (todo, in-progress, or done)
        #[arg(short, long)]
        status: Option<StepStatusArg>,
        /// Update title
        #[arg(short, long)]
        title: Option<String>,
        /// Update description
        #[arg(short, long)]
        description: Option<String>,
        /// Update acceptance criteria
        #[arg(short, long)]
        acceptance_criteria: Option<String>,
        /// Update references (comma-separated URLs or file paths)
        #[arg(short, long, value_delimiter = ',')]
        references: Option<Vec<String>>,
        /// Result description (required when setting status to done)
        #[arg(
            long,
            help = "Description of what was accomplished (required when setting status to done)"
        )]
        result: Option<String>,
    },
    /// Show details of a specific step
    Show {
        /// Step ID to show
        id: u64,
        /// Output format
        #[arg(short, long, default_value = "text")]
        format: OutputFormat,
    },
    /// Swap the order of two steps within the same plan
    Swap {
        /// First step ID
        step1: u64,
        /// Second step ID
        step2: u64,
    },
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
