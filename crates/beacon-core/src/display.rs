//! Display formatting functions and result types.
//!
//! This module provides helper functions for formatting collections and wrapper types
//! for operation results, enabling consistent formatting across different
//! output contexts (lists, operations, etc.).
//!
//! # Architecture: Display Functions and Wrappers
//!
//! The Display architecture combines direct Display implementations on domain models
//! with formatting functions for collections and wrapper types for operation results.
//! This approach provides both idiomatic Rust patterns and context-specific formatting.
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
//! 1. **Idiomatic Collections**: Helper functions format slices without wrapper overhead
//! 2. **Separation of Concerns**: Business logic in models, presentation in functions
//! 3. **Flexibility**: Functions can handle different contexts (titles, empty collections)
//! 4. **Consistency**: All output goes through standardized display logic
//!
//! ## Types and Functions
//!
//! - [`format_plan_list`]: Formats collections of plans with optional titles
//! - [`format_step_list`]: Formats collections of steps with optional titles
//! - [`CreateResult`]: Formats creation operation results
//! - [`UpdateResult`]: Formats update operation results with change tracking
//! - [`DeleteResult`]: Formats deletion confirmations
//! - [`OperationStatus`]: Formats success/failure messages
//!
//! ## Usage Examples
//!
//! ### Basic List Formatting
//!
//! ```rust
//! use beacon_core::display::format_plan_list;
//! use beacon_core::models::{PlanSummary, PlanStatus};
//! use jiff::Timestamp;
//! 
//! // Create a sample plan
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
//! // Format a collection of plans  
//! let output = format_plan_list(&plans, None);
//! assert!(output.contains("My Project"));
//! 
//! // With a title header
//! let titled_output = format_plan_list(&plans, Some("Active Plans"));
//! assert!(titled_output.contains("# Active Plans"));
//! ```
//!
//! ### Operation Results
//!
//! ```rust
//! use beacon_core::display::{CreateResult, UpdateResult};
//! use beacon_core::models::{Plan, PlanStatus};
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
//! 3. **Markdown Output**: All formatters produce markdown for rich terminal display
//! 4. **Consistent Structure**: Headers, metadata, content follow standard patterns

use std::fmt;

use crate::models::{Plan, PlanSummary, Step};



/// Wrapper type for displaying the result of create operations.
///
/// This provides consistent formatting for creation results,
/// including success messages and the created resource information.
///
/// The wrapper formats creation results with:
/// - Success message with resource type and ID
/// - Full details of the created resource
/// - Consistent markdown structure
///
/// # Examples
///
/// ```rust
/// use beacon_core::display::CreateResult;
/// use beacon_core::models::{Plan, PlanStatus};
/// use jiff::Timestamp;
///
/// let plan = Plan {
///     id: 1,
///     title: "New Project".to_string(),
///     description: Some("A newly created project".to_string()),
///     status: PlanStatus::Active,
///     directory: Some("/home/user/new-project".to_string()),
///     created_at: Timestamp::now(),
///     updated_at: Timestamp::now(),
///     steps: vec![],
/// };
///
/// let result = CreateResult::new(plan);
/// println!("{}", result);
/// ```
pub struct CreateResult<T> {
    pub resource: T,
}

impl<T> CreateResult<T> {
    /// Create a new CreateResult wrapper.
    pub fn new(resource: T) -> Self {
        Self { resource }
    }
}

impl fmt::Display for CreateResult<Plan> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Created plan with ID: {}", self.resource.id)?;
        writeln!(f)?;
        write!(f, "{}", self.resource)
    }
}

impl fmt::Display for CreateResult<Step> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Created step with ID: {}", self.resource.id)?;
        writeln!(f)?;
        write!(f, "{}", self.resource)
    }
}

/// Wrapper type for displaying the result of update operations.
///
/// This provides consistent formatting for update results,
/// including success messages and the updated resource information.
///
/// The wrapper can track and display specific changes made during the update,
/// providing users with clear feedback about what was modified.
///
/// # Examples
///
/// ```rust
/// use beacon_core::display::UpdateResult;
/// use beacon_core::models::{Step, StepStatus};
/// use jiff::Timestamp;
///
/// let updated_step = Step {
///     id: 1,
///     plan_id: 42,
///     title: "Updated step".to_string(),
///     description: Some("Updated description".to_string()),
///     acceptance_criteria: None,
///     references: vec![],
///     status: StepStatus::Done,
///     result: Some("Task completed successfully".to_string()),
///     order: 0,
///     created_at: Timestamp::now(),
///     updated_at: Timestamp::now(),
/// };
///
/// let changes = vec![
///     "Updated title".to_string(),
///     "Changed status to Done".to_string(),
///     "Added result description".to_string(),
/// ];
///
/// let result = UpdateResult::with_changes(updated_step, changes);
/// println!("{}", result);
/// ```
pub struct UpdateResult<T> {
    pub resource: T,
    pub changes: Vec<String>,
}

impl<T> UpdateResult<T> {
    /// Create a new UpdateResult wrapper.
    pub fn new(resource: T) -> Self {
        Self {
            resource,
            changes: Vec::new(),
        }
    }

    /// Create an UpdateResult with a list of changes made.
    pub fn with_changes(resource: T, changes: Vec<String>) -> Self {
        Self {
            resource,
            changes,
        }
    }
}

impl fmt::Display for UpdateResult<Plan> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Updated plan with ID: {}", self.resource.id)?;
        
        if !self.changes.is_empty() {
            writeln!(f)?;
            writeln!(f, "Changes made:")?;
            for change in &self.changes {
                writeln!(f, "- {change}")?;
            }
        }
        
        writeln!(f)?;
        write!(f, "{}", self.resource)
    }
}

impl fmt::Display for UpdateResult<Step> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Updated step with ID: {}", self.resource.id)?;
        
        if !self.changes.is_empty() {
            writeln!(f)?;
            writeln!(f, "Changes made:")?;
            for change in &self.changes {
                writeln!(f, "- {change}")?;
            }
        }
        
        writeln!(f)?;
        write!(f, "{}", self.resource)
    }
}

/// Wrapper type for displaying the result of delete operations.
///
/// This provides consistent formatting for deletion results,
/// including confirmation messages and resource identification.
pub struct DeleteResult<T> {
    pub resource: T,
}

impl<T> DeleteResult<T> {
    /// Create a new DeleteResult wrapper.
    pub fn new(resource: T) -> Self {
        Self { resource }
    }
}

impl fmt::Display for DeleteResult<Plan> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Deleted plan '{}' (ID: {})", self.resource.title, self.resource.id)
    }
}

impl fmt::Display for DeleteResult<Step> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Deleted step '{}' (ID: {})", self.resource.title, self.resource.id)
    }
}

/// Wrapper type for displaying operation confirmation messages.
///
/// This provides consistent formatting for operations that require
/// user confirmation or status updates.
pub struct OperationStatus {
    pub message: String,
    pub success: bool,
}

impl OperationStatus {
    /// Create a new success status.
    pub fn success(message: String) -> Self {
        Self {
            message,
            success: true,
        }
    }

    /// Create a new failure status.
    pub fn failure(message: String) -> Self {
        Self {
            message,
            success: false,
        }
    }
}

impl fmt::Display for OperationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prefix = if self.success { "Success:" } else { "Error:" };
        writeln!(f, "{} {}", prefix, self.message)
    }
}

// ============================================================================
// Direct collection formatting functions
// ============================================================================

/// Format a collection of plans with an optional title.
///
/// This provides idiomatic Rust formatting for collections of plans
/// without requiring wrapper types. Handles empty collections gracefully.
pub fn format_plan_list(plans: &[PlanSummary], title: Option<&str>) -> String {
    let mut result = String::new();
    
    if let Some(title) = title {
        result.push_str(&format!("# {title}\n\n"));
    }
    
    if plans.is_empty() {
        result.push_str("No plans found.\n");
        return result;
    }
    
    for plan in plans {
        result.push_str(&format!("{plan}"));
    }
    
    result
}

/// Format a collection of steps with an optional title.
///
/// This provides idiomatic Rust formatting for collections of steps
/// without requiring wrapper types. Handles empty collections gracefully.
pub fn format_step_list(steps: &[Step], title: Option<&str>) -> String {
    let mut result = String::new();
    
    if let Some(title) = title {
        result.push_str(&format!("# {title}\n\n"));
    }
    
    if steps.is_empty() {
        result.push_str("No steps found.\n");
        return result;
    }
    
    for step in steps {
        result.push_str(&format!("{step}"));
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{PlanStatus, StepStatus};
    use jiff::Timestamp;

    fn create_test_plan_summary() -> PlanSummary {
        PlanSummary {
            id: 1,
            title: "Test Plan".to_string(),
            description: Some("A test plan".to_string()),
            status: PlanStatus::Active,
            directory: Some("/test".to_string()),
            created_at: Timestamp::from_second(1640995200).unwrap(), // 2022-01-01 00:00:00 UTC
            updated_at: Timestamp::from_second(1640995200).unwrap(),
            total_steps: 3,
            completed_steps: 1,
            pending_steps: 2,
        }
    }

    fn create_test_step() -> Step {
        Step {
            id: 1,
            plan_id: 1,
            title: "Test Step".to_string(),
            description: Some("A test step".to_string()),
            acceptance_criteria: Some("Should work".to_string()),
            references: vec!["http://example.com".to_string()],
            status: StepStatus::Todo,
            result: None,
            order: 0,
            created_at: Timestamp::from_second(1640995200).unwrap(),
            updated_at: Timestamp::from_second(1640995200).unwrap(),
        }
    }

    #[test]
    fn test_format_plan_list() {
        let plans = vec![create_test_plan_summary()];
        let output = format_plan_list(&plans, None);
        assert!(output.contains("Test Plan"));
        assert!(output.contains("ID: 1"));
    }

    #[test]
    fn test_format_step_list() {
        let steps = vec![create_test_step()];
        let output = format_step_list(&steps, None);
        assert!(output.contains("Test Step"));
        assert!(output.contains("○ Todo"));
    }

    #[test]
    fn test_operation_status_display() {
        let success = OperationStatus::success("Operation completed".to_string());
        assert!(format!("{}", success).contains("Success:"));

        let failure = OperationStatus::failure("Operation failed".to_string());
        assert!(format!("{}", failure).contains("Error:"));
    }
}