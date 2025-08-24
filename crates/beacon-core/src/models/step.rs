//! Step model definition and related functionality.

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use super::StepStatus;

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
