//! Step CRUD operations and queries.

use jiff::Timestamp;
use rusqlite::{params, types::Type, OptionalExtension};

use crate::{
    error::{DatabaseResultExt, PlannerError, Result},
    models::{Step, StepStatus, UpdateStepRequest},
};

// Optimized SQL queries as const strings for compile-time optimization
const CHECK_PLAN_EXISTS_SQL: &str = "SELECT EXISTS(SELECT 1 FROM plans WHERE id = ?1)";
const GET_MAX_STEP_ORDER_SQL: &str =
    "SELECT COALESCE(MAX(step_order), -1) + 1 FROM steps WHERE plan_id = ?1";
const INSERT_STEP_SQL: &str = "INSERT INTO steps (plan_id, title, description, acceptance_criteria, step_references, status, result, step_order, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)";
const UPDATE_PLAN_TIMESTAMP_SQL: &str = "UPDATE plans SET updated_at = ?1 WHERE id = ?2";
const UPDATE_PLAN_TIMESTAMP_BY_STEP_SQL: &str =
    "UPDATE plans SET updated_at = ?1 WHERE id = (SELECT plan_id FROM steps WHERE id = ?2)";
const GET_MAX_STEP_ORDER_ONLY_SQL: &str = "SELECT MAX(step_order) FROM steps WHERE plan_id = ?1";
const UPDATE_STEP_ORDERS_INCREMENT_SQL: &str =
    "UPDATE steps SET step_order = step_order + 1 WHERE plan_id = ?1 AND step_order >= ?2";
const SELECT_STEP_DETAILS_SQL: &str = "SELECT title, description, acceptance_criteria, step_references, status, result FROM steps WHERE id = ?1";
const UPDATE_STEP_SQL: &str = "UPDATE steps SET title = ?1, description = ?2, acceptance_criteria = ?3, step_references = ?4, status = ?5, result = ?6, updated_at = ?7 WHERE id = ?8";
const SELECT_STEPS_BY_PLAN_SQL: &str = "SELECT id, plan_id, title, description, acceptance_criteria, step_references, status, result, step_order, created_at, updated_at FROM steps WHERE plan_id = ?1 ORDER BY step_order";
const SELECT_STEP_BY_ID_SQL: &str = "SELECT id, plan_id, title, description, acceptance_criteria, step_references, status, result, step_order, created_at, updated_at FROM steps WHERE id = ?1";
const SELECT_STEP_STATUS_SQL: &str = "SELECT status FROM steps WHERE id = ?1";
const UPDATE_STEP_STATUS_CLAIMED_SQL: &str =
    "UPDATE steps SET status = ?1, updated_at = ?2 WHERE id = ?3 AND status = ?4";
const SELECT_STEP_ORDER_SQL: &str = "SELECT plan_id, step_order FROM steps WHERE id = ?1";
const UPDATE_STEP_ORDER_TEMP_SQL: &str =
    "UPDATE steps SET step_order = -1, updated_at = ?1 WHERE id = ?2";
const UPDATE_STEP_ORDER_SQL: &str =
    "UPDATE steps SET step_order = ?1, updated_at = ?2 WHERE id = ?3";
const DELETE_STEP_SQL: &str = "DELETE FROM steps WHERE id = ?1";
const UPDATE_STEP_ORDERS_DECREMENT_SQL: &str =
    "UPDATE steps SET step_order = step_order - 1 WHERE plan_id = ?1 AND step_order > ?2";

impl super::Database {
    /// Helper function to construct a Step from a database row
    fn build_step_from_row(row: &rusqlite::Row) -> rusqlite::Result<Step> {
        let status_str: String = row.get(6)?;
        let status = status_str.parse::<StepStatus>().map_err(|_| {
            rusqlite::Error::FromSqlConversionFailure(
                6,
                Type::Text,
                format!("Invalid status: {status_str}").into(),
            )
        })?;

        // Parse references from comma-separated string
        let references_str: Option<String> = row.get(5)?;
        let references = references_str
            .map(|s| s.split(',').map(String::from).collect())
            .unwrap_or_default();

        Ok(Step {
            id: row.get::<_, i64>(0)? as u64,
            plan_id: row.get::<_, i64>(1)? as u64,
            title: row.get(2)?,
            description: row.get(3)?,
            acceptance_criteria: row.get(4)?,
            references,
            status,
            result: row.get(7)?,
            order: row.get::<_, i64>(8)? as u32,
            created_at: row.get::<_, String>(9)?.parse::<Timestamp>().map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(9, Type::Text, Box::new(e))
            })?,
            updated_at: row
                .get::<_, String>(10)?
                .parse::<Timestamp>()
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(10, Type::Text, Box::new(e))
                })?,
        })
    }
    /// Adds a new step to the specified plan.
    pub fn add_step(
        &mut self,
        plan_id: u64,
        title: &str,
        description: Option<&str>,
        acceptance_criteria: Option<&str>,
        references: Vec<String>,
    ) -> Result<Step> {
        let tx = self
            .connection
            .transaction()
            .db_context("Failed to begin transaction")?;

        // Check if plan exists
        let plan_exists: bool = tx
            .query_row(CHECK_PLAN_EXISTS_SQL, params![plan_id as i64], |row| {
                row.get(0)
            })
            .map_err(|e| PlannerError::database_error("Failed to check plan existence", e))?;

        if !plan_exists {
            return Err(PlannerError::PlanNotFound { id: plan_id });
        }

        let next_order: i64 = tx
            .query_row(GET_MAX_STEP_ORDER_SQL, params![plan_id as i64], |row| {
                row.get(0)
            })
            .map_err(|e| PlannerError::database_error("Failed to get next step order", e))?;

        let now = Timestamp::now();
        let now_str = now.to_string();

        // Store references as comma-separated string
        let references_str = if references.is_empty() {
            None
        } else {
            Some(references.join(","))
        };

        tx.execute(
            INSERT_STEP_SQL,
            params![
                plan_id as i64,
                title,
                description,
                acceptance_criteria,
                references_str.as_deref(),
                "todo",
                None::<String>, // result is NULL for new steps
                next_order,
                &now_str,
                &now_str
            ],
        )
        .map_err(|e| PlannerError::database_error("Failed to insert step", e))?;

        let id = tx.last_insert_rowid() as u64;

        // Update plan's updated_at
        tx.execute(UPDATE_PLAN_TIMESTAMP_SQL, params![&now_str, plan_id as i64])
            .map_err(|e| PlannerError::database_error("Failed to update plan timestamp", e))?;

        tx.commit().db_context("Failed to commit transaction")?;

        Ok(Step {
            id,
            plan_id,
            title: title.into(),
            description: description.map(String::from),
            acceptance_criteria: acceptance_criteria.map(String::from),
            references,
            status: StepStatus::Todo,
            result: None, // New steps have no result
            order: next_order as u32,
            created_at: now,
            updated_at: now,
        })
    }

    /// Inserts a new step at a specific position in the plan's step order.
    /// All steps at or after the specified position will have their order
    /// incremented.
    pub fn insert_step(
        &mut self,
        plan_id: u64,
        position: u32,
        title: &str,
        description: Option<&str>,
        acceptance_criteria: Option<&str>,
        references: Vec<String>,
    ) -> Result<Step> {
        let tx = self
            .connection
            .transaction()
            .db_context("Failed to begin transaction")?;

        // Check if plan exists
        let plan_exists: bool = tx
            .query_row(CHECK_PLAN_EXISTS_SQL, params![plan_id as i64], |row| {
                row.get(0)
            })
            .map_err(|e| PlannerError::database_error("Failed to check plan existence", e))?;

        if !plan_exists {
            return Err(PlannerError::PlanNotFound { id: plan_id });
        }

        let max_order: Option<i64> = tx
            .query_row(
                GET_MAX_STEP_ORDER_ONLY_SQL,
                params![plan_id as i64],
                |row| row.get(0),
            )
            .map_err(|e| PlannerError::database_error("Failed to get max step order", e))?;

        // Validate position - allow inserting at the end (position == count)
        let step_count = max_order.map(|m| m + 1).unwrap_or(0) as u32;
        if position > step_count {
            return Err(PlannerError::InvalidInput {
                field: "position".into(),
                reason: format!("Position {position} is out of range. Plan has {step_count} steps"),
            });
        }

        // Update existing steps' order to make room for the new step
        tx.execute(
            UPDATE_STEP_ORDERS_INCREMENT_SQL,
            params![plan_id as i64, position as i64],
        )
        .map_err(|e| PlannerError::database_error("Failed to update step orders", e))?;

        let now = Timestamp::now();
        let now_str = now.to_string();

        // Store references as comma-separated string
        let references_str = if references.is_empty() {
            None
        } else {
            Some(references.join(","))
        };

        // Insert the new step at the specified position
        tx.execute(
            INSERT_STEP_SQL,
            params![
                plan_id as i64,
                title,
                description,
                acceptance_criteria,
                references_str.as_deref(),
                "todo",
                None::<String>, // result is NULL for new steps
                position as i64,
                &now_str,
                &now_str
            ],
        )
        .map_err(|e| PlannerError::database_error("Failed to insert step", e))?;

        let id = tx.last_insert_rowid() as u64;

        // Update plan's updated_at
        tx.execute(UPDATE_PLAN_TIMESTAMP_SQL, params![&now_str, plan_id as i64])
            .map_err(|e| PlannerError::database_error("Failed to update plan timestamp", e))?;

        tx.commit().db_context("Failed to commit transaction")?;

        Ok(Step {
            id,
            plan_id,
            title: title.into(),
            description: description.map(String::from),
            acceptance_criteria: acceptance_criteria.map(String::from),
            references,
            status: StepStatus::Todo,
            result: None, // New steps have no result
            order: position,
            created_at: now,
            updated_at: now,
        })
    }

    /// Updates step details using a request struct to reduce argument count.
    /// When changing status to Done, result is required.
    /// Result is ignored when changing to Todo or InProgress.
    pub fn update_step(&mut self, step_id: u64, request: UpdateStepRequest) -> Result<()> {
        // Validate result requirement when changing status to Done
        if let Some(StepStatus::Done) = request.status {
            if request.result.is_none() {
                return Err(PlannerError::InvalidInput {
                    field: "result".into(),
                    reason: "Result description is required when marking a step as done".into(),
                });
            }
        }

        // Check if there's anything to update
        if request.title.is_none()
            && request.description.is_none()
            && request.acceptance_criteria.is_none()
            && request.references.is_none()
            && request.status.is_none()
            && request.result.is_none()
        {
            return Ok(());
        }

        let tx = self
            .connection
            .transaction()
            .db_context("Failed to begin transaction")?;

        // First, get the current step to preserve unchanged fields
        let (
            current_title,
            current_desc,
            current_criteria,
            current_refs,
            current_status,
            current_result,
        ): (
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            String,
            Option<String>,
        ) = {
            let mut stmt = tx.prepare(SELECT_STEP_DETAILS_SQL).map_err(|e| {
                PlannerError::database_error("Failed to prepare select statement", e)
            })?;

            stmt.query_row(params![step_id as i64], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            })
            .map_err(|e| {
                if matches!(e, rusqlite::Error::QueryReturnedNoRows) {
                    PlannerError::StepNotFound { id: step_id }
                } else {
                    PlannerError::database_error("Failed to get current step", e)
                }
            })?
        };

        // Use provided values or keep current ones
        let new_title = request.title.unwrap_or(current_title);
        let new_description = request.description.or(current_desc);
        let new_criteria = request.acceptance_criteria.or(current_criteria);
        let new_references = request
            .references
            .map(|refs| refs.join(","))
            .or(current_refs);
        let new_status_str = request
            .status
            .map(|s| s.as_str().into())
            .unwrap_or(current_status);

        // Determine the result value based on the status change
        let new_result = if let Some(new_status) = request.status {
            match new_status {
                StepStatus::Done => request.result, // Use provided result (already validated as
                // required)
                StepStatus::Todo | StepStatus::InProgress => None, /* Clear result for non-done
                                                                    * statuses */
            }
        } else {
            // Status not changing, preserve existing result
            current_result
        };

        let now_str = Timestamp::now().to_string();

        // Update the step
        tx.execute(
            UPDATE_STEP_SQL,
            params![
                &new_title,
                &new_description,
                &new_criteria,
                &new_references,
                &new_status_str,
                &new_result,
                &now_str,
                step_id as i64
            ],
        )
        .map_err(|e| PlannerError::database_error("Failed to update step", e))?;

        // Update plan's updated_at
        tx.execute(
            UPDATE_PLAN_TIMESTAMP_BY_STEP_SQL,
            params![&now_str, step_id as i64],
        )
        .map_err(|e| PlannerError::database_error("Failed to update plan timestamp", e))?;

        tx.commit().db_context("Failed to commit transaction")?;

        Ok(())
    }

    /// Retrieves all steps for a given plan.
    pub fn get_steps(&self, plan_id: u64) -> Result<Vec<Step>> {
        let mut stmt = self
            .connection
            .prepare(SELECT_STEPS_BY_PLAN_SQL)
            .map_err(|e| PlannerError::database_error("Failed to prepare query", e))?;

        let steps = stmt
            .query_map(params![plan_id as i64], Self::build_step_from_row)
            .map_err(|e| PlannerError::database_error("Failed to query steps", e))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| PlannerError::database_error("Failed to fetch steps", e))?;

        Ok(steps)
    }

    /// Retrieves a single step by its ID.
    pub fn get_step(&self, step_id: u64) -> Result<Option<Step>> {
        let mut stmt = self
            .connection
            .prepare(SELECT_STEP_BY_ID_SQL)
            .map_err(|e| PlannerError::database_error("Failed to prepare query", e))?;

        let step = stmt
            .query_row(params![step_id as i64], Self::build_step_from_row)
            .optional()
            .map_err(|e| PlannerError::database_error("Failed to get step", e))?;

        Ok(step)
    }

    /// Atomically claims a step for processing by transitioning it from Todo to
    /// InProgress. Returns the step details if successfully claimed, None if
    /// the step doesn't exist or cannot be claimed.
    pub fn claim_step(&mut self, step_id: u64) -> Result<Option<Step>> {
        let tx = self
            .connection
            .transaction()
            .db_context("Failed to begin transaction")?;

        // Check current status and update atomically
        let current_status: Option<String> = tx
            .query_row(SELECT_STEP_STATUS_SQL, params![step_id as i64], |row| {
                row.get(0)
            })
            .optional()
            .map_err(|e| PlannerError::database_error("Failed to query step status", e))?;

        match current_status {
            None => {
                // Step doesn't exist, return None
                Ok(None)
            }
            Some(status) if status == "todo" => {
                // Atomically update to in_progress
                let now_str = Timestamp::now().to_string();
                tx.execute(
                    UPDATE_STEP_STATUS_CLAIMED_SQL,
                    params![
                        StepStatus::InProgress.as_str(),
                        &now_str,
                        step_id as i64,
                        "todo"
                    ],
                )
                .map_err(|e| PlannerError::database_error("Failed to claim step", e))?;

                // Update plan's updated_at
                tx.execute(
                    UPDATE_PLAN_TIMESTAMP_BY_STEP_SQL,
                    params![&now_str, step_id as i64],
                )
                .map_err(|e| PlannerError::database_error("Failed to update plan timestamp", e))?;

                // Get the updated step details
                let step = tx
                    .query_row(
                        SELECT_STEP_BY_ID_SQL,
                        params![step_id as i64],
                        Self::build_step_from_row,
                    )
                    .optional()
                    .map_err(|e| PlannerError::database_error("Failed to query claimed step", e))?;

                tx.commit().db_context("Failed to commit transaction")?;

                Ok(step)
            }
            _ => {
                // Step is not in Todo status, cannot claim
                Ok(None)
            }
        }
    }

    /// Swaps the order of two steps within the same plan.
    pub fn swap_steps(&mut self, step_id1: u64, step_id2: u64) -> Result<()> {
        // Don't do anything if swapping with self
        if step_id1 == step_id2 {
            return Ok(());
        }

        let tx = self
            .connection
            .transaction()
            .db_context("Failed to begin transaction")?;

        let (plan_id1, order1): (i64, i64) = tx
            .query_row(SELECT_STEP_ORDER_SQL, params![step_id1 as i64], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .map_err(|e| {
                if matches!(e, rusqlite::Error::QueryReturnedNoRows) {
                    PlannerError::StepNotFound { id: step_id1 }
                } else {
                    PlannerError::database_error("Failed to query first step", e)
                }
            })?;

        let (plan_id2, order2): (i64, i64) = tx
            .query_row(SELECT_STEP_ORDER_SQL, params![step_id2 as i64], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .map_err(|e| {
                if matches!(e, rusqlite::Error::QueryReturnedNoRows) {
                    PlannerError::StepNotFound { id: step_id2 }
                } else {
                    PlannerError::database_error("Failed to query second step", e)
                }
            })?;

        // Verify both steps are in the same plan
        if plan_id1 != plan_id2 {
            return Err(PlannerError::InvalidInput {
                field: "step_ids".into(),
                reason: "Steps must be from the same plan to swap".into(),
            });
        }

        // Swap the orders
        let now_str = Timestamp::now().to_string();

        // Use a temporary negative value to avoid unique constraint violation
        tx.execute(
            UPDATE_STEP_ORDER_TEMP_SQL,
            params![&now_str, step_id1 as i64],
        )
        .map_err(|e| PlannerError::database_error("Failed to update first step order", e))?;

        tx.execute(
            UPDATE_STEP_ORDER_SQL,
            params![order1, &now_str, step_id2 as i64],
        )
        .map_err(|e| PlannerError::database_error("Failed to update second step order", e))?;

        tx.execute(
            UPDATE_STEP_ORDER_SQL,
            params![order2, &now_str, step_id1 as i64],
        )
        .map_err(|e| PlannerError::database_error("Failed to update first step final order", e))?;

        // Update plan's updated_at
        tx.execute(UPDATE_PLAN_TIMESTAMP_SQL, params![&now_str, plan_id1])
            .map_err(|e| PlannerError::database_error("Failed to update plan timestamp", e))?;

        tx.commit().db_context("Failed to commit transaction")?;

        Ok(())
    }

    /// Removes a step from a plan.
    pub fn remove_step(&mut self, step_id: u64) -> Result<()> {
        let tx = self
            .connection
            .transaction()
            .db_context("Failed to begin transaction")?;

        let (plan_id, step_order): (i64, i64) = tx
            .query_row(SELECT_STEP_ORDER_SQL, params![step_id as i64], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .map_err(|e| {
                if matches!(e, rusqlite::Error::QueryReturnedNoRows) {
                    PlannerError::StepNotFound { id: step_id }
                } else {
                    PlannerError::database_error("Failed to query step", e)
                }
            })?;

        // Delete the step
        tx.execute(DELETE_STEP_SQL, params![step_id as i64])
            .map_err(|e| PlannerError::database_error("Failed to delete step", e))?;

        // Update order of subsequent steps
        tx.execute(
            UPDATE_STEP_ORDERS_DECREMENT_SQL,
            params![plan_id, step_order],
        )
        .map_err(|e| PlannerError::database_error("Failed to update step orders", e))?;

        // Update plan's updated_at
        let now_str = Timestamp::now().to_string();
        tx.execute(UPDATE_PLAN_TIMESTAMP_SQL, params![&now_str, plan_id])
            .map_err(|e| PlannerError::database_error("Failed to update plan timestamp", e))?;

        tx.commit().db_context("Failed to commit transaction")?;

        Ok(())
    }
}
