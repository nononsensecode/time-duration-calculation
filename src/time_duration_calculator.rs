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

/// Parses a time string (e.g., "9:00AM", "09:00", "10:30PM") into its components.
///
/// # Arguments
/// * `time_str` - A string slice representing the time.
///   Formats supported: "H:MMam/pm", "HH:MMam/pm", "H:MM", "HH:MM".
///
/// # Returns
/// A `Result` containing a tuple `(hour, minute, Option<ampm_indicator_string>)`
/// if successful, or a `TimeError` if parsing fails.
/// Hour is in 12-hour format (1-12).
fn parse_time_components(time_str: &str) -> Result<(u32, u32, Option<String>), TimeError> {
    let original_time_str = time_str; // For rich error messages
    let mut time_part = time_str.trim(); // Handle potential surrounding spaces
    let mut ampm_opt: Option<String> = None;

    // Check for AM/PM suffix (case-insensitive)
    // It must be exactly "AM" or "PM" and at the end.
    if time_part.len() >= 2 {
        let potential_ampm = &time_part[time_part.len() - 2..];
        if potential_ampm.eq_ignore_ascii_case("AM") || potential_ampm.eq_ignore_ascii_case("PM") {
            // Ensure that what precedes AM/PM is not just another letter (e.g. "XAM")
            if time_part.len() > 2 {
                // e.g. "9AM" is valid, "AM" alone is not a time_part
                // Check if the character before AM/PM is a digit. If not, it's not a valid time like "XAM:PM"
                let char_before_ampm = time_part.chars().nth(time_part.len() - 3);
                if char_before_ampm.map_or(false, |c| c.is_alphabetic()) {
                    // e.g. "FOOAM", this is not an AM/PM marker for a time like "H:MMAM"
                    // Let it be parsed as part of HH:MM or H:MM if it matches
                } else {
                    ampm_opt = Some(potential_ampm.to_uppercase());
                    time_part = &time_part[..time_part.len() - 2];
                }
            } else {
                // Case like "AM" or "PM" as the whole string, or "9AM"
                // If time_part is just "AM" or "PM", it's invalid.
                // If it's "9AM", time_part becomes "9", ampm_opt is "AM"
                // This check might be redundant if parts.len() !=2 handles it later
                if potential_ampm.len() == time_part.len() {
                    // time_part is just "AM" or "PM"
                    return Err(TimeError(format!(
                        "Invalid time format: '{}'. Time string is too short or just an AM/PM indicator.",
                        original_time_str
                    )));
                }
                ampm_opt = Some(potential_ampm.to_uppercase());
                time_part = &time_part[..time_part.len() - 2];
            }
        }
    }

    // Now time_part should be "H:MM" or "HH:MM"
    let parts: Vec<&str> = time_part.split(':').collect();
    if parts.len() != 2 {
        return Err(TimeError(format!(
            "Invalid time format: '{}'. Expected H:MM or HH:MM (optionally followed by AM/PM). Missing or too many colons.",
            original_time_str
        )));
    }

    let h_str = parts[0];
    let m_str = parts[1];

    if h_str.is_empty() || !(1..=2).contains(&h_str.len()) {
        return Err(TimeError(format!(
            "Invalid hour format in '{}'. Hour part '{}' must be 1 or 2 digits.",
            original_time_str, h_str
        )));
    }
    if m_str.len() != 2 {
        return Err(TimeError(format!(
            "Invalid minute format in '{}'. Minute part '{}' must be 2 digits.",
            original_time_str, m_str
        )));
    }

    let hour12: u32 = h_str.parse().map_err(|_| {
        TimeError(format!(
            "Invalid hour value: '{}' in '{}'. Hour must be a number.",
            h_str, original_time_str
        ))
    })?;
    let minute: u32 = m_str.parse().map_err(|_| {
        TimeError(format!(
            "Invalid minute value: '{}' in '{}'. Minute must be a number.",
            m_str, original_time_str
        ))
    })?;

    // Validate hour (1-12 for 12-hour format) and minute (0-59)
    if !(1..=12).contains(&hour12) {
        return Err(TimeError(format!(
            "Invalid hour: {}. Hour must be between 1 and 12 for 12-hour format in '{}'.",
            hour12, original_time_str
        )));
    }
    if minute > 59 {
        return Err(TimeError(format!(
            "Invalid minute: {}. Minute must be between 0 and 59 in '{}'.",
            minute, original_time_str
        )));
    }

    Ok((hour12, minute, ampm_opt))
}

/// Converts 12-hour format components (hour, minute, AM/PM) into total minutes from midnight.
///
/// # Arguments
/// * `hour12` - Hour in 12-hour format (1-12).
/// * `minute` - Minute (0-59).
/// * `ampm_indicator` - "AM" or "PM".
/// * `original_time_str_for_error` - The original string for context in error messages.
///
/// # Returns
/// A `Result` containing total minutes from midnight (u32) or a `TimeError`.
fn convert_components_to_minutes(
    hour12: u32,
    minute: u32,
    ampm_indicator: &str,
    original_time_str_for_error: &str,
) -> Result<u32, TimeError> {
    // hour12 is assumed to be validated (1-12), minute (0-59)
    let mut hour24 = hour12;

    match ampm_indicator {
        // ampm_indicator is already Uppercase
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
            // This case should ideally not be reached if ampm_indicator is always "AM" or "PM".
            // It might be reached if parse_time_components incorrectly returns Some("") for ampm_opt.
            return Err(TimeError(format!(
                "Internal error or invalid AM/PM indicator: '{}' for time '{}'. Expected 'AM' or 'PM'.",
                ampm_indicator, original_time_str_for_error
            )));
        }
    }

    Ok(hour24 * 60 + minute)
}

/// Calculates the difference in hours between two time strings.
/// Input format: "H(H):MM[am/pm]-H(H):MM[am/pm]".
/// If AM/PM is omitted for both, start is assumed AM, end is assumed PM.
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
            "Invalid input format: '{}'. Expected format is H(H):MM[am/pm]-H(H):MM[am/pm].",
            range_str
        )));
    }

    let raw_start_time_str = parts[0].trim();
    let raw_end_time_str = parts[1].trim();

    if raw_start_time_str.is_empty() || raw_end_time_str.is_empty() {
        return Err(TimeError(format!(
            "Invalid input format: '{}'. Start or end time string is empty after splitting by '-'.",
            range_str
        )));
    }

    let (start_h, start_m, start_ampm_opt) = parse_time_components(raw_start_time_str)?;
    let (end_h, end_m, end_ampm_opt) = parse_time_components(raw_end_time_str)?;

    let start_minutes;
    let end_minutes;
    let determined_start_ampm_str;
    let determined_end_ampm_str;

    match (start_ampm_opt, end_ampm_opt) {
        (Some(start_ampm), Some(end_ampm)) => {
            // Both times explicitly specify AM/PM
            determined_start_ampm_str = start_ampm;
            determined_end_ampm_str = end_ampm;
            start_minutes = convert_components_to_minutes(
                start_h,
                start_m,
                &determined_start_ampm_str,
                raw_start_time_str,
            )?;
            end_minutes = convert_components_to_minutes(
                end_h,
                end_m,
                &determined_end_ampm_str,
                raw_end_time_str,
            )?;
        }
        (None, None) => {
            // Neither time specifies AM/PM: assume start is AM, end is PM
            determined_start_ampm_str = "AM".to_string();
            determined_end_ampm_str = "PM".to_string();
            start_minutes = convert_components_to_minutes(
                start_h,
                start_m,
                &determined_start_ampm_str,
                raw_start_time_str,
            )?;
            end_minutes = convert_components_to_minutes(
                end_h,
                end_m,
                &determined_end_ampm_str,
                raw_end_time_str,
            )?;
        }
        (Some(_), None) | (None, Some(_)) => {
            // Mixed specification: one has AM/PM, the other doesn't. This is ambiguous.
            return Err(TimeError(format!(
                "Ambiguous time range: '{}'. Both times must specify AM/PM, or neither should. If neither, start is assumed AM and end is assumed PM.",
                range_str
            )));
        }
    }

    if end_minutes < start_minutes {
        return Err(TimeError(format!(
            "End time {} (interpreted as {}:{:02}{}) is before start time {} (interpreted as {}:{:02}{}). The range must be within a single day and end time must be after start time.",
            raw_end_time_str, end_h, end_m, determined_end_ampm_str, // AM/PM already uppercase
            raw_start_time_str, start_h, start_m, determined_start_ampm_str // AM/PM already uppercase
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
            "Usage: {} \"H(H):MM[am/pm]-H(H):MM[am/pm]\"",
            args.get(0).map_or("time_diff_calculator", |s| s.as_str())
        );
        eprintln!(
            "Example (with AM/PM): {} \"09:00AM-05:30PM\"",
            args.get(0).map_or("time_diff_calculator", |s| s.as_str())
        );
        eprintln!(
            "Example (single digit hour): {} \"9:00AM-5:30PM\"",
            args.get(0).map_or("time_diff_calculator", |s| s.as_str())
        );
        eprintln!(
            "Example (implicit AM/PM): {} \"9:00-5:30\"",
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

    // Tests for parse_time_components function
    #[test]
    fn test_parse_time_components_valid() {
        assert_eq!(
            parse_time_components("09:00AM"),
            Ok((9, 0, Some("AM".to_string())))
        );
        assert_eq!(
            parse_time_components("9:00am"),
            Ok((9, 0, Some("AM".to_string())))
        );
        assert_eq!(
            parse_time_components("12:30PM"),
            Ok((12, 30, Some("PM".to_string())))
        );
        assert_eq!(
            parse_time_components("01:15pm"),
            Ok((1, 15, Some("PM".to_string())))
        );
        assert_eq!(parse_time_components("09:00"), Ok((9, 0, None)));
        assert_eq!(parse_time_components("9:00"), Ok((9, 0, None)));
        assert_eq!(parse_time_components("12:00"), Ok((12, 0, None)));
        assert_eq!(
            parse_time_components(" 07:00AM "),
            Ok((7, 0, Some("AM".to_string())))
        ); // With spaces
        assert_eq!(parse_time_components("7:00"), Ok((7, 0, None)));
    }

    #[test]
    fn test_parse_time_components_invalid_format() {
        assert!(parse_time_components("900AM").is_err()); // Missing colon
        assert!(parse_time_components("09:00XM").is_err()); // Invalid AM/PM
        assert!(parse_time_components("09:00PMM").is_err()); // Invalid AM/PM (too long)
        assert!(parse_time_components("090:00AM").is_err()); // Hour too long
        assert!(parse_time_components("09:0AM").is_err()); // Minute too short
        assert!(parse_time_components("09:000AM").is_err()); // Minute too long
        assert!(parse_time_components(":00AM").is_err()); // Missing hour
        assert!(parse_time_components("09:AM").is_err()); // Missing minute
        assert!(parse_time_components("9").is_err());
        assert!(parse_time_components("9AM").is_err()); // Needs colon
        assert!(parse_time_components("AM").is_err());
        assert!(parse_time_components("").is_err());
        assert!(parse_time_components("10:30 AM").is_err()); // Space before AM/PM
    }

    #[test]
    fn test_parse_time_components_invalid_values() {
        assert!(parse_time_components("00:00AM").is_err()); // Hour 00 invalid
        assert!(parse_time_components("13:00AM").is_err()); // Hour 13 invalid
        assert!(parse_time_components("09:60AM").is_err()); // Minute 60 invalid
        assert!(parse_time_components("AA:00AM").is_err()); // Hour not a number
        assert!(parse_time_components("09:BBAM").is_err()); // Minute not a number
    }

    // Tests for convert_components_to_minutes function
    #[test]
    fn test_convert_components_to_minutes_valid() {
        assert_eq!(
            convert_components_to_minutes(9, 0, "AM", "9:00AM"),
            Ok(9 * 60)
        );
        assert_eq!(
            convert_components_to_minutes(12, 0, "AM", "12:00AM"),
            Ok(0 * 60)
        ); // Midnight
        assert_eq!(
            convert_components_to_minutes(12, 30, "AM", "12:30AM"),
            Ok(30)
        );
        assert_eq!(
            convert_components_to_minutes(1, 15, "AM", "01:15AM"),
            Ok(1 * 60 + 15)
        );
        assert_eq!(
            convert_components_to_minutes(5, 30, "PM", "05:30PM"),
            Ok(17 * 60 + 30)
        );
        assert_eq!(
            convert_components_to_minutes(12, 0, "PM", "12:00PM"),
            Ok(12 * 60)
        ); // Noon
        assert_eq!(
            convert_components_to_minutes(12, 45, "PM", "12:45PM"),
            Ok(12 * 60 + 45)
        );
        assert_eq!(
            convert_components_to_minutes(1, 0, "PM", "01:00PM"),
            Ok(13 * 60)
        );
        assert_eq!(
            convert_components_to_minutes(11, 59, "PM", "11:59PM"),
            Ok(23 * 60 + 59)
        );
    }

    // Tests for calculate_time_difference_from_range_str function
    #[test]
    fn test_calculate_difference_explicit_ampm() {
        assert_eq!(
            calculate_time_difference_from_range_str("09:00AM-05:30PM"),
            Ok(8.5)
        );
        assert_eq!(
            calculate_time_difference_from_range_str("9:00AM-5:30PM"),
            Ok(8.5)
        );
        assert_eq!(
            calculate_time_difference_from_range_str("10:00AM-10:00AM"),
            Ok(0.0)
        );
        assert_eq!(
            calculate_time_difference_from_range_str("12:00AM-11:59PM"),
            Ok(1439.0 / 60.0)
        );
        assert_eq!(
            calculate_time_difference_from_range_str("01:00PM-05:00PM"),
            Ok(4.0)
        );
        assert_eq!(
            calculate_time_difference_from_range_str("1:00PM-5:00PM"),
            Ok(4.0)
        );
        assert_eq!(
            calculate_time_difference_from_range_str("11:00AM-01:00PM"),
            Ok(2.0)
        );
        assert_eq!(
            calculate_time_difference_from_range_str(" 11:00AM - 01:00PM "),
            Ok(2.0)
        ); // with spaces
    }

    #[test]
    fn test_calculate_difference_implicit_ampm() {
        assert_eq!(
            calculate_time_difference_from_range_str("09:00-05:30"),
            Ok(8.5)
        ); // 9AM to 5:30PM
        assert_eq!(
            calculate_time_difference_from_range_str("9:00-5:30"),
            Ok(8.5)
        ); // 9AM to 5:30PM
        assert_eq!(
            calculate_time_difference_from_range_str("10:00-02:00"),
            Ok(4.0)
        ); // 10AM to 2PM
        assert_eq!(
            calculate_time_difference_from_range_str("12:00-11:59"),
            Ok(1439.0 / 60.0)
        ); // 12AM to 11:59PM
        assert_eq!(
            calculate_time_difference_from_range_str("01:00-05:00"),
            Ok(16.0)
        ); // 1AM to 5PM
        assert_eq!(
            calculate_time_difference_from_range_str("11:00-01:00"),
            Ok(2.0)
        ); // 11AM to 1PM
    }

    #[test]
    fn test_calculate_difference_mixed_ampm_error() {
        assert!(calculate_time_difference_from_range_str("09:00AM-05:00").is_err());
        assert!(calculate_time_difference_from_range_str("09:00-05:00PM").is_err());
    }

    #[test]
    fn test_calculate_difference_invalid_range_explicit_ampm() {
        assert!(calculate_time_difference_from_range_str("05:00PM-09:00AM").is_err());
        assert!(calculate_time_difference_from_range_str("10:00AM-09:00AM").is_err());
    }

    #[test]
    fn test_calculate_difference_invalid_range_implicit_ampm() {
        // 5:00 (AM) - 9:00 (PM) -> This is valid: 16 hours
        assert_eq!(
            calculate_time_difference_from_range_str("05:00-09:00"),
            Ok(16.0)
        );
        // 10:00 (AM) - 09:00 (PM) -> This is valid: 11 hours
        assert_eq!(
            calculate_time_difference_from_range_str("10:00-09:00"),
            Ok(11.0)
        );
        // However, if the interpretation leads to start_minutes > end_minutes, it should fail.
        // This is already covered by the logic if e.g. 10:00PM-02:00AM was allowed and then parsed.
        // The current error message for end_minutes < start_minutes is generic and covers this.
        // Example: "12:00PM-10:00AM" (explicit) -> error
        assert!(calculate_time_difference_from_range_str("12:00PM-10:00AM").is_err());
    }

    #[test]
    fn test_calculate_difference_invalid_input_format() {
        assert!(calculate_time_difference_from_range_str("invalid-input").is_err());
        assert!(calculate_time_difference_from_range_str("09:00AM").is_err()); // Missing second part
        assert!(calculate_time_difference_from_range_str("09:00AM-").is_err());
        assert!(calculate_time_difference_from_range_str("-05:00PM").is_err());
        assert!(calculate_time_difference_from_range_str("09:00AM - ").is_err());
        // Empty second part after trim
    }

    #[test]
    fn test_calculate_difference_propagates_parse_error() {
        assert!(calculate_time_difference_from_range_str("09:70AM-05:00PM").is_err()); // Invalid minute in first
        assert!(calculate_time_difference_from_range_str("09:00AM-05:70PM").is_err()); // Invalid minute in second
        assert!(calculate_time_difference_from_range_str("13:00AM-05:00PM").is_err()); // Invalid hour in first
        assert!(calculate_time_difference_from_range_str("13:00-05:00").is_err());
        // Invalid hour in first (implicit)
    }

    #[test]
    fn test_end_time_before_start_time_error_message() {
        let result = calculate_time_difference_from_range_str("05:00PM-09:00AM");
        assert!(result.is_err());
        if let Err(TimeError(msg)) = result {
            assert!(msg.contains("End time 09:00AM (interpreted as 9:00AM) is before start time 05:00PM (interpreted as 5:00PM)"));
        }

        let result_implicit = calculate_time_difference_from_range_str("10:00PM-02:00AM"); // This should be an error
        assert!(result_implicit.is_err());
        if let Err(TimeError(msg)) = result_implicit {
            // 10:00PM -> 22*60 = 1320. 02:00AM -> 2*60 = 120. 120 < 1320.
            assert!(msg.contains("End time 02:00AM (interpreted as 2:00AM) is before start time 10:00PM (interpreted as 10:00PM)"));
        }
    }
}
