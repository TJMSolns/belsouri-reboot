/// Validates first or last name: non-empty, max 100 chars.
pub fn validate_name(name: &str, field: &str) -> Result<(), String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(format!("{} is required", field));
    }
    if trimmed.len() > 100 {
        return Err(format!("{} must be 100 characters or fewer", field));
    }
    Ok(())
}

/// Validates that at least one of phone or email is provided.
pub fn validate_contact_required(
    phone: Option<&str>,
    email: Option<&str>,
) -> Result<(), String> {
    let has_phone = phone.map(|s| !s.trim().is_empty()).unwrap_or(false);
    let has_email = email.map(|s| !s.trim().is_empty()).unwrap_or(false);
    if !has_phone && !has_email {
        return Err("At least one of phone or email is required".to_string());
    }
    Ok(())
}

/// Validates preferred_contact_channel is one of the allowed values (or None).
pub fn validate_preferred_channel(channel: Option<&str>) -> Result<(), String> {
    const ALLOWED: &[&str] = &["Phone", "Email", "WhatsApp"];
    if let Some(c) = channel {
        if !ALLOWED.contains(&c) {
            return Err(format!(
                "preferred_contact_channel must be one of: {}",
                ALLOWED.join(", ")
            ));
        }
    }
    Ok(())
}

/// Validates a date string in YYYY-MM-DD format (basic check).
pub fn validate_date_ymd_opt(date: Option<&str>, field: &str) -> Result<(), String> {
    if let Some(d) = date {
        let d = d.trim();
        if d.len() != 10 || !d.chars().nth(4).map(|c| c == '-').unwrap_or(false)
            || !d.chars().nth(7).map(|c| c == '-').unwrap_or(false)
        {
            return Err(format!("{} must be in YYYY-MM-DD format", field));
        }
    }
    Ok(())
}

/// Validates note text is non-empty and ≤ 2000 chars.
pub fn validate_note_text(text: &str) -> Result<(), String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err("Note text is required".to_string());
    }
    if trimmed.len() > 2000 {
        return Err("Note text must be 2000 characters or fewer".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_name_empty() {
        assert!(validate_name("", "First name").is_err());
        assert!(validate_name("  ", "First name").is_err());
    }

    #[test]
    fn test_validate_name_too_long() {
        let long = "a".repeat(101);
        assert!(validate_name(&long, "First name").is_err());
    }

    #[test]
    fn test_validate_name_ok() {
        assert!(validate_name("Maria", "First name").is_ok());
    }

    #[test]
    fn test_contact_required_both_missing() {
        assert!(validate_contact_required(None, None).is_err());
        assert!(validate_contact_required(Some(""), Some("")).is_err());
        assert!(validate_contact_required(Some("  "), None).is_err());
    }

    #[test]
    fn test_contact_required_phone_only() {
        assert!(validate_contact_required(Some("+18765551234"), None).is_ok());
    }

    #[test]
    fn test_contact_required_email_only() {
        assert!(validate_contact_required(None, Some("a@b.com")).is_ok());
    }

    #[test]
    fn test_contact_required_both() {
        assert!(validate_contact_required(Some("123"), Some("a@b.com")).is_ok());
    }

    #[test]
    fn test_validate_preferred_channel_valid() {
        assert!(validate_preferred_channel(None).is_ok());
        assert!(validate_preferred_channel(Some("Phone")).is_ok());
        assert!(validate_preferred_channel(Some("Email")).is_ok());
        assert!(validate_preferred_channel(Some("WhatsApp")).is_ok());
    }

    #[test]
    fn test_validate_preferred_channel_invalid() {
        assert!(validate_preferred_channel(Some("SMS")).is_err());
        assert!(validate_preferred_channel(Some("phone")).is_err());
    }

    #[test]
    fn test_validate_date_ymd_opt_valid() {
        assert!(validate_date_ymd_opt(None, "dob").is_ok());
        assert!(validate_date_ymd_opt(Some("1990-05-15"), "dob").is_ok());
    }

    #[test]
    fn test_validate_date_ymd_opt_invalid() {
        assert!(validate_date_ymd_opt(Some("15/05/1990"), "dob").is_err());
        assert!(validate_date_ymd_opt(Some("1990-5-15"), "dob").is_err());
    }

    #[test]
    fn test_validate_note_text() {
        assert!(validate_note_text("").is_err());
        assert!(validate_note_text("Allergic to penicillin").is_ok());
        assert!(validate_note_text(&"x".repeat(2001)).is_err());
        assert!(validate_note_text(&"x".repeat(2000)).is_ok());
    }
}
