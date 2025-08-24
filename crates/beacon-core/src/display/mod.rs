//! Display formatting functions and result types.
//!
//! This module provides helper functions for formatting collections and wrapper
//! types for operation results, enabling consistent formatting across different
//! output contexts.

pub mod collections;
pub mod datetime;
pub mod models;
pub mod results;
pub mod status;

// Re-export commonly used types for convenience
pub use collections::{PlanSummaries, Steps};
pub use datetime::LocalDateTime;
pub use results::{CreateResult, DeleteResult, UpdateResult};
pub use status::OperationStatus;
