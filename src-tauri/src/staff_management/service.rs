const VALID_ROLES: &[&str] = &["PracticeManager", "Provider", "Staff"];
const VALID_CHANNELS: &[&str] = &["WhatsApp", "SMS", "Phone", "Email"];
const VALID_CLINICAL_SPECIALIZATIONS: &[&str] = &["Dentist", "Hygienist", "Specialist"];

pub fn validate_name(name: &str) -> Result<(), String> {
    let t = name.trim();
    if t.is_empty() {
        return Err("Name is required".to_string());
    }
    if t.len() > 100 {
        return Err("Name must be 100 characters or fewer".to_string());
    }
    Ok(())
}

pub fn validate_role(role: &str) -> Result<(), String> {
    if VALID_ROLES.contains(&role) {
        Ok(())
    } else {
        Err("Role must be PracticeManager, Provider, or Staff".to_string())
    }
}

pub fn validate_preferred_channel(channel: Option<&str>) -> Result<(), String> {
    if let Some(c) = channel {
        if !VALID_CHANNELS.contains(&c) {
            return Err(format!(
                "preferred_contact_channel must be one of: {}",
                VALID_CHANNELS.join(", ")
            ));
        }
    }
    Ok(())
}

pub fn validate_clinical_specialization(s: &str) -> Result<(), String> {
    if VALID_CLINICAL_SPECIALIZATIONS.contains(&s) {
        Ok(())
    } else {
        Err(format!(
            "Invalid clinical specialization '{}'. Must be one of: {}",
            s,
            VALID_CLINICAL_SPECIALIZATIONS.join(", ")
        ))
    }
}

/// Validates HH:MM format with valid hour (0–23) and minute (0–59), zero-padded.
pub fn validate_hhmm(t: &str) -> Result<(), String> {
    let parts: Vec<&str> = t.split(':').collect();
    if parts.len() != 2 {
        return Err(format!("Time '{}' must be in HH:MM format", t));
    }
    let h: u32 = parts[0].parse().map_err(|_| format!("Invalid hour in '{}'", t))?;
    let m: u32 = parts[1].parse().map_err(|_| format!("Invalid minute in '{}'", t))?;
    if h > 23 || m > 59 {
        return Err(format!("Time '{}' out of range (00:00–23:59)", t));
    }
    if parts[0].len() != 2 || parts[1].len() != 2 {
        return Err(format!("Time '{}' must be zero-padded (e.g. 08:00)", t));
    }
    Ok(())
}

/// Validates that end time is strictly after start time (both HH:MM strings).
pub fn validate_time_range(start: &str, end: &str) -> Result<(), String> {
    if end <= start {
        return Err(format!(
            "End time '{}' must be after start time '{}'",
            end, start
        ));
    }
    Ok(())
}

/// Validates YYYY-MM-DD format (basic structural check).
pub fn validate_date_ymd(d: &str) -> Result<(), String> {
    let parts: Vec<&str> = d.split('-').collect();
    if parts.len() != 3 || parts[0].len() != 4 || parts[1].len() != 2 || parts[2].len() != 2 {
        return Err(format!("Date '{}' must be in YYYY-MM-DD format", d));
    }
    let _y: u32 = parts[0].parse().map_err(|_| format!("Invalid year in '{}'", d))?;
    let m: u32 = parts[1].parse().map_err(|_| format!("Invalid month in '{}'", d))?;
    let day: u32 = parts[2].parse().map_err(|_| format!("Invalid day in '{}'", d))?;
    if m < 1 || m > 12 {
        return Err(format!("Month {} is out of range in '{}'", m, d));
    }
    if day < 1 || day > 31 {
        return Err(format!("Day {} is out of range in '{}'", day, d));
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

/// Check for cross-office time overlap on the same day.
/// Returns Err if any existing window at a DIFFERENT office on the same day overlaps
/// the proposed window.
pub fn check_no_cross_office_overlap(
    existing: &[crate::db::StaffAvailabilityRow],
    office_id: &str,
    day: &str,
    start: &str,
    end: &str,
) -> Result<(), String> {
    for w in existing {
        if w.office_id == office_id || w.day_of_week != day {
            continue;
        }
        // Overlap: not (new_end <= existing_start || new_start >= existing_end)
        if start < w.end_time.as_str() && w.start_time.as_str() < end {
            return Err(format!(
                "Proposed window ({} {}-{} at office {}) overlaps with existing window \
                 ({}-{} at office {}) on {}",
                day, start, end, office_id,
                w.start_time, w.end_time, w.office_id,
                day
            ));
        }
    }
    Ok(())
}

/// PIN must be 4–6 ASCII digits.
pub fn validate_pin(pin: &str) -> Result<(), String> {
    let len = pin.len();
    if len < 4 || len > 6 {
        return Err("PIN must be 4 to 6 digits".to_string());
    }
    if !pin.chars().all(|c| c.is_ascii_digit()) {
        return Err("PIN must contain digits only".to_string());
    }
    Ok(())
}

/// Hash a PIN using bcrypt with a sensible cost factor.
pub fn hash_pin(pin: &str) -> Result<String, String> {
    bcrypt::hash(pin, 10).map_err(|e| format!("Failed to hash PIN: {}", e))
}

/// Verify a raw PIN against a stored bcrypt hash.
pub fn verify_pin(pin: &str, hash: &str) -> bool {
    bcrypt::verify(pin, hash).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_name_empty() {
        assert!(validate_name("").is_err());
        assert!(validate_name("  ").is_err());
    }

    #[test]
    fn test_validate_name_ok() {
        assert!(validate_name("Dr. Spence").is_ok());
    }

    #[test]
    fn test_validate_role_valid() {
        assert!(validate_role("PracticeManager").is_ok());
        assert!(validate_role("Provider").is_ok());
        assert!(validate_role("Staff").is_ok());
    }

    #[test]
    fn test_validate_role_invalid() {
        assert!(validate_role("Admin").is_err());
        assert!(validate_role("practice_manager").is_err());
    }

    #[test]
    fn test_validate_channel_all_valid() {
        for c in ["WhatsApp", "SMS", "Phone", "Email"] {
            assert!(validate_preferred_channel(Some(c)).is_ok(), "should accept {}", c);
        }
        assert!(validate_preferred_channel(Some("Telegram")).is_err());
    }

    #[test]
    fn test_validate_pin_valid() {
        assert!(validate_pin("1234").is_ok());
        assert!(validate_pin("12345").is_ok());
        assert!(validate_pin("123456").is_ok());
    }

    #[test]
    fn test_validate_pin_too_short() {
        assert!(validate_pin("123").is_err());
    }

    #[test]
    fn test_validate_pin_too_long() {
        assert!(validate_pin("1234567").is_err());
    }

    #[test]
    fn test_validate_pin_non_digit() {
        assert!(validate_pin("12ab").is_err());
    }

    #[test]
    fn test_hash_and_verify_pin() {
        let hash = hash_pin("5678").unwrap();
        assert!(verify_pin("5678", &hash));
        assert!(!verify_pin("0000", &hash));
    }

    #[test]
    fn test_validate_clinical_specialization() {
        assert!(validate_clinical_specialization("Dentist").is_ok());
        assert!(validate_clinical_specialization("Hygienist").is_ok());
        assert!(validate_clinical_specialization("Specialist").is_ok());
        assert!(validate_clinical_specialization("Nurse").is_err());
        assert!(validate_clinical_specialization("dentist").is_err());
    }

    #[test]
    fn test_validate_hhmm_valid() {
        assert!(validate_hhmm("08:00").is_ok());
        assert!(validate_hhmm("23:59").is_ok());
        assert!(validate_hhmm("00:00").is_ok());
    }

    #[test]
    fn test_validate_hhmm_invalid() {
        assert!(validate_hhmm("8:00").is_err());   // not zero-padded
        assert!(validate_hhmm("24:00").is_err());  // hour out of range
        assert!(validate_hhmm("12:60").is_err());  // minute out of range
        assert!(validate_hhmm("1200").is_err());   // wrong format
    }

    #[test]
    fn test_validate_time_range() {
        assert!(validate_time_range("08:00", "17:00").is_ok());
        assert!(validate_time_range("08:00", "08:00").is_err());
        assert!(validate_time_range("17:00", "08:00").is_err());
    }

    #[test]
    fn test_validate_date_ymd() {
        assert!(validate_date_ymd("2026-03-04").is_ok());
        assert!(validate_date_ymd("2026-3-4").is_err());
        assert!(validate_date_ymd("26-03-04").is_err());
        assert!(validate_date_ymd("2026-13-01").is_err());
    }

    #[test]
    fn test_validate_date_range() {
        assert!(validate_date_range("2026-03-01", "2026-03-31").is_ok());
        assert!(validate_date_range("2026-03-01", "2026-03-01").is_ok());
        assert!(validate_date_range("2026-03-31", "2026-03-01").is_err());
    }

    #[test]
    fn test_cross_office_overlap() {
        use crate::db::StaffAvailabilityRow;
        let existing = vec![StaffAvailabilityRow {
            staff_member_id: "sm-1".to_string(),
            office_id: "off-a".to_string(),
            day_of_week: "Monday".to_string(),
            start_time: "10:00".to_string(),
            end_time: "14:00".to_string(),
        }];
        // Same office — no error
        assert!(check_no_cross_office_overlap(&existing, "off-a", "Monday", "09:00", "16:00").is_ok());
        // Different day — no error
        assert!(check_no_cross_office_overlap(&existing, "off-b", "Tuesday", "12:00", "16:00").is_ok());
        // Different office, overlapping time — error
        assert!(check_no_cross_office_overlap(&existing, "off-b", "Monday", "12:00", "16:00").is_err());
        // Adjacent (no overlap)
        assert!(check_no_cross_office_overlap(&existing, "off-b", "Monday", "14:00", "17:00").is_ok());
    }
}
