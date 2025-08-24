//! DateTime display utilities.
//!
//! This module provides wrapper types for formatting timestamps in a
//! consistent, human-readable format using system timezone.

use std::fmt;

use jiff::{Timestamp, tz::TimeZone};

/// A wrapper around `Timestamp` that provides system timezone formatting via
/// the `Display` trait.
///
/// This struct encapsulates a `Timestamp` reference and implements `Display` to
/// format it in a consistent, human-readable format using the system timezone.
/// It provides an ergonomic and type-safe approach to timestamp formatting in
/// display contexts.
///
/// # Format
///
/// The display format follows the pattern: `YYYY-MM-DD HH:MM:SS TZ`
/// - Year, month, and day are zero-padded
/// - Time is in 24-hour format with zero-padded components
/// - Timezone abbreviation is included (e.g., UTC, EST, JST)
pub struct LocalDateTime<'a>(pub &'a Timestamp);

impl<'a> fmt::Display for LocalDateTime<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .to_zoned(TimeZone::system())
                .strftime("%Y-%m-%d %H:%M:%S %Z")
        )
    }
}
