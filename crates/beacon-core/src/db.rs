//! Database operations for plans and steps.

use std::path::Path;

use jiff::Timestamp;
use rusqlite::{params, Connection, OptionalExtension};

use crate::{
    error::{PlannerError, Result},
    models::{CompletionFilter, Plan, PlanFilter, PlanStatus, Step, StepStatus},
};

/// Database connection and operations handler.
pub struct Database {
    connection: Connection,
}

impl Database {
    /// Creates a new database connection and initializes the schema.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let connection = Connection::open(path)
            .map_err(|e| PlannerError::database_error("Failed to open database connection", e))?;

        let db = Self { connection };
        db.initialize_schema()?;
        Ok(db)
    }

    /// Canonicalize a directory path for search purposes using the same logic
    /// as plan creation
    pub fn canonicalize_directory_for_search(&self, directory: &str) -> Result<String> {
        let path = std::path::Path::new(directory);
        if path.is_absolute() {
            Ok(directory.to_string())
        } else {
            // Convert relative path to absolute
            let cwd = std::env::current_dir().map_err(|_| PlannerError::InvalidInput {
                field: "directory".to_string(),
                reason: "Cannot resolve current working directory to make path absolute"
                    .to_string(),
            })?;
            let absolute_path = cwd.join(path);
            // Normalize the path to resolve ".." and "." components without requiring the
            // path to exist
            let normalized_path = Self::normalize_path(&absolute_path);
            normalized_path
                .to_str()
                .map(String::from)
                .ok_or_else(|| PlannerError::InvalidInput {
                    field: "directory".to_string(),
                    reason: "Cannot convert path to string".to_string(),
                })
        }
    }

    /// Normalizes a path by resolving "." and ".." components without requiring
    /// the path to exist
    fn normalize_path(path: &std::path::Path) -> std::path::PathBuf {
        path.components()
            .fold(std::path::PathBuf::new(), |mut acc, component| {
                match component {
                    std::path::Component::CurDir => acc, // Skip "." components
                    std::path::Component::ParentDir => {
                        // Handle ".." by popping the last component if possible
                        acc.pop();
                        acc
                    }
                    _ => {
                        // Keep all other components (Normal, RootDir, Prefix)
                        acc.push(component);
                        acc
                    }
                }
            })
    }

    /// Ensures a directory path is absolute. Converts relative paths to
    /// absolute using the current working directory.
    fn ensure_absolute_directory(directory: Option<&str>) -> Result<Option<String>> {
        match directory {
            Some(dir) => {
                let path = std::path::Path::new(dir);
                if path.is_absolute() {
                    Ok(Some(dir.to_string()))
                } else {
                    // Convert relative path to absolute
                    let cwd = std::env::current_dir().map_err(|_| PlannerError::InvalidInput {
                        field: "directory".to_string(),
                        reason: "Cannot resolve current working directory to make path absolute"
                            .to_string(),
                    })?;
                    let absolute_path = cwd.join(path);
                    // Normalize the path to resolve ".." and "." components without requiring the
                    // path to exist
                    let normalized_path = Self::normalize_path(&absolute_path);
                    Ok(normalized_path.to_str().map(String::from))
                }
            }
            None => {
                // Use current working directory as default
                let cwd = std::env::current_dir().map_err(|_| PlannerError::InvalidInput {
                    field: "directory".to_string(),
                    reason: "Cannot determine current working directory".to_string(),
                })?;
                let normalized_cwd = Self::normalize_path(&cwd);
                Ok(normalized_cwd.to_str().map(String::from))
            }
        }
    }

    /// Initializes the database schema using the embedded SQL file.
    fn initialize_schema(&self) -> Result<()> {
        // Enable foreign keys for this connection
        self.connection
            .execute("PRAGMA foreign_keys = ON", [])
            .map_err(|e| PlannerError::database_error("Failed to enable foreign keys", e))?;

        // Execute the schema SQL
        let schema_sql = include_str!("../assets/schema.sql");
        self.connection
            .execute_batch(schema_sql)
            .map_err(|e| PlannerError::database_error("Failed to initialize database schema", e))?;

        // Apply migrations for existing databases
        self.apply_migrations()?;

        Ok(())
    }

    /// Apply database migrations for existing databases
    fn apply_migrations(&self) -> Result<()> {
        // Check if result column exists in steps table
        let has_result_column: bool = self
            .connection
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('steps') WHERE name = 'result'",
                [],
                |row| row.get(0),
            )
            .map(|count: i64| count > 0)
            .unwrap_or(false);

        // Add result column if it doesn't exist
        if !has_result_column {
            self.connection
                .execute("ALTER TABLE steps ADD COLUMN result TEXT", [])
                .map_err(|e| {
                    PlannerError::database_error("Failed to add result column to steps table", e)
                })?;
        }

        Ok(())
    }

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
            .map_err(|e| PlannerError::database_error("Failed to begin transaction", e))?;

        let now = Timestamp::now();
        let now_str = now.to_string();

        // Ensure directory is always absolute
        let directory = Self::ensure_absolute_directory(directory)?;

        tx.execute(
            "INSERT INTO plans (title, description, directory, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![title, description, directory.as_deref(), &now_str, &now_str],
        )
        .map_err(|e| PlannerError::database_error("Failed to insert plan", e))?;

        let id = tx.last_insert_rowid() as u64;

        tx.commit()
            .map_err(|e| PlannerError::database_error("Failed to commit transaction", e))?;

        Ok(Plan {
            id,
            title: title.to_string(),
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
            .prepare(
                "SELECT id, title, description, status, directory, created_at, updated_at FROM plans WHERE id = ?1",
            )
            .map_err(|e| PlannerError::database_error("Failed to prepare query", e))?;

        let plan = stmt
            .query_row(params![id as i64], |row| {
                let status_str: String = row.get(3)?;
                let status = PlanStatus::parse(&status_str).ok_or_else(|| {
                    rusqlite::Error::FromSqlConversionFailure(
                        3,
                        rusqlite::types::Type::Text,
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
                        rusqlite::Error::FromSqlConversionFailure(
                            5,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?,
                    updated_at: row.get::<_, String>(6)?.parse::<Timestamp>().map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            6,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?,
                    steps: Vec::new(),
                })
            })
            .optional()
            .map_err(|e| PlannerError::database_error("Failed to query plan", e))?;

        Ok(plan)
    }

    /// Lists all plans with optional filtering.
    pub fn list_plans(&self, filter: Option<&PlanFilter>) -> Result<Vec<Plan>> {
        // Choose the appropriate view based on whether we want to include archived
        // plans
        let view_name = if filter.as_ref().is_some_and(|f| f.include_archived) {
            "all_plan_summaries"
        } else {
            "plan_summaries" // Only shows active plans
        };

        let mut query = format!(
            "SELECT id, title, description, status, directory, created_at, updated_at, total_steps, completed_steps, pending_steps
             FROM {view_name}"
        );

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
                let status = PlanStatus::parse(&status_str).ok_or_else(|| {
                    rusqlite::Error::FromSqlConversionFailure(
                        3,
                        rusqlite::types::Type::Text,
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
                        rusqlite::Error::FromSqlConversionFailure(
                            5,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?,
                    updated_at: row.get::<_, String>(6)?.parse::<Timestamp>().map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            6,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?,
                    steps: Vec::new(),
                };
                Ok((plan, total_steps, completed_steps))
            })
            .map_err(|e| PlannerError::database_error("Failed to query plans", e))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| PlannerError::database_error("Failed to fetch plans", e))?;

        // Apply completion filter if specified
        if let Some(f) = filter {
            if let Some(ref completion) = f.completion_status {
                return Ok(self.filter_by_completion_with_counts(plans_with_counts, completion));
            }
        }

        // If no completion filter, just return the plans
        let plans = plans_with_counts
            .into_iter()
            .map(|(plan, _, _)| plan)
            .collect();

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

                if should_include {
                    Some(plan)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Archives a plan (soft delete).
    pub fn archive_plan(&mut self, id: u64) -> Result<()> {
        let tx = self
            .connection
            .transaction()
            .map_err(|e| PlannerError::database_error("Failed to begin transaction", e))?;

        let now = Timestamp::now().to_string();
        let rows_affected = tx
            .execute(
                "UPDATE plans SET status = ?1, updated_at = ?2 WHERE id = ?3 AND status = ?4",
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
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM plans WHERE id = ?1)",
                    params![id as i64],
                    |row| row.get(0),
                )
                .map_err(|e| PlannerError::database_error("Failed to check plan existence", e))?;

            if !exists {
                return Err(PlannerError::PlanNotFound { id });
            }
            // Plan exists but is already archived, which is okay
        }

        tx.commit()
            .map_err(|e| PlannerError::database_error("Failed to commit transaction", e))?;

        Ok(())
    }

    /// Unarchives a plan (restores from archive).
    pub fn unarchive_plan(&mut self, id: u64) -> Result<()> {
        let tx = self
            .connection
            .transaction()
            .map_err(|e| PlannerError::database_error("Failed to begin transaction", e))?;

        let now = Timestamp::now().to_string();
        let rows_affected = tx
            .execute(
                "UPDATE plans SET status = ?1, updated_at = ?2 WHERE id = ?3 AND status = ?4",
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
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM plans WHERE id = ?1)",
                    params![id as i64],
                    |row| row.get(0),
                )
                .map_err(|e| PlannerError::database_error("Failed to check plan existence", e))?;

            if !exists {
                return Err(PlannerError::PlanNotFound { id });
            }
            // Plan exists but is already active, which is okay
        }

        tx.commit()
            .map_err(|e| PlannerError::database_error("Failed to commit transaction", e))?;

        Ok(())
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
            .map_err(|e| PlannerError::database_error("Failed to begin transaction", e))?;

        // Check if plan exists
        let plan_exists: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM plans WHERE id = ?1)",
                params![plan_id as i64],
                |row| row.get(0),
            )
            .map_err(|e| PlannerError::database_error("Failed to check plan existence", e))?;

        if !plan_exists {
            return Err(PlannerError::PlanNotFound { id: plan_id });
        }

        // Get the next order number
        let next_order: i64 = tx
            .query_row(
                "SELECT COALESCE(MAX(step_order), -1) + 1 FROM steps WHERE plan_id = ?1",
                params![plan_id as i64],
                |row| row.get(0),
            )
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
            "INSERT INTO steps (plan_id, title, description, acceptance_criteria, step_references, status, result, step_order, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                plan_id as i64,
                title,
                description,
                acceptance_criteria,
                references_str.as_deref(),
                "todo",
                None::<String>,  // result is NULL for new steps
                next_order,
                &now_str,
                &now_str
            ],
        )
        .map_err(|e| PlannerError::database_error("Failed to insert step", e))?;

        let id = tx.last_insert_rowid() as u64;

        // Update plan's updated_at
        tx.execute(
            "UPDATE plans SET updated_at = ?1 WHERE id = ?2",
            params![&now_str, plan_id as i64],
        )
        .map_err(|e| PlannerError::database_error("Failed to update plan timestamp", e))?;

        tx.commit()
            .map_err(|e| PlannerError::database_error("Failed to commit transaction", e))?;

        Ok(Step {
            id,
            plan_id,
            title: title.to_string(),
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
            .map_err(|e| PlannerError::database_error("Failed to begin transaction", e))?;

        // Check if plan exists
        let plan_exists: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM plans WHERE id = ?1)",
                params![plan_id as i64],
                |row| row.get(0),
            )
            .map_err(|e| PlannerError::database_error("Failed to check plan existence", e))?;

        if !plan_exists {
            return Err(PlannerError::PlanNotFound { id: plan_id });
        }

        // Get the max order for validation
        let max_order: Option<i64> = tx
            .query_row(
                "SELECT MAX(step_order) FROM steps WHERE plan_id = ?1",
                params![plan_id as i64],
                |row| row.get(0),
            )
            .map_err(|e| PlannerError::database_error("Failed to get max step order", e))?;

        // Validate position - allow inserting at the end (position == count)
        let step_count = max_order.map(|m| m + 1).unwrap_or(0) as u32;
        if position > step_count {
            return Err(PlannerError::InvalidInput {
                field: "position".to_string(),
                reason: format!("Position {position} is out of range. Plan has {step_count} steps"),
            });
        }

        // Update existing steps' order to make room for the new step
        tx.execute(
            "UPDATE steps SET step_order = step_order + 1 
             WHERE plan_id = ?1 AND step_order >= ?2",
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
            "INSERT INTO steps (plan_id, title, description, acceptance_criteria, step_references, status, result, step_order, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                plan_id as i64,
                title,
                description,
                acceptance_criteria,
                references_str.as_deref(),
                "todo",
                None::<String>,  // result is NULL for new steps
                position as i64,
                &now_str,
                &now_str
            ],
        )
        .map_err(|e| PlannerError::database_error("Failed to insert step", e))?;

        let id = tx.last_insert_rowid() as u64;

        // Update plan's updated_at
        tx.execute(
            "UPDATE plans SET updated_at = ?1 WHERE id = ?2",
            params![&now_str, plan_id as i64],
        )
        .map_err(|e| PlannerError::database_error("Failed to update plan timestamp", e))?;

        tx.commit()
            .map_err(|e| PlannerError::database_error("Failed to commit transaction", e))?;

        Ok(Step {
            id,
            plan_id,
            title: title.to_string(),
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

    /// Updates step details (title, description, acceptance criteria,
    /// references, status, and result).
    /// When changing status to Done, result is required.
    /// Result is ignored when changing to Todo or InProgress.
    pub fn update_step(
        &mut self,
        step_id: u64,
        title: Option<String>,
        description: Option<String>,
        acceptance_criteria: Option<String>,
        references: Option<Vec<String>>,
        status: Option<StepStatus>,
        result: Option<String>,
    ) -> Result<()> {
        // Validate result requirement when changing status to Done
        if let Some(StepStatus::Done) = status {
            if result.is_none() {
                return Err(PlannerError::InvalidInput {
                    field: "result".to_string(),
                    reason: "Result description is required when marking a step as done"
                        .to_string(),
                });
            }
        }

        // Check if there's anything to update
        if title.is_none()
            && description.is_none()
            && acceptance_criteria.is_none()
            && references.is_none()
            && status.is_none()
            && result.is_none()
        {
            return Ok(());
        }

        let tx = self
            .connection
            .transaction()
            .map_err(|e| PlannerError::database_error("Failed to begin transaction", e))?;

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
            let mut stmt = tx
                .prepare("SELECT title, description, acceptance_criteria, step_references, status, result FROM steps WHERE id = ?1")
                .map_err(|e| PlannerError::database_error("Failed to prepare select statement", e))?;

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
        let new_title = title.unwrap_or(current_title);
        let new_description = description.or(current_desc);
        let new_criteria = acceptance_criteria.or(current_criteria);
        let new_references = references.map(|refs| refs.join(",")).or(current_refs);
        let new_status_str = status
            .map(|s| s.as_str().to_string())
            .unwrap_or(current_status.clone());

        // Determine the result value based on the status change
        let new_result = if let Some(new_status) = status {
            match new_status {
                StepStatus::Done => result, // Use provided result (already validated as required)
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
            "UPDATE steps SET title = ?1, description = ?2, acceptance_criteria = ?3, step_references = ?4, status = ?5, result = ?6, updated_at = ?7 WHERE id = ?8",
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
            "UPDATE plans SET updated_at = ?1 
             WHERE id = (SELECT plan_id FROM steps WHERE id = ?2)",
            params![&now_str, step_id as i64],
        )
        .map_err(|e| PlannerError::database_error("Failed to update plan timestamp", e))?;

        tx.commit()
            .map_err(|e| PlannerError::database_error("Failed to commit transaction", e))?;

        Ok(())
    }

    /// Retrieves all steps for a given plan.
    pub fn get_steps(&self, plan_id: u64) -> Result<Vec<Step>> {
        let mut stmt = self
            .connection
            .prepare(
                "SELECT id, plan_id, title, description, acceptance_criteria, step_references, status, result, step_order, created_at, updated_at
                 FROM steps WHERE plan_id = ?1 ORDER BY step_order",
            )
            .map_err(|e| PlannerError::database_error("Failed to prepare query", e))?;

        let steps = stmt
            .query_map(params![plan_id as i64], |row| {
                let status_str: String = row.get(6)?;
                let status = StepStatus::parse(&status_str).ok_or_else(|| {
                    rusqlite::Error::FromSqlConversionFailure(
                        6,
                        rusqlite::types::Type::Text,
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
                        rusqlite::Error::FromSqlConversionFailure(
                            9,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?,
                    updated_at: row
                        .get::<_, String>(10)?
                        .parse::<Timestamp>()
                        .map_err(|e| {
                            rusqlite::Error::FromSqlConversionFailure(
                                10,
                                rusqlite::types::Type::Text,
                                Box::new(e),
                            )
                        })?,
                })
            })
            .map_err(|e| PlannerError::database_error("Failed to query steps", e))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| PlannerError::database_error("Failed to fetch steps", e))?;

        Ok(steps)
    }

    /// Retrieves a single step by its ID.
    pub fn get_step(&self, step_id: u64) -> Result<Option<Step>> {
        let mut stmt = self
            .connection
            .prepare(
                "SELECT id, plan_id, title, description, acceptance_criteria, step_references, status, result, step_order, created_at, updated_at
                 FROM steps WHERE id = ?1",
            )
            .map_err(|e| PlannerError::database_error("Failed to prepare query", e))?;

        let step = stmt
            .query_row(params![step_id as i64], |row| {
                let status_str: String = row.get(6)?;
                let status = StepStatus::parse(&status_str).ok_or_else(|| {
                    rusqlite::Error::FromSqlConversionFailure(
                        6,
                        rusqlite::types::Type::Text,
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
                        rusqlite::Error::FromSqlConversionFailure(
                            9,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?,
                    updated_at: row
                        .get::<_, String>(10)?
                        .parse::<Timestamp>()
                        .map_err(|e| {
                            rusqlite::Error::FromSqlConversionFailure(
                                10,
                                rusqlite::types::Type::Text,
                                Box::new(e),
                            )
                        })?,
                })
            })
            .optional()
            .map_err(|e| PlannerError::database_error("Failed to get step", e))?;

        Ok(step)
    }

    /// Atomically claims a step for processing by transitioning it from Todo to
    /// InProgress. Returns Ok(true) if the step was successfully claimed,
    /// Ok(false) if the step was not in Todo status.
    pub fn claim_step(&mut self, step_id: u64) -> Result<bool> {
        let tx = self
            .connection
            .transaction()
            .map_err(|e| PlannerError::database_error("Failed to begin transaction", e))?;

        // Check current status and update atomically
        let current_status: Option<String> = tx
            .query_row(
                "SELECT status FROM steps WHERE id = ?1",
                params![step_id as i64],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| PlannerError::database_error("Failed to query step status", e))?;

        match current_status {
            None => Err(PlannerError::StepNotFound { id: step_id }),
            Some(status) if status == "todo" => {
                // Atomically update to in_progress
                let now_str = Timestamp::now().to_string();
                tx.execute(
                    "UPDATE steps SET status = ?1, updated_at = ?2 WHERE id = ?3 AND status = ?4",
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
                    "UPDATE plans SET updated_at = ?1 
                     WHERE id = (SELECT plan_id FROM steps WHERE id = ?2)",
                    params![&now_str, step_id as i64],
                )
                .map_err(|e| PlannerError::database_error("Failed to update plan timestamp", e))?;

                tx.commit()
                    .map_err(|e| PlannerError::database_error("Failed to commit transaction", e))?;

                Ok(true)
            }
            _ => {
                // Step is not in Todo status, cannot claim
                Ok(false)
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
            .map_err(|e| PlannerError::database_error("Failed to begin transaction", e))?;

        // Get both steps' info to verify they're in the same plan
        let (plan_id1, order1): (i64, i64) = tx
            .query_row(
                "SELECT plan_id, step_order FROM steps WHERE id = ?1",
                params![step_id1 as i64],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|e| {
                if matches!(e, rusqlite::Error::QueryReturnedNoRows) {
                    PlannerError::StepNotFound { id: step_id1 }
                } else {
                    PlannerError::database_error("Failed to query first step", e)
                }
            })?;

        let (plan_id2, order2): (i64, i64) = tx
            .query_row(
                "SELECT plan_id, step_order FROM steps WHERE id = ?1",
                params![step_id2 as i64],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
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
                field: "step_ids".to_string(),
                reason: "Steps must be from the same plan to swap".to_string(),
            });
        }

        // Swap the orders
        let now_str = Timestamp::now().to_string();

        // Use a temporary negative value to avoid unique constraint violation
        tx.execute(
            "UPDATE steps SET step_order = -1, updated_at = ?1 WHERE id = ?2",
            params![&now_str, step_id1 as i64],
        )
        .map_err(|e| PlannerError::database_error("Failed to update first step order", e))?;

        tx.execute(
            "UPDATE steps SET step_order = ?1, updated_at = ?2 WHERE id = ?3",
            params![order1, &now_str, step_id2 as i64],
        )
        .map_err(|e| PlannerError::database_error("Failed to update second step order", e))?;

        tx.execute(
            "UPDATE steps SET step_order = ?1, updated_at = ?2 WHERE id = ?3",
            params![order2, &now_str, step_id1 as i64],
        )
        .map_err(|e| PlannerError::database_error("Failed to update first step final order", e))?;

        // Update plan's updated_at
        tx.execute(
            "UPDATE plans SET updated_at = ?1 WHERE id = ?2",
            params![&now_str, plan_id1],
        )
        .map_err(|e| PlannerError::database_error("Failed to update plan timestamp", e))?;

        tx.commit()
            .map_err(|e| PlannerError::database_error("Failed to commit transaction", e))?;

        Ok(())
    }

    /// Removes a step from a plan.
    pub fn remove_step(&mut self, step_id: u64) -> Result<()> {
        let tx = self
            .connection
            .transaction()
            .map_err(|e| PlannerError::database_error("Failed to begin transaction", e))?;

        // Get plan_id and order before deletion
        let (plan_id, step_order): (i64, i64) = tx
            .query_row(
                "SELECT plan_id, step_order FROM steps WHERE id = ?1",
                params![step_id as i64],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|e| {
                if matches!(e, rusqlite::Error::QueryReturnedNoRows) {
                    PlannerError::StepNotFound { id: step_id }
                } else {
                    PlannerError::database_error("Failed to query step", e)
                }
            })?;

        // Delete the step
        tx.execute("DELETE FROM steps WHERE id = ?1", params![step_id as i64])
            .map_err(|e| PlannerError::database_error("Failed to delete step", e))?;

        // Update order of subsequent steps
        tx.execute(
            "UPDATE steps SET step_order = step_order - 1 
             WHERE plan_id = ?1 AND step_order > ?2",
            params![plan_id, step_order],
        )
        .map_err(|e| PlannerError::database_error("Failed to update step orders", e))?;

        // Update plan's updated_at
        let now_str = Timestamp::now().to_string();
        tx.execute(
            "UPDATE plans SET updated_at = ?1 WHERE id = ?2",
            params![&now_str, plan_id],
        )
        .map_err(|e| PlannerError::database_error("Failed to update plan timestamp", e))?;

        tx.commit()
            .map_err(|e| PlannerError::database_error("Failed to commit transaction", e))?;

        Ok(())
    }
}
