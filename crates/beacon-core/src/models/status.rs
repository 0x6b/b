//! Status enumerations for plans and steps.

use std::str::FromStr;

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

impl PlanStatus {
    /// Convert to database string representation (for backwards compatibility)
    pub fn as_str(&self) -> &'static str {
        match self {
            PlanStatus::Active => "active",
            PlanStatus::Archived => "archived",
        }
    }
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
