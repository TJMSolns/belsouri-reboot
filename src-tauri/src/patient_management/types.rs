use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PatientDto {
    pub patient_id: String,
    pub first_name: String,
    pub last_name: String,
    pub full_name_display: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub preferred_contact_channel: Option<String>,
    pub preferred_office_id: Option<String>,
    pub date_of_birth: Option<String>,
    pub address_line_1: Option<String>,
    pub city_town: Option<String>,
    pub subdivision: Option<String>,
    pub country: Option<String>,
    pub registered_by: String,
    pub registered_at: String,
    pub archived: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PatientNoteDto {
    pub note_id: String,
    pub patient_id: String,
    pub text: String,
    pub recorded_by: String,
    pub recorded_at: String,
}

/// Returned from register_patient. Includes the new patient record and
/// an optional soft duplicate warning (does NOT block registration).
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RegisterPatientResult {
    pub patient: PatientDto,
    pub duplicate_warning: Option<String>,
}

/// Returned from get_patient — includes the patient plus their notes.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PatientWithNotesDto {
    pub patient: PatientDto,
    pub notes: Vec<PatientNoteDto>,
}
