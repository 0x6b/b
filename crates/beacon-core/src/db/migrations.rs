//! Database schema initialization and migrations.

use crate::error::{PlannerError, Result, DatabaseResultExt};

impl super::Database {
    /// Initializes the database schema using the embedded SQL file.
    pub(super) fn initialize_schema(&self) -> Result<()> {
        // Enable foreign keys for this connection
        self.connection
            .execute("PRAGMA foreign_keys = ON", [])
            .db_context("Failed to enable foreign keys")?;

        // Execute the schema SQL
        let schema_sql = include_str!("../../assets/schema.sql");
        self.connection
            .execute_batch(schema_sql)
            .db_context("Failed to initialize database schema")?;

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
}
