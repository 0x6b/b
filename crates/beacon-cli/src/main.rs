//! Beacon CLI Application
//!
//! Command-line interface for the beacon task planning tool.
//! Following the m43 pattern for clean CLI implementation.

mod args;
mod cli;
mod mcp;
mod renderer;

use std::env::var;

use Commands::*;
use anyhow::{Context, Result};
use args::{Args, Commands};
use beacon_core::{PlannerBuilder, params::ListPlans};
use clap::Parser;
use cli::Cli;
use log::info;
use mcp::{BeaconMcpServer, run_stdio_server};
use pager::Pager;
use renderer::TerminalRenderer;
use tokio::runtime::Runtime;

fn main() -> Result<()> {
    env_logger::init();

    let Args {
        database_file,
        no_color,
        no_pager,
        command,
    } = Args::parse();

    if !no_pager {
        // Set up the pager before starting async runtime to avoid I/O conflicts
        Pager::with_pager(
            &var("BEACON_PAGER")
                .or_else(|_| var("PAGER"))
                .unwrap_or_else(|_| "less -FRX".to_string()),
        )
        .setup();
    }

    let renderer = TerminalRenderer::new(!no_color);

    Runtime::new()
        .context("Failed to create tokio runtime")?
        .block_on(async move {
            let planner = PlannerBuilder::new()
                .with_database_path(database_file)
                .build()
                .await
                .context("Failed to initialize planner")?;

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
        })
}
