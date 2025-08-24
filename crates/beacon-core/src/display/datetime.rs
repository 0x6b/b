//! DateTime display utilities.
//!
//! This module provides wrapper types for formatting timestamps in a consistent,
//! human-readable format using system timezone.

use std::fmt;
use jiff::{tz::TimeZone, Timestamp};

/// A wrapper around `Timestamp` that provides system timezone formatting via the `Display` trait.
///
/// This struct encapsulates a `Timestamp` reference and implements `Display` to format it
/// in a consistent, human-readable format using the system timezone. It provides an ergonomic
/// and type-safe approach to timestamp formatting in display contexts.
///
/// # Format
///
/// The display format follows the pattern: `YYYY-MM-DD HH:MM:SS TZ`
/// - Year, month, and day are zero-padded
/// - Time is in 24-hour format with zero-padded components
/// - Timezone abbreviation is included (e.g., UTC, EST, JST)
///
/// # Examples
///
/// ```rust
/// use beacon_core::display::LocalDateTime;
/// use jiff::Timestamp;
///
/// let timestamp = Timestamp::from_second(1640995200).unwrap(); // 2022-01-01 00:00:00 UTC
/// let local_dt = LocalDateTime(&timestamp);
/// 
/// // Display automatically formats using system timezone
/// println!("Created: {}", local_dt);
/// // Output (example): "Created: 2022-01-01 09:00:00 JST"
///
/// // Can be used in format strings and templates
/// let message = format!("Plan updated at {}", LocalDateTime(&timestamp));
/// ```
///
/// # Design Rationale
///
/// This wrapper provides several advantages over direct function calls:
/// - **Type Safety**: Encapsulates formatting logic in a dedicated type
/// - **Ergonomics**: Integrates seamlessly with `Display` trait usage
/// - **Consistency**: Ensures uniform timestamp formatting across the application
/// - **Future-proofing**: Allows format changes without affecting call sites
///
/// # Performance
///
/// The wrapper is zero-cost at runtime - it only holds a reference to the timestamp
/// and performs formatting only when `Display::fmt` is called.
pub struct LocalDateTime<'a>(pub &'a Timestamp);


impl<'a> fmt::Display for LocalDateTime<'a> {
    /// Format the wrapped timestamp using system timezone in YYYY-MM-DD HH:MM:SS TZ format.
    ///
    /// This implementation converts the UTC timestamp to the system timezone and formats it
    /// in a consistent, human-readable format.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter to write the timestamp string to
    ///
    /// # Returns
    ///
    /// `fmt::Result` indicating success or failure of the formatting operation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use beacon_core::display::LocalDateTime;
    /// use jiff::Timestamp;
    ///
    /// let timestamp = Timestamp::from_second(1640995200).unwrap();
    /// let local_dt = LocalDateTime(&timestamp);
    /// 
    /// // Formats with system timezone
    /// println!("{}", local_dt); // e.g., "2022-01-01 09:00:00 JST"
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.0.to_zoned(TimeZone::system()).strftime("%Y-%m-%d %H:%M:%S %Z")
        )
    }
}