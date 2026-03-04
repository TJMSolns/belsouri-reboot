use serde::{Deserialize, Serialize};

// ── Event type constants ──────────────────────────────────────────────────────

pub const STAFF_MEMBER_REGISTERED: &str = "StaffMemberRegistered";
pub const PRACTICE_MANAGER_CLAIMED: &str = "PracticeManagerClaimed";
pub const ROLE_ASSIGNED: &str = "RoleAssigned";
pub const ROLE_REMOVED: &str = "RoleRemoved";
pub const PIN_SET: &str = "PINSet";
pub const PIN_CHANGED: &str = "PINChanged";
pub const PIN_RESET: &str = "PINReset";
pub const STAFF_MEMBER_ARCHIVED: &str = "StaffMemberArchived";
pub const STAFF_MEMBER_UNARCHIVED: &str = "StaffMemberUnarchived";

pub const ALL_EVENT_TYPES: &[&str] = &[
    STAFF_MEMBER_REGISTERED,
    PRACTICE_MANAGER_CLAIMED,
    ROLE_ASSIGNED,
    ROLE_REMOVED,
    PIN_SET,
    PIN_CHANGED,
    PIN_RESET,
    STAFF_MEMBER_ARCHIVED,
    STAFF_MEMBER_UNARCHIVED,
];

// ── Payload structs ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffMemberRegisteredPayload {
    pub staff_member_id: String,
    pub name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub preferred_contact_channel: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PracticeManagerClaimedPayload {
    pub staff_member_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleAssignedPayload {
    pub staff_member_id: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleRemovedPayload {
    pub staff_member_id: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PINSetPayload {
    pub staff_member_id: String,
    pub pin_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PINChangedPayload {
    pub staff_member_id: String,
    pub pin_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PINResetPayload {
    pub staff_member_id: String,
    pub reset_by_staff_member_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffMemberArchivedPayload {
    pub staff_member_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffMemberUnarchivedPayload {
    pub staff_member_id: String,
}
