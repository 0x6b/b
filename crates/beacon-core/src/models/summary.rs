//! Plan summary types and functionality.

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use super::{Plan, PlanStatus, StepStatus};

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
