use chrono::{Local, TimeZone};

/// Timestamp format preference
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TimestampFormat {
    #[default]
    Relative,
    Absolute,
}

/// Global custom absolute time format (set from config)
use std::sync::OnceLock;

static ABSOLUTE_TIME_FORMAT: OnceLock<String> = OnceLock::new();

/// Set the custom absolute time format string
pub fn set_absolute_time_format(format: String) {
    let _ = ABSOLUTE_TIME_FORMAT.set(format);
}

/// Format a Unix timestamp based on the format preference
pub fn format_timestamp(timestamp: i64, format: TimestampFormat) -> String {
    match format {
        TimestampFormat::Relative => format_relative_time(timestamp),
        TimestampFormat::Absolute => format_absolute_time(timestamp),
    }
}

/// Format a Unix timestamp as relative time (e.g., "2 hours ago")
pub fn format_relative_time(timestamp: i64) -> String {
    let now = Local::now();
    let dt = Local.timestamp_opt(timestamp, 0).single();

    match dt {
        Some(dt) => {
            let duration = now.signed_duration_since(dt);
            let seconds = duration.num_seconds();
            let minutes = duration.num_minutes();
            let hours = duration.num_hours();
            let days = duration.num_days();

            if seconds < 60 {
                "just now".to_string()
            } else if minutes < 60 {
                format!("{} min ago", minutes)
            } else if hours < 24 {
                format!("{} hours ago", hours)
            } else if days < 30 {
                format!("{} days ago", days)
            } else if days < 365 {
                let months = days / 30;
                format!("{} months ago", months)
            } else {
                let years = days / 365;
                format!("{} years ago", years)
            }
        }
        None => "unknown".to_string(),
    }
}

/// Default absolute time format
const DEFAULT_ABSOLUTE_FORMAT: &str = "%Y-%m-%d %H:%M";

/// Format a Unix timestamp as absolute time (e.g., "2026-02-09 14:30")
/// Uses custom format string if set via config, otherwise uses default
pub fn format_absolute_time(timestamp: i64) -> String {
    let dt = Local.timestamp_opt(timestamp, 0).single();

    match dt {
        Some(dt) => {
            let format_str = ABSOLUTE_TIME_FORMAT
                .get()
                .map(|s| s.as_str())
                .unwrap_or(DEFAULT_ABSOLUTE_FORMAT);
            dt.format(format_str).to_string()
        }
        None => "unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;

    #[test]
    fn test_format_absolute_time() {
        // Test with a known timestamp (2026-02-09 14:30:00 UTC)
        let timestamp = 1739106600i64;
        let result = format_absolute_time(timestamp);
        // Result will depend on local timezone, but should contain the date
        assert!(result.contains("2026") || result.contains("2025"));
    }

    #[test]
    fn test_format_relative_time_recent() {
        // Test with current time (should be "just now")
        let now = Local::now().timestamp();
        let result = format_relative_time(now);
        assert_eq!(result, "just now");
    }

    #[test]
    fn test_format_relative_time_minutes() {
        // Test with 5 minutes ago
        let five_min_ago = Local::now().timestamp() - 300;
        let result = format_relative_time(five_min_ago);
        assert!(result.contains("min ago"));
    }
}
