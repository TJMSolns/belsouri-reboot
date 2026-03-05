use serde::{Deserialize, Serialize};

// ── Event type constants ──────────────────────────────────────────────────────

pub const PRACTICE_DETAILS_UPDATED: &str = "PracticeDetailsUpdated";

pub const OFFICE_CREATED: &str = "OfficeCreated";
pub const OFFICE_RENAMED: &str = "OfficeRenamed";
pub const OFFICE_CHAIR_COUNT_UPDATED: &str = "OfficeChairCountUpdated";
pub const OFFICE_HOURS_SET: &str = "OfficeHoursSet";
pub const OFFICE_DAY_CLOSED: &str = "OfficeDayClosed";
pub const OFFICE_ADDRESS_SET: &str = "OfficeAddressSet";
pub const OFFICE_ARCHIVED: &str = "OfficeArchived";

pub const PROVIDER_REGISTERED: &str = "ProviderRegistered";
pub const PROVIDER_RENAMED: &str = "ProviderRenamed";
pub const PROVIDER_TYPE_CHANGED: &str = "ProviderTypeChanged";
pub const PROVIDER_ASSIGNED_TO_OFFICE: &str = "ProviderAssignedToOffice";
pub const PROVIDER_REMOVED_FROM_OFFICE: &str = "ProviderRemovedFromOffice";
pub const PROVIDER_AVAILABILITY_SET: &str = "ProviderAvailabilitySet";
pub const PROVIDER_AVAILABILITY_CLEARED: &str = "ProviderAvailabilityCleared";
pub const PROVIDER_EXCEPTION_SET: &str = "ProviderExceptionSet";
pub const PROVIDER_EXCEPTION_REMOVED: &str = "ProviderExceptionRemoved";
pub const PROVIDER_ARCHIVED: &str = "ProviderArchived";
pub const PROVIDER_UNARCHIVED: &str = "ProviderUnarchived";

pub const PROCEDURE_TYPE_DEFINED: &str = "ProcedureTypeDefined";
pub const PROCEDURE_TYPE_UPDATED: &str = "ProcedureTypeUpdated";
pub const PROCEDURE_TYPE_DEACTIVATED: &str = "ProcedureTypeDeactivated";
pub const PROCEDURE_TYPE_REACTIVATED: &str = "ProcedureTypeReactivated";
pub const PROCEDURE_TYPE_CAPABILITY_SET: &str = "ProcedureTypeCapabilitySet";

// ── Practice payloads ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PracticeDetailsUpdatedPayload {
    pub name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub address_line_1: Option<String>,
    pub address_line_2: Option<String>,
    pub city_town: Option<String>,
    pub subdivision: Option<String>,
    pub country: Option<String>,
}

// ── Office payloads ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfficeCreatedPayload {
    pub id: String,
    pub name: String,
    pub chair_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfficeRenamedPayload {
    pub id: String,
    pub new_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfficeChairCountUpdatedPayload {
    pub id: String,
    pub new_chair_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfficeHoursSetPayload {
    pub id: String,
    pub day_of_week: String,
    pub open_time: String,
    pub close_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfficeDayClosedPayload {
    pub id: String,
    pub day_of_week: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfficeAddressSetPayload {
    pub id: String,
    pub address_line_1: Option<String>,
    pub address_line_2: Option<String>,
    pub city_town: Option<String>,
    pub subdivision: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfficeArchivedPayload {
    pub id: String,
}

// ── Provider payloads ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderRegisteredPayload {
    pub id: String,
    pub name: String,
    pub provider_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderRenamedPayload {
    pub id: String,
    pub new_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderTypeChangedPayload {
    pub id: String,
    pub new_provider_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderAssignedToOfficePayload {
    pub id: String,
    pub office_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderRemovedFromOfficePayload {
    pub id: String,
    pub office_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderAvailabilitySetPayload {
    pub id: String,
    pub office_id: String,
    pub day_of_week: String,
    pub start_time: String,
    pub end_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderAvailabilityClearedPayload {
    pub id: String,
    pub office_id: String,
    pub day_of_week: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderExceptionSetPayload {
    pub id: String,
    pub start_date: String,
    pub end_date: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderExceptionRemovedPayload {
    pub id: String,
    pub start_date: String,
    pub end_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderArchivedPayload {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderUnarchivedPayload {
    pub id: String,
}

// ── ProcedureType payloads ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureTypeDefinedPayload {
    pub id: String,
    pub name: String,
    pub category: String,
    pub default_duration_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureTypeUpdatedPayload {
    pub id: String,
    pub name: Option<String>,
    pub category: Option<String>,
    pub default_duration_minutes: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureTypeDeactivatedPayload {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureTypeReactivatedPayload {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureTypeCapabilitySetPayload {
    pub id: String,
    /// None | "Hygienist" | "Dentist" | "Specialist"
    pub required_provider_type: Option<String>,
}

// ── All event types slice (used by projection rebuild) ────────────────────────

pub const ALL_EVENT_TYPES: &[&str] = &[
    PRACTICE_DETAILS_UPDATED,
    OFFICE_CREATED,
    OFFICE_RENAMED,
    OFFICE_CHAIR_COUNT_UPDATED,
    OFFICE_HOURS_SET,
    OFFICE_DAY_CLOSED,
    OFFICE_ADDRESS_SET,
    OFFICE_ARCHIVED,
    PROVIDER_REGISTERED,
    PROVIDER_RENAMED,
    PROVIDER_TYPE_CHANGED,
    PROVIDER_ASSIGNED_TO_OFFICE,
    PROVIDER_REMOVED_FROM_OFFICE,
    PROVIDER_AVAILABILITY_SET,
    PROVIDER_AVAILABILITY_CLEARED,
    PROVIDER_EXCEPTION_SET,
    PROVIDER_EXCEPTION_REMOVED,
    PROVIDER_ARCHIVED,
    PROVIDER_UNARCHIVED,
    PROCEDURE_TYPE_DEFINED,
    PROCEDURE_TYPE_UPDATED,
    PROCEDURE_TYPE_DEACTIVATED,
    PROCEDURE_TYPE_REACTIVATED,
    PROCEDURE_TYPE_CAPABILITY_SET,
];
