//! Filter types for querying plans and steps.

use jiff::Timestamp;

use super::PlanStatus;

/// Filter options for querying plans.
#[derive(Debug, Clone, Default)]
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
    /// Create a directory-specific plan filter for search operations.
    ///
    /// This associated function creates a plan filter that combines directory
    /// filtering with archived status filtering for search operations. It
    /// provides the same functionality as the `create_directory_filter`
    /// function but as an idiomatic associated constructor.
    ///
    /// # Arguments
    ///
    /// * `directory` - Directory path to filter by
    /// * `archived` - Whether to include archived plans
    ///
    /// # Returns
    ///
    /// A PlanFilter configured for directory and status filtering
    ///
    /// # Examples
    ///
    /// ```rust
    /// use beacon_core::models::PlanFilter;
    ///
    /// // Filter for active plans in a specific directory
    /// let filter = PlanFilter::for_directory("/path/to/project".to_string(), false);
    /// assert_eq!(filter.directory, Some("/path/to/project".to_string()));
    /// assert!(!filter.include_archived);
    ///
    /// // Filter for archived plans in a specific directory
    /// let filter = PlanFilter::for_directory("/path/to/archived".to_string(), true);
    /// assert_eq!(filter.directory, Some("/path/to/archived".to_string()));
    /// assert!(filter.include_archived);
    /// ```
    pub fn for_directory(directory: String, archived: bool) -> Self {
        Self {
            status: Some(if archived {
                PlanStatus::Archived
            } else {
                PlanStatus::Active
            }),
            directory: Some(directory),
            include_archived: archived,
            ..Default::default()
        }
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
    /// Convert ListPlans parameters to a PlanFilter for plan queries.
    ///
    /// This implementation replaces the `create_plan_filter` function with an
    /// idiomatic Rust trait conversion. The conversion creates appropriate
    /// filters based on the archived flag:
    ///
    /// - `archived: false` → Filter for active plans only
    /// - `archived: true` → Filter for archived plans only
    ///
    /// # Examples
    ///
    /// ```rust
    /// use beacon_core::{models::PlanFilter, params::ListPlans};
    ///
    /// // Filter for active plans
    /// let params = ListPlans { archived: false };
    /// let filter: PlanFilter = (&params).into();
    /// assert_eq!(filter.status, Some(beacon_core::models::PlanStatus::Active));
    /// assert!(!filter.include_archived);
    ///
    /// // Filter for archived plans
    /// let params = ListPlans { archived: true };
    /// let filter: PlanFilter = (&params).into();
    /// assert_eq!(
    ///     filter.status,
    ///     Some(beacon_core::models::PlanStatus::Archived)
    /// );
    /// assert!(filter.include_archived);
    /// ```
    fn from(params: &crate::params::ListPlans) -> Self {
        if params.archived {
            Self {
                status: Some(PlanStatus::Archived),
                include_archived: true,
                ..Default::default()
            }
        } else {
            Self {
                status: Some(PlanStatus::Active),
                include_archived: false,
                ..Default::default()
            }
        }
    }
}
