//! Database operations and SQLite management for plans and steps.
//!
//! This module provides low-level database operations for the Beacon task
//! planning system. It handles SQLite database connections, schema management,
//! and provides specialized query interfaces for plans and steps.

use std::path::Path;

use rusqlite::Connection;

use crate::error::{DatabaseResultExt, Result};

pub mod migrations;
pub mod plan_queries;
pub mod step_queries;
pub mod utils;

/// Database connection and operations handler.
pub struct Database {
    connection: Connection,
}

impl Database {
    /// Creates a new database connection and initializes the schema.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let connection = Connection::open(path).db_context("Failed to open database connection")?;

        let db = Self { connection };
        db.initialize_schema()?;
        Ok(db)
    }
}
