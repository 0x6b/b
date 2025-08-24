use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::cli::{PlanCommands, StepCommands};

/// Main command-line interface for Beacon task management tool
///
/// Beacon is a hierarchical task planning and management system that helps
/// organize work into structured plans and steps. It provides a command-line
/// interface for creating, managing, and tracking tasks with support for both
/// local CLI operations and MCP (Model Context Protocol) server mode for
/// integration with AI assistants.
#[derive(Parser)]
#[command(version, about, name = "b")]
pub struct Args {
    /// Path to the SQLite database file. Defaults to
    /// $XDG_DATA_HOME/beacon/beacon.db
    #[arg(long, global = true)]
    pub database_file: Option<PathBuf>,

    /// Disable colored output and use plain text
    #[arg(long, global = true)]
    pub no_color: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
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
