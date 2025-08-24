//! Display implementations for domain models.
//!
//! This module contains all Display trait implementations for the core domain
//! models, separated from the model definitions to maintain clean separation of
//! concerns.
//!
//! The Display implementations provide:
//! - Markdown-formatted output for rich terminal display
//! - Consistent formatting with status icons and structured sections
//! - Context-aware display behavior for different use cases

use std::fmt;

use super::datetime::LocalDateTime;
use crate::models::{Plan, PlanStatus, PlanSummary, Step, StepStatus};

impl fmt::Display for PlanStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl fmt::Display for StepStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
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
        if self.status == StepStatus::Done
            && let Some(result) = &self.result
        {
            writeln!(f, "#### Result")?;
            writeln!(f)?;
            writeln!(f, "{result}")?;
            writeln!(f)?;
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
