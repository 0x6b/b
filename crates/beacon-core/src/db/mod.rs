//! Database operations and SQLite management for plans and steps.
//!
//! This module provides low-level database operations for the Beacon task
//! planning system. It handles SQLite database connections, schema management,
//! and provides specialized query interfaces for plans and steps.
//!
//! # Architecture Overview
//!
//! The database layer is organized into specialized modules:
//!
//! ```text
//! ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
//! │   Migrations    │    │     Queries     │    │    Database     │
//! │   (schema)      │───▶│ (plan_queries,  │───▶│  (Connection)   │
//! │                 │    │  step_queries)  │    │                 │
//! └─────────────────┘    └─────────────────┘    └─────────────────┘
//!     Schema Updates        Typed Queries         SQLite Storage
//! ```
//!
//! ## Submodules
//!
//! - [`migrations`]: Database schema initialization and version management
//! - [`plan_queries`]: Specialized queries for plan operations (CRUD, filtering, search)
//! - [`step_queries`]: Specialized queries for step operations (CRUD, ordering, status updates)
//! - [`utils`]: Common database utilities and helper functions
//!
//! ## Features
//!
//! ### Schema Management
//! - **Automatic Initialization**: Database schema created on first connection
//! - **Version Control**: Future migration support for schema updates
//! - **Constraint Enforcement**: Foreign keys, uniqueness, and data integrity
//!
//! ### Query Organization
//! - **Type Safety**: Strongly typed query parameters and results
//! - **Transaction Support**: Proper transaction boundaries for multi-operation workflows
//! - **Error Handling**: Comprehensive error propagation with context
//! - **Performance**: Optimized queries with proper indexing
//!
//! ### Data Integrity
//! - **Foreign Key Constraints**: Plans and steps properly linked
//! - **Status Validation**: Step and plan statuses enforced at database level
//! - **Ordering Consistency**: Step order maintained automatically
//! - **Audit Trail**: Created/updated timestamps for all records
//!
//! # Usage
//!
//! The [`Database`] struct is typically used through the planner layer rather
//! than directly. However, for testing or advanced use cases:
//!
//! ```rust
//! use beacon_core::db::Database;
//!
//! # async fn example() -> beacon_core::Result<()> {
//! // Create database connection (initializes schema)
//! let db = Database::new("test.db")?;
//!
//! // Database operations are typically done through
//! // the query modules or planner methods
//! # Ok(())
//! # }
//! ```
//!
//! ## Database Schema
//!
//! The database uses SQLite with the following key tables:
//!
//! - **plans**: Main plan storage with metadata and status tracking
//! - **steps**: Step storage with plan relationships and ordering
//! - **Indexes**: Performance optimizations for common query patterns
//!
//! See [`migrations`] for complete schema details.

use std::path::Path;

use rusqlite::Connection;

use crate::error::{Result, ResultExt};

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
        let connection = Connection::open(path).db_err("Failed to open database connection")?;

        let db = Self { connection };
        db.initialize_schema()?;
        Ok(db)
    }

}