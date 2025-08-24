//! High-level planner API for managing plans and steps.
//!
//! This module provides the main [`Planner`] interface for interacting with the
//! Beacon task planning system. The planner acts as the central coordinator
//! between the application layers and the database, implementing all business
//! logic for plan and step operations.
//!
//! # Architecture Overview
//!
//! The planner module is organized into several submodules that handle different
//! aspects of the task planning system:
//!
//! ```text
//! ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
//! │    Handlers     │    │   Operations    │    │    Database     │
//! │ (plan_handlers, │───▶│ (plan_ops,      │───▶│   (via db/)     │
//! │  step_handlers) │    │  step_ops)      │    │                 │
//! └─────────────────┘    └─────────────────┘    └─────────────────┘
//!     User Interface      Business Logic         Data Persistence
//! ```
//!
//! ## Submodules
//!
//! - [`builder`]: Factory for creating [`Planner`] instances with configuration
//! - [`plan_handlers`]: High-level plan operations (create, list, show, archive, etc.)
//! - [`step_handlers`]: High-level step operations (add, update, show, swap, etc.)
//! - [`plan_ops`]: Lower-level plan database operations and queries
//! - [`step_ops`]: Lower-level step database operations and queries
//!
//! ## Design Principles
//!
//! 1. **Async First**: All operations are async-compatible for better performance
//! 2. **Error Propagation**: Comprehensive error handling with context
//! 3. **Transaction Safety**: Database operations use proper transaction boundaries
//! 4. **Type Safety**: Strong typing for IDs, statuses, and parameters
//! 5. **Display Integration**: Results formatted via the display system
//!
//! # Usage Examples
//!
//! ## Creating a Planner
//!
//! ```rust
//! use beacon_core::{PlannerBuilder, params::CreatePlan};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create with default database path
//! let planner = PlannerBuilder::new()
//!     .build()
//!     .await?;
//!
//! // Or specify custom database path
//! let planner = PlannerBuilder::new()
//!     .with_database_path("/custom/path/beacon.db")
//!     .build()
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Plan Operations
//!
//! ```rust
//! use beacon_core::{PlannerBuilder, params::{CreatePlan, ListPlans}};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let planner = PlannerBuilder::new().build().await?;
//!
//! // Create a new plan
//! let create_params = CreatePlan {
//!     title: "My Project".to_string(),
//!     description: Some("A sample project".to_string()),
//!     directory: Some("/home/user/project".to_string()),
//! };
//! let plan = planner.create_plan_result(&create_params).await?;
//!
//! // List active plans
//! let active_plans = planner.list_plans_summary(&ListPlans::default()).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Step Operations
//!
//! ```rust
//! use beacon_core::{PlannerBuilder, params::{CreatePlan, StepCreate}};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let planner = PlannerBuilder::new().build().await?;
//!
//! // Create plan first
//! let plan = planner.create_plan_result(&CreatePlan {
//!     title: "Test Plan".to_string(),
//!     description: None,
//!     directory: None,
//! }).await?;
//!
//! // Add a step to the plan
//! let step_params = StepCreate {
//!     plan_id: plan.id,
//!     title: "First step".to_string(),
//!     description: Some("Complete the initial setup".to_string()),
//!     acceptance_criteria: Some("Setup is verified and documented".to_string()),
//!     references: vec!["https://docs.example.com/setup".to_string()],
//! };
//! let step = planner.add_step_to_plan(&step_params).await?;
//! # Ok(())
//! # }
//! ```

use std::path::PathBuf;

// Module declarations
pub mod builder;
pub mod plan_ops;
pub mod step_ops;
pub mod plan_handlers;
pub mod step_handlers;

#[cfg(test)]
mod tests;

// Re-export the main types
pub use builder::PlannerBuilder;

/// Main planner interface for managing plans and steps.
pub struct Planner {
    pub(crate) db_path: PathBuf,
}

impl Planner {
    /// Creates a new planner with the specified database path.
    pub(crate) fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }
}