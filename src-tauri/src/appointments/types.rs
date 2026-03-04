use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AppointmentNoteDto {
    pub note_id: String,
    pub appointment_id: String,
    pub text: String,
    pub recorded_by: String,
    pub recorded_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AppointmentDto {
    pub appointment_id: String,
    pub office_id: String,
    pub patient_id: String,
    pub patient_name: String,
    pub patient_phone: Option<String>,
    pub procedure_type_id: String,
    pub procedure_name: String,
    pub procedure_category: String,
    pub provider_id: String,
    pub provider_name: String,
    pub start_time: String,
    pub end_time: String,
    pub duration_minutes: u32,
    pub status: String,
    pub rescheduled_to_id: Option<String>,
    pub rescheduled_from_id: Option<String>,
    pub booked_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AppointmentWithNotesDto {
    pub appointment: AppointmentDto,
    pub notes: Vec<AppointmentNoteDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CallListEntryDto {
    pub appointment_id: String,
    pub office_id: String,
    pub patient_name: String,
    pub patient_phone: Option<String>,
    pub patient_email: Option<String>,
    pub preferred_contact_channel: Option<String>,
    pub procedure_name: String,
    pub provider_name: String,
    pub start_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BookAppointmentResult {
    pub appointment_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RescheduleAppointmentResult {
    pub new_appointment_id: String,
}
