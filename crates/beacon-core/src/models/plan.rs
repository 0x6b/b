//! Plan model definition and related functionality.

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use super::{PlanStatus, Step};

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
    #[serde(default)]
    pub steps: Vec<Step>,
}
