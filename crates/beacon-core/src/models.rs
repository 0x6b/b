//! Data models for plans and steps.
//!
//! This module contains the core domain models that represent plans and steps
//! in the Beacon task planning system. Each model implements Display for
//! direct formatting while supporting more sophisticated display via wrapper
//! types in the [`crate::display`] module.
//!
//! # Display Architecture
//!
//! The models follow a dual-display approach:
//!
//! 1. **Direct Display**: Models implement [`std::fmt::Display`] for standalone
//!    formatting
//! 2. **Wrapper Display**: Specialized wrappers in [`crate::display`] for
//!    contextual formatting
//!
//! ## Direct Display Features
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

use std::{fmt, str::FromStr};

use jiff::{tz::TimeZone, Timestamp};
use serde::{Deserialize, Serialize};

/// Type-safe enumeration of plan statuses.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum PlanStatus {
    /// Plan is active and visible
    #[default]
    Active,

    /// Plan is archived and hidden from normal views
    Archived,
}

impl FromStr for PlanStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "active" => Ok(PlanStatus::Active),
            "archived" => Ok(PlanStatus::Archived),
            _ => Err(format!("Invalid plan status: {s}")),
        }
    }
}

impl fmt::Display for PlanStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl PlanStatus {
    /// Convert to database string representation (for backwards compatibility)
    pub fn as_str(&self) -> &'static str {
        match self {
            PlanStatus::Active => "active",
            PlanStatus::Archived => "archived",
        }
    }
}

/// Represents a complete plan with metadata and steps.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Plan {
    /// Unique identifier for the plan
    pub id: u64,

    /// Title of the plan
    pub title: String,

    /// Detailed multi-line description of the plan
    pub description: Option<String>,

    /// Status of the plan (active or archived)
    #[serde(default)]
    pub status: PlanStatus,

    /// Working directory for the plan (defaults to CWD when created)
    pub directory: Option<String>,

    /// Timestamp when the plan was created (UTC)
    pub created_at: Timestamp,

    /// Timestamp when the plan was last modified (UTC)
    pub updated_at: Timestamp,

    /// Associated steps (lazy-loaded by default)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub steps: Vec<Step>,
}

/// Represents an individual step within a plan.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Step {
    /// Unique identifier for the step
    pub id: u64,

    /// ID of the parent plan
    pub plan_id: u64,

    /// Brief title/summary of the step
    pub title: String,

    /// Detailed multi-line description of the step
    pub description: Option<String>,

    /// Clear completion criteria for the step
    pub acceptance_criteria: Option<String>,

    /// References to relevant resources (URLs, file paths)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub references: Vec<String>,

    /// Current status of the step
    pub status: StepStatus,

    /// Description of what was accomplished (required when status = Done)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,

    /// Order of the step within the plan (0-indexed)
    pub order: u32,

    /// Timestamp when the step was created (UTC)
    pub created_at: Timestamp,

    /// Timestamp when the step was last updated (UTC)
    pub updated_at: Timestamp,
}

/// Type-safe enumeration of step statuses.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StepStatus {
    /// Step is pending completion
    Todo,

    /// Step is being worked on
    InProgress,

    /// Step has been completed
    Done,
}

impl FromStr for StepStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "todo" => Ok(StepStatus::Todo),
            "inprogress" | "in_progress" => Ok(StepStatus::InProgress),
            "done" => Ok(StepStatus::Done),
            _ => Err(format!("Invalid step status: {s}")),
        }
    }
}

impl fmt::Display for StepStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl StepStatus {
    /// Convert to database string representation (for backwards compatibility)
    pub fn as_str(&self) -> &'static str {
        match self {
            StepStatus::Todo => "todo",
            StepStatus::InProgress => "inprogress",
            StepStatus::Done => "done",
        }
    }

    /// Get status with consistent icon formatting for display.
    ///
    /// Returns a formatted string that includes both an icon and the status
    /// name. This method ensures consistent visual representation across
    /// all display contexts.
    ///
    /// # Icons Used
    /// - `✓ Done` - Checkmark for completed steps
    /// - `➤ In Progress` - Arrow for active steps
    /// - `○ Todo` - Circle for pending steps
    ///
    /// # Examples
    ///
    /// ```rust
    /// use beacon_core::models::StepStatus;
    ///
    /// assert_eq!(StepStatus::Done.with_icon(), "✓ Done");
    /// assert_eq!(StepStatus::InProgress.with_icon(), "➤ In Progress");
    /// assert_eq!(StepStatus::Todo.with_icon(), "○ Todo");
    /// ```
    pub fn with_icon(&self) -> &'static str {
        match self {
            StepStatus::Done => "✓ Done",
            StepStatus::InProgress => "➤ In Progress",
            StepStatus::Todo => "○ Todo",
        }
    }
}

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
    /// This associated function creates a plan filter that combines directory filtering
    /// with archived status filtering for search operations. It provides the same
    /// functionality as the `create_directory_filter` function but as an idiomatic
    /// associated constructor.
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

/// Summary information about a plan with step statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanSummary {
    /// Plan ID
    pub id: u64,
    /// Title of the plan
    pub title: String,
    /// Detailed multi-line description of the plan
    pub description: Option<String>,
    /// Plan status
    pub status: PlanStatus,
    /// Working directory for the plan
    pub directory: Option<String>,
    /// Creation timestamp
    pub created_at: Timestamp,
    /// Last update timestamp
    pub updated_at: Timestamp,
    /// Total number of steps
    pub total_steps: u32,
    /// Number of completed steps
    pub completed_steps: u32,
    /// Number of pending steps
    pub pending_steps: u32,
}

/// Parameters for updating a step to reduce function argument count
#[derive(Debug, Default)]
pub struct UpdateStepRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub acceptance_criteria: Option<String>,
    pub references: Option<Vec<String>>,
    pub status: Option<StepStatus>,
    pub result: Option<String>,
}

impl UpdateStepRequest {
    /// Create an UpdateStepRequest from individual validated parameters.
    ///
    /// This constructor method creates an UpdateStepRequest from pre-validated
    /// components, typically used when validation has already been performed
    /// elsewhere in the system.
    ///
    /// # Arguments
    ///
    /// * `title` - Optional new title for the step
    /// * `description` - Optional new description for the step
    /// * `acceptance_criteria` - Optional new acceptance criteria for the step
    /// * `references` - Optional new references list for the step
    /// * `status` - Optional validated StepStatus (already parsed and validated)
    /// * `result` - Optional result description for the step
    ///
    /// # Returns
    ///
    /// A new UpdateStepRequest with all provided parameters set
    ///
    /// # Examples
    ///
    /// ```rust
    /// use beacon_core::models::{UpdateStepRequest, StepStatus};
    ///
    /// let request = UpdateStepRequest::new(
    ///     Some("Updated title".to_string()),
    ///     None,
    ///     None,
    ///     None,
    ///     Some(StepStatus::Done),
    ///     Some("Task completed successfully".to_string()),
    /// );
    ///
    /// assert_eq!(request.title, Some("Updated title".to_string()));
    /// assert_eq!(request.status, Some(StepStatus::Done));
    /// assert_eq!(request.result, Some("Task completed successfully".to_string()));
    /// ```
    pub fn new(
        title: Option<String>,
        description: Option<String>,
        acceptance_criteria: Option<String>,
        references: Option<Vec<String>>,
        status: Option<StepStatus>,
        result: Option<String>,
    ) -> Self {
        Self {
            title,
            description,
            acceptance_criteria,
            references,
            status,
            result,
        }
    }
}

impl TryFrom<crate::params::UpdateStep> for UpdateStepRequest {
    type Error = crate::PlannerError;

    /// Convert an UpdateStep parameter into a validated UpdateStepRequest.
    ///
    /// This trait implementation replaces the `create_update_request` function
    /// with an idiomatic Rust conversion. It performs validation of the status
    /// field and ensures result requirements are met for 'done' status.
    ///
    /// # Arguments
    ///
    /// * `params` - UpdateStep parameters from the params module
    ///
    /// # Returns
    ///
    /// A Result containing the validated UpdateStepRequest, or a PlannerError
    /// if validation fails
    ///
    /// # Errors
    ///
    /// * `PlannerError::InvalidInput` - When status string is invalid
    /// * `PlannerError::InvalidInput` - When result is missing for 'done' status
    ///
    /// # Examples
    ///
    /// ```rust
    /// use beacon_core::{params::UpdateStep, models::UpdateStepRequest};
    ///
    /// // Valid conversion with status change
    /// let mut params = UpdateStep::default();
    /// params.id = 1;
    /// params.status = Some("done".to_string());
    /// params.result = Some("Completed successfully".to_string());
    /// params.title = Some("New title".to_string());
    ///
    /// let request: UpdateStepRequest = params.try_into()?;
    /// assert_eq!(request.title, Some("New title".to_string()));
    /// # use beacon_core::Result;
    /// # Result::<()>::Ok(())
    /// ```
    fn try_from(params: crate::params::UpdateStep) -> Result<Self, Self::Error> {
        // Use the existing validation method from UpdateStep
        let (validated_status, validated_result) = params.validate()?;

        Ok(Self {
            title: params.title,
            description: params.description,
            acceptance_criteria: params.acceptance_criteria,
            references: params.references,
            status: validated_status,
            result: validated_result,
        })
    }
}

impl PlanSummary {
    /// Create a PlanSummary from a Plan and step counts
    pub fn from_plan(plan: Plan, total_steps: u32, completed_steps: u32) -> Self {
        Self {
            id: plan.id,
            title: plan.title,
            description: plan.description,
            status: plan.status,
            directory: plan.directory,
            created_at: plan.created_at,
            updated_at: plan.updated_at,
            total_steps,
            completed_steps,
            pending_steps: total_steps - completed_steps,
        }
    }
}

impl From<&Plan> for PlanSummary {
    fn from(plan: &Plan) -> Self {
        let total_steps = plan.steps.len() as u32;
        let completed_steps = plan
            .steps
            .iter()
            .filter(|step| step.status == StepStatus::Done)
            .count() as u32;
        let pending_steps = total_steps - completed_steps;

        Self {
            id: plan.id,
            title: plan.title.clone(),
            description: plan.description.clone(),
            status: plan.status,
            directory: plan.directory.clone(),
            created_at: plan.created_at,
            updated_at: plan.updated_at,
            total_steps,
            completed_steps,
            pending_steps,
        }
    }
}

impl fmt::Display for Plan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "# {}. {}", self.id, self.title)?;
        writeln!(f)?;

        // Metadata section
        writeln!(f, "- Status: {}", self.status.as_str())?;
        if let Some(dir) = &self.directory {
            writeln!(f, "- Directory: {dir}")?;
        }
        writeln!(f, "- Created: {}", LocalDateTime(&self.created_at))?;
        writeln!(f, "- Updated: {}", LocalDateTime(&self.updated_at))?;

        // Description as a paragraph
        if let Some(desc) = &self.description {
            writeln!(f)?;
            writeln!(f, "{desc}")?;
        }

        if !self.steps.is_empty() {
            writeln!(f, "\n## Steps")?;
            writeln!(f)?;
            for step in &self.steps {
                write!(f, "{}", step)?;
            }
        } else {
            writeln!(f, "\nNo steps in this plan.")?;
        }

        Ok(())
    }
}

impl Step {
    /// Format the step using the clean, compact display format.
    ///
    /// This uses the same format whether the step is displayed standalone
    /// or within a plan context.
    fn fmt_step(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "### {}. {} ({})",
            self.id,
            self.title,
            self.status.with_icon()
        )?;
        writeln!(f)?;

        if let Some(desc) = &self.description {
            writeln!(f, "{desc}")?;
            writeln!(f)?;
        }

        if let Some(criteria) = &self.acceptance_criteria {
            writeln!(f, "#### Acceptance")?;
            writeln!(f)?;
            writeln!(f, "{criteria}")?;
            writeln!(f)?;
        }

        // Show result only for completed steps
        if self.status == StepStatus::Done {
            if let Some(result) = &self.result {
                writeln!(f, "#### Result")?;
                writeln!(f)?;
                writeln!(f, "{result}")?;
                writeln!(f)?;
            }
        }

        if !self.references.is_empty() {
            writeln!(f, "#### References")?;
            writeln!(f)?;
            for reference in &self.references {
                writeln!(f, "- {reference}")?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

impl fmt::Display for Step {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_step(f)
    }
}

impl fmt::Display for PlanSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let progress = if self.total_steps > 0 {
            format!(" ({}/{})", self.completed_steps, self.total_steps)
        } else {
            String::new()
        };

        writeln!(f, "## {} (ID: {}){progress}", self.title, self.id)?;
        writeln!(f)?;

        if let Some(desc) = &self.description {
            writeln!(f, "- **Description**: {desc}")?;
        }

        if let Some(dir) = &self.directory {
            writeln!(f, "- **Directory**: {dir}")?;
        }

        writeln!(f, "- **Created**: {}", LocalDateTime(&self.created_at))?;
        writeln!(f)?; // Add blank line after each plan

        Ok(())
    }
}


/// A wrapper around `Timestamp` that provides system timezone formatting via the `Display` trait.
///
/// This struct encapsulates a `Timestamp` reference and implements `Display` to format it
/// in a consistent, human-readable format using the system timezone. It provides an ergonomic
/// and type-safe approach to timestamp formatting in display contexts.
///
/// # Format
///
/// The display format follows the pattern: `YYYY-MM-DD HH:MM:SS TZ`
/// - Year, month, and day are zero-padded
/// - Time is in 24-hour format with zero-padded components
/// - Timezone abbreviation is included (e.g., UTC, EST, JST)
///
/// # Examples
///
/// ```rust
/// use beacon_core::models::LocalDateTime;
/// use jiff::Timestamp;
///
/// let timestamp = Timestamp::from_second(1640995200).unwrap(); // 2022-01-01 00:00:00 UTC
/// let local_dt = LocalDateTime::new(&timestamp);
/// 
/// // Display automatically formats using system timezone
/// println!("Created: {}", local_dt);
/// // Output (example): "Created: 2022-01-01 09:00:00 JST"
///
/// // Can be used in format strings and templates
/// let message = format!("Plan updated at {}", LocalDateTime::new(&timestamp));
/// ```
///
/// # Design Rationale
///
/// This wrapper provides several advantages over direct function calls:
/// - **Type Safety**: Encapsulates formatting logic in a dedicated type
/// - **Ergonomics**: Integrates seamlessly with `Display` trait usage
/// - **Consistency**: Ensures uniform timestamp formatting across the application
/// - **Future-proofing**: Allows format changes without affecting call sites
///
/// # Performance
///
/// The wrapper is zero-cost at runtime - it only holds a reference to the timestamp
/// and performs formatting only when `Display::fmt` is called.
pub struct LocalDateTime<'a>(&'a Timestamp);

impl<'a> LocalDateTime<'a> {
    /// Create a new `LocalDateTime` wrapper around a timestamp reference.
    ///
    /// # Arguments
    ///
    /// * `timestamp` - Reference to the timestamp to wrap for display formatting
    ///
    /// # Returns
    ///
    /// A new `LocalDateTime` instance that will format the timestamp using
    /// the system timezone when displayed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use beacon_core::models::LocalDateTime;
    /// use jiff::Timestamp;
    ///
    /// let now = Timestamp::now();
    /// let local_dt = LocalDateTime::new(&now);
    /// println!("Current time: {}", local_dt);
    /// ```
    pub fn new(timestamp: &'a Timestamp) -> Self {
        Self(timestamp)
    }
}

impl<'a> fmt::Display for LocalDateTime<'a> {
    /// Format the wrapped timestamp using system timezone in YYYY-MM-DD HH:MM:SS TZ format.
    ///
    /// This implementation converts the UTC timestamp to the system timezone and formats it
    /// in a consistent, human-readable format.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter to write the timestamp string to
    ///
    /// # Returns
    ///
    /// `fmt::Result` indicating success or failure of the formatting operation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use beacon_core::models::LocalDateTime;
    /// use jiff::Timestamp;
    ///
    /// let timestamp = Timestamp::from_second(1640995200).unwrap();
    /// let local_dt = LocalDateTime::new(&timestamp);
    /// 
    /// // Formats with system timezone
    /// println!("{}", local_dt); // e.g., "2022-01-01 09:00:00 JST"
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.0.to_zoned(TimeZone::system()).strftime("%Y-%m-%d %H:%M:%S %Z")
        )
    }
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
    /// use beacon_core::{params::ListPlans, models::PlanFilter};
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
    /// assert_eq!(filter.status, Some(beacon_core::models::PlanStatus::Archived));
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

#[cfg(test)]
mod tests {
    use jiff::Timestamp;

    use super::*;

    fn create_test_step(status: StepStatus) -> Step {
        Step {
            id: 123,
            plan_id: 456,
            title: "Test Step Title".to_string(),
            description: Some("This is a test step description".to_string()),
            acceptance_criteria: Some("Should pass all tests".to_string()),
            references: vec!["https://example.com".to_string(), "file.txt".to_string()],
            status,
            result: if status == StepStatus::Done {
                Some("Successfully completed the test".to_string())
            } else {
                None
            },
            order: 2,
            created_at: Timestamp::from_second(1640995200).unwrap(), // 2022-01-01 00:00:00 UTC
            updated_at: Timestamp::from_second(1641081600).unwrap(), // 2022-01-02 00:00:00 UTC
        }
    }

    fn create_test_plan() -> Plan {
        Plan {
            id: 789,
            title: "Test Plan Title".to_string(),
            description: Some("This is a test plan".to_string()),
            status: PlanStatus::Active,
            directory: Some("/test/path".to_string()),
            created_at: Timestamp::from_second(1640995200).unwrap(),
            updated_at: Timestamp::from_second(1641081600).unwrap(),
            steps: vec![
                create_test_step(StepStatus::Done),
                create_test_step(StepStatus::InProgress),
                create_test_step(StepStatus::Todo),
            ],
        }
    }

    fn create_test_plan_summary() -> PlanSummary {
        PlanSummary {
            id: 789,
            title: "Test Plan Summary".to_string(),
            description: Some("Summary description".to_string()),
            status: PlanStatus::Active,
            directory: Some("/test/summary".to_string()),
            created_at: Timestamp::from_second(1640995200).unwrap(),
            updated_at: Timestamp::from_second(1641081600).unwrap(),
            total_steps: 5,
            completed_steps: 2,
            pending_steps: 3,
        }
    }

    #[test]
    fn test_step_status_with_icon() {
        assert_eq!(StepStatus::Done.with_icon(), "✓ Done");
        assert_eq!(StepStatus::InProgress.with_icon(), "➤ In Progress");
        assert_eq!(StepStatus::Todo.with_icon(), "○ Todo");
    }

    #[test]
    fn test_step_display_independently_todo() {
        let step = create_test_step(StepStatus::Todo);
        let output = format!("{}", step);

        // Should contain step header with ID and status
        assert!(output.contains("### 123. Test Step Title (○ Todo)"));

        // Should contain description and acceptance criteria
        assert!(output.contains("This is a test step description"));
        assert!(output.contains("#### Acceptance"));
        assert!(output.contains("Should pass all tests"));

        // Should contain references
        assert!(output.contains("#### References"));
        assert!(output.contains("- https://example.com"));
        assert!(output.contains("- file.txt"));

        // Should NOT contain result section for todo steps
        assert!(!output.contains("#### Result"));
    }

    #[test]
    fn test_step_display_independently_in_progress() {
        let step = create_test_step(StepStatus::InProgress);
        let output = format!("{}", step);

        assert!(output.contains("### 123. Test Step Title (➤ In Progress)"));
        assert!(!output.contains("#### Result"));
    }

    #[test]
    fn test_step_display_independently_done() {
        let step = create_test_step(StepStatus::Done);
        let output = format!("{}", step);

        assert!(output.contains("### 123. Test Step Title (✓ Done)"));
        assert!(output.contains("#### Result"));
        assert!(output.contains("Successfully completed the test"));
    }

    #[test]
    fn test_step_display_within_plan_context() {
        let step = create_test_step(StepStatus::InProgress);
        let output = format!("{}", step);

        // Should use consistent formatting with step ID
        assert!(output.contains("### 123. Test Step Title (➤ In Progress)"));
        assert!(output.contains("#### Acceptance"));
        assert!(output.contains("#### References"));
    }

    #[test]
    fn test_plan_display_with_steps() {
        let plan = create_test_plan();
        let output = format!("{}", plan);

        // Should contain plan header
        assert!(output.contains("# 789. Test Plan Title"));

        // Should contain metadata
        assert!(output.contains("- Status: active"));
        assert!(output.contains("- Directory: /test/path"));
        assert!(output.contains("- Created: 2022-01-01"));
        assert!(output.contains("- Updated: 2022-01-02"));

        // Should contain description
        assert!(output.contains("This is a test plan"));

        // Should contain steps section
        assert!(output.contains("## Steps"));

        // Should contain step status icons in plan context
        assert!(output.contains("✓ Done"));
        assert!(output.contains("➤ In Progress"));
        assert!(output.contains("○ Todo"));
    }

    #[test]
    fn test_plan_display_empty_steps() {
        let mut plan = create_test_plan();
        plan.steps.clear();
        let output = format!("{}", plan);

        assert!(output.contains("No steps in this plan."));
        assert!(!output.contains("## Steps"));
    }

    #[test]
    fn test_plan_summary_display_with_progress() {
        let summary = create_test_plan_summary();
        let output = format!("{}", summary);

        // Should contain title with progress
        assert!(output.contains("## Test Plan Summary (ID: 789) (2/5)"));

        // Should contain metadata
        assert!(output.contains("- **Description**: Summary description"));
        assert!(output.contains("- **Directory**: /test/summary"));
        assert!(output.contains("- **Created**: 2022-01-01"));

        // Should have blank line at end
        assert!(output.ends_with("\n\n"));
    }

    #[test]
    fn test_plan_summary_display_no_steps() {
        let mut summary = create_test_plan_summary();
        summary.total_steps = 0;
        summary.completed_steps = 0;
        summary.pending_steps = 0;
        let output = format!("{}", summary);

        // Should not show progress when no steps
        assert!(output.contains("## Test Plan Summary (ID: 789)"));
        assert!(!output.contains("(0/0)"));
    }

    #[test]
    fn test_plan_summary_display_minimal_info() {
        let mut summary = create_test_plan_summary();
        summary.description = None;
        summary.directory = None;
        let output = format!("{}", summary);

        // Should still contain basic info
        assert!(output.contains("## Test Plan Summary (ID: 789) (2/5)"));
        assert!(output.contains("- **Created**: 2022-01-01"));

        // Should not contain optional fields
        assert!(!output.contains("- **Description**:"));
        assert!(!output.contains("- **Directory**:"));
    }

    #[test]
    fn test_step_status_display_consistency() {
        // Test that status icons are consistent across all display contexts
        let todo_step = create_test_step(StepStatus::Todo);
        let in_progress_step = create_test_step(StepStatus::InProgress);
        let done_step = create_test_step(StepStatus::Done);

        // Independent step display
        let todo_output = format!("{}", todo_step);
        let in_progress_output = format!("{}", in_progress_step);
        let done_output = format!("{}", done_step);

        assert!(todo_output.contains("○ Todo"));
        assert!(in_progress_output.contains("➤ In Progress"));
        assert!(done_output.contains("✓ Done"));

        // Plan context display
        let todo_pos_output = format!("{}", todo_step);
        let in_progress_pos_output = format!("{}", in_progress_step);
        let done_pos_output = format!("{}", done_step);

        assert!(todo_pos_output.contains("○ Todo"));
        assert!(in_progress_pos_output.contains("➤ In Progress"));
        assert!(done_pos_output.contains("✓ Done"));
    }

    #[test]
    fn test_plan_summary_from_plan_trait() {
        let plan = create_test_plan();
        let summary = PlanSummary::from(&plan);

        // Verify basic plan information is copied correctly
        assert_eq!(summary.id, plan.id);
        assert_eq!(summary.title, plan.title);
        assert_eq!(summary.description, plan.description);
        assert_eq!(summary.status, plan.status);
        assert_eq!(summary.directory, plan.directory);
        assert_eq!(summary.created_at, plan.created_at);
        assert_eq!(summary.updated_at, plan.updated_at);

        // Verify step counts are calculated correctly
        // The test plan has 3 steps: Done, InProgress, Todo
        assert_eq!(summary.total_steps, 3);
        assert_eq!(summary.completed_steps, 1); // Only the Done step
        assert_eq!(summary.pending_steps, 2); // InProgress + Todo steps
    }

    #[test]
    fn test_plan_summary_from_plan_trait_empty_steps() {
        let mut plan = create_test_plan();
        plan.steps.clear();
        let summary = PlanSummary::from(&plan);

        // Verify step counts for empty plan
        assert_eq!(summary.total_steps, 0);
        assert_eq!(summary.completed_steps, 0);
        assert_eq!(summary.pending_steps, 0);
    }

    #[test]
    fn test_plan_summary_from_plan_trait_all_completed() {
        let mut plan = create_test_plan();
        // Make all steps completed
        for step in &mut plan.steps {
            step.status = StepStatus::Done;
        }
        let summary = PlanSummary::from(&plan);

        // Verify step counts when all steps are completed
        assert_eq!(summary.total_steps, 3);
        assert_eq!(summary.completed_steps, 3);
        assert_eq!(summary.pending_steps, 0);
    }

    #[test]
    fn test_plan_filter_from_list_plans_active() {
        use crate::params::ListPlans;

        let params = ListPlans { archived: false };
        let filter: PlanFilter = (&params).into();

        assert_eq!(filter.status, Some(PlanStatus::Active));
        assert!(!filter.include_archived);
        assert_eq!(filter.title_contains, None);
        assert_eq!(filter.directory, None);
        assert_eq!(filter.created_after, None);
        assert_eq!(filter.created_before, None);
        assert_eq!(filter.completion_status, None);
    }

    #[test]
    fn test_plan_filter_from_list_plans_archived() {
        use crate::params::ListPlans;

        let params = ListPlans { archived: true };
        let filter: PlanFilter = (&params).into();

        assert_eq!(filter.status, Some(PlanStatus::Archived));
        assert!(filter.include_archived);
        assert_eq!(filter.title_contains, None);
        assert_eq!(filter.directory, None);
        assert_eq!(filter.created_after, None);
        assert_eq!(filter.created_before, None);
        assert_eq!(filter.completion_status, None);
    }

    #[test]
    fn test_plan_filter_for_directory_active() {
        let directory = "/path/to/project".to_string();
        let filter = PlanFilter::for_directory(directory.clone(), false);

        assert_eq!(filter.status, Some(PlanStatus::Active));
        assert_eq!(filter.directory, Some(directory));
        assert!(!filter.include_archived);
        // Verify other fields use defaults
        assert_eq!(filter.title_contains, None);
        assert_eq!(filter.created_after, None);
        assert_eq!(filter.created_before, None);
        assert_eq!(filter.completion_status, None);
    }

    #[test]
    fn test_plan_filter_for_directory_archived() {
        let directory = "/path/to/archived".to_string();
        let filter = PlanFilter::for_directory(directory.clone(), true);

        assert_eq!(filter.status, Some(PlanStatus::Archived));
        assert_eq!(filter.directory, Some(directory));
        assert!(filter.include_archived);
        // Verify other fields use defaults
        assert_eq!(filter.title_contains, None);
        assert_eq!(filter.created_after, None);
        assert_eq!(filter.created_before, None);
        assert_eq!(filter.completion_status, None);
    }

    #[test]
    fn test_update_step_request_new_constructor() {
        let request = UpdateStepRequest::new(
            Some("Test Title".to_string()),
            Some("Test Description".to_string()),
            Some("Test Acceptance".to_string()),
            Some(vec!["ref1.txt".to_string(), "ref2.txt".to_string()]),
            Some(StepStatus::Done),
            Some("Test Result".to_string()),
        );

        assert_eq!(request.title, Some("Test Title".to_string()));
        assert_eq!(request.description, Some("Test Description".to_string()));
        assert_eq!(
            request.acceptance_criteria,
            Some("Test Acceptance".to_string())
        );
        assert_eq!(
            request.references,
            Some(vec!["ref1.txt".to_string(), "ref2.txt".to_string()])
        );
        assert_eq!(request.status, Some(StepStatus::Done));
        assert_eq!(request.result, Some("Test Result".to_string()));
    }

    #[test]
    fn test_update_step_request_new_constructor_minimal() {
        let request = UpdateStepRequest::new(None, None, None, None, None, None);

        assert_eq!(request.title, None);
        assert_eq!(request.description, None);
        assert_eq!(request.acceptance_criteria, None);
        assert_eq!(request.references, None);
        assert_eq!(request.status, None);
        assert_eq!(request.result, None);
    }

    #[test]
    fn test_update_step_request_try_from_valid_todo() {
        use crate::params::UpdateStep;

        let mut params = UpdateStep::default();
        params.id = 1;
        params.status = Some("todo".to_string());
        params.title = Some("Updated Title".to_string());
        params.description = Some("Updated Description".to_string());

        let result: Result<UpdateStepRequest, _> = params.try_into();
        assert!(result.is_ok());

        let request = result.unwrap();
        assert_eq!(request.title, Some("Updated Title".to_string()));
        assert_eq!(request.description, Some("Updated Description".to_string()));
        assert_eq!(request.status, Some(StepStatus::Todo));
        assert_eq!(request.result, None);
    }

    #[test]
    fn test_update_step_request_try_from_valid_done_with_result() {
        use crate::params::UpdateStep;

        let mut params = UpdateStep::default();
        params.id = 1;
        params.status = Some("done".to_string());
        params.result = Some("Task completed successfully".to_string());
        params.acceptance_criteria = Some("Must pass all tests".to_string());
        params.references = Some(vec!["file.txt".to_string()]);

        let result: Result<UpdateStepRequest, _> = params.try_into();
        assert!(result.is_ok());

        let request = result.unwrap();
        assert_eq!(request.status, Some(StepStatus::Done));
        assert_eq!(
            request.result,
            Some("Task completed successfully".to_string())
        );
        assert_eq!(
            request.acceptance_criteria,
            Some("Must pass all tests".to_string())
        );
        assert_eq!(request.references, Some(vec!["file.txt".to_string()]));
    }

    #[test]
    fn test_update_step_request_try_from_done_missing_result() {
        use crate::params::UpdateStep;

        let mut params = UpdateStep::default();
        params.id = 1;
        params.status = Some("done".to_string());
        params.result = None; // Missing result for done status

        let result: Result<UpdateStepRequest, _> = params.try_into();
        assert!(result.is_err());

        match result.unwrap_err() {
            crate::PlannerError::InvalidInput { field, reason } => {
                assert_eq!(field, "result");
                assert!(reason.contains("Result description is required"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[test]
    fn test_update_step_request_try_from_invalid_status() {
        use crate::params::UpdateStep;

        let mut params = UpdateStep::default();
        params.id = 1;
        params.status = Some("invalid_status".to_string());

        let result: Result<UpdateStepRequest, _> = params.try_into();
        assert!(result.is_err());

        match result.unwrap_err() {
            crate::PlannerError::InvalidInput { field, reason } => {
                assert_eq!(field, "status");
                assert!(reason.contains("Invalid status: invalid_status"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[test]
    fn test_update_step_request_try_from_no_changes() {
        use crate::params::UpdateStep;

        let params = UpdateStep::default(); // All fields None

        let result: Result<UpdateStepRequest, _> = params.try_into();
        assert!(result.is_ok());

        let request = result.unwrap();
        assert_eq!(request.title, None);
        assert_eq!(request.description, None);
        assert_eq!(request.acceptance_criteria, None);
        assert_eq!(request.references, None);
        assert_eq!(request.status, None);
        assert_eq!(request.result, None);
    }

    #[test]
    fn test_create_update_request_all_fields() {
        let request = UpdateStepRequest::new(
            Some("New Title".to_string()),
            Some("New Description".to_string()),
            Some("New Acceptance".to_string()),
            Some(vec!["ref1.txt".to_string(), "ref2.txt".to_string()]),
            Some(StepStatus::Done),
            Some("Completed successfully".to_string()),
        );

        assert_eq!(request.title, Some("New Title".to_string()));
        assert_eq!(request.description, Some("New Description".to_string()));
        assert_eq!(
            request.acceptance_criteria,
            Some("New Acceptance".to_string())
        );
        assert_eq!(
            request.references,
            Some(vec!["ref1.txt".to_string(), "ref2.txt".to_string()])
        );
        assert_eq!(request.status, Some(StepStatus::Done));
        assert_eq!(request.result, Some("Completed successfully".to_string()));
    }

    #[test]
    fn test_create_update_request_minimal() {
        let request = UpdateStepRequest::new(None, None, None, None, None, None);

        assert_eq!(request.title, None);
        assert_eq!(request.description, None);
        assert_eq!(request.acceptance_criteria, None);
        assert_eq!(request.references, None);
        assert_eq!(request.status, None);
        assert_eq!(request.result, None);
    }

    #[test]
    fn test_create_directory_filter_active() {
        let directory = "/path/to/project".to_string();
        let filter = PlanFilter::for_directory(directory.clone(), false);

        assert_eq!(filter.status, Some(PlanStatus::Active));
        assert_eq!(filter.directory, Some(directory));
        assert!(!filter.include_archived);
    }

    #[test]
    fn test_create_directory_filter_archived() {
        let directory = "/path/to/project".to_string();
        let filter = PlanFilter::for_directory(directory.clone(), true);

        assert_eq!(filter.status, Some(PlanStatus::Archived));
        assert_eq!(filter.directory, Some(directory));
        assert!(filter.include_archived);
    }

    #[test]
    fn test_local_date_time_new() {
        let timestamp = Timestamp::from_second(1640995200).unwrap(); // 2022-01-01 00:00:00 UTC
        let local_dt = LocalDateTime::new(&timestamp);
        
        // Verify the wrapper holds the correct timestamp
        assert_eq!(local_dt.0, &timestamp);
    }

    #[test]
    fn test_local_date_time_display_format() {
        let timestamp = Timestamp::from_second(1640995200).unwrap(); // 2022-01-01 00:00:00 UTC
        let local_dt = LocalDateTime::new(&timestamp);
        let output = format!("{}", local_dt);
        
        // Should contain date in YYYY-MM-DD format
        assert!(output.contains("2022-01-01"));
        // Should contain time components (exact time depends on system timezone)
        assert!(output.contains(":"));
        // Should contain timezone info
        let parts: Vec<&str> = output.split_whitespace().collect();
        assert_eq!(parts.len(), 3); // Date, Time, Timezone
        assert_eq!(parts[0], "2022-01-01");
        assert!(parts[1].contains(":")); // Time has colons
        assert!(!parts[2].is_empty()); // Timezone is non-empty
    }


    #[test]
    fn test_local_date_time_different_timestamps() {
        // Test with different timestamps to ensure formatting works consistently
        let timestamps = vec![
            Timestamp::from_second(1640995200).unwrap(), // 2022-01-01 00:00:00 UTC
            Timestamp::from_second(1672531200).unwrap(), // 2023-01-01 00:00:00 UTC
            Timestamp::from_second(1704067200).unwrap(), // 2024-01-01 00:00:00 UTC
        ];
        
        for timestamp in timestamps {
            let local_dt = LocalDateTime::new(&timestamp);
            let local_dt_output = format!("{}", local_dt);
            
            // Each should have the expected format structure
            let parts: Vec<&str> = local_dt_output.split_whitespace().collect();
            assert_eq!(parts.len(), 3); // Date, Time, Timezone
            assert!(parts[1].contains(":")); // Time component
            assert!(!local_dt_output.is_empty()); // Output should not be empty
        }
    }

    #[test] 
    fn test_local_date_time_lifetime_safety() {
        // Test that LocalDateTime correctly holds lifetime to timestamp
        let timestamp = Timestamp::from_second(1640995200).unwrap();
        let local_dt = LocalDateTime::new(&timestamp);
        
        // Should be able to format multiple times
        let output1 = format!("{}", local_dt);
        let output2 = format!("{}", local_dt);
        
        assert_eq!(output1, output2);
        assert!(!output1.is_empty());
    }
}
