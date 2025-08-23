//! Display wrapper types for formatting different contexts.
//!
//! This module provides wrapper types that implement Display for collections
//! and operation results, enabling consistent formatting across different
//! output contexts (lists, operations, etc.).
//!
//! # Architecture: Display Wrapper Pattern
//!
//! The Display architecture follows a wrapper pattern that separates presentation
//! logic from business logic. Instead of implementing Display directly on domain
//! models, we use specialized wrapper types that can format the same data
//! differently depending on context.
//!
//! ```text
//! ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
//! │  Domain Models  │    │ Display Wrapper │    │   Formatted     │
//! │  (Plan, Step)   │───▶│    Types        │───▶│    Output       │
//! │                 │    │                 │    │  (Terminal/MCP) │
//! └─────────────────┘    └─────────────────┘    └─────────────────┘
//! ```
//!
//! ## Benefits
//!
//! 1. **Context-Aware Formatting**: Same data can be formatted differently
//!    (e.g., PlanList vs individual Plan display)
//! 2. **Separation of Concerns**: Business logic in models, presentation in wrappers
//! 3. **Composability**: Wrappers can be nested and combined
//! 4. **Consistency**: All output goes through standardized display logic
//!
//! ## Wrapper Types
//!
//! - [`PlanList`]: Formats collections of plans with optional titles
//! - [`StepList`]: Formats collections of steps with contextual information
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
//! use beacon_core::display::PlanList;
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
//! let list = PlanList::new(&plans);
//! let output = format!("{}", list);
//! assert!(output.contains("My Project"));
//! 
//! // With a title header
//! let titled_list = PlanList::with_title(&plans, "Active Plans");
//! let titled_output = format!("{}", titled_list);
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
//! let update_result = UpdateResult::with_changes(plan, "plan", changes);
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

/// Wrapper type for displaying a collection of plans as a formatted list.
///
/// This provides a consistent display format for plan collections,
/// typically used when listing plans or showing search results.
///
/// The wrapper formats each plan with:
/// - Plan title and ID
/// - Progress indicator (completed/total steps)
/// - Creation timestamp
/// - Optional description and directory
///
/// # Examples
///
/// ```rust
/// use beacon_core::display::PlanList;
/// use beacon_core::models::{PlanSummary, PlanStatus};
/// use jiff::Timestamp;
///
/// let plan = PlanSummary {
///     id: 1,
///     title: "My Project".to_string(),
///     description: Some("A test project".to_string()),
///     status: PlanStatus::Active,
///     directory: Some("/home/user/project".to_string()),
///     created_at: Timestamp::now(),
///     updated_at: Timestamp::now(),
///     total_steps: 5,
///     completed_steps: 2,
///     pending_steps: 3,
/// };
///
/// let plans = vec![plan];
/// let list = PlanList::with_title(&plans, "Active Projects");
/// println!("{}", list);
/// ```
pub struct PlanList<'a> {
    plans: &'a [PlanSummary],
    title: Option<&'a str>,
}

impl<'a> PlanList<'a> {
    /// Create a new PlanList wrapper.
    pub fn new(plans: &'a [PlanSummary]) -> Self {
        Self {
            plans,
            title: None,
        }
    }

    /// Create a PlanList with a title header.
    pub fn with_title(plans: &'a [PlanSummary], title: &'a str) -> Self {
        Self {
            plans,
            title: Some(title),
        }
    }
}

impl<'a> fmt::Display for PlanList<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(title) = self.title {
            writeln!(f, "# {title}")?;
            writeln!(f)?;
        }

        if self.plans.is_empty() {
            writeln!(f, "No plans found.")?;
            return Ok(());
        }

        for plan in self.plans {
            write!(f, "{plan}")?;
        }

        Ok(())
    }
}

/// Wrapper type for displaying a collection of steps as a formatted list.
///
/// This provides a consistent display format for step collections,
/// typically used when showing steps from a plan or filtered step results.
///
/// The wrapper can format steps in different contexts:
/// - Within a plan (showing step position and status icons)
/// - As a filtered list (optionally showing plan ID for each step)
/// - With custom titles for grouped displays
///
/// # Examples
///
/// ```rust
/// use beacon_core::display::StepList;
/// use beacon_core::models::{Step, StepStatus};
/// use jiff::Timestamp;
///
/// let step = Step {
///     id: 1,
///     plan_id: 42,
///     title: "Complete setup".to_string(),
///     description: Some("Set up the development environment".to_string()),
///     acceptance_criteria: Some("All dependencies installed".to_string()),
///     references: vec!["https://docs.example.com".to_string()],
///     status: StepStatus::InProgress,
///     result: None,
///     order: 0,
///     created_at: Timestamp::now(),
///     updated_at: Timestamp::now(),
/// };
///
/// let steps = vec![step];
/// let list = StepList::with_plan_info(&steps, Some("Steps from Multiple Plans"));
/// println!("{}", list);
/// ```
pub struct StepList<'a> {
    steps: &'a [Step],
    title: Option<&'a str>,
    show_plan_info: bool,
}

impl<'a> StepList<'a> {
    /// Create a new StepList wrapper.
    pub fn new(steps: &'a [Step]) -> Self {
        Self {
            steps,
            title: None,
            show_plan_info: false,
        }
    }

    /// Create a StepList with a title header.
    pub fn with_title(steps: &'a [Step], title: &'a str) -> Self {
        Self {
            steps,
            title: Some(title),
            show_plan_info: false,
        }
    }

    /// Create a StepList that includes plan information for each step.
    pub fn with_plan_info(steps: &'a [Step], title: Option<&'a str>) -> Self {
        Self {
            steps,
            title,
            show_plan_info: true,
        }
    }
}

impl<'a> fmt::Display for StepList<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(title) = self.title {
            writeln!(f, "# {title}")?;
            writeln!(f)?;
        }

        if self.steps.is_empty() {
            writeln!(f, "No steps found.")?;
            return Ok(());
        }

        for step in self.steps {
            if self.show_plan_info {
                writeln!(f, "**Plan ID: {}**", step.plan_id)?;
                writeln!(f)?;
            }
            write!(f, "{step}")?;
        }

        Ok(())
    }
}

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
/// let result = UpdateResult::with_changes(updated_step, "step", changes);
/// println!("{}", result);
/// ```
pub struct UpdateResult<T> {
    pub resource: T,
    pub resource_type: &'static str,
    pub changes: Vec<String>,
}

impl<T> UpdateResult<T> {
    /// Create a new UpdateResult wrapper.
    pub fn new(resource: T, resource_type: &'static str) -> Self {
        Self {
            resource,
            resource_type,
            changes: Vec::new(),
        }
    }

    /// Create an UpdateResult with a list of changes made.
    pub fn with_changes(resource: T, resource_type: &'static str, changes: Vec<String>) -> Self {
        Self {
            resource,
            resource_type,
            changes,
        }
    }
}

impl fmt::Display for UpdateResult<Plan> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Updated {} with ID: {}", self.resource_type, self.resource.id)?;
        
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
        writeln!(f, "Updated {} with ID: {}", self.resource_type, self.resource.id)?;
        
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
pub struct DeleteResult {
    pub resource_id: u64,
    pub resource_type: &'static str,
    pub resource_title: Option<String>,
}

impl DeleteResult {
    /// Create a new DeleteResult wrapper.
    pub fn new(resource_id: u64, resource_type: &'static str) -> Self {
        Self {
            resource_id,
            resource_type,
            resource_title: None,
        }
    }

    /// Create a DeleteResult with the resource title for better context.
    pub fn with_title(resource_id: u64, resource_type: &'static str, title: String) -> Self {
        Self {
            resource_id,
            resource_type,
            resource_title: Some(title),
        }
    }
}

impl fmt::Display for DeleteResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.resource_title {
            Some(title) => writeln!(f, "Deleted {} '{}' (ID: {})", self.resource_type, title, self.resource_id),
            None => writeln!(f, "Deleted {} with ID: {}", self.resource_type, self.resource_id),
        }
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
    fn test_plan_list_display() {
        let plans = vec![create_test_plan_summary()];
        let list = PlanList::new(&plans);
        let output = format!("{}", list);
        assert!(output.contains("Test Plan"));
        assert!(output.contains("ID: 1"));
    }

    #[test]
    fn test_step_list_display() {
        let steps = vec![create_test_step()];
        let list = StepList::new(&steps);
        let output = format!("{}", list);
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