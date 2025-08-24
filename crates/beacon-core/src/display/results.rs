//! Result wrapper types for displaying operation outcomes.
//!
//! This module provides wrapper types that format the results of create, update,
//! and delete operations with consistent messaging and resource display.

use std::fmt;

use crate::models::{Plan, Step};

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
/// use beacon_core::{
///     display::CreateResult,
///     models::{Plan, PlanStatus},
/// };
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
/// use beacon_core::{
///     display::UpdateResult,
///     models::{Step, StepStatus},
/// };
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
        Self { resource, changes }
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
        writeln!(
            f,
            "Deleted plan '{}' (ID: {})",
            self.resource.title, self.resource.id
        )
    }
}

impl fmt::Display for DeleteResult<Step> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Deleted step '{}' (ID: {})",
            self.resource.title, self.resource.id
        )
    }
}