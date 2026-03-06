/// Pure validation functions for Practice Setup domain rules.
/// All functions are side-effect-free and fully testable without a database.

const VALID_DAYS: &[&str] = &[
    "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday",
];

const VALID_CATEGORIES: &[&str] = &[
    "Consult", "Preventive", "Restorative", "Invasive", "Cosmetic", "Diagnostic",
];

const MIN_DURATION: u32 = 15;
const MAX_DURATION: u32 = 240;

pub fn validate_name(name: &str) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("Name must not be empty".to_string());
    }
    Ok(())
}

pub fn validate_chair_count(count: u32) -> Result<(), String> {
    if count < 1 {
        return Err("Chair count must be at least 1".to_string());
    }
    Ok(())
}

pub fn validate_day_of_week(day: &str) -> Result<(), String> {
    if !VALID_DAYS.contains(&day) {
        return Err(format!(
            "Invalid day '{}'. Must be one of: {}",
            day,
            VALID_DAYS.join(", ")
        ));
    }
    Ok(())
}

/// Validates HH:MM format with valid hour (0–23) and minute (0–59).
pub fn validate_hhmm(time: &str) -> Result<(), String> {
    let parts: Vec<&str> = time.split(':').collect();
    if parts.len() != 2 {
        return Err(format!("Time '{}' must be in HH:MM format", time));
    }
    let h: u32 = parts[0].parse().map_err(|_| format!("Invalid hour in '{}'", time))?;
    let m: u32 = parts[1].parse().map_err(|_| format!("Invalid minute in '{}'", time))?;
    if h > 23 || m > 59 {
        return Err(format!("Time '{}' out of range (00:00–23:59)", time));
    }
    // Require zero-padded 2-digit components
    if parts[0].len() != 2 || parts[1].len() != 2 {
        return Err(format!("Time '{}' must be zero-padded (e.g. 08:00)", time));
    }
    Ok(())
}

/// Validates that close > open (both HH:MM strings, assumed already validated).
pub fn validate_time_range(open: &str, close: &str) -> Result<(), String> {
    if close <= open {
        return Err(format!(
            "Close time '{}' must be after open time '{}'",
            close, open
        ));
    }
    Ok(())
}

pub fn validate_duration(mins: u32) -> Result<(), String> {
    if mins < MIN_DURATION || mins > MAX_DURATION {
        return Err(format!(
            "Duration {} minutes is outside allowed range ({MIN_DURATION}–{MAX_DURATION})",
            mins
        ));
    }
    Ok(())
}

pub fn validate_category(cat: &str) -> Result<(), String> {
    if !VALID_CATEGORIES.contains(&cat) {
        return Err(format!(
            "Invalid category '{}'. Must be one of: {}",
            cat,
            VALID_CATEGORIES.join(", ")
        ));
    }
    Ok(())
}

/// Validates YYYY-MM-DD format (used by office-related date checks).
pub fn validate_date_ymd(date: &str) -> Result<(), String> {
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 || parts[0].len() != 4 || parts[1].len() != 2 || parts[2].len() != 2 {
        return Err(format!("Date '{}' must be in YYYY-MM-DD format", date));
    }
    let _y: u32 = parts[0].parse().map_err(|_| format!("Invalid year in '{}'", date))?;
    let m: u32 = parts[1].parse().map_err(|_| format!("Invalid month in '{}'", date))?;
    let d: u32 = parts[2].parse().map_err(|_| format!("Invalid day in '{}'", date))?;
    if m < 1 || m > 12 {
        return Err(format!("Month {} is out of range in '{}'", m, date));
    }
    if d < 1 || d > 31 {
        return Err(format!("Day {} is out of range in '{}'", d, date));
    }
    Ok(())
}

/// Validates that end_date >= start_date (lexicographic comparison valid for YYYY-MM-DD).
pub fn validate_date_range(start: &str, end: &str) -> Result<(), String> {
    if end < start {
        return Err(format!(
            "End date '{}' must be on or after start date '{}'",
            end, start
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_name_rejects_empty() {
        assert!(validate_name("").is_err());
        assert!(validate_name("   ").is_err());
    }

    #[test]
    fn test_validate_name_accepts_nonempty() {
        assert!(validate_name("Kingston").is_ok());
    }

    #[test]
    fn test_validate_chair_count_rejects_zero() {
        assert!(validate_chair_count(0).is_err());
    }

    #[test]
    fn test_validate_chair_count_accepts_positive() {
        assert!(validate_chair_count(1).is_ok());
        assert!(validate_chair_count(10).is_ok());
    }

    #[test]
    fn test_validate_day_of_week() {
        assert!(validate_day_of_week("Monday").is_ok());
        assert!(validate_day_of_week("Sunday").is_ok());
        assert!(validate_day_of_week("monday").is_err());
        assert!(validate_day_of_week("Funday").is_err());
    }

    #[test]
    fn test_validate_hhmm() {
        assert!(validate_hhmm("08:00").is_ok());
        assert!(validate_hhmm("23:59").is_ok());
        assert!(validate_hhmm("8:00").is_err());   // not zero-padded
        assert!(validate_hhmm("24:00").is_err());  // hour out of range
        assert!(validate_hhmm("12:60").is_err());  // minute out of range
        assert!(validate_hhmm("1200").is_err());   // wrong format
    }

    #[test]
    fn test_validate_time_range() {
        assert!(validate_time_range("08:00", "17:00").is_ok());
        assert!(validate_time_range("08:00", "08:00").is_err()); // equal
        assert!(validate_time_range("17:00", "08:00").is_err()); // reversed
    }

    #[test]
    fn test_validate_duration() {
        assert!(validate_duration(15).is_ok());
        assert!(validate_duration(240).is_ok());
        assert!(validate_duration(14).is_err());
        assert!(validate_duration(241).is_err());
    }

    #[test]
    fn test_validate_category() {
        assert!(validate_category("Preventive").is_ok());
        assert!(validate_category("Invasive").is_ok());
        assert!(validate_category("Dental").is_err());
    }

    #[test]
    fn test_validate_date_ymd() {
        assert!(validate_date_ymd("2026-03-04").is_ok());
        assert!(validate_date_ymd("2026-3-4").is_err());   // not zero-padded
        assert!(validate_date_ymd("26-03-04").is_err());   // year wrong length
        assert!(validate_date_ymd("2026-13-01").is_err()); // invalid month
    }

    #[test]
    fn test_validate_date_range() {
        assert!(validate_date_range("2026-03-01", "2026-03-31").is_ok());
        assert!(validate_date_range("2026-03-01", "2026-03-01").is_ok()); // same day ok
        assert!(validate_date_range("2026-03-31", "2026-03-01").is_err());
    }

}
