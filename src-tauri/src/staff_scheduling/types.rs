use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ProviderAvailabilityResult {
    pub available: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ProviderScheduleEntry {
    pub provider_id: String,
    pub provider_name: String,
    pub start_time: String,
    pub end_time: String,
}
