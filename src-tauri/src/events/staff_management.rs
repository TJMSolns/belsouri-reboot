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

// ── Provider clinical events (staff member IS the provider) ───────────────────
pub const PROVIDER_TYPE_SET: &str = "ProviderTypeSet";
pub const PROVIDER_ASSIGNED_TO_OFFICE: &str = "ProviderAssignedToOffice";
pub const PROVIDER_REMOVED_FROM_OFFICE: &str = "ProviderRemovedFromOffice";
pub const PROVIDER_AVAILABILITY_SET: &str = "ProviderAvailabilitySet";
pub const PROVIDER_AVAILABILITY_CLEARED: &str = "ProviderAvailabilityCleared";
pub const PROVIDER_EXCEPTION_SET: &str = "ProviderExceptionSet";
pub const PROVIDER_EXCEPTION_REMOVED: &str = "ProviderExceptionRemoved";

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
    PROVIDER_TYPE_SET,
    PROVIDER_ASSIGNED_TO_OFFICE,
    PROVIDER_REMOVED_FROM_OFFICE,
    PROVIDER_AVAILABILITY_SET,
    PROVIDER_AVAILABILITY_CLEARED,
    PROVIDER_EXCEPTION_SET,
    PROVIDER_EXCEPTION_REMOVED,
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

// ── Provider clinical event payloads ──────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderTypeSetPayload {
    pub staff_member_id: String,
    pub clinical_specialization: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderAssignedToOfficePayload {
    pub staff_member_id: String,
    pub office_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderRemovedFromOfficePayload {
    pub staff_member_id: String,
    pub office_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderAvailabilitySetPayload {
    pub staff_member_id: String,
    pub office_id: String,
    pub day_of_week: String,
    pub start_time: String,
    pub end_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderAvailabilityClearedPayload {
    pub staff_member_id: String,
    pub office_id: String,
    pub day_of_week: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderExceptionSetPayload {
    pub staff_member_id: String,
    pub start_date: String,
    pub end_date: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderExceptionRemovedPayload {
    pub staff_member_id: String,
    pub start_date: String,
    pub end_date: String,
}
