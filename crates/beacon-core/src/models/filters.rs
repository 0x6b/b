//! Filter types for querying plans and steps.

use jiff::Timestamp;

use super::PlanStatus;

/// Filter options for querying plans.
#[derive(Debug, Clone)]
pub struct PlanFilter {
    /// Filter by plan title (case-insensitive partial match)
    pub title_contains: Option<String>,
    /// Filter by directory path (exact match or prefix match)
    pub directory: Option<String>,
    /// Filter by creation date range
    pub created_after: Option<Timestamp>,
    pub created_before: Option<Timestamp>,
    /// Filter by completion status
    pub completion_status: Option<CompletionFilter>,
    /// Filter by plan status (active/archived)
    /// If None, defaults to showing only active plans
    pub status: Option<PlanStatus>,
    /// Show all plans regardless of status
    pub include_archived: bool,
}

impl PlanFilter {
    /// Create a new filter builder with default values.
    pub const fn new() -> Self {
        Self {
            title_contains: None,
            directory: None,
            created_after: None,
            created_before: None,
            completion_status: None,
            status: None,
            include_archived: false,
        }
    }

    /// Set directory filter.
    pub fn directory(mut self, directory: String) -> Self {
        self.directory = Some(directory);
        self
    }

    /// Set archived status and corresponding plan status.
    pub fn archived(mut self, archived: bool) -> Self {
        self.include_archived = archived;
        self.status = Some(if archived {
            PlanStatus::Archived
        } else {
            PlanStatus::Active
        });
        self
    }

    /// Create a directory-specific plan filter for search operations.
    pub fn for_directory(directory: String, archived: bool) -> Self {
        Self::new().directory(directory).archived(archived)
    }
}

impl Default for PlanFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Completion status filter options.
#[derive(Debug, Clone, PartialEq)]
pub enum CompletionFilter {
    /// Plans with all steps completed
    Complete,
    /// Plans with at least one incomplete step
    Incomplete,
    /// Plans with no steps
    Empty,
}

impl From<&crate::params::ListPlans> for PlanFilter {
    fn from(params: &crate::params::ListPlans) -> Self {
        Self::new().archived(params.archived)
    }
}
