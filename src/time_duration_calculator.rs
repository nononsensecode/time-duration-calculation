use std::env;
use std::process;

/// Represents an error that can occur during time parsing or calculation.
#[derive(Debug, PartialEq)]
struct TimeError(String);

impl std::fmt::Display for TimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for TimeError {}

/// Parses a time string in "HH:MMam" or "HH:MMpm" format into total minutes from midnight.
///
/// # Arguments
/// * `time_str` - A string slice representing the time (e.g., "09:00AM").
///
/// # Returns
/// A `Result` containing the total minutes from midnight (u32) if successful,
/// or a `TimeError` if parsing fails.
fn parse_time_to_minutes(time_str: &str) -> Result<u32, TimeError> {
    if time_str.len() != 7 {
        return Err(TimeError(format!(
            "Invalid time format length for '{}'. Expected HH:MMam/pm (7 characters).",
            time_str
        )));
    }

    // Using .get() for safe slicing, converting Option to Result
    let h_str = time_str
        .get(0..2)
        .ok_or_else(|| TimeError("Time string too short for hour.".to_string()))?;
    let colon = time_str
        .get(2..3)
        .ok_or_else(|| TimeError("Time string too short for colon.".to_string()))?;
    let m_str = time_str
        .get(3..5)
        .ok_or_else(|| TimeError("Time string too short for minute.".to_string()))?;
    let ampm_indicator = time_str
        .get(5..7)
        .ok_or_else(|| TimeError("Time string too short for AM/PM indicator.".to_string()))?;

    if colon != ":" {
        return Err(TimeError(format!(
            "Invalid time format: Missing colon in '{}'. Expected HH:MMam/pm.",
            time_str
        )));
    }

    let hour12: u32 = h_str.parse().map_err(|_| {
        TimeError(format!(
            "Invalid hour value: '{}'. Hour must be a number.",
            h_str
        ))
    })?;
    let minute: u32 = m_str.parse().map_err(|_| {
        TimeError(format!(
            "Invalid minute value: '{}'. Minute must be a number.",
            m_str
        ))
    })?;

    if !(1..=12).contains(&hour12) {
        return Err(TimeError(format!(
            "Invalid hour: {}. Hour must be between 01 and 12 for 12-hour format.",
            hour12
        )));
    }
    if minute > 59 {
        return Err(TimeError(format!(
            "Invalid minute: {}. Minute must be between 00 and 59.",
            minute
        )));
    }

    let ampm = ampm_indicator.to_uppercase();
    let mut hour24 = hour12;

    match ampm.as_str() {
        "AM" => {
            if hour12 == 12 {
                // 12 AM (midnight) is 00 hours in 24-hour format
                hour24 = 0;
            }
            // For 1 AM to 11 AM, hour12 is already the correct hour24
        }
        "PM" => {
            if hour12 != 12 {
                // For 1 PM to 11 PM, add 12 hours
                hour24 += 12;
            }
            // 12 PM (noon) is 12 hours in 24-hour format, so no change needed if hour12 is 12
        }
        _ => {
            return Err(TimeError(format!(
                "Invalid AM/PM indicator: '{}'. Must be 'AM' or 'PM'.",
                ampm_indicator
            )))
        }
    }

    Ok(hour24 * 60 + minute)
}

/// Calculates the difference in hours between two time strings.
/// The input format is "HH:MMa-HH:MMa" (e.g., "09:00AM-05:30PM").
/// The calculation assumes the time range is within a single day.
///
/// # Arguments
/// * `range_str` - A string slice representing the time range.
///
/// # Returns
/// A `Result` containing the difference in hours (f64) if successful,
/// or a `TimeError` if parsing or calculation fails.
fn calculate_time_difference_from_range_str(range_str: &str) -> Result<f64, TimeError> {
    let parts: Vec<&str> = range_str.split('-').collect();
    if parts.len() != 2 {
        return Err(TimeError(format!(
            "Invalid input format: '{}'. Expected format is HH:MMa-HH:MMa.",
            range_str
        )));
    }

    let start_time_str = parts[0].trim();
    let end_time_str = parts[1].trim();

    let start_minutes = parse_time_to_minutes(start_time_str)?;
    let end_minutes = parse_time_to_minutes(end_time_str)?;

    if end_minutes < start_minutes {
        return Err(TimeError(format!(
            "End time {} is before start time {}. The range must be within a single day and end time must be after start time.",
            end_time_str, start_time_str
        )));
    }

    let diff_minutes = end_minutes - start_minutes;
    Ok(diff_minutes as f64 / 60.0)
}

fn main() {
    // Collect command-line arguments
    let args: Vec<String> = env::args().collect();

    // Check if the correct number of arguments is provided
    if args.len() != 2 {
        eprintln!("Calculates the difference in hours between two times in a day.");
        eprintln!(
            "Usage: {} \"HH:MMa-HH:MMa\"",
            args.get(0).map_or("time_diff_calculator", |s| s.as_str())
        );
        eprintln!(
            "Example: {} \"09:00AM-05:30PM\"",
            args.get(0).map_or("time_diff_calculator", |s| s.as_str())
        );
        process::exit(1); // Exit with an error code
    }

    let input_str = &args[1];

    // Calculate the time difference
    match calculate_time_difference_from_range_str(input_str) {
        Ok(hours) => {
            // Print the result formatted to two decimal places
            println!("{:.2} hours", hours);
        }
        Err(e) => {
            // Print the error message to stderr
            eprintln!("Error: {}", e);
            process::exit(1); // Exit with an error code
        }
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*; // Import items from the parent module

    // Tests for parse_time_to_minutes function
    #[test]
    fn test_parse_time_am() {
        assert_eq!(parse_time_to_minutes("09:00AM"), Ok(9 * 60)); // 540
        assert_eq!(parse_time_to_minutes("12:00AM"), Ok(0 * 60)); // 0 (midnight)
        assert_eq!(parse_time_to_minutes("12:30AM"), Ok(0 * 60 + 30)); // 30
        assert_eq!(parse_time_to_minutes("01:15AM"), Ok(1 * 60 + 15)); // 75
        assert_eq!(parse_time_to_minutes("11:59AM"), Ok(11 * 60 + 59)); // 719
    }

    #[test]
    fn test_parse_time_pm() {
        assert_eq!(parse_time_to_minutes("05:30PM"), Ok(17 * 60 + 30)); // 1050
        assert_eq!(parse_time_to_minutes("12:00PM"), Ok(12 * 60)); // 720 (noon)
        assert_eq!(parse_time_to_minutes("12:45PM"), Ok(12 * 60 + 45)); // 765
        assert_eq!(parse_time_to_minutes("01:00PM"), Ok(13 * 60)); // 780
        assert_eq!(parse_time_to_minutes("11:59PM"), Ok(23 * 60 + 59)); // 1439
    }

    #[test]
    fn test_parse_time_invalid_format() {
        assert!(parse_time_to_minutes("9:00AM").is_err()); // Invalid length
        assert!(parse_time_to_minutes("0900AM").is_err()); // Missing colon
        assert!(parse_time_to_minutes("09:00XM").is_err()); // Invalid AM/PM
        assert!(parse_time_to_minutes("09:00PMM").is_err()); // Invalid length
        assert!(parse_time_to_minutes("09:00").is_err()); // Invalid length
    }

    #[test]
    fn test_parse_time_invalid_values() {
        assert!(parse_time_to_minutes("00:00AM").is_err()); // Hour 00 invalid for 12h format
        assert!(parse_time_to_minutes("13:00AM").is_err()); // Hour 13 invalid for 12h format
        assert!(parse_time_to_minutes("09:60AM").is_err()); // Minute 60 invalid
        assert!(parse_time_to_minutes("AA:00AM").is_err()); // Hour not a number
        assert!(parse_time_to_minutes("09:BBAM").is_err()); // Minute not a number
    }

    // Tests for calculate_time_difference_from_range_str function
    #[test]
    fn test_calculate_difference_valid() {
        assert_eq!(
            calculate_time_difference_from_range_str("09:00AM-05:30PM"),
            Ok(8.5)
        );
        assert_eq!(
            calculate_time_difference_from_range_str("10:00AM-10:00AM"),
            Ok(0.0)
        );
        assert_eq!(
            calculate_time_difference_from_range_str("12:00AM-11:59PM"),
            Ok(1439.0 / 60.0)
        ); // 23.9833...
        assert_eq!(
            calculate_time_difference_from_range_str("01:00PM-05:00PM"),
            Ok(4.0)
        );
        assert_eq!(
            calculate_time_difference_from_range_str("11:00AM-01:00PM"),
            Ok(2.0)
        );
        // Test with spaces
        assert_eq!(
            calculate_time_difference_from_range_str(" 08:00AM - 10:00AM "),
            Ok(2.0)
        );
    }

    #[test]
    fn test_calculate_difference_invalid_range() {
        // End time before start time
        assert!(calculate_time_difference_from_range_str("05:00PM-09:00AM").is_err());
        assert!(calculate_time_difference_from_range_str("10:00AM-09:00AM").is_err());
    }

    #[test]
    fn test_calculate_difference_invalid_input_format() {
        assert!(calculate_time_difference_from_range_str("invalid-input").is_err());
        assert!(calculate_time_difference_from_range_str("09:00AM").is_err()); // Missing second part
        assert!(calculate_time_difference_from_range_str("09:00AM-").is_err());
        assert!(calculate_time_difference_from_range_str("-05:00PM").is_err());
    }

    #[test]
    fn test_calculate_difference_propagates_parse_error() {
        // Error from first time string
        assert!(calculate_time_difference_from_range_str("09:70AM-05:00PM").is_err());
        // Error from second time string
        assert!(calculate_time_difference_from_range_str("09:00AM-05:70PM").is_err());
    }
}
