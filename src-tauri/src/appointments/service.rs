/// Pure validation and constraint-check functions for Patient Scheduling.
/// All functions are side-effect-free and testable without a database.

use chrono::{NaiveDateTime, Duration, Datelike};
use crate::db::{OfficeHoursRow, StaffAvailabilityRow, StaffExceptionRow};

pub fn validate_duration(minutes: u32) -> Result<(), String> {
    if minutes < 15 || minutes > 240 {
        return Err("Duration must be between 15 and 240 minutes".to_string());
    }
    Ok(())
}

/// Parse a local datetime string "YYYY-MM-DDTHH:MM:SS".
pub fn parse_datetime(s: &str) -> Result<NaiveDateTime, String> {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
        .map_err(|e| format!("Invalid datetime '{}': {}", s, e))
}

/// Compute end_time = start_time + duration_minutes, returned as "YYYY-MM-DDTHH:MM:SS".
pub fn compute_end_time(start_dt: &NaiveDateTime, duration_minutes: u32) -> String {
    let end_dt = *start_dt + Duration::minutes(duration_minutes as i64);
    end_dt.format("%Y-%m-%dT%H:%M:%S").to_string()
}

fn weekday_name(dt: &NaiveDateTime) -> &'static str {
    match dt.weekday() {
        chrono::Weekday::Mon => "Monday",
        chrono::Weekday::Tue => "Tuesday",
        chrono::Weekday::Wed => "Wednesday",
        chrono::Weekday::Thu => "Thursday",
        chrono::Weekday::Fri => "Friday",
        chrono::Weekday::Sat => "Saturday",
        chrono::Weekday::Sun => "Sunday",
    }
}

/// C1: Office is open at the requested time on that day.
/// start_dt and end_dt are both local times for the same booking.
pub fn check_c1_office_open(
    office_hours: &[OfficeHoursRow],
    office_name: &str,
    start_dt: &NaiveDateTime,
    end_dt: &NaiveDateTime,
) -> Result<(), String> {
    let day = weekday_name(start_dt);
    let start_hhmm = start_dt.format("%H:%M").to_string();
    let end_hhmm = end_dt.format("%H:%M").to_string();

    let hours = office_hours
        .iter()
        .find(|h| h.day_of_week == day)
        .ok_or_else(|| {
            format!("Office {} is not open at {} on {}", office_name, start_hhmm, day)
        })?;

    if start_hhmm < hours.open_time {
        return Err(format!("Office {} is not open at {} on {}", office_name, start_hhmm, day));
    }
    if end_hhmm > hours.close_time {
        return Err(format!("Office {} is not open at {} on {}", office_name, start_hhmm, day));
    }
    Ok(())
}

/// C2: Provider is available at that office at that time (no exception, within availability window).
pub fn check_c2_provider_available(
    availability: &[StaffAvailabilityRow],
    exceptions: &[StaffExceptionRow],
    provider_name: &str,
    office_id: &str,
    office_name: &str,
    start_dt: &NaiveDateTime,
    end_dt: &NaiveDateTime,
) -> Result<(), String> {
    let day = weekday_name(start_dt);
    let start_hhmm = start_dt.format("%H:%M").to_string();
    let end_hhmm = end_dt.format("%H:%M").to_string();
    let date_str = start_dt.format("%Y-%m-%d").to_string();

    let err = || format!("Provider {} is not available at {} at {}", provider_name, office_name, start_hhmm);

    // Check for active exception covering this date
    let has_exception = exceptions
        .iter()
        .any(|e| e.start_date <= date_str && date_str <= e.end_date);
    if has_exception {
        return Err(err());
    }

    // Check availability window for this office+day
    let avail = availability
        .iter()
        .find(|a| a.office_id == office_id && a.day_of_week == day)
        .ok_or_else(err)?;

    // Provider must be available for the entire booking duration
    if start_hhmm < avail.start_time || end_hhmm > avail.end_time {
        return Err(err());
    }
    Ok(())
}

/// C3: Chair capacity not exceeded.
pub fn check_c3_chair_capacity(
    overlapping_count: i64,
    chair_count: u32,
    office_name: &str,
    start_dt: &NaiveDateTime,
) -> Result<(), String> {
    if overlapping_count >= chair_count as i64 {
        let start_hhmm = start_dt.format("%H:%M").to_string();
        return Err(format!(
            "No chairs available at {} at {} — all {} chairs are booked",
            office_name, start_hhmm, chair_count
        ));
    }
    Ok(())
}

/// C4: Patient is active (not archived).
pub fn check_c4_patient_active(archived: bool) -> Result<(), String> {
    if archived {
        return Err("Patient is archived and cannot be booked".to_string());
    }
    Ok(())
}

/// C5: Procedure type is active.
pub fn check_c5_procedure_active(is_active: bool, procedure_name: &str) -> Result<(), String> {
    if !is_active {
        return Err(format!("Procedure type {} is no longer active", procedure_name));
    }
    Ok(())
}

pub fn validate_note_text(text: &str) -> Result<(), String> {
    if text.trim().is_empty() {
        return Err("Note text is required".to_string());
    }
    Ok(())
}

pub fn validate_recorded_by(recorded_by: &str) -> Result<(), String> {
    if recorded_by.trim().is_empty() {
        return Err("recorded_by is required".to_string());
    }
    Ok(())
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{OfficeHoursRow, StaffAvailabilityRow, StaffExceptionRow};

    fn make_hours(day: &str, open: &str, close: &str) -> OfficeHoursRow {
        OfficeHoursRow {
            office_id: "o1".to_string(),
            day_of_week: day.to_string(),
            open_time: open.to_string(),
            close_time: close.to_string(),
        }
    }

    fn make_avail(office_id: &str, day: &str, start: &str, end: &str) -> StaffAvailabilityRow {
        StaffAvailabilityRow {
            staff_member_id: "sm-1".to_string(),
            office_id: office_id.to_string(),
            day_of_week: day.to_string(),
            start_time: start.to_string(),
            end_time: end.to_string(),
        }
    }

    #[test]
    fn test_validate_duration_ok() {
        assert!(validate_duration(15).is_ok());
        assert!(validate_duration(60).is_ok());
        assert!(validate_duration(240).is_ok());
    }

    #[test]
    fn test_validate_duration_err() {
        assert!(validate_duration(0).is_err());
        assert!(validate_duration(14).is_err());
        assert!(validate_duration(241).is_err());
        let e = validate_duration(14).unwrap_err();
        assert_eq!(e, "Duration must be between 15 and 240 minutes");
    }

    #[test]
    fn test_parse_datetime() {
        let dt = parse_datetime("2026-03-09T10:00:00").unwrap();
        assert_eq!(dt.format("%H:%M").to_string(), "10:00");
    }

    #[test]
    fn test_compute_end_time() {
        let dt = parse_datetime("2026-03-09T10:00:00").unwrap();
        let end = compute_end_time(&dt, 60);
        assert_eq!(end, "2026-03-09T11:00:00");
    }

    #[test]
    fn test_c1_office_open_ok() {
        let hours = vec![make_hours("Monday", "08:00", "17:00")];
        let start = parse_datetime("2026-03-09T10:00:00").unwrap(); // Monday
        let end   = parse_datetime("2026-03-09T11:00:00").unwrap();
        assert!(check_c1_office_open(&hours, "Main Office", &start, &end).is_ok());
    }

    #[test]
    fn test_c1_office_open_exactly_at_opening() {
        let hours = vec![make_hours("Monday", "08:00", "17:00")];
        let start = parse_datetime("2026-03-09T08:00:00").unwrap();
        let end   = parse_datetime("2026-03-09T09:00:00").unwrap();
        assert!(check_c1_office_open(&hours, "Main Office", &start, &end).is_ok());
    }

    #[test]
    fn test_c1_office_closed_day() {
        let hours = vec![make_hours("Monday", "08:00", "17:00")];
        let start = parse_datetime("2026-03-07T10:00:00").unwrap(); // Saturday
        let end   = parse_datetime("2026-03-07T11:00:00").unwrap();
        let err = check_c1_office_open(&hours, "Main Office", &start, &end).unwrap_err();
        assert!(err.contains("Saturday"));
    }

    #[test]
    fn test_c1_before_opening() {
        let hours = vec![make_hours("Monday", "08:00", "17:00")];
        let start = parse_datetime("2026-03-09T07:30:00").unwrap();
        let end   = parse_datetime("2026-03-09T08:30:00").unwrap();
        let err = check_c1_office_open(&hours, "Main Office", &start, &end).unwrap_err();
        assert!(err.contains("07:30"));
    }

    #[test]
    fn test_c1_end_extends_beyond_closing() {
        let hours = vec![make_hours("Monday", "08:00", "17:00")];
        let start = parse_datetime("2026-03-09T16:45:00").unwrap();
        let end   = parse_datetime("2026-03-09T17:45:00").unwrap();
        let err = check_c1_office_open(&hours, "Main Office", &start, &end).unwrap_err();
        assert!(err.contains("16:45"));
    }

    #[test]
    fn test_c2_provider_available_ok() {
        let avail = vec![make_avail("o1", "Monday", "09:00", "16:00")];
        let start = parse_datetime("2026-03-09T10:00:00").unwrap();
        let end   = parse_datetime("2026-03-09T11:00:00").unwrap();
        assert!(check_c2_provider_available(&avail, &[], "Dr. Spence", "o1", "Main Office", &start, &end).is_ok());
    }

    #[test]
    fn test_c2_exactly_at_availability_start() {
        let avail = vec![make_avail("o1", "Monday", "09:00", "16:00")];
        let start = parse_datetime("2026-03-09T09:00:00").unwrap();
        let end   = parse_datetime("2026-03-09T10:00:00").unwrap();
        assert!(check_c2_provider_available(&avail, &[], "Dr. Spence", "o1", "Main Office", &start, &end).is_ok());
    }

    #[test]
    fn test_c2_before_availability() {
        let avail = vec![make_avail("o1", "Monday", "09:00", "16:00")];
        let start = parse_datetime("2026-03-09T08:00:00").unwrap();
        let end   = parse_datetime("2026-03-09T09:00:00").unwrap();
        let err = check_c2_provider_available(&avail, &[], "Dr. Spence", "o1", "Main Office", &start, &end).unwrap_err();
        assert!(err.contains("Dr. Spence"));
        assert!(err.contains("08:00"));
    }

    #[test]
    fn test_c2_active_exception() {
        let avail = vec![make_avail("o1", "Monday", "09:00", "16:00")];
        let exc = vec![StaffExceptionRow {
            staff_member_id: "sm-1".to_string(),
            start_date: "2026-03-09".to_string(),
            end_date: "2026-03-09".to_string(),
            reason: None,
        }];
        let start = parse_datetime("2026-03-09T10:00:00").unwrap();
        let end   = parse_datetime("2026-03-09T11:00:00").unwrap();
        assert!(check_c2_provider_available(&avail, &exc, "Dr. Spence", "o1", "Main Office", &start, &end).is_err());
    }

    #[test]
    fn test_c2_wrong_office() {
        let avail = vec![make_avail("o1", "Monday", "09:00", "16:00")];
        let start = parse_datetime("2026-03-09T10:00:00").unwrap();
        let end   = parse_datetime("2026-03-09T11:00:00").unwrap();
        // Provider has no availability at "o2"
        assert!(check_c2_provider_available(&avail, &[], "Dr. Spence", "o2", "Other Office", &start, &end).is_err());
    }

    #[test]
    fn test_c3_under_capacity() {
        assert!(check_c3_chair_capacity(2, 3, "Main Office", &parse_datetime("2026-03-09T10:00:00").unwrap()).is_ok());
    }

    #[test]
    fn test_c3_at_capacity() {
        let err = check_c3_chair_capacity(3, 3, "Main Office", &parse_datetime("2026-03-09T10:00:00").unwrap()).unwrap_err();
        assert!(err.contains("all 3 chairs"));
        assert!(err.contains("10:00"));
    }

    #[test]
    fn test_c4_active_patient() {
        assert!(check_c4_patient_active(false).is_ok());
    }

    #[test]
    fn test_c4_archived_patient() {
        let err = check_c4_patient_active(true).unwrap_err();
        assert!(err.contains("archived"));
    }

    #[test]
    fn test_c5_active_procedure() {
        assert!(check_c5_procedure_active(true, "Cleaning").is_ok());
    }

    #[test]
    fn test_c5_inactive_procedure() {
        let err = check_c5_procedure_active(false, "Cleaning").unwrap_err();
        assert!(err.contains("Cleaning"));
        assert!(err.contains("no longer active"));
    }

    #[test]
    fn test_validate_note_text() {
        assert!(validate_note_text("Patient arrived late").is_ok());
        assert!(validate_note_text("").is_err());
        assert!(validate_note_text("   ").is_err());
    }
}
