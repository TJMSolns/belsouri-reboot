use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct StaffShiftDto {
    pub shift_id: String,
    pub staff_member_id: String,
    pub staff_name: String,
    pub office_id: String,
    pub office_name: String,
    pub date: String,
    pub start_time: String,
    pub end_time: String,
    pub role: String,
    pub created_by: String,
    pub cancelled: bool,
    pub cancel_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct PlanShiftResult {
    pub shift_id: String,
}
