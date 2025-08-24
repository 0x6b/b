//! Collection wrapper types for displaying groups of domain objects.
//!
//! This module provides wrapper types that format collections of domain objects
//! with consistent structure and empty collection handling.

use std::{fmt, ops::Index};

use crate::models::{PlanSummary, Step};

/// Newtype wrapper for displaying collections of plan summaries.
///
/// This provides clean Display formatting for plan collections without title
/// handling, allowing consumers to handle titles separately. Handles empty
/// collections gracefully.
///
/// # Examples
///
/// ```rust
/// use beacon_core::{
///     display::PlanSummaries,
///     models::{PlanStatus, PlanSummary},
/// };
/// use jiff::Timestamp;
///
/// let plan = PlanSummary {
///     id: 1,
///     title: "My Project".to_string(),
///     description: Some("A test project".to_string()),
///     status: PlanStatus::Active,
///     directory: Some("/home/user/project".to_string()),
///     created_at: Timestamp::now(),
///     updated_at: Timestamp::now(),
///     total_steps: 5,
///     completed_steps: 2,
///     pending_steps: 3,
/// };
/// let plans = vec![plan];
///
/// // Format a collection of plans
/// let summaries = PlanSummaries(plans);
/// let output = format!("{}", summaries);
/// assert!(output.contains("My Project"));
/// ```
pub struct PlanSummaries(pub Vec<PlanSummary>);

impl PlanSummaries {
    /// Check if the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the number of plan summaries in the collection.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Get a reference to the plan summary at the given index.
    pub fn get(&self, index: usize) -> Option<&PlanSummary> {
        self.0.get(index)
    }

    /// Get an iterator over the plan summaries.
    pub fn iter(&self) -> std::slice::Iter<'_, PlanSummary> {
        self.0.iter()
    }
}

impl Index<usize> for PlanSummaries {
    type Output = PlanSummary;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IntoIterator for PlanSummaries {
    type Item = PlanSummary;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a PlanSummaries {
    type Item = &'a PlanSummary;
    type IntoIter = std::slice::Iter<'a, PlanSummary>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl fmt::Display for PlanSummaries {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            writeln!(f, "No plans found.")
        } else {
            for plan in &self.0 {
                write!(f, "{}", plan)?;
            }
            Ok(())
        }
    }
}

/// Newtype wrapper for displaying collections of steps.
///
/// This wrapper provides Display implementation for collections of steps
/// without requiring title formatting logic. It handles empty collections
/// gracefully and formats each step using the existing Step Display trait.
///
/// # Examples
///
/// ```rust
/// use beacon_core::{
///     display::Steps,
///     models::{Step, StepStatus},
/// };
/// use jiff::Timestamp;
///
/// // Create a collection of steps
/// let step = Step {
///     id: 1,
///     plan_id: 42,
///     title: "Example step".to_string(),
///     description: None,
///     acceptance_criteria: None,
///     references: vec![],
///     status: StepStatus::Todo,
///     result: None,
///     order: 0,
///     created_at: Timestamp::now(),
///     updated_at: Timestamp::now(),
/// };
/// let steps = Steps(vec![step]);
/// println!("{}", steps);
/// ```
pub struct Steps(pub Vec<Step>);

impl Steps {
    /// Check if the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the number of steps in the collection.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Get a reference to the step at the given index.
    pub fn get(&self, index: usize) -> Option<&Step> {
        self.0.get(index)
    }

    /// Get an iterator over the steps.
    pub fn iter(&self) -> std::slice::Iter<'_, Step> {
        self.0.iter()
    }
}

impl Index<usize> for Steps {
    type Output = Step;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IntoIterator for Steps {
    type Item = Step;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Steps {
    type Item = &'a Step;
    type IntoIter = std::slice::Iter<'a, Step>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl fmt::Display for Steps {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            writeln!(f, "No steps found.")
        } else {
            for step in &self.0 {
                write!(f, "{}", step)?;
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use jiff::Timestamp;

    use super::*;
    use crate::models::{PlanStatus, StepStatus};

    fn create_test_plan_summary() -> PlanSummary {
        PlanSummary {
            id: 1,
            title: "Test Plan".to_string(),
            description: Some("A test plan".to_string()),
            status: PlanStatus::Active,
            directory: Some("/test".to_string()),
            created_at: Timestamp::from_second(1640995200).unwrap(), // 2022-01-01 00:00:00 UTC
            updated_at: Timestamp::from_second(1640995200).unwrap(),
            total_steps: 3,
            completed_steps: 1,
            pending_steps: 2,
        }
    }

    fn create_test_step() -> Step {
        Step {
            id: 1,
            plan_id: 1,
            title: "Test Step".to_string(),
            description: Some("A test step".to_string()),
            acceptance_criteria: Some("Should work".to_string()),
            references: vec!["http://example.com".to_string()],
            status: StepStatus::Todo,
            result: None,
            order: 0,
            created_at: Timestamp::from_second(1640995200).unwrap(),
            updated_at: Timestamp::from_second(1640995200).unwrap(),
        }
    }

    #[test]
    fn test_plan_summaries_display() {
        // Test with plans
        let plans = vec![create_test_plan_summary()];
        let summaries = PlanSummaries(plans);
        let output = format!("{}", summaries);
        assert!(output.contains("Test Plan"));
        assert!(output.contains("ID: 1"));

        // Test empty collection
        let empty_summaries = PlanSummaries(vec![]);
        let empty_output = format!("{}", empty_summaries);
        assert_eq!(empty_output, "No plans found.\n");

        // Test multiple plans
        let plan1 = create_test_plan_summary();
        let mut plan2 = create_test_plan_summary();
        plan2.id = 2;
        plan2.title = "Second Plan".to_string();
        let plans = vec![plan1, plan2];
        let summaries = PlanSummaries(plans);
        let output = format!("{}", summaries);
        assert!(output.contains("Test Plan"));
        assert!(output.contains("Second Plan"));
        assert!(output.contains("ID: 1"));
        assert!(output.contains("ID: 2"));

        // Verify the output uses PlanSummary's own Display format (which includes ##)
        // but doesn't add additional title formatting
        assert!(output.contains("## Test Plan"));
        assert!(output.contains("## Second Plan"));
        // Verify it doesn't start with a title header
        assert!(!output.starts_with("# "));
    }

    #[test]
    fn test_steps_display_empty() {
        let steps = Steps(vec![]);
        let output = format!("{}", steps);
        assert_eq!(output, "No steps found.\n");
    }

    #[test]
    fn test_steps_display_single_step() {
        let step = create_test_step();
        let steps = Steps(vec![step]);
        let output = format!("{}", steps);

        assert!(output.contains("Test Step"));
        assert!(output.contains("○ Todo"));
        assert!(output.contains("Should work"));
    }

    #[test]
    fn test_steps_display_multiple_steps() {
        let step1 = create_test_step();
        let mut step2 = create_test_step();
        step2.id = 2;
        step2.title = "Second Step".to_string();
        step2.status = StepStatus::Done;

        let steps = Steps(vec![step1, step2]);
        let output = format!("{}", steps);

        assert!(output.contains("Test Step"));
        assert!(output.contains("Second Step"));
        assert!(output.contains("○ Todo"));
        assert!(output.contains("✓ Done"));
    }
}
