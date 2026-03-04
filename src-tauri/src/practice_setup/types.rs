use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PracticeDto {
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

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct OfficeDto {
    pub id: String,
    pub name: String,
    pub chair_count: u32,
    pub hours: Vec<OfficeHoursDto>,
    pub archived: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct OfficeHoursDto {
    pub day_of_week: String,
    pub open_time: String,
    pub close_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ProviderDto {
    pub id: String,
    pub name: String,
    pub provider_type: String,
    pub office_ids: Vec<String>,
    pub availability: Vec<AvailabilityWindowDto>,
    pub exceptions: Vec<AvailabilityExceptionDto>,
    pub archived: bool,
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

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ProcedureTypeDto {
    pub id: String,
    pub name: String,
    pub category: String,
    pub default_duration_minutes: u32,
    pub is_active: bool,
}
