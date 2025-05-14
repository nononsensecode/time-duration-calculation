/// Represents an error that can occur during time parsing or calculation.
#[derive(Debug, PartialEq)]
struct TimeError(String);

impl std::fmt::Display for TimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for TimeError {}

mod time_parsing {
    use super::TimeError;

    /// Parses a time string (e.g., "9:00AM", "09:00", "10:30PM") into its components.
    /// Returns (hour, minute, Option<AM/PM>)
    pub fn parse_time_components(time_str: &str) -> Result<(u32, u32, Option<String>), TimeError> {
        let original_time_str = time_str;
        let mut time_part = time_str.trim();
        let mut ampm_opt: Option<String> = None;

        // Check for AM/PM suffix (case-insensitive)
        if time_part.len() >= 2 {
            let potential_ampm = &time_part[time_part.len() - 2..];
            if potential_ampm.eq_ignore_ascii_case("AM")
                || potential_ampm.eq_ignore_ascii_case("PM")
            {
                if time_part.len() > 2 {
                    let char_before_ampm = time_part.chars().nth(time_part.len() - 3);
                    if char_before_ampm.map_or(false, |c| c.is_alphabetic()) {
                        // Not a valid AM/PM marker
                    } else {
                        ampm_opt = Some(potential_ampm.to_uppercase());
                        time_part = &time_part[..time_part.len() - 2];
                    }
                } else {
                    if potential_ampm.len() == time_part.len() {
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
}

mod time_conversion {
    use super::TimeError;
    /// Converts 12-hour format components (hour, minute, AM/PM) into total minutes from midnight.
    pub fn to_minutes(
        hour12: u32,
        minute: u32,
        ampm_indicator: &str,
        original_time_str: &str,
    ) -> Result<u32, TimeError> {
        let mut hour24 = hour12;
        match ampm_indicator {
            "AM" => if hour12 == 12 { hour24 = 0; },
            "PM" => if hour12 != 12 { hour24 += 12; },
            _ => return Err(TimeError(format!(
                "Internal error or invalid AM/PM indicator: '{}' for time '{}'. Expected 'AM' or 'PM'.",
                ampm_indicator, original_time_str
            ))),
        }
        Ok(hour24 * 60 + minute)
    }
}

mod time_difference {
    use super::{time_conversion::to_minutes, time_parsing::parse_time_components, TimeError};
    /// Calculates the difference in hours between two time strings.
    pub fn calculate(range_str: &str) -> Result<f64, TimeError> {
        let parts: Vec<&str> = range_str.split('-').collect();
        if parts.len() != 2 {
            return Err(TimeError(format!(
                "Invalid input format: '{}'. Expected format is H(H):MM[am/pm]-H(H):MM[am/pm].",
                range_str
            )));
        }
        let raw_start = parts[0].trim();
        let raw_end = parts[1].trim();
        if raw_start.is_empty() || raw_end.is_empty() {
            return Err(TimeError(format!(
                "Invalid input format: '{}'. Start or end time string is empty after splitting by '-'.",
                range_str
            )));
        }
        let (start_h, start_m, start_ampm) = parse_time_components(raw_start)?;
        let (end_h, end_m, end_ampm) = parse_time_components(raw_end)?;
        let (start_minutes, end_minutes, start_ampm_str, end_ampm_str) = match (start_ampm, end_ampm) {
            (Some(s), Some(e)) => (
                to_minutes(start_h, start_m, &s, raw_start)?,
                to_minutes(end_h, end_m, &e, raw_end)?,
                s, e
            ),
            (None, None) => (
                to_minutes(start_h, start_m, "AM", raw_start)?,
                to_minutes(end_h, end_m, "PM", raw_end)?,
                "AM".to_string(), "PM".to_string()
            ),
            _ => return Err(TimeError(format!(
                "Ambiguous time range: '{}'. Both times must specify AM/PM, or neither should. If neither, start is assumed AM and end is assumed PM.",
                range_str
            ))),
        };
        if end_minutes < start_minutes {
            return Err(TimeError(format!(
                "End time {} (interpreted as {}:{:02}{}) is before start time {} (interpreted as {}:{:02}{}). The range must be within a single day and end time must be after start time.",
                raw_end, end_h, end_m, end_ampm_str,
                raw_start, start_h, start_m, start_ampm_str
            )));
        }
        Ok((end_minutes - start_minutes) as f64 / 60.0)
    }
}

fn main() {
    use chrono::Local;
    use std::env;
    use std::process;
    use time_difference::calculate;
    use time_parsing::parse_time_components;

    let args: Vec<String> = env::args().collect();
    let program_name = args
        .get(0)
        .map_or("time_duration_calculator", |s| s.as_str());

    if args.len() != 2 {
        eprintln!("Calculates the difference in hours between two times in a day.");
        eprintln!("Usage:");
        eprintln!(
            "  1. Time range: {} \"H(H):MM[am/pm]-H(H):MM[am/pm]\"",
            program_name
        );
        eprintln!("     Example: {} \"09:00AM-05:30PM\"", program_name);
        eprintln!("     Example (implicit AM/PM for range): {} \"9:00-5:30\" (interprets as 9:00AM-5:30PM)", program_name);
        eprintln!("  2. Single time (start time assumed AM, end time is current system time): {} \"H(H):MM\"", program_name);
        eprintln!(
            "     Example: {} \"09:15\" (interprets as 09:15AM - CurrentSystemTime)",
            program_name
        );
        process::exit(1);
    }

    let input_str = args[1].trim();
    let final_result: Result<f64, TimeError>;

    if input_str.contains('-') {
        final_result = calculate(input_str);
    } else {
        let (input_h, input_m, ampm_opt) = match parse_time_components(input_str) {
            Ok(components) => components,
            Err(e) => {
                eprintln!("Error parsing input time '{}': {}", input_str, e);
                process::exit(1);
            }
        };
        if ampm_opt.is_some() {
            eprintln!("Error: For single time input (e.g., '9:15'), do not specify AM/PM.");
            eprintln!(
                "The input time is assumed to be AM, and the end time is the current system time."
            );
            process::exit(1);
        }
        let start_time_str = format!("{}:{:02}AM", input_h, input_m);
        let now = Local::now();
        let current_hour_12 = now.format("%I").to_string();
        let current_minute = now.format("%M").to_string();
        let current_ampm = now.format("%p").to_string().to_uppercase();
        let end_time_str = format!("{}:{}{}", current_hour_12, current_minute, current_ampm);
        let range_str = format!("{}-{}", start_time_str, end_time_str);
        eprintln!(
            "Interpreting single time input '{}' as range: {}",
            input_str, range_str
        );
        final_result = calculate(&range_str);
    }

    match final_result {
        Ok(hours) => println!("{:.2} hours", hours),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_time_components_valid() {
        assert_eq!(
            time_parsing::parse_time_components("09:00AM"),
            Ok((9, 0, Some("AM".to_string())))
        );
        assert_eq!(
            time_parsing::parse_time_components("9:00am"),
            Ok((9, 0, Some("AM".to_string())))
        );
        assert_eq!(
            time_parsing::parse_time_components("12:30PM"),
            Ok((12, 30, Some("PM".to_string())))
        );
        assert_eq!(
            time_parsing::parse_time_components("01:15pm"),
            Ok((1, 15, Some("PM".to_string())))
        );
        assert_eq!(
            time_parsing::parse_time_components("09:00"),
            Ok((9, 0, None))
        );
        assert_eq!(
            time_parsing::parse_time_components("9:00"),
            Ok((9, 0, None))
        );
        assert_eq!(
            time_parsing::parse_time_components("12:00"),
            Ok((12, 0, None))
        );
        assert_eq!(
            time_parsing::parse_time_components(" 07:00AM "),
            Ok((7, 0, Some("AM".to_string())))
        );
        assert_eq!(
            time_parsing::parse_time_components("7:00"),
            Ok((7, 0, None))
        );
    }

    #[test]
    fn test_parse_time_components_invalid_format() {
        assert!(time_parsing::parse_time_components("900AM").is_err());
        assert!(time_parsing::parse_time_components("09:00XM").is_err());
        assert!(time_parsing::parse_time_components("09:00PMM").is_err());
        assert!(time_parsing::parse_time_components("090:00AM").is_err());
        assert!(time_parsing::parse_time_components("09:0AM").is_err());
        assert!(time_parsing::parse_time_components("09:000AM").is_err());
        assert!(time_parsing::parse_time_components(":00AM").is_err());
        assert!(time_parsing::parse_time_components("09:AM").is_err());
        assert!(time_parsing::parse_time_components("9").is_err());
        assert!(time_parsing::parse_time_components("9AM").is_err());
        assert!(time_parsing::parse_time_components("AM").is_err());
        assert!(time_parsing::parse_time_components("").is_err());
        assert!(time_parsing::parse_time_components("10:30 AM").is_err());
    }

    #[test]
    fn test_parse_time_components_invalid_values() {
        assert!(time_parsing::parse_time_components("00:00AM").is_err());
        assert!(time_parsing::parse_time_components("13:00AM").is_err());
        assert!(time_parsing::parse_time_components("09:60AM").is_err());
        assert!(time_parsing::parse_time_components("AA:00AM").is_err());
        assert!(time_parsing::parse_time_components("09:BBAM").is_err());
    }

    #[test]
    fn test_convert_components_to_minutes_valid() {
        assert_eq!(
            time_conversion::to_minutes(9, 0, "AM", "9:00AM"),
            Ok(9 * 60)
        );
        assert_eq!(
            time_conversion::to_minutes(12, 0, "AM", "12:00AM"),
            Ok(0 * 60)
        );
        assert_eq!(
            time_conversion::to_minutes(5, 30, "PM", "05:30PM"),
            Ok(17 * 60 + 30)
        );
        assert_eq!(
            time_conversion::to_minutes(12, 0, "PM", "12:00PM"),
            Ok(12 * 60)
        );
    }

    #[test]
    fn test_calculate_difference_explicit_ampm() {
        assert_eq!(time_difference::calculate("09:00AM-05:30PM"), Ok(8.5));
        assert_eq!(time_difference::calculate("9:00AM-5:30PM"), Ok(8.5));
    }

    #[test]
    fn test_calculate_difference_implicit_ampm_range() {
        assert_eq!(time_difference::calculate("09:00-05:30"), Ok(8.5));
        assert_eq!(time_difference::calculate("9:00-5:30"), Ok(8.5));
    }

    #[test]
    fn test_calculate_difference_mixed_ampm_error() {
        assert!(time_difference::calculate("09:00AM-05:00").is_err());
        assert!(time_difference::calculate("09:00-05:00PM").is_err());
    }

    #[test]
    fn test_calculate_difference_end_before_start_error() {
        let result = time_difference::calculate("05:00PM-09:00AM");
        assert!(result.is_err());
        if let Err(TimeError(msg)) = result {
            assert!(msg.contains("End time 09:00AM (interpreted as 9:00AM) is before start time 05:00PM (interpreted as 5:00PM)"));
        }
    }
}
