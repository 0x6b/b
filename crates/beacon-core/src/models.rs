//! Data models for plans and steps.

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
        write!(
            f,
            "{}",
            match self {
                PlanStatus::Active => "active",
                PlanStatus::Archived => "archived",
            }
        )
    }
}

impl PlanStatus {
    /// Parse a status from a string (for backwards compatibility)
    pub fn parse(s: &str) -> Option<Self> {
        s.parse().ok()
    }

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
        write!(
            f,
            "{}",
            match self {
                StepStatus::Todo => "todo",
                StepStatus::InProgress => "inprogress",
                StepStatus::Done => "done",
            }
        )
    }
}

impl StepStatus {
    /// Parse a status from a string (for backwards compatibility)
    pub fn parse(s: &str) -> Option<Self> {
        s.parse().ok()
    }

    /// Convert to database string representation (for backwards compatibility)
    pub fn as_str(&self) -> &'static str {
        match self {
            StepStatus::Todo => "todo",
            StepStatus::InProgress => "inprogress",
            StepStatus::Done => "done",
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

/// Summary information about a plan.
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

/// Wrapper to display a step with its position number
struct StepWithPosition<'a> {
    step: &'a Step,
    position: usize,
}

impl<'a> fmt::Display for StepWithPosition<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status_text = match self.step.status {
            StepStatus::Done => "done",
            StepStatus::InProgress => "in progress",
            StepStatus::Todo => "todo",
        };

        writeln!(
            f,
            "### {}. {} ({})",
            self.position, self.step.title, status_text
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
        // Default display for Step when used outside of a Plan context
        // Uses the order field as position (1-indexed)
        let position = self.order + 1;
        write!(
            f,
            "{}",
            StepWithPosition {
                step: self,
                position: position as usize
            }
        )
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

/// Format datetime for display
fn format_datetime(dt: &Timestamp) -> String {
    dt.strftime("%Y-%m-%d %H:%M:%S UTC").to_string()
}
