use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StaffMemberDto {
    pub staff_member_id: String,
    pub name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub preferred_contact_channel: Option<String>,
    /// True if the staff member has a PIN set (PIN hash itself is never exposed).
    pub has_pin: bool,
    pub roles: Vec<String>,
    pub archived: bool,
    pub clinical_specialization: Option<String>,
    pub office_ids: Vec<String>,
    pub availability: Vec<AvailabilityWindowDto>,
    pub exceptions: Vec<AvailabilityExceptionDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AvailabilityWindowDto {
    pub office_id: String,
    pub day_of_week: String,
    pub start_time: String,
    pub end_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AvailabilityExceptionDto {
    pub start_date: String,
    pub end_date: String,
    pub reason: Option<String>,
}

/// Returns whether the Staff Management setup step is complete.
/// Complete = at least one active PracticeManager has a PIN set.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StaffSetupStatusDto {
    pub complete: bool,
}
