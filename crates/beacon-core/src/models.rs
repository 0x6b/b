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

use jiff::Timestamp;
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

/// Completion status filter options.
#[derive(Debug, Clone)]
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

impl fmt::Display for Plan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "# {}. {}", self.id, self.title)?;
        writeln!(f)?;

        // Metadata section
        writeln!(f, "- Status: {}", self.status.as_str())?;
        if let Some(dir) = &self.directory {
            writeln!(f, "- Directory: {dir}")?;
        }
        writeln!(f, "- Created: {}", format_datetime(&self.created_at))?;
        writeln!(f, "- Updated: {}", format_datetime(&self.updated_at))?;

        // Description as a paragraph
        if let Some(desc) = &self.description {
            writeln!(f)?;
            writeln!(f, "{desc}")?;
        }

        if !self.steps.is_empty() {
            writeln!(f, "\n## Steps")?;
            writeln!(f)?;
            for (index, step) in self.steps.iter().enumerate() {
                // Pass the position (1-indexed) to the step display
                write!(
                    f,
                    "{}",
                    StepWithPosition {
                        step,
                        position: index + 1
                    }
                )?;
            }
        } else {
            writeln!(f, "\nNo steps in this plan.")?;
        }

        Ok(())
    }
}

/// Internal wrapper to display a step with its position number within a plan.
///
/// This wrapper is used internally by [`Plan::fmt`] to display steps with
/// their position numbers (1-indexed) and provide contextual formatting
/// that differs from standalone step display.
///
/// The formatting includes:
/// - Position number in the step header
/// - Compact status display with icons
/// - Structured sections for description, acceptance criteria, result, and
///   references
struct StepWithPosition<'a> {
    step: &'a Step,
    position: usize,
}

impl<'a> fmt::Display for StepWithPosition<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "### {}. {} ({})",
            self.position,
            self.step.title,
            self.step.status.with_icon()
        )?;
        writeln!(f)?;

        if let Some(desc) = &self.step.description {
            writeln!(f, "{desc}")?;
            writeln!(f)?;
        }

        if let Some(criteria) = &self.step.acceptance_criteria {
            writeln!(f, "#### Acceptance")?;
            writeln!(f)?;
            writeln!(f, "{criteria}")?;
            writeln!(f)?;
        }

        // Show result only for completed steps
        if self.step.status == StepStatus::Done {
            if let Some(result) = &self.step.result {
                writeln!(f, "#### Result")?;
                writeln!(f)?;
                writeln!(f, "{result}")?;
                writeln!(f)?;
            }
        }

        if !self.step.references.is_empty() {
            writeln!(f, "#### References")?;
            writeln!(f)?;
            for reference in &self.step.references {
                writeln!(f, "- {reference}")?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

impl fmt::Display for Step {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Enhanced display for Step when used independently
        writeln!(f, "# Step {} Details", self.id)?;
        writeln!(f)?;
        writeln!(f, "Title: {}", self.title)?;
        writeln!(f, "Status: {}", self.status.with_icon())?;
        writeln!(f, "Plan ID: {}", self.plan_id)?;

        if let Some(desc) = &self.description {
            writeln!(f)?;
            writeln!(f, "## Description")?;
            writeln!(f, "{}", desc)?;
        }

        if let Some(criteria) = &self.acceptance_criteria {
            writeln!(f)?;
            writeln!(f, "## Acceptance Criteria")?;
            writeln!(f, "{}", criteria)?;
        }

        // Show result only for completed steps
        if self.status == StepStatus::Done {
            if let Some(result) = &self.result {
                writeln!(f)?;
                writeln!(f, "## Result")?;
                writeln!(f, "{}", result)?;
            }
        }

        if !self.references.is_empty() {
            writeln!(f)?;
            writeln!(f, "## References")?;
            for reference in &self.references {
                writeln!(f, "- {}", reference)?;
            }
        }

        writeln!(f)?;
        writeln!(f, "Created: {}", self.created_at)?;
        writeln!(f, "Updated: {}", self.updated_at)?;

        Ok(())
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
            writeln!(f, "- Description: {desc}")?;
        }

        if let Some(dir) = &self.directory {
            writeln!(f, "- Directory: {dir}")?;
        }

        writeln!(f, "- Created: {}", format_datetime(&self.created_at))?;
        writeln!(f)?; // Add blank line after each plan

        Ok(())
    }
}

/// Format datetime for consistent display across all models.
///
/// This function provides a standardized timestamp format used throughout
/// the display system. The format is human-readable and includes timezone
/// information for clarity.
///
/// # Format
/// `YYYY-MM-DD HH:MM:SS UTC` (e.g., "2022-01-01 15:30:45 UTC")
///
/// # Examples
///
/// ```rust
/// use beacon_core::format_datetime;
/// use jiff::Timestamp;
///
/// let timestamp = Timestamp::from_second(1640995200).unwrap();
/// let formatted = format!("{}", format_datetime(&timestamp));
/// assert_eq!(formatted, "2022-01-01 00:00:00 UTC");
/// ```
pub fn format_datetime(dt: &Timestamp) -> impl fmt::Display + '_ {
    dt.strftime("%Y-%m-%d %H:%M:%S UTC")
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

        // Should contain step header and basic info
        assert!(output.contains("# Step 123 Details"));
        assert!(output.contains("Title: Test Step Title"));
        assert!(output.contains("Status: ○ Todo"));
        assert!(output.contains("Plan ID: 456"));

        // Should contain description and acceptance criteria
        assert!(output.contains("## Description"));
        assert!(output.contains("This is a test step description"));
        assert!(output.contains("## Acceptance Criteria"));
        assert!(output.contains("Should pass all tests"));

        // Should contain references
        assert!(output.contains("## References"));
        assert!(output.contains("- https://example.com"));
        assert!(output.contains("- file.txt"));

        // Should contain timestamps (ISO format)
        assert!(output.contains("Created: 2022-01-01T00:00:00Z"));
        assert!(output.contains("Updated: 2022-01-02T00:00:00Z"));

        // Should NOT contain result section for todo steps
        assert!(!output.contains("## Result"));
    }

    #[test]
    fn test_step_display_independently_in_progress() {
        let step = create_test_step(StepStatus::InProgress);
        let output = format!("{}", step);

        assert!(output.contains("Status: ➤ In Progress"));
        assert!(!output.contains("## Result"));
    }

    #[test]
    fn test_step_display_independently_done() {
        let step = create_test_step(StepStatus::Done);
        let output = format!("{}", step);

        assert!(output.contains("Status: ✓ Done"));
        assert!(output.contains("## Result"));
        assert!(output.contains("Successfully completed the test"));
    }

    #[test]
    fn test_step_display_within_plan_context() {
        let step = create_test_step(StepStatus::InProgress);
        let step_with_position = StepWithPosition {
            step: &step,
            position: 3,
        };
        let output = format!("{}", step_with_position);

        // Should use plan context formatting
        assert!(output.contains("### 3. Test Step Title (➤ In Progress)"));
        assert!(output.contains("#### Acceptance"));
        assert!(output.contains("#### References"));

        // Should NOT contain the independent step header
        assert!(!output.contains("# Step 123 Details"));
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
        assert!(output.contains("- Created: 2022-01-01 00:00:00 UTC"));
        assert!(output.contains("- Updated: 2022-01-02 00:00:00 UTC"));

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
        assert!(output.contains("- Description: Summary description"));
        assert!(output.contains("- Directory: /test/summary"));
        assert!(output.contains("- Created: 2022-01-01 00:00:00 UTC"));

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
        assert!(output.contains("- Created: 2022-01-01 00:00:00 UTC"));

        // Should not contain optional fields
        assert!(!output.contains("- Description:"));
        assert!(!output.contains("- Directory:"));
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
        let todo_with_pos = StepWithPosition {
            step: &todo_step,
            position: 1,
        };
        let in_progress_with_pos = StepWithPosition {
            step: &in_progress_step,
            position: 2,
        };
        let done_with_pos = StepWithPosition {
            step: &done_step,
            position: 3,
        };

        let todo_pos_output = format!("{}", todo_with_pos);
        let in_progress_pos_output = format!("{}", in_progress_with_pos);
        let done_pos_output = format!("{}", done_with_pos);

        assert!(todo_pos_output.contains("○ Todo"));
        assert!(in_progress_pos_output.contains("➤ In Progress"));
        assert!(done_pos_output.contains("✓ Done"));
    }
}
