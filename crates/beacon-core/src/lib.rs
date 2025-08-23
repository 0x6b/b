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
//! use beacon_core::{
//!     format_plan_list,
//!     models::{PlanStatus, PlanSummary},
//! };
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
pub mod handlers;
pub mod models;
pub mod operations;
pub mod params;
pub mod planner;

// Re-export commonly used types
pub use db::Database;
pub use display::{
    format_plan_list, format_step_list, CreateResult, DeleteResult, OperationStatus, UpdateResult,
};
pub use error::{PlannerError, Result};
pub use handlers::{
    handle_add_step, handle_archive_plan, handle_claim_step, handle_create_plan,
    handle_delete_plan, handle_insert_step, handle_list_plans, handle_search_plans,
    handle_show_plan, handle_show_step, handle_swap_steps, handle_unarchive_plan,
    handle_update_step,
};
pub use models::{
    format_datetime, CompletionFilter, Plan, PlanFilter, PlanStatus, PlanSummary, Step, StepStatus,
    UpdateStepRequest,
};
pub use operations::{
    create_directory_filter, create_plan_filter, create_update_request, validate_step_update,
};
pub use params::{
    CreatePlan, Id, InsertStep, ListPlans, SearchPlans, StepCreate, SwapSteps, UpdateStep,
};
pub use planner::{Planner, PlannerBuilder};
