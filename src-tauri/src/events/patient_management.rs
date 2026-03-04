use serde::{Deserialize, Serialize};

// ── Event type constants ──────────────────────────────────────────────────────

pub const PATIENT_REGISTERED: &str = "PatientRegistered";
pub const PATIENT_DEMOGRAPHICS_UPDATED: &str = "PatientDemographicsUpdated";
pub const PATIENT_CONTACT_INFO_UPDATED: &str = "PatientContactInfoUpdated";
pub const PATIENT_NOTE_ADDED: &str = "PatientNoteAdded";
pub const PATIENT_ARCHIVED: &str = "PatientArchived";
pub const PATIENT_UNARCHIVED: &str = "PatientUnarchived";

pub const ALL_EVENT_TYPES: &[&str] = &[
    PATIENT_REGISTERED,
    PATIENT_DEMOGRAPHICS_UPDATED,
    PATIENT_CONTACT_INFO_UPDATED,
    PATIENT_NOTE_ADDED,
    PATIENT_ARCHIVED,
    PATIENT_UNARCHIVED,
];

// ── Payload structs ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientRegisteredPayload {
    pub patient_id: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub preferred_contact_channel: Option<String>,
    pub preferred_office_id: Option<String>,
    pub date_of_birth: Option<String>,
    pub registered_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientDemographicsUpdatedPayload {
    pub patient_id: String,
    pub first_name: String,
    pub last_name: String,
    pub date_of_birth: Option<String>,
    pub address_line_1: Option<String>,
    pub city_town: Option<String>,
    pub subdivision: Option<String>,
    pub country: Option<String>,
    pub updated_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientContactInfoUpdatedPayload {
    pub patient_id: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub preferred_contact_channel: Option<String>,
    pub updated_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientNoteAddedPayload {
    pub patient_id: String,
    pub note_id: String,
    pub text: String,
    pub recorded_by: String,
    pub recorded_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientArchivedPayload {
    pub patient_id: String,
    pub archived_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientUnarchivedPayload {
    pub patient_id: String,
    pub unarchived_by: String,
}
