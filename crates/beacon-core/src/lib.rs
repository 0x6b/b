//! Core library for the Beacon task planning application.
//!
//! This crate provides the core business logic for managing plans and steps,
//! including database operations, data models, and error handling.
//!
//! # Display Architecture
//!
//! The crate implements a Display-based architecture for formatting output:
//!
//! - **Domain Models** ([`models`]): Implement [`std::fmt::Display`] for direct
//!   formatting
//! - **Display Wrappers** ([`display`]): Provide contextual and specialized
//!   formatting
//! - **Terminal Rendering**: Rich markdown output via the CLI's terminal
//!   renderer
//!
//! This separation allows the same data to be formatted differently depending
//! on context (lists vs. individual items, creation results vs. updates, etc.)
//! while maintaining consistency across all output.
//!
//! # Quick Start
//!
//! ```rust
//! use beacon_core::{PlannerBuilder, params::CreatePlan};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a planner instance
//! let planner = PlannerBuilder::new()
//!     .with_database_path("test.db")
//!     .build()
//!     .await?;
//!
//! // Create a new plan using planner methods
//! let create_params = CreatePlan {
//!     title: "My Project".to_string(),
//!     description: Some("A test project".to_string()),
//!     directory: Some("/home/user/project".to_string()),
//! };
//!
//! let plan = planner.create_plan(&create_params).await?;
//! println!("Created plan: {}", plan);
//!
//! // List plans as summaries
//! use beacon_core::params::ListPlans;
//! let plans = planner.list_plans_summary(&ListPlans::default()).await?;
//! for plan in &plans {
//!     println!("Plan: {}", plan.title);
//! }
//! # Ok(())
//! # }
//! ```

pub mod db;
pub mod display;
pub mod error;
pub mod models;
pub mod params;
pub mod planner;

// Re-export commonly used types
pub use db::Database;
pub use display::{
    CreateResult, DeleteResult, OperationStatus, PlanSummaries, Steps, UpdateResult,
};
pub use error::{PlannerError, Result};
pub use models::{
    CompletionFilter, LocalDateTime, Plan, PlanFilter, PlanStatus, PlanSummary, Step, StepStatus,
    UpdateStepRequest,
};
pub use params::{
    CreatePlan, Id, InsertStep, ListPlans, SearchPlans, StepCreate, SwapSteps, UpdateStep,
};
pub use planner::{Planner, PlannerBuilder};
