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
}

/// Returns whether the Staff Management setup step is complete.
/// Complete = at least one active PracticeManager has a PIN set.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StaffSetupStatusDto {
    pub complete: bool,
}
