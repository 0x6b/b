//! Plan CRUD operations and queries.

use jiff::Timestamp;
use rusqlite::{OptionalExtension, params, types::Type};

use crate::{
    error::{DatabaseResultExt, PlannerError, Result},
    models::{CompletionFilter, Plan, PlanFilter, PlanStatus},
};

// Optimized SQL queries as const strings for compile-time optimization
const INSERT_PLAN_SQL: &str = "INSERT INTO plans (title, description, directory, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)";
const SELECT_PLAN_SQL: &str = "SELECT id, title, description, status, directory, created_at, updated_at FROM plans WHERE id = ?1";
const CHECK_PLAN_EXISTS_SQL: &str = "SELECT EXISTS(SELECT 1 FROM plans WHERE id = ?1)";
const UPDATE_PLAN_ARCHIVE_SQL: &str =
    "UPDATE plans SET status = ?1, updated_at = ?2 WHERE id = ?3 AND status = ?4";
const UPDATE_PLAN_UNARCHIVE_SQL: &str =
    "UPDATE plans SET status = ?1, updated_at = ?2 WHERE id = ?3 AND status = ?4";
const DELETE_PLAN_STEPS_SQL: &str = "DELETE FROM steps WHERE plan_id = ?1";
const DELETE_PLAN_SQL: &str = "DELETE FROM plans WHERE id = ?1";

// Base queries for plan listing
const PLAN_SUMMARY_COLUMNS: &str = "id, title, description, status, directory, created_at, updated_at, total_steps, completed_steps, pending_steps";
const PLAN_SUMMARIES_VIEW: &str = "plan_summaries";
const ALL_PLAN_SUMMARIES_VIEW: &str = "all_plan_summaries";

impl super::Database {
    /// Creates a new plan with the given title, optional description, and
    /// directory. The directory path will always be stored as an absolute path.
    /// If a relative path is provided, it will be converted to absolute using
    /// the current working directory. If no directory is provided, the current
    /// working directory will be used.
    pub fn create_plan(
        &mut self,
        title: &str,
        description: Option<&str>,
        directory: Option<&str>,
    ) -> Result<Plan> {
        let tx = self
            .connection
            .transaction()
            .db_context("Failed to begin transaction")?;

        let now = Timestamp::now();
        let now_str = now.to_string();

        // Ensure directory is always absolute
        let directory = Self::ensure_absolute_directory(directory)?;

        tx.execute(
            INSERT_PLAN_SQL,
            params![title, description, directory.as_deref(), &now_str, &now_str],
        )
        .map_err(|e| PlannerError::database_error("Failed to insert plan", e))?;

        let id = tx.last_insert_rowid() as u64;

        tx.commit().db_context("Failed to commit transaction")?;

        Ok(Plan {
            id,
            title: title.into(),
            description: description.map(String::from),
            status: PlanStatus::Active,
            directory,
            created_at: now,
            updated_at: now,
            steps: Vec::new(),
        })
    }

    /// Retrieves a plan by its ID.
    pub fn get_plan(&self, id: u64) -> Result<Option<Plan>> {
        let mut stmt = self
            .connection
            .prepare(SELECT_PLAN_SQL)
            .map_err(|e| PlannerError::database_error("Failed to prepare query", e))?;

        let mut plan = stmt
            .query_row(params![id as i64], |row| {
                let status_str: String = row.get(3)?;
                let status = status_str.parse::<PlanStatus>().map_err(|_| {
                    rusqlite::Error::FromSqlConversionFailure(
                        3,
                        Type::Text,
                        Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Invalid plan status: {status_str}"),
                        )),
                    )
                })?;

                Ok(Plan {
                    id: row.get::<_, i64>(0)? as u64,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    status,
                    directory: row.get(4)?,
                    created_at: row.get::<_, String>(5)?.parse::<Timestamp>().map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(5, Type::Text, Box::new(e))
                    })?,
                    updated_at: row.get::<_, String>(6)?.parse::<Timestamp>().map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(6, Type::Text, Box::new(e))
                    })?,
                    steps: Vec::new(),
                })
            })
            .optional()
            .map_err(|e| PlannerError::database_error("Failed to query plan", e))?;

        // Eagerly load steps if plan exists
        if let Some(ref mut plan) = plan {
            plan.steps = self.get_steps(plan.id)?;
        }

        Ok(plan)
    }

    /// Lists all plans with optional filtering.
    pub fn list_plans(&self, filter: Option<&PlanFilter>) -> Result<Vec<Plan>> {
        // Choose the appropriate view based on whether we want to include archived
        // plans
        let view_name = if filter.as_ref().is_some_and(|f| f.include_archived) {
            ALL_PLAN_SUMMARIES_VIEW
        } else {
            PLAN_SUMMARIES_VIEW
        };

        let mut query = format!("SELECT {PLAN_SUMMARY_COLUMNS} FROM {view_name}");

        let mut conditions = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(f) = filter {
            if let Some(ref title) = f.title_contains {
                conditions.push("title LIKE ?");
                params_vec.push(Box::new(format!("%{title}%")));
            }

            if let Some(ref directory) = f.directory {
                conditions.push("directory LIKE ?");
                params_vec.push(Box::new(format!("{directory}%")));
            }

            if let Some(ref after) = f.created_after {
                conditions.push("created_at >= ?");
                params_vec.push(Box::new(after.to_string()));
            }

            if let Some(ref before) = f.created_before {
                conditions.push("created_at <= ?");
                params_vec.push(Box::new(before.to_string()));
            }

            // Filter by specific status if provided
            if let Some(ref status) = f.status {
                conditions.push("status = ?");
                params_vec.push(Box::new(status.as_str().to_string()));
            }
        }

        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        query.push_str(" ORDER BY created_at DESC");

        let mut stmt = self
            .connection
            .prepare(&query)
            .map_err(|e| PlannerError::database_error("Failed to prepare query", e))?;

        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| &**b).collect();

        let plans_with_counts: Vec<(Plan, i64, i64)> = stmt
            .query_map(&params_refs[..], |row| {
                let status_str: String = row.get(3)?;
                let status = status_str.parse::<PlanStatus>().map_err(|_| {
                    rusqlite::Error::FromSqlConversionFailure(
                        3,
                        Type::Text,
                        Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Invalid plan status: {status_str}"),
                        )),
                    )
                })?;

                let total_steps: i64 = row.get(7)?;
                let completed_steps: i64 = row.get(8)?;
                let _pending_steps: i64 = row.get(9)?; // Not used but part of the view

                let plan = Plan {
                    id: row.get::<_, i64>(0)? as u64,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    status,
                    directory: row.get(4)?,
                    created_at: row.get::<_, String>(5)?.parse::<Timestamp>().map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(5, Type::Text, Box::new(e))
                    })?,
                    updated_at: row.get::<_, String>(6)?.parse::<Timestamp>().map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(6, Type::Text, Box::new(e))
                    })?,
                    steps: Vec::new(),
                };
                Ok((plan, total_steps, completed_steps))
            })
            .map_err(|e| PlannerError::database_error("Failed to query plans", e))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| PlannerError::database_error("Failed to fetch plans", e))?;

        // Apply completion filter if specified
        if let Some(f) = filter
            && let Some(ref completion) = f.completion_status {
                let mut filtered_plans =
                    self.filter_by_completion_with_counts(plans_with_counts, completion);
                // Eagerly load steps for each filtered plan
                for plan in &mut filtered_plans {
                    plan.steps = self.get_steps(plan.id)?;
                }
                return Ok(filtered_plans);
            }

        // If no completion filter, populate steps for each plan and return
        let mut plans: Vec<Plan> = plans_with_counts
            .into_iter()
            .map(|(plan, _, _)| plan)
            .collect();

        // Eagerly load steps for each plan
        for plan in &mut plans {
            plan.steps = self.get_steps(plan.id)?;
        }

        Ok(plans)
    }

    /// Filters plans by completion status using counts from the view.
    fn filter_by_completion_with_counts(
        &self,
        plans_with_counts: Vec<(Plan, i64, i64)>,
        filter: &CompletionFilter,
    ) -> Vec<Plan> {
        plans_with_counts
            .into_iter()
            .filter_map(|(plan, total_steps, completed_steps)| {
                let should_include = match filter {
                    CompletionFilter::Complete => total_steps > 0 && total_steps == completed_steps,
                    CompletionFilter::Incomplete => {
                        total_steps > 0 && completed_steps < total_steps
                    }
                    CompletionFilter::Empty => total_steps == 0,
                };

                should_include.then_some(plan)
            })
            .collect()
    }

    /// Archives a plan (soft delete).
    /// Returns the archived plan details if successful, None if the plan
    /// doesn't exist.
    pub fn archive_plan(&mut self, id: u64) -> Result<Option<Plan>> {
        let tx = self
            .connection
            .transaction()
            .db_context("Failed to begin transaction")?;

        let now = Timestamp::now().to_string();
        let rows_affected = tx
            .execute(
                UPDATE_PLAN_ARCHIVE_SQL,
                params![
                    PlanStatus::Archived.as_str(),
                    &now,
                    id as i64,
                    PlanStatus::Active.as_str()
                ],
            )
            .map_err(|e| PlannerError::database_error("Failed to archive plan", e))?;

        if rows_affected == 0 {
            // Check if plan exists
            let exists: bool = tx
                .query_row(CHECK_PLAN_EXISTS_SQL, params![id as i64], |row| row.get(0))
                .map_err(|e| PlannerError::database_error("Failed to check plan existence", e))?;

            if !exists {
                // Plan doesn't exist, return None
                return Ok(None);
            }
            // Plan exists but is already archived - still return its details
        }

        // Get the updated plan details
        let mut plan = tx
            .query_row(SELECT_PLAN_SQL, params![id as i64], |row| {
                let status_str: String = row.get(3)?;
                let status = status_str.parse::<PlanStatus>().map_err(|_| {
                    rusqlite::Error::FromSqlConversionFailure(
                        3,
                        Type::Text,
                        Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Invalid plan status: {status_str}"),
                        )),
                    )
                })?;

                Ok(Plan {
                    id: row.get::<_, i64>(0)? as u64,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    status,
                    directory: row.get(4)?,
                    created_at: row.get::<_, String>(5)?.parse::<Timestamp>().map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(5, Type::Text, Box::new(e))
                    })?,
                    updated_at: row.get::<_, String>(6)?.parse::<Timestamp>().map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(6, Type::Text, Box::new(e))
                    })?,
                    steps: Vec::new(),
                })
            })
            .optional()
            .map_err(|e| PlannerError::database_error("Failed to query archived plan", e))?;

        tx.commit().db_context("Failed to commit transaction")?;

        // Load steps for the plan if it exists
        if let Some(ref mut plan) = plan {
            plan.steps = self.get_steps(plan.id)?;
        }

        Ok(plan)
    }

    /// Unarchives a plan (restores from archive).
    /// Returns the unarchived plan details if successful, None if the plan
    /// doesn't exist.
    pub fn unarchive_plan(&mut self, id: u64) -> Result<Option<Plan>> {
        let tx = self
            .connection
            .transaction()
            .db_context("Failed to begin transaction")?;

        let now = Timestamp::now().to_string();
        let rows_affected = tx
            .execute(
                UPDATE_PLAN_UNARCHIVE_SQL,
                params![
                    PlanStatus::Active.as_str(),
                    &now,
                    id as i64,
                    PlanStatus::Archived.as_str()
                ],
            )
            .map_err(|e| PlannerError::database_error("Failed to unarchive plan", e))?;

        if rows_affected == 0 {
            // Check if plan exists
            let exists: bool = tx
                .query_row(CHECK_PLAN_EXISTS_SQL, params![id as i64], |row| row.get(0))
                .map_err(|e| PlannerError::database_error("Failed to check plan existence", e))?;

            if !exists {
                // Plan doesn't exist, return None
                return Ok(None);
            }
            // Plan exists but is already active - still return its details
        }

        // Get the updated plan details
        let mut plan = tx
            .query_row(SELECT_PLAN_SQL, params![id as i64], |row| {
                let status_str: String = row.get(3)?;
                let status = status_str.parse::<PlanStatus>().map_err(|_| {
                    rusqlite::Error::FromSqlConversionFailure(
                        3,
                        Type::Text,
                        Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Invalid plan status: {status_str}"),
                        )),
                    )
                })?;

                Ok(Plan {
                    id: row.get::<_, i64>(0)? as u64,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    status,
                    directory: row.get(4)?,
                    created_at: row.get::<_, String>(5)?.parse::<Timestamp>().map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(5, Type::Text, Box::new(e))
                    })?,
                    updated_at: row.get::<_, String>(6)?.parse::<Timestamp>().map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(6, Type::Text, Box::new(e))
                    })?,
                    steps: Vec::new(),
                })
            })
            .optional()
            .map_err(|e| PlannerError::database_error("Failed to query unarchived plan", e))?;

        tx.commit().db_context("Failed to commit transaction")?;

        // Load steps for the plan if it exists
        if let Some(ref mut plan) = plan {
            plan.steps = self.get_steps(plan.id)?;
        }

        Ok(plan)
    }

    /// Permanently deletes a plan and all its associated steps from the
    /// database. This operation cannot be undone.
    pub fn delete_plan(&mut self, id: u64) -> Result<()> {
        let tx = self
            .connection
            .transaction()
            .db_context("Failed to begin transaction")?;

        // Check if plan exists
        let exists: bool = tx
            .query_row(CHECK_PLAN_EXISTS_SQL, params![id as i64], |row| row.get(0))
            .map_err(|e| PlannerError::database_error("Failed to check plan existence", e))?;

        if !exists {
            return Err(PlannerError::PlanNotFound { id });
        }

        // Delete all steps associated with this plan first
        // (Foreign key constraints should handle this automatically, but we'll be
        // explicit)
        tx.execute(DELETE_PLAN_STEPS_SQL, params![id as i64])
            .map_err(|e| PlannerError::database_error("Failed to delete plan steps", e))?;

        // Delete the plan itself
        tx.execute(DELETE_PLAN_SQL, params![id as i64])
            .map_err(|e| PlannerError::database_error("Failed to delete plan", e))?;

        tx.commit().db_context("Failed to commit transaction")?;

        Ok(())
    }
}
