//! Data models for plans and steps.
//!
//! This module contains the core domain models that represent plans and steps
//! in the Beacon task planning system. Display implementations for these models
//! are located in [`crate::display::models`].

pub mod filters;
pub mod plan;
pub mod requests;
pub mod status;
pub mod step;
pub mod summary;

#[cfg(test)]
mod tests;

// Re-export all public types at the models level for backward compatibility
pub use filters::{CompletionFilter, PlanFilter};
pub use plan::Plan;
pub use requests::UpdateStepRequest;
pub use status::{PlanStatus, StepStatus};
pub use step::Step;
pub use summary::PlanSummary;
