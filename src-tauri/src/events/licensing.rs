use serde::{Deserialize, Serialize};

// Event type string constants — used as the `event_type` column in the event store
pub const PRACTICE_IDENTITY_ESTABLISHED: &str = "PracticeIdentityEstablished";
pub const EVAL_STARTED: &str = "EvalStarted";
pub const LICENSE_ISSUED: &str = "LicenseIssued";
pub const LICENSE_RENEWED: &str = "LicenseRenewed";
pub const LICENSE_VALIDATION_SUCCEEDED: &str = "LicenseValidationSucceeded";
pub const LICENSE_DEGRADED: &str = "LicenseDegraded";
pub const LICENSE_EXPIRED: &str = "LicenseExpired";
pub const CLOCK_ROLLBACK_DETECTED: &str = "ClockRollbackDetected";

// Stream IDs
pub const STREAM_PRACTICE_IDENTITY: &str = "practice_identity";
pub const STREAM_LICENSE: &str = "license";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PracticeIdentityEstablishedPayload {
    pub practice_id: String,
    pub machine_id_hash: String,
    pub install_date: String,   // YYYY-MM-DD
    pub established_at: String, // ISO 8601
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalStartedPayload {
    pub practice_id: String,
    pub started_at: String,
    pub eval_expires_at: String,
    pub modules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseModuleEntryPayload {
    pub name: String,
    pub expires_at: String,
    pub grace_period_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseIssuedPayload {
    pub practice_id: String,
    pub license_type: String, // "paid"
    pub issued_at: String,
    pub modules: Vec<LicenseModuleEntryPayload>,
    pub schema_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseRenewedPayload {
    pub practice_id: String,
    pub renewed_at: String,
    pub modules: Vec<LicenseModuleEntryPayload>,
    pub schema_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseValidationSucceededPayload {
    pub validated_at: String,
    pub next_check_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseDegradedPayload {
    pub module_name: String,
    pub degraded_at: String,
    pub grace_expires_at: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseExpiredPayload {
    pub module_name: String,
    pub expired_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClockRollbackDetectedPayload {
    pub detected_at: String,
    pub last_seen_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_practice_identity_established_roundtrip() {
        let payload = PracticeIdentityEstablishedPayload {
            practice_id: "a".repeat(64),
            machine_id_hash: "b".repeat(64),
            install_date: "2026-03-04".to_string(),
            established_at: "2026-03-04T10:00:00.000Z".to_string(),
        };
        let json = serde_json::to_string(&payload).unwrap();
        let back: PracticeIdentityEstablishedPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(back.practice_id, payload.practice_id);
        assert_eq!(back.install_date, payload.install_date);
    }

    #[test]
    fn test_eval_started_roundtrip() {
        let payload = EvalStartedPayload {
            practice_id: "a".repeat(64),
            started_at: "2026-03-04T10:00:00.000Z".to_string(),
            eval_expires_at: "2026-04-03T10:00:00.000Z".to_string(),
            modules: vec!["scheduling".to_string()],
        };
        let json = serde_json::to_string(&payload).unwrap();
        let back: EvalStartedPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(back.modules, vec!["scheduling"]);
    }

    #[test]
    fn test_license_issued_roundtrip() {
        let payload = LicenseIssuedPayload {
            practice_id: "a".repeat(64),
            license_type: "paid".to_string(),
            issued_at: "2026-03-04T10:00:00.000Z".to_string(),
            modules: vec![LicenseModuleEntryPayload {
                name: "scheduling".to_string(),
                expires_at: "2027-03-04T10:00:00.000Z".to_string(),
                grace_period_days: 90,
            }],
            schema_version: 2,
        };
        let json = serde_json::to_string(&payload).unwrap();
        let back: LicenseIssuedPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(back.modules[0].name, "scheduling");
        assert_eq!(back.schema_version, 2);
    }
}
