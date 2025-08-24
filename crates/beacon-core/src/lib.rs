//! Core library for the Beacon task planning application.
//!
//! This crate provides the core business logic for managing plans and steps,
//! including database operations, data models, and error handling.

pub mod db;
pub mod display;
pub mod error;
pub mod models;
pub mod params;
pub mod planner;

// Re-export commonly used types
pub use db::Database;
pub use display::{
    CreateResult, DeleteResult, LocalDateTime, OperationStatus, PlanSummaries, Steps, UpdateResult,
};
pub use error::{PlannerError, Result};
pub use models::{
    CompletionFilter, Plan, PlanFilter, PlanStatus, PlanSummary, Step, StepStatus,
    UpdateStepRequest,
};
pub use params::{
    CreatePlan, Id, InsertStep, ListPlans, SearchPlans, StepCreate, SwapSteps, UpdateStep,
};
pub use planner::{Planner, PlannerBuilder};
