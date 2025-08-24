//! High-level planner API for managing plans and steps.
//!
//! This module provides the main [`Planner`] interface for interacting with the
//! Beacon task planning system. The planner acts as the central coordinator
//! between the application layers and the database, implementing all business
//! logic for plan and step operations.

use std::path::PathBuf;

// Module declarations
pub mod builder;
pub mod plan_handlers;
pub mod plan_ops;
pub mod step_handlers;
pub mod step_ops;

// Integration tests moved to /tests/planner_integration_tests.rs

// Re-export the main types
pub use builder::PlannerBuilder;

/// Main planner interface for managing plans and steps.
#[derive(Clone)]
pub struct Planner {
    pub(crate) db_path: PathBuf,
}

impl Planner {
    /// Creates a new planner with the specified database path.
    pub(crate) fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }
}
