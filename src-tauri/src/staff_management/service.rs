const VALID_ROLES: &[&str] = &["PracticeManager", "Provider", "Staff"];
const VALID_CHANNELS: &[&str] = &["WhatsApp", "SMS", "Phone", "Email"];

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
}
