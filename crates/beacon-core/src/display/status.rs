//! Status and confirmation message types for operation feedback.

use std::fmt;

/// Wrapper type for displaying operation confirmation messages.
///
/// This provides consistent formatting for operations that require
/// user confirmation or status updates.
pub struct OperationStatus {
    pub message: String,
    pub success: bool,
}

impl OperationStatus {
    /// Create a new success status.
    pub fn success(message: String) -> Self {
        Self {
            message,
            success: true,
        }
    }

    /// Create a new failure status.
    pub fn failure(message: String) -> Self {
        Self {
            message,
            success: false,
        }
    }
}

impl fmt::Display for OperationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} {}", if self.success { "Success:" } else { "Error:" }, self.message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_status_display() {
        let success = OperationStatus::success("Operation completed".to_string());
        assert!(format!("{success}").contains("Success:"));

        let failure = OperationStatus::failure("Operation failed".to_string());
        assert!(format!("{failure}").contains("Error:"));
    }
}
