use std::env;
use std::process;
use chrono::Local; // Added for getting current system time

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
        let potential_ampm = &time_part[time_part.len()-2..];
        if potential_ampm.eq_ignore_ascii_case("AM") || potential_ampm.eq_ignore_ascii_case("PM") {
            if time_part.len() > 2 { 
                let char_before_ampm = time_part.chars().nth(time_part.len() - 3);
                if char_before_ampm.map_or(false, |c| c.is_alphabetic()) {
                    // e.g. "FOOAM", this is not an AM/PM marker for a time like "H:MMAM"
                } else {
                    ampm_opt = Some(potential_ampm.to_uppercase());
                    time_part = &time_part[..time_part.len()-2];
                }
            } else { 
                 if potential_ampm.len() == time_part.len() { 
                    return Err(TimeError(format!(
                        "Invalid time format: '{}'. Time string is too short or just an AM/PM indicator.",
                        original_time_str
                    )));
                 }
                 // This case is tricky, e.g. "9AM". time_part becomes "9".
                 // This will likely fail later at split(':') if not "H:MM" structure.
                 // The current logic expects H:MM before AM/PM.
                 // Let's assume if AM/PM is stripped, what remains must be H:MM or HH:MM.
                 // This is implicitly handled by the colon check later.
                 ampm_opt = Some(potential_ampm.to_uppercase());
                 time_part = &time_part[..time_part.len()-2];
            }
        }
    }

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

    let hour12: u32 = h_str.parse().map_err(|_| TimeError(format!("Invalid hour value: '{}' in '{}'. Hour must be a number.", h_str, original_time_str)))?;
    let minute: u32 = m_str.parse().map_err(|_| TimeError(format!("Invalid minute value: '{}' in '{}'. Minute must be a number.", m_str, original_time_str)))?;

    if !(1..=12).contains(&hour12) {
        return Err(TimeError(format!("Invalid hour: {}. Hour must be between 1 and 12 for 12-hour format in '{}'.", hour12, original_time_str)));
    }
    if minute > 59 {
        return Err(TimeError(format!("Invalid minute: {}. Minute must be between 0 and 59 in '{}'.", minute, original_time_str)));
    }

    Ok((hour12, minute, ampm_opt))
}

/// Converts 12-hour format components (hour, minute, AM/PM) into total minutes from midnight.
fn convert_components_to_minutes(hour12: u32, minute: u32, ampm_indicator: &str, original_time_str_for_error: &str) -> Result<u32, TimeError> {
    let mut hour24 = hour12;
    match ampm_indicator { 
        "AM" => {
            if hour12 == 12 { hour24 = 0; }
        }
        "PM" => {
            if hour12 != 12 { hour24 += 12; }
        }
        _ => {
            return Err(TimeError(format!(
                "Internal error or invalid AM/PM indicator: '{}' for time '{}'. Expected 'AM' or 'PM'.",
                ampm_indicator, original_time_str_for_error
            )));
        }
    }
    Ok(hour24 * 60 + minute)
}

/// Calculates the difference in hours between two time strings.
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
            determined_start_ampm_str = start_ampm;
            determined_end_ampm_str = end_ampm;
            start_minutes = convert_components_to_minutes(start_h, start_m, &determined_start_ampm_str, raw_start_time_str)?;
            end_minutes = convert_components_to_minutes(end_h, end_m, &determined_end_ampm_str, raw_end_time_str)?;
        }
        (None, None) => {
            determined_start_ampm_str = "AM".to_string();
            determined_end_ampm_str = "PM".to_string();
            start_minutes = convert_components_to_minutes(start_h, start_m, &determined_start_ampm_str, raw_start_time_str)?;
            end_minutes = convert_components_to_minutes(end_h, end_m, &determined_end_ampm_str, raw_end_time_str)?;
        }
        (Some(_), None) | (None, Some(_)) => {
            return Err(TimeError(format!(
                "Ambiguous time range: '{}'. Both times must specify AM/PM, or neither should. If neither, start is assumed AM and end is assumed PM.",
                range_str
            )));
        }
    }

    if end_minutes < start_minutes {
         return Err(TimeError(format!(
            "End time {} (interpreted as {}:{:02}{}) is before start time {} (interpreted as {}:{:02}{}). The range must be within a single day and end time must be after start time.",
            raw_end_time_str, end_h, end_m, determined_end_ampm_str, 
            raw_start_time_str, start_h, start_m, determined_start_ampm_str 
        )));
    }

    let diff_minutes = end_minutes - start_minutes;
    Ok(diff_minutes as f64 / 60.0)
}

fn main() {
    // Collect command-line arguments
    let args: Vec<String> = env::args().collect();
    let program_name = args.get(0).map_or("time_diff_calculator", |s| s.as_str());

    // Check if the correct number of arguments is provided
    if args.len() != 2 {
        eprintln!("Calculates the difference in hours between two times in a day.");
        eprintln!("Usage:");
        eprintln!("  1. Time range: {} \"H(H):MM[am/pm]-H(H):MM[am/pm]\"", program_name);
        eprintln!("     Example: {} \"09:00AM-05:30PM\"", program_name);
        eprintln!("     Example (implicit AM/PM for range): {} \"9:00-5:30\" (interprets as 9:00AM-5:30PM)", program_name);
        eprintln!("  2. Single time (start time assumed AM, end time is current system time): {} \"H(H):MM\"", program_name);
        eprintln!("     Example: {} \"09:15\" (interprets as 09:15AM - CurrentSystemTime)", program_name);
        process::exit(1); 
    }

    let input_str = args[1].trim();
    let final_result: Result<f64, TimeError>;

    if input_str.contains('-') {
        // Input is a time range
        final_result = calculate_time_difference_from_range_str(input_str);
    } else {
        // Input is a single time
        // Attempt to parse it. parse_time_components will also identify if AM/PM was unexpectedly included.
        let (input_h, input_m, ampm_opt) = match parse_time_components(input_str) {
            Ok(components) => components,
            Err(e) => {
                eprintln!("Error parsing input time '{}': {}", input_str, e);
                process::exit(1);
            }
        };

        // For single time input, AM/PM should NOT be specified by the user.
        if ampm_opt.is_some() {
            eprintln!("Error: For single time input (e.g., '9:15'), do not specify AM/PM.");
            eprintln!("The input time is assumed to be AM, and the end time is the current system time.");
            process::exit(1);
        }

        // Construct the start time string, assuming AM for the input time.
        let start_time_str_constructed = format!("{}:{:02}AM", input_h, input_m);

        // Get current system time and format it
        let now = Local::now();
        // %I gives 12-hour format (01-12), %M gives minute (00-59), %p gives AM/PM
        let current_hour_12_str = now.format("%I").to_string(); 
        let current_minute_str = now.format("%M").to_string();
        let current_ampm_str = now.format("%p").to_string().to_uppercase(); // Ensure uppercase AM/PM

        let end_time_str_constructed = format!("{}:{}{}", current_hour_12_str, current_minute_str, current_ampm_str);
        
        // Construct the full range string to be processed
        let range_str_constructed = format!("{}-{}", start_time_str_constructed, end_time_str_constructed);
        
        // Provide feedback to the user about the interpretation
        eprintln!("Interpreting single time input '{}' as range: {}", input_str, range_str_constructed);

        final_result = calculate_time_difference_from_range_str(&range_str_constructed);
    }

    match final_result {
        Ok(hours) => {
            println!("{:.2} hours", hours);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1); 
        }
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;

    // Tests for parse_time_components function (no changes needed here from previous version)
    #[test]
    fn test_parse_time_components_valid() {
        assert_eq!(parse_time_components("09:00AM"), Ok((9, 0, Some("AM".to_string()))));
        assert_eq!(parse_time_components("9:00am"), Ok((9, 0, Some("AM".to_string()))));
        assert_eq!(parse_time_components("12:30PM"), Ok((12, 30, Some("PM".to_string()))));
        assert_eq!(parse_time_components("01:15pm"), Ok((1, 15, Some("PM".to_string()))));
        assert_eq!(parse_time_components("09:00"), Ok((9, 0, None))); // No AM/PM
        assert_eq!(parse_time_components("9:00"), Ok((9, 0, None)));   // Single digit hour, no AM/PM
        assert_eq!(parse_time_components("12:00"), Ok((12, 0, None))); // No AM/PM
        assert_eq!(parse_time_components(" 07:00AM "), Ok((7,0,Some("AM".to_string())))); // With spaces
        assert_eq!(parse_time_components("7:00"), Ok((7,0,None)));
    }

    #[test]
    fn test_parse_time_components_invalid_format() {
        assert!(parse_time_components("900AM").is_err()); 
        assert!(parse_time_components("09:00XM").is_err()); 
        assert!(parse_time_components("09:00PMM").is_err());
        assert!(parse_time_components("090:00AM").is_err()); 
        assert!(parse_time_components("09:0AM").is_err()); 
        assert!(parse_time_components("09:000AM").is_err());
        assert!(parse_time_components(":00AM").is_err()); 
        assert!(parse_time_components("09:AM").is_err()); 
        assert!(parse_time_components("9").is_err());
        assert!(parse_time_components("9AM").is_err()); // Needs colon
        assert!(parse_time_components("AM").is_err());
        assert!(parse_time_components("").is_err());
        assert!(parse_time_components("10:30 AM").is_err()); // Space before AM/PM not handled by current stripping
    }

    #[test]
    fn test_parse_time_components_invalid_values() {
        assert!(parse_time_components("00:00AM").is_err()); 
        assert!(parse_time_components("13:00AM").is_err()); 
        assert!(parse_time_components("09:60AM").is_err()); 
        assert!(parse_time_components("AA:00AM").is_err()); 
        assert!(parse_time_components("09:BBAM").is_err()); 
    }

    // Tests for convert_components_to_minutes (no changes needed)
    #[test]
    fn test_convert_components_to_minutes_valid() {
        assert_eq!(convert_components_to_minutes(9, 0, "AM", "9:00AM"), Ok(9 * 60));
        assert_eq!(convert_components_to_minutes(12, 0, "AM", "12:00AM"), Ok(0 * 60));
        assert_eq!(convert_components_to_minutes(5, 30, "PM", "05:30PM"), Ok(17 * 60 + 30));
        assert_eq!(convert_components_to_minutes(12, 0, "PM", "12:00PM"), Ok(12 * 60));
    }
    
    // Tests for calculate_time_difference_from_range_str (no changes needed for its direct logic)
    // These tests ensure it still works correctly when main constructs the string for it.
    #[test]
    fn test_calculate_difference_explicit_ampm() {
        assert_eq!(calculate_time_difference_from_range_str("09:00AM-05:30PM"), Ok(8.5));
        assert_eq!(calculate_time_difference_from_range_str("9:00AM-5:30PM"), Ok(8.5));
    }

    #[test]
    fn test_calculate_difference_implicit_ampm_range() { // Renamed for clarity
        assert_eq!(calculate_time_difference_from_range_str("09:00-05:30"), Ok(8.5)); 
        assert_eq!(calculate_time_difference_from_range_str("9:00-5:30"), Ok(8.5));   
    }

    #[test]
    fn test_calculate_difference_mixed_ampm_error() {
        assert!(calculate_time_difference_from_range_str("09:00AM-05:00").is_err());
        assert!(calculate_time_difference_from_range_str("09:00-05:00PM").is_err());
    }

    #[test]
    fn test_calculate_difference_end_before_start_error() { // Renamed for clarity
        let result = calculate_time_difference_from_range_str("05:00PM-09:00AM");
        assert!(result.is_err());
        if let Err(TimeError(msg)) = result {
            assert!(msg.contains("End time 09:00AM (interpreted as 9:00AM) is before start time 05:00PM (interpreted as 5:00PM)"));
        }
    }
    // Note: Testing the new single-time functionality fully requires knowing the current system time,
    // which is difficult in automated unit tests without mocking time.
    // However, we can test the components:
    // 1. `parse_time_components` is tested for "H:MM" inputs.
    // 2. `calculate_time_difference_from_range_str` is tested for various constructed strings.
    // Manual testing of the main binary with inputs like "9:15" would be needed to verify
    // the chrono integration and string construction in `main`.
    // Example manual test:
    // If current time is 06:30 PM, running `./your_program "09:15"`
    // should internally process "09:15AM-06:30PM" and output 9.25 hours.
    // If current time is 08:00 AM, running `./your_program "09:15"`
    // should internally process "09:15AM-08:00AM" and give an error (end before start).
}
