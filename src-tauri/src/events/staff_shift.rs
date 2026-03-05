use serde::{Deserialize, Serialize};

pub const STAFF_SHIFT_PLANNED: &str = "StaffShiftPlanned";
pub const STAFF_SHIFT_CANCELLED: &str = "StaffShiftCancelled";

pub const ALL_EVENT_TYPES: &[&str] = &[STAFF_SHIFT_PLANNED, STAFF_SHIFT_CANCELLED];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffShiftPlannedPayload {
    pub shift_id: String,
    pub staff_member_id: String,
    pub office_id: String,
    pub date: String,
    pub start_time: String,
    pub end_time: String,
    pub role: String,
    pub created_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffShiftCancelledPayload {
    pub shift_id: String,
    pub cancel_reason: Option<String>,
    pub cancelled_by: String,
}
