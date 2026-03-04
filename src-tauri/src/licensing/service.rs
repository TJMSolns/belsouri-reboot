use chrono::{DateTime, Duration, Utc};
use sha2::{Digest, Sha256};

use crate::db::StoredEvent;
use crate::events::licensing::*;
use crate::licensing::types::{LicensePayload, LicenseStatusDto, ModuleStatusDto};

pub use machine_uid;

/// Returns the machine's stable unique identifier.
/// On Windows: MachineGuid from registry.
/// On Linux: /etc/machine-id.
pub fn get_machine_id() -> Result<String, String> {
    machine_uid::get().map_err(|e| format!("Failed to get machine ID: {e}"))
}

/// Derives a 64-char lowercase hex PracticeId from machine_id and install_date.
/// PracticeId = SHA-256(machine_id || ":" || install_date), hex-encoded.
/// `install_date` must be in YYYY-MM-DD format.
pub fn derive_practice_id(machine_id: &str, install_date: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(machine_id.as_bytes());
    hasher.update(b":");
    hasher.update(install_date.as_bytes());
    hex::encode(hasher.finalize())
}

/// Derives a 64-char lowercase hex hash of the machine_id alone.
/// Stored in PracticeIdentityEstablished for support purposes.
/// Distinct from PracticeId (which also incorporates install_date).
pub fn derive_machine_id_hash(machine_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(machine_id.as_bytes());
    hex::encode(hasher.finalize())
}

/// Computes the current license status from all licensing events.
/// Pure function — deterministic given the same inputs.
/// `now` is passed as a parameter to keep this testable.
pub fn compute_license_status(events: &[StoredEvent], now: DateTime<Utc>) -> LicenseStatusDto {
    // Walk events in order to build current state
    let mut license_type: Option<String> = None;
    let mut eval_started_at: Option<DateTime<Utc>> = None;
    let mut eval_duration_days: Option<u32> = None;
    let mut module_expires: std::collections::HashMap<String, (String, u32)> = Default::default(); // name -> (expires_at, grace_days)
    let mut last_validated_at: Option<String> = None;
    let mut clock_rollback = false;
    let mut is_degraded: std::collections::HashSet<String> = Default::default();
    let mut is_expired: std::collections::HashSet<String> = Default::default();

    for event in events {
        match event.event_type.as_str() {
            t if t == EVAL_STARTED => {
                if let Ok(p) = serde_json::from_str::<EvalStartedPayload>(&event.payload) {
                    license_type = Some("eval".to_string());
                    eval_started_at = parse_dt(&p.started_at);
                    // Re-read max_duration from the payload via eval_expires_at
                    if let Some(started) = eval_started_at {
                        if let Some(expires) = parse_dt(&p.eval_expires_at) {
                            let days = (expires - started).num_days();
                            eval_duration_days = Some(days.max(0) as u32);
                        }
                    }
                    for module_name in &p.modules {
                        // Eval modules expire at started_at + duration
                        if let (Some(started), Some(duration)) = (eval_started_at, eval_duration_days) {
                            let expires = started + Duration::days(duration as i64);
                            module_expires.insert(
                                module_name.clone(),
                                (expires.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(), 90),
                            );
                        }
                    }
                }
            }
            t if t == LICENSE_ISSUED || t == LICENSE_RENEWED => {
                if let Ok(p) = serde_json::from_str::<LicenseIssuedPayload>(&event.payload) {
                    license_type = Some("paid".to_string());
                    eval_started_at = None;
                    eval_duration_days = None;
                    module_expires.clear();
                    is_degraded.clear();
                    is_expired.clear();
                    for m in &p.modules {
                        module_expires.insert(m.name.clone(), (m.expires_at.clone(), m.grace_period_days));
                    }
                }
            }
            t if t == LICENSE_VALIDATION_SUCCEEDED => {
                if let Ok(p) = serde_json::from_str::<LicenseValidationSucceededPayload>(&event.payload) {
                    last_validated_at = Some(p.validated_at);
                }
            }
            t if t == LICENSE_DEGRADED => {
                if let Ok(p) = serde_json::from_str::<LicenseDegradedPayload>(&event.payload) {
                    is_degraded.insert(p.module_name);
                }
            }
            t if t == LICENSE_EXPIRED => {
                if let Ok(p) = serde_json::from_str::<LicenseExpiredPayload>(&event.payload) {
                    is_degraded.remove(&p.module_name);
                    is_expired.insert(p.module_name);
                }
            }
            t if t == CLOCK_ROLLBACK_DETECTED => {
                clock_rollback = true;
            }
            _ => {}
        }
    }

    if clock_rollback {
        return LicenseStatusDto {
            overall_validity: "Invalid".to_string(),
            license_type,
            eval_expires_at: None,
            modules: vec![],
            last_validated_at,
        };
    }

    // Build module statuses
    let mut modules: Vec<ModuleStatusDto> = module_expires
        .iter()
        .map(|(name, (expires_at_str, grace_days))| {
            let expires_at = parse_dt(expires_at_str).unwrap_or(now);
            let grace_expires = expires_at + Duration::days(*grace_days as i64);

            let (status, days_remaining, grace_expires_at) = if is_expired.contains(name) {
                ("Expired".to_string(), None, None)
            } else if is_degraded.contains(name) || now > expires_at {
                let grace_remaining = (grace_expires - now).num_days();
                (
                    "Degraded".to_string(),
                    Some(grace_remaining.max(0)),
                    Some(grace_expires.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()),
                )
            } else {
                let days = (expires_at - now).num_days();
                ("Active".to_string(), Some(days.max(0)), None)
            };

            ModuleStatusDto {
                module_name: name.clone(),
                status,
                expires_at: expires_at_str.clone(),
                grace_period_days: *grace_days,
                grace_expires_at,
                days_remaining,
            }
        })
        .collect();

    modules.sort_by(|a, b| a.module_name.cmp(&b.module_name));

    let all_expired = !modules.is_empty() && modules.iter().all(|m| m.status == "Expired");
    let overall_validity = if all_expired { "Invalid" } else { "Valid" }.to_string();

    let eval_expires_at = if license_type.as_deref() == Some("eval") {
        eval_started_at.and_then(|started| {
            eval_duration_days.map(|days| {
                (started + Duration::days(days as i64))
                    .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                    .to_string()
            })
        })
    } else {
        None
    };

    LicenseStatusDto {
        overall_validity,
        license_type,
        eval_expires_at,
        modules,
        last_validated_at,
    }
}

fn parse_dt(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

/// Returns today's date as YYYY-MM-DD (for use as install_date on first run).
pub fn today_as_install_date() -> String {
    Utc::now().format("%Y-%m-%d").to_string()
}

/// Parses a LicensePayload from raw JSON bytes (after signature verification).
pub fn parse_license_payload(payload_bytes: &[u8]) -> Result<LicensePayload, String> {
    serde_json::from_slice(payload_bytes).map_err(|e| format!("Invalid license payload: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_event(id: i64, event_type: &str, payload: &str) -> StoredEvent {
        StoredEvent {
            id,
            stream_id: "license".to_string(),
            stream_version: id as u64,
            event_type: event_type.to_string(),
            payload: payload.to_string(),
            created_at: "2026-03-04T10:00:00.000Z".to_string(),
        }
    }

    fn now() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-03-04T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn test_derive_practice_id_is_deterministic() {
        let id1 = derive_practice_id("machine-123", "2026-03-04");
        let id2 = derive_practice_id("machine-123", "2026-03-04");
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_derive_practice_id_is_64_hex_chars() {
        let id = derive_practice_id("machine-123", "2026-03-04");
        assert_eq!(id.len(), 64);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(id, id.to_lowercase());
    }

    #[test]
    fn test_derive_practice_id_changes_with_date() {
        let id1 = derive_practice_id("machine-123", "2026-03-04");
        let id2 = derive_practice_id("machine-123", "2026-03-05");
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_machine_id_hash_differs_from_practice_id() {
        let hash = derive_machine_id_hash("machine-123");
        let practice_id = derive_practice_id("machine-123", "2026-03-04");
        assert_ne!(hash, practice_id);
    }

    #[test]
    fn test_compute_status_no_events() {
        let status = compute_license_status(&[], now());
        // No license type, no modules, invalid overall
        assert_eq!(status.license_type, None);
        assert!(status.modules.is_empty());
    }

    #[test]
    fn test_compute_status_eval_active() {
        let eval_payload = serde_json::json!({
            "practice_id": "a".repeat(64),
            "started_at": "2026-03-04T00:00:00Z",
            "eval_expires_at": "2026-04-03T00:00:00Z",
            "modules": ["scheduling"]
        });
        let events = vec![make_event(1, EVAL_STARTED, &eval_payload.to_string())];
        let status = compute_license_status(&events, now());
        assert_eq!(status.license_type, Some("eval".to_string()));
        assert_eq!(status.overall_validity, "Valid");
        assert_eq!(status.modules.len(), 1);
        assert_eq!(status.modules[0].status, "Active");
        assert!(status.modules[0].days_remaining.unwrap() > 0);
    }

    #[test]
    fn test_compute_status_eval_expired() {
        let eval_payload = serde_json::json!({
            "practice_id": "a".repeat(64),
            "started_at": "2026-01-01T00:00:00Z",
            "eval_expires_at": "2026-01-31T00:00:00Z",
            "modules": ["scheduling"]
        });
        // now() is 2026-03-04, well past Jan 31 + 90 days grace
        let events = vec![make_event(1, EVAL_STARTED, &eval_payload.to_string())];
        // Use a "now" past the grace period too (90 days after Jan 31 = May 1)
        let future = DateTime::parse_from_rfc3339("2026-06-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let status = compute_license_status(&events, future);
        // Still Degraded (not yet Expired event) but grace is past
        assert_eq!(status.modules[0].status, "Degraded");
        assert_eq!(status.modules[0].days_remaining.unwrap(), 0);
    }

    #[test]
    fn test_compute_status_paid_license_active() {
        let license_payload = serde_json::json!({
            "practice_id": "a".repeat(64),
            "license_type": "paid",
            "issued_at": "2026-03-04T00:00:00Z",
            "modules": [{
                "name": "scheduling",
                "expires_at": "2027-03-04T00:00:00Z",
                "grace_period_days": 90
            }],
            "schema_version": 2
        });
        let events = vec![make_event(1, LICENSE_ISSUED, &license_payload.to_string())];
        let status = compute_license_status(&events, now());
        assert_eq!(status.license_type, Some("paid".to_string()));
        assert_eq!(status.overall_validity, "Valid");
        assert_eq!(status.modules[0].status, "Active");
    }

    #[test]
    fn test_compute_status_clock_rollback() {
        let rollback_payload = serde_json::json!({
            "detected_at": "2026-03-04T10:00:00Z",
            "last_seen_at": "2026-03-05T10:00:00Z"
        });
        let events = vec![make_event(1, CLOCK_ROLLBACK_DETECTED, &rollback_payload.to_string())];
        let status = compute_license_status(&events, now());
        assert_eq!(status.overall_validity, "Invalid");
        assert!(status.modules.is_empty());
    }

    #[test]
    fn test_compute_status_license_renewed_replaces_eval() {
        let eval_payload = serde_json::json!({
            "practice_id": "a".repeat(64),
            "started_at": "2026-03-01T00:00:00Z",
            "eval_expires_at": "2026-03-31T00:00:00Z",
            "modules": ["scheduling"]
        });
        let license_payload = serde_json::json!({
            "practice_id": "a".repeat(64),
            "license_type": "paid",
            "issued_at": "2026-03-04T00:00:00Z",
            "modules": [{
                "name": "scheduling",
                "expires_at": "2027-03-04T00:00:00Z",
                "grace_period_days": 90
            }],
            "schema_version": 2
        });
        let events = vec![
            make_event(1, EVAL_STARTED, &eval_payload.to_string()),
            make_event(2, LICENSE_ISSUED, &license_payload.to_string()),
        ];
        let status = compute_license_status(&events, now());
        assert_eq!(status.license_type, Some("paid".to_string()));
        assert_eq!(status.eval_expires_at, None);
    }
}
