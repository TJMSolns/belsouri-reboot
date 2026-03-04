use chrono::{Duration, Local, NaiveDate};
use tauri::State;
use crate::app_state::AppState;
use super::service::{self, OfficeProviderData};
use super::types::{ProviderAvailabilityResult, ProviderScheduleEntry};

/// Query whether a provider is available at an office on a given date and time.
///
/// date: "YYYY-MM-DD", time: "HH:MM"
/// Returns {available, reason} where reason is null when available is true.
#[specta::specta]
#[tauri::command(rename_all = "snake_case")]
pub fn query_provider_availability(
    state: State<'_, AppState>,
    provider_id: String,
    office_id: String,
    date: String,
    time: String,
) -> Result<ProviderAvailabilityResult, String> {
    let proj = state.projections.lock().unwrap();

    let provider = proj.get_provider(&provider_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Provider not found: {provider_id}"))?;

    let office = proj.get_office(&office_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Office not found: {office_id}"))?;

    let assignments = proj.list_provider_offices(&provider_id)
        .map_err(|e| e.to_string())?;
    let assigned_to_office = assignments.contains(&office_id);

    let day_of_week = service::weekday_of(&date)
        .ok_or_else(|| format!("Invalid date: {date}"))?;

    let office_hours = proj.list_office_hours(&office_id)
        .map_err(|e| e.to_string())?;
    let office_has_hours_for_day = office_hours.iter().any(|h| h.day_of_week == day_of_week);

    let exceptions = proj.list_provider_exceptions(&provider_id)
        .map_err(|e| e.to_string())?;
    let exception_pairs: Vec<(String, String)> = exceptions.iter()
        .map(|e| (e.start_date.clone(), e.end_date.clone()))
        .collect();

    let availability = proj.list_provider_availability(&provider_id)
        .map_err(|e| e.to_string())?;
    let avail_for_day: Option<(String, String)> = availability.iter()
        .find(|a| a.office_id == office_id && a.day_of_week == day_of_week)
        .map(|a| (a.start_time.clone(), a.end_time.clone()));
    let avail_ref = avail_for_day.as_ref().map(|(s, e)| (s.as_str(), e.as_str()));

    Ok(service::compute_provider_availability(
        provider.archived,
        assigned_to_office,
        office.archived,
        office_has_hours_for_day,
        &exception_pairs,
        avail_ref,
        &date,
        &time,
    ))
}

/// Get all providers working at an office on a given date.
///
/// date: "YYYY-MM-DD". Returns empty list if date is more than 90 days from today.
/// Each entry includes provider_id, provider_name, start_time, end_time (availability window for the day).
#[specta::specta]
#[tauri::command(rename_all = "snake_case")]
pub fn get_office_provider_schedule(
    state: State<'_, AppState>,
    office_id: String,
    date: String,
) -> Result<Vec<ProviderScheduleEntry>, String> {
    // 90-day pre-materialisation window guard (SS6g)
    let today = Local::now().date_naive();
    let requested = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .map_err(|_| format!("Invalid date: {date}"))?;
    if requested > today + Duration::days(90) {
        return Ok(vec![]);
    }

    let proj = state.projections.lock().unwrap();

    let office = proj.get_office(&office_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Office not found: {office_id}"))?;

    if office.archived {
        return Ok(vec![]);
    }

    let day_of_week = format!("{}", requested.format("%A"));

    let office_hours = proj.list_office_hours(&office_id)
        .map_err(|e| e.to_string())?;
    if !office_hours.iter().any(|h| h.day_of_week == day_of_week) {
        return Ok(vec![]);
    }

    let provider_ids = proj.list_providers_for_office(&office_id)
        .map_err(|e| e.to_string())?;

    let mut providers_data: Vec<OfficeProviderData> = Vec::new();
    for provider_id in &provider_ids {
        let provider = match proj.get_provider(provider_id).map_err(|e| e.to_string())? {
            Some(p) => p,
            None => continue,
        };
        let exceptions = proj.list_provider_exceptions(provider_id)
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(|e| (e.start_date, e.end_date))
            .collect();
        let availability = proj.list_provider_availability(provider_id)
            .map_err(|e| e.to_string())?;
        let avail_for_day = availability.into_iter()
            .find(|a| a.office_id == office_id && a.day_of_week == day_of_week)
            .map(|a| (a.start_time, a.end_time));
        providers_data.push(OfficeProviderData {
            provider_id: provider.id,
            provider_name: provider.name,
            provider_archived: provider.archived,
            exceptions,
            availability_for_day: avail_for_day,
        });
    }

    Ok(service::compute_office_schedule(&providers_data, &date))
}
