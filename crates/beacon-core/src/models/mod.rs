//! Data models for plans and steps.
//!
//! This module contains the core domain models that represent plans and steps
//! in the Beacon task planning system. Display implementations for these models
//! are located in [`crate::display::models`] to maintain clean separation of
//! concerns between data structures and presentation logic.
//!
//! # Display Architecture
//!
//! The models follow a dual-display approach:
//!
//! 1. **Model Display**: Display implementations in [`crate::display::models`]
//!    for standalone formatting
//! 2. **Wrapper Display**: Specialized wrappers in [`crate::display`] for
//!    contextual formatting
//!
//! ## Display Features
//!
//! All Display implementations (in [`crate::display::models`]) provide:
//!
//! - **Markdown Output**: All models format as readable markdown
//! - **Rich Information**: Includes metadata, timestamps, and structured
//!   content
//! - **Context Awareness**: Different display behavior when used independently
//!   vs. in lists
//! - **Status Icons**: Visual indicators for step statuses (✓ Done, ➤ In
//!   Progress, ○ Todo)
//!
//! ## Model-Specific Formatting
//!
//! ### Plan Display
//! - Header with ID and title
//! - Metadata (status, directory, timestamps)
//! - Optional description
//! - Nested step list with position numbers
//!
//! ### Step Display
//! - **Independent**: Full step details with sections
//! - **Within Plan**: Compact format with position numbers
//! - **Status Icons**: Consistent visual indicators
//! - **Conditional Sections**: Result shown only for completed steps
//!
//! ### PlanSummary Display
//! - Compact format for lists
//! - Progress indicators (completed/total)
//! - Essential metadata only
//!
//! # Examples
//!
//! ```rust
//! use beacon_core::models::{Plan, Step, StepStatus};
//! use jiff::Timestamp;
//!
//! // Direct plan display
//! let plan = Plan {
//!     id: 1,
//!     title: "My Project".to_string(),
//!     description: Some("Project description".to_string()),
//!     // ... other fields
//! #   status: beacon_core::models::PlanStatus::Active,
//! #   directory: None,
//! #   created_at: Timestamp::now(),
//! #   updated_at: Timestamp::now(),
//! #   steps: vec![],
//! };
//! println!("{}", plan); // Formats with markdown headers and metadata
//!
//! // Direct step display
//! let step = Step {
//!     id: 1,
//!     plan_id: 1,
//!     title: "Complete setup".to_string(),
//!     status: StepStatus::InProgress,
//!     // ... other fields
//! #   description: None,
//! #   acceptance_criteria: None,
//! #   references: vec![],
//! #   result: None,
//! #   order: 0,
//! #   created_at: Timestamp::now(),
//! #   updated_at: Timestamp::now(),
//! };
//! println!("{}", step); // Shows ➤ In Progress status icon
//! ```

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