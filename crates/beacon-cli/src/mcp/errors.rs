//! Error handling utilities for MCP server

use beacon_core::PlannerError;
use rmcp::ErrorData;

/// Helper to convert planner errors to MCP errors
pub fn to_mcp_error(message: &str, error: PlannerError) -> ErrorData {
    ErrorData::internal_error(format!("{}: {}", message, error), None)
}
