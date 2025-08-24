//! Beacon CLI Application
//!
//! Command-line interface for the beacon task planning tool.
//! Following the m43 pattern for clean CLI implementation.

mod args;
mod cli;
mod mcp;
mod renderer;

use anyhow::{Context, Result};
use args::{Args, Commands};
use beacon_core::{params::ListPlans, PlannerBuilder};
use clap::Parser;
use cli::Cli;
use log::info;
use mcp::{run_stdio_server, BeaconMcpServer};
use renderer::TerminalRenderer;
use Commands::*;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let Args { database_file, no_color, command } = Args::parse();

    let planner = PlannerBuilder::new()
        .with_database_path(database_file)
        .build()
        .await
        .context("Failed to initialize planner")?;

    let renderer = TerminalRenderer::new(!no_color);

    info!("Beacon started");

    match command {
        Some(Plan { command }) => {
            Cli::new(planner, renderer)
                .handle_plan_command(command)
                .await
        }
        Some(Step { command }) => {
            Cli::new(planner, renderer)
                .handle_step_command(command)
                .await
        }
        Some(Serve) => {
            info!("Starting Beacon MCP server");
            run_stdio_server(BeaconMcpServer::new(planner))
                .await
                .context("MCP server failed")
        }
        None => {
            Cli::new(planner, renderer)
                .list_plans(&ListPlans { archived: false })
                .await
        }
    }
}
