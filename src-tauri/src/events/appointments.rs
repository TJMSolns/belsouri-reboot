use serde::{Deserialize, Serialize};

pub const APPOINTMENT_BOOKED: &str = "AppointmentBooked";
pub const APPOINTMENT_RESCHEDULED: &str = "AppointmentRescheduled";
pub const APPOINTMENT_CANCELLED: &str = "AppointmentCancelled";
pub const APPOINTMENT_COMPLETED: &str = "AppointmentCompleted";
pub const APPOINTMENT_MARKED_NO_SHOW: &str = "AppointmentMarkedNoShow";
pub const APPOINTMENT_NOTE_ADDED: &str = "AppointmentNoteAdded";

pub const ALL_EVENT_TYPES: &[&str] = &[
    APPOINTMENT_BOOKED,
    APPOINTMENT_RESCHEDULED,
    APPOINTMENT_CANCELLED,
    APPOINTMENT_COMPLETED,
    APPOINTMENT_MARKED_NO_SHOW,
    APPOINTMENT_NOTE_ADDED,
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppointmentBookedPayload {
    pub appointment_id: String,
    pub office_id: String,
    pub patient_id: String,
    pub procedure_type_id: String,
    pub staff_member_id: String,
    /// Local datetime "YYYY-MM-DDTHH:MM:SS"
    pub start_time: String,
    /// Local datetime "YYYY-MM-DDTHH:MM:SS", computed: start + duration
    pub end_time: String,
    pub duration_minutes: u32,
    pub booked_by: String,
    /// Set when this appointment was created as part of a reschedule
    pub rescheduled_from_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppointmentRescheduledPayload {
    pub appointment_id: String,
    pub rescheduled_to_id: String,
    pub rescheduled_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppointmentCancelledPayload {
    pub appointment_id: String,
    pub cancelled_by: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppointmentCompletedPayload {
    pub appointment_id: String,
    pub completed_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppointmentMarkedNoShowPayload {
    pub appointment_id: String,
    pub recorded_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppointmentNoteAddedPayload {
    pub appointment_id: String,
    pub note_id: String,
    pub text: String,
    pub recorded_by: String,
    pub recorded_at: String,
}
