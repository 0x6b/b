//! Display formatting functions and result types.
//!
//! This module provides helper functions for formatting collections and wrapper
//! types for operation results, enabling consistent formatting across different
//! output contexts (lists, operations, etc.).
//!
//! # Architecture: Display Functions and Wrappers
//!
//! The Display architecture combines direct Display implementations on domain
//! models with formatting functions for collections and wrapper types for
//! operation results. This approach provides both idiomatic Rust patterns and
//! context-specific formatting.
//!
//! ```text
//! ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
//! │  Domain Models  │    │ Format Functions│    │   Formatted     │
//! │  (Plan, Step)   │───▶│ & Result Types  │───▶│    Output       │
//! │                 │    │                 │    │  (Terminal/MCP) │
//! └─────────────────┘    └─────────────────┘    └─────────────────┘
//! ```
//!
//! ## Benefits
//!
//! 1. **Idiomatic Rust**: Newtype wrappers provide Display implementations for
//!    collections
//! 2. **Separation of Concerns**: Business logic in models, presentation in
//!    wrappers
//! 3. **Type Safety**: Newtype wrappers ensure proper formatting without runtime
//!    errors
//! 4. **Consistency**: All output goes through standardized display logic
//!
//! ## Module Organization
//!
//! - [`collections`]: Collection wrapper types (PlanSummaries, Steps)
//! - [`results`]: Operation result types (CreateResult, UpdateResult, DeleteResult)
//! - [`status`]: Status and confirmation messages (OperationStatus)
//! - [`datetime`]: Date/time formatting utilities
//! - [`models`]: Display implementations for domain models
//!
//! ## Usage Examples
//!
//! ### Operation Results
//!
//! ```rust
//! use beacon_core::{
//!     display::{CreateResult, UpdateResult},
//!     models::{Plan, PlanStatus},
//! };
//! use jiff::Timestamp;
//!
//! // Create a sample plan for testing
//! let plan = Plan {
//!     id: 1,
//!     title: "New Project".to_string(),
//!     description: Some("A newly created project".to_string()),
//!     status: PlanStatus::Active,
//!     directory: Some("/home/user/new-project".to_string()),
//!     created_at: Timestamp::now(),
//!     updated_at: Timestamp::now(),
//!     steps: vec![],
//! };
//!
//! // Format creation results
//! let result = CreateResult::new(plan.clone());
//! let output = format!("{}", result);
//! assert!(output.contains("Created plan with ID: 1"));
//!
//! // Format updates with change tracking
//! let changes = vec!["Updated title".to_string(), "Added description".to_string()];
//! let update_result = UpdateResult::with_changes(plan, changes);
//! let update_output = format!("{}", update_result);
//! assert!(update_output.contains("Changes made:"));
//! ```
//!
//! ### Status Messages
//!
//! ```rust
//! use beacon_core::display::OperationStatus;
//!
//! // Success messages
//! let success = OperationStatus::success("Operation completed successfully".to_string());
//! println!("{}", success);
//!
//! // Error messages
//! let error = OperationStatus::failure("Operation failed".to_string());
//! println!("{}", error);
//! ```
//!
//! ## Design Principles
//!
//! 1. **Immutable Wrappers**: Wrappers hold references, not owned data
//! 2. **Builder Pattern**: Optional configurations via chained methods
//! 3. **Markdown Output**: All formatters produce markdown for rich terminal
//!    display
//! 4. **Consistent Structure**: Headers, metadata, content follow standard
//!    patterns

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


