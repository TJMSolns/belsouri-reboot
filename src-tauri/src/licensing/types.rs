use serde::{Deserialize, Serialize};
use specta::Type;

/// Payload deserialized from a license key's JSON body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicensePayload {
    pub schema_version: u32,
    pub license_type: String,          // "eval" | "paid"
    pub practice_id: Option<String>,   // null for eval tokens
    pub issued_at: String,             // ISO 8601
    pub max_duration_days: Option<u32>, // eval only
    pub modules: Vec<ModulePayloadEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModulePayloadEntry {
    pub name: String,
    pub grace_period_days: u32,
    pub expires_at: Option<String>, // None for eval (computed from issued_at + max_duration_days)
}

/// Per-module status returned to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ModuleStatusDto {
    pub module_name: String,
    pub status: String,                  // "Active" | "Degraded" | "Expired"
    pub expires_at: String,              // ISO 8601
    pub grace_period_days: u32,
    pub grace_expires_at: Option<String>, // ISO 8601, present when Degraded
    pub days_remaining: Option<i64>,     // computed at read time; None when Expired
}

/// Overall license status returned to the frontend.
/// All datetime fields are ISO 8601 strings — DateTime<Utc> does not impl specta::Type.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct LicenseStatusDto {
    pub overall_validity: String,          // "Valid" | "Invalid"
    pub license_type: Option<String>,      // "eval" | "paid" | null (not yet activated)
    pub eval_expires_at: Option<String>,   // ISO 8601, present during eval
    pub modules: Vec<ModuleStatusDto>,
    pub last_validated_at: Option<String>, // ISO 8601
}

impl LicenseStatusDto {
    /// Returns a "not activated" status — used before startup_license_check runs.
    pub fn not_activated() -> Self {
        Self {
            overall_validity: "Invalid".to_string(),
            license_type: None,
            eval_expires_at: None,
            modules: vec![],
            last_validated_at: None,
        }
    }
}
