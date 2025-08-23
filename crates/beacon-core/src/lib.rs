//! Core library for the Beacon task planning application.
//!
//! This crate provides the core business logic for managing plans and steps,
//! including database operations, data models, and error handling.
//!
//! # Display Architecture
//!
//! The crate implements a Display-based architecture for formatting output:
//!
//! - **Domain Models** ([`models`]): Implement [`std::fmt::Display`] for direct formatting
//! - **Display Wrappers** ([`display`]): Provide contextual and specialized formatting
//! - **Terminal Rendering**: Rich markdown output via the CLI's terminal renderer
//!
//! This separation allows the same data to be formatted differently depending on
//! context (lists vs. individual items, creation results vs. updates, etc.) while
//! maintaining consistency across all output.
//!
//! # Quick Start
//!
//! ```rust
//! use beacon_core::{format_plan_list, models::{PlanSummary, PlanStatus}};
//! use jiff::Timestamp;
//!
//! // Create a sample plan summary
//! let plan = PlanSummary {
//!     id: 1,
//!     title: "My Project".to_string(),
//!     description: Some("A test project".to_string()),
//!     status: PlanStatus::Active,
//!     directory: Some("/home/user/project".to_string()),
//!     created_at: Timestamp::now(),
//!     updated_at: Timestamp::now(),
//!     total_steps: 5,
//!     completed_steps: 2,
//!     pending_steps: 3,
//! };
//! let plans = vec![plan];
//!
//! // Use helper functions for formatted output
//! let output = format_plan_list(&plans, Some("My Plans"));
//! assert!(output.contains("# My Plans"));
//! assert!(output.contains("My Project"));
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
    CreateResult, DeleteResult, OperationStatus, UpdateResult,
    format_plan_list, format_step_list,
};
pub use error::{PlannerError, Result};
pub use models::{
    CompletionFilter, Plan, PlanFilter, PlanStatus, PlanSummary, Step, StepStatus,
    UpdateStepRequest, format_datetime,
};
pub use params::{
    CreatePlan, Id, InsertStep, ListPlans, SearchPlans, StepCreate, SwapSteps, UpdateStep,
};
pub use planner::{Planner, PlannerBuilder};
