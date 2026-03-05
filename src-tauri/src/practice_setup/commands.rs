use tauri::State;
use uuid::Uuid;
use crate::app_state::AppState;
use crate::db::{OfficeRow, OfficeHoursRow, ProviderRow, ProcedureTypeRow};
use crate::events::practice_setup::*;
use crate::practice_setup::service::{self, AvailabilityWindow};
use crate::practice_setup::types::*;
use crate::projections::practice_setup::rebuild;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn append_event(
    state: &State<'_, AppState>,
    stream_id: &str,
    event_type: &str,
    payload: &impl serde::Serialize,
) -> Result<(), String> {
    let events = state.events.lock().map_err(|e| e.to_string())?;
    let ver = events.current_version(stream_id).map_err(|e| e.to_string())?;
    let json = serde_json::to_string(payload).map_err(|e| e.to_string())?;
    events.append(stream_id, ver, event_type, &json).map_err(|e| e.to_string())?;
    Ok(())
}

fn do_rebuild(state: &State<'_, AppState>) -> Result<(), String> {
    let events = state.events.lock().map_err(|e| e.to_string())?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    rebuild(&events, &proj)
}

fn build_office_dto(row: OfficeRow, hours: Vec<OfficeHoursRow>) -> OfficeDto {
    OfficeDto {
        id: row.id,
        name: row.name,
        chair_count: row.chair_count,
        hours: hours.into_iter().map(|h| OfficeHoursDto {
            day_of_week: h.day_of_week,
            open_time: h.open_time,
            close_time: h.close_time,
        }).collect(),
        archived: row.archived,
        address_line_1: row.address_line_1,
        address_line_2: row.address_line_2,
        city_town: row.city_town,
        subdivision: row.subdivision,
        country: row.country,
    }
}

fn build_office_dto_from_proj(
    proj: &std::sync::MutexGuard<'_, crate::db::ProjectionStore>,
    office_id: &str,
) -> Result<OfficeDto, String> {
    let row = proj.get_office(office_id).map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Office '{}' not found", office_id))?;
    let hours = proj.list_office_hours(office_id).map_err(|e| e.to_string())?;
    Ok(build_office_dto(row, hours))
}

fn procedure_type_to_dto(row: &ProcedureTypeRow) -> ProcedureTypeDto {
    ProcedureTypeDto {
        id: row.id.clone(),
        name: row.name.clone(),
        category: row.category.clone(),
        default_duration_minutes: row.default_duration_minutes,
        is_active: row.is_active,
    }
}

// ── Practice commands ─────────────────────────────────────────────────────────

#[specta::specta]
#[tauri::command]
pub async fn update_practice_details(
    state: State<'_, AppState>,
    name: String,
    phone: Option<String>,
    email: Option<String>,
    website: Option<String>,
    address_line_1: Option<String>,
    address_line_2: Option<String>,
    city_town: Option<String>,
    subdivision: Option<String>,
    country: Option<String>,
) -> Result<PracticeDto, String> {
    service::validate_name(&name)?;
    append_event(
        &state,
        "practice",
        PRACTICE_DETAILS_UPDATED,
        &PracticeDetailsUpdatedPayload {
            name: name.clone(),
            phone: phone.clone(),
            email: email.clone(),
            website: website.clone(),
            address_line_1: address_line_1.clone(),
            address_line_2: address_line_2.clone(),
            city_town: city_town.clone(),
            subdivision: subdivision.clone(),
            country: country.clone(),
        },
    )?;
    do_rebuild(&state)?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    let row = proj.get_practice_settings().map_err(|e| e.to_string())?
        .ok_or_else(|| "Practice settings not found after update".to_string())?;
    Ok(PracticeDto {
        name: row.name,
        phone: row.phone,
        email: row.email,
        website: row.website,
        address_line_1: row.address_line_1,
        address_line_2: row.address_line_2,
        city_town: row.city_town,
        subdivision: row.subdivision,
        country: row.country,
    })
}

#[specta::specta]
#[tauri::command]
pub async fn get_practice(state: State<'_, AppState>) -> Result<Option<PracticeDto>, String> {
    do_rebuild(&state)?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    let row = proj.get_practice_settings().map_err(|e| e.to_string())?;
    Ok(row.map(|r| PracticeDto {
        name: r.name,
        phone: r.phone,
        email: r.email,
        website: r.website,
        address_line_1: r.address_line_1,
        address_line_2: r.address_line_2,
        city_town: r.city_town,
        subdivision: r.subdivision,
        country: r.country,
    }))
}

// ── Office commands ───────────────────────────────────────────────────────────

#[specta::specta]
#[tauri::command]
pub async fn create_office(
    state: State<'_, AppState>,
    name: String,
    chair_count: u32,
) -> Result<OfficeDto, String> {
    service::validate_name(&name)?;
    service::validate_chair_count(chair_count)?;
    let id = Uuid::new_v4().to_string();
    let stream_id = format!("office:{id}");
    append_event(
        &state,
        &stream_id,
        OFFICE_CREATED,
        &OfficeCreatedPayload { id: id.clone(), name, chair_count },
    )?;
    do_rebuild(&state)?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    build_office_dto_from_proj(&proj, &id)
}

#[specta::specta]
#[tauri::command]
pub async fn rename_office(
    state: State<'_, AppState>,
    office_id: String,
    new_name: String,
) -> Result<OfficeDto, String> {
    service::validate_name(&new_name)?;
    do_rebuild(&state)?;
    {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        let row = proj.get_office(&office_id).map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Office '{}' not found", office_id))?;
        if row.archived {
            return Err(format!("Office '{}' is archived", office_id));
        }
    }
    let stream_id = format!("office:{office_id}");
    append_event(
        &state,
        &stream_id,
        OFFICE_RENAMED,
        &OfficeRenamedPayload { id: office_id.clone(), new_name },
    )?;
    do_rebuild(&state)?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    build_office_dto_from_proj(&proj, &office_id)
}

#[specta::specta]
#[tauri::command]
pub async fn update_office_chair_count(
    state: State<'_, AppState>,
    office_id: String,
    new_chair_count: u32,
) -> Result<OfficeDto, String> {
    service::validate_chair_count(new_chair_count)?;
    do_rebuild(&state)?;
    {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        let row = proj.get_office(&office_id).map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Office '{}' not found", office_id))?;
        if row.archived {
            return Err(format!("Office '{}' is archived", office_id));
        }
    }
    let stream_id = format!("office:{office_id}");
    append_event(
        &state,
        &stream_id,
        OFFICE_CHAIR_COUNT_UPDATED,
        &OfficeChairCountUpdatedPayload { id: office_id.clone(), new_chair_count },
    )?;
    do_rebuild(&state)?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    build_office_dto_from_proj(&proj, &office_id)
}

#[specta::specta]
#[tauri::command]
pub async fn set_office_hours(
    state: State<'_, AppState>,
    office_id: String,
    day_of_week: String,
    open_time: String,
    close_time: String,
) -> Result<OfficeDto, String> {
    service::validate_day_of_week(&day_of_week)?;
    service::validate_hhmm(&open_time)?;
    service::validate_hhmm(&close_time)?;
    service::validate_time_range(&open_time, &close_time)?;
    do_rebuild(&state)?;
    {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        let row = proj.get_office(&office_id).map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Office '{}' not found", office_id))?;
        if row.archived {
            return Err(format!("Office '{}' is archived", office_id));
        }
    }
    let stream_id = format!("office:{office_id}");
    append_event(
        &state,
        &stream_id,
        OFFICE_HOURS_SET,
        &OfficeHoursSetPayload {
            id: office_id.clone(),
            day_of_week,
            open_time,
            close_time,
        },
    )?;
    do_rebuild(&state)?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    build_office_dto_from_proj(&proj, &office_id)
}

#[specta::specta]
#[tauri::command]
pub async fn close_office_day(
    state: State<'_, AppState>,
    office_id: String,
    day_of_week: String,
) -> Result<OfficeDto, String> {
    service::validate_day_of_week(&day_of_week)?;
    do_rebuild(&state)?;
    {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        proj.get_office(&office_id).map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Office '{}' not found", office_id))?;
    }
    let stream_id = format!("office:{office_id}");
    append_event(
        &state,
        &stream_id,
        OFFICE_DAY_CLOSED,
        &OfficeDayClosedPayload { id: office_id.clone(), day_of_week },
    )?;
    do_rebuild(&state)?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    build_office_dto_from_proj(&proj, &office_id)
}

#[specta::specta]
#[tauri::command]
pub async fn archive_office(
    state: State<'_, AppState>,
    office_id: String,
) -> Result<OfficeDto, String> {
    do_rebuild(&state)?;
    {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        let row = proj.get_office(&office_id).map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Office '{}' not found", office_id))?;
        if row.archived {
            return Err(format!("Office '{}' is already archived", office_id));
        }
    }
    let stream_id = format!("office:{office_id}");
    append_event(&state, &stream_id, OFFICE_ARCHIVED, &OfficeArchivedPayload { id: office_id.clone() })?;
    do_rebuild(&state)?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    build_office_dto_from_proj(&proj, &office_id)
}

#[specta::specta]
#[tauri::command]
pub async fn list_offices(state: State<'_, AppState>) -> Result<Vec<OfficeDto>, String> {
    do_rebuild(&state)?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    let office_rows = proj.list_offices().map_err(|e| e.to_string())?;
    let mut result = Vec::with_capacity(office_rows.len());
    for row in office_rows {
        let hours = proj.list_office_hours(&row.id).map_err(|e| e.to_string())?;
        result.push(build_office_dto(row, hours));
    }
    Ok(result)
}

#[specta::specta]
#[tauri::command]
pub async fn get_office(
    state: State<'_, AppState>,
    office_id: String,
) -> Result<OfficeDto, String> {
    do_rebuild(&state)?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    build_office_dto_from_proj(&proj, &office_id)
}

#[specta::specta]
#[tauri::command]
pub async fn set_office_address(
    state: State<'_, AppState>,
    office_id: String,
    address_line_1: Option<String>,
    address_line_2: Option<String>,
    city_town: Option<String>,
    subdivision: Option<String>,
    country: Option<String>,
) -> Result<OfficeDto, String> {
    do_rebuild(&state)?;
    {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        let row = proj.get_office(&office_id).map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Office '{}' not found", office_id))?;
        if row.archived {
            return Err(format!("Office '{}' is archived", office_id));
        }
    }
    let stream_id = format!("office:{office_id}");
    append_event(
        &state,
        &stream_id,
        OFFICE_ADDRESS_SET,
        &OfficeAddressSetPayload {
            id: office_id.clone(),
            address_line_1,
            address_line_2,
            city_town,
            subdivision,
            country,
        },
    )?;
    do_rebuild(&state)?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    build_office_dto_from_proj(&proj, &office_id)
}

// ── Provider commands ─────────────────────────────────────────────────────────

#[specta::specta]
#[tauri::command]
pub async fn register_provider(
    state: State<'_, AppState>,
    name: String,
    provider_type: String,
) -> Result<ProviderDto, String> {
    service::validate_name(&name)?;
    service::validate_provider_type(&provider_type)?;
    let id = Uuid::new_v4().to_string();
    let stream_id = format!("provider:{id}");
    append_event(
        &state,
        &stream_id,
        PROVIDER_REGISTERED,
        &ProviderRegisteredPayload { id: id.clone(), name, provider_type },
    )?;
    do_rebuild(&state)?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    let row = proj.get_provider(&id).map_err(|e| e.to_string())?.unwrap();
    let office_ids = proj.list_provider_offices(&id).map_err(|e| e.to_string())?;
    Ok(ProviderDto {
        id: row.id,
        name: row.name,
        provider_type: row.provider_type,
        office_ids,
        availability: vec![],
        exceptions: vec![],
        archived: row.archived,
    })
}

#[specta::specta]
#[tauri::command]
pub async fn rename_provider(
    state: State<'_, AppState>,
    provider_id: String,
    new_name: String,
) -> Result<ProviderDto, String> {
    service::validate_name(&new_name)?;
    do_rebuild(&state)?;
    {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        let row = proj.get_provider(&provider_id).map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Provider '{}' not found", provider_id))?;
        if row.archived {
            return Err(format!("Provider '{}' is archived", provider_id));
        }
    }
    let stream_id = format!("provider:{provider_id}");
    append_event(
        &state,
        &stream_id,
        PROVIDER_RENAMED,
        &ProviderRenamedPayload { id: provider_id.clone(), new_name },
    )?;
    do_rebuild(&state)?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    let row = proj.get_provider(&provider_id).map_err(|e| e.to_string())?.unwrap();
    build_provider_dto_from_proj(&proj, &row)
}

#[specta::specta]
#[tauri::command]
pub async fn change_provider_type(
    state: State<'_, AppState>,
    provider_id: String,
    new_provider_type: String,
) -> Result<ProviderDto, String> {
    service::validate_provider_type(&new_provider_type)?;
    do_rebuild(&state)?;
    {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        let row = proj.get_provider(&provider_id).map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Provider '{}' not found", provider_id))?;
        if row.archived {
            return Err(format!("Provider '{}' is archived", provider_id));
        }
    }
    let stream_id = format!("provider:{provider_id}");
    append_event(
        &state,
        &stream_id,
        PROVIDER_TYPE_CHANGED,
        &ProviderTypeChangedPayload { id: provider_id.clone(), new_provider_type },
    )?;
    do_rebuild(&state)?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    let row = proj.get_provider(&provider_id).map_err(|e| e.to_string())?.unwrap();
    build_provider_dto_from_proj(&proj, &row)
}

#[specta::specta]
#[tauri::command]
pub async fn assign_provider_to_office(
    state: State<'_, AppState>,
    provider_id: String,
    office_id: String,
) -> Result<ProviderDto, String> {
    do_rebuild(&state)?;
    {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        let provider = proj.get_provider(&provider_id).map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Provider '{}' not found", provider_id))?;
        if provider.archived {
            return Err(format!("Provider '{}' is archived", provider_id));
        }
        let office = proj.get_office(&office_id).map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Office '{}' not found", office_id))?;
        if office.archived {
            return Err(format!("Office '{}' is archived", office_id));
        }
        let current_offices = proj.list_provider_offices(&provider_id).map_err(|e| e.to_string())?;
        if current_offices.contains(&office_id) {
            return Err(format!("Provider '{}' is already assigned to office '{}'", provider_id, office_id));
        }
    }
    let stream_id = format!("provider:{provider_id}");
    append_event(
        &state,
        &stream_id,
        PROVIDER_ASSIGNED_TO_OFFICE,
        &ProviderAssignedToOfficePayload { id: provider_id.clone(), office_id },
    )?;
    do_rebuild(&state)?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    let row = proj.get_provider(&provider_id).map_err(|e| e.to_string())?.unwrap();
    build_provider_dto_from_proj(&proj, &row)
}

#[specta::specta]
#[tauri::command]
pub async fn remove_provider_from_office(
    state: State<'_, AppState>,
    provider_id: String,
    office_id: String,
) -> Result<ProviderDto, String> {
    do_rebuild(&state)?;
    // Gather availability days to clear BEFORE appending any events
    let days_to_clear: Vec<String> = {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        let current_offices = proj.list_provider_offices(&provider_id).map_err(|e| e.to_string())?;
        if !current_offices.contains(&office_id) {
            return Err(format!(
                "Provider '{}' is not assigned to office '{}'",
                provider_id, office_id
            ));
        }
        proj.delete_provider_availability_for_office(&provider_id, &office_id)
            .map_err(|e| e.to_string())?
    };

    // Append ProviderRemovedFromOffice + N x ProviderAvailabilityCleared
    let stream_id = format!("provider:{provider_id}");
    {
        let events = state.events.lock().map_err(|e| e.to_string())?;
        let mut ver = events.current_version(&stream_id).map_err(|e| e.to_string())?;
        let json = serde_json::to_string(&ProviderRemovedFromOfficePayload {
            id: provider_id.clone(),
            office_id: office_id.clone(),
        }).map_err(|e| e.to_string())?;
        events.append(&stream_id, ver, PROVIDER_REMOVED_FROM_OFFICE, &json)
            .map_err(|e| e.to_string())?;
        ver += 1;
        for day in &days_to_clear {
            let json = serde_json::to_string(&ProviderAvailabilityClearedPayload {
                id: provider_id.clone(),
                office_id: office_id.clone(),
                day_of_week: day.clone(),
            }).map_err(|e| e.to_string())?;
            events.append(&stream_id, ver, PROVIDER_AVAILABILITY_CLEARED, &json)
                .map_err(|e| e.to_string())?;
            ver += 1;
        }
    }
    do_rebuild(&state)?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    let row = proj.get_provider(&provider_id).map_err(|e| e.to_string())?.unwrap();
    build_provider_dto_from_proj(&proj, &row)
}

#[specta::specta]
#[tauri::command]
pub async fn set_provider_availability(
    state: State<'_, AppState>,
    provider_id: String,
    office_id: String,
    day_of_week: String,
    start_time: String,
    end_time: String,
) -> Result<ProviderDto, String> {
    service::validate_day_of_week(&day_of_week)?;
    service::validate_hhmm(&start_time)?;
    service::validate_hhmm(&end_time)?;
    service::validate_time_range(&start_time, &end_time)?;
    do_rebuild(&state)?;
    {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        let provider = proj.get_provider(&provider_id).map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Provider '{}' not found", provider_id))?;
        if provider.archived {
            return Err(format!("Provider '{}' is archived", provider_id));
        }
        let current_offices = proj.list_provider_offices(&provider_id).map_err(|e| e.to_string())?;
        if !current_offices.contains(&office_id) {
            return Err(format!(
                "Provider '{}' is not assigned to office '{}'",
                provider_id, office_id
            ));
        }
        // Cross-office overlap check
        let avail = proj.list_provider_availability(&provider_id).map_err(|e| e.to_string())?;
        let existing: Vec<AvailabilityWindow> = avail.into_iter().map(|a| AvailabilityWindow {
            office_id: a.office_id,
            day_of_week: a.day_of_week,
            start_time: a.start_time,
            end_time: a.end_time,
        }).collect();
        let proposed = AvailabilityWindow {
            office_id: office_id.clone(),
            day_of_week: day_of_week.clone(),
            start_time: start_time.clone(),
            end_time: end_time.clone(),
        };
        service::check_no_cross_office_overlap(&existing, &proposed)?;
    }
    let stream_id = format!("provider:{provider_id}");
    append_event(
        &state,
        &stream_id,
        PROVIDER_AVAILABILITY_SET,
        &ProviderAvailabilitySetPayload {
            id: provider_id.clone(),
            office_id,
            day_of_week,
            start_time,
            end_time,
        },
    )?;
    do_rebuild(&state)?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    let row = proj.get_provider(&provider_id).map_err(|e| e.to_string())?.unwrap();
    build_provider_dto_from_proj(&proj, &row)
}

#[specta::specta]
#[tauri::command]
pub async fn clear_provider_availability(
    state: State<'_, AppState>,
    provider_id: String,
    office_id: String,
    day_of_week: String,
) -> Result<ProviderDto, String> {
    service::validate_day_of_week(&day_of_week)?;
    do_rebuild(&state)?;
    {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        let avail = proj.list_provider_availability(&provider_id).map_err(|e| e.to_string())?;
        let has = avail.iter().any(|a| a.office_id == office_id && a.day_of_week == day_of_week);
        if !has {
            return Err(format!(
                "Provider '{}' has no availability for office '{}' on {}",
                provider_id, office_id, day_of_week
            ));
        }
    }
    let stream_id = format!("provider:{provider_id}");
    append_event(
        &state,
        &stream_id,
        PROVIDER_AVAILABILITY_CLEARED,
        &ProviderAvailabilityClearedPayload {
            id: provider_id.clone(),
            office_id,
            day_of_week,
        },
    )?;
    do_rebuild(&state)?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    let row = proj.get_provider(&provider_id).map_err(|e| e.to_string())?.unwrap();
    build_provider_dto_from_proj(&proj, &row)
}

#[specta::specta]
#[tauri::command]
pub async fn set_provider_exception(
    state: State<'_, AppState>,
    provider_id: String,
    start_date: String,
    end_date: String,
    reason: Option<String>,
) -> Result<ProviderDto, String> {
    service::validate_date_ymd(&start_date)?;
    service::validate_date_ymd(&end_date)?;
    service::validate_date_range(&start_date, &end_date)?;
    do_rebuild(&state)?;
    {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        let row = proj.get_provider(&provider_id).map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Provider '{}' not found", provider_id))?;
        if row.archived {
            return Err(format!("Provider '{}' is archived", provider_id));
        }
    }
    let stream_id = format!("provider:{provider_id}");
    append_event(
        &state,
        &stream_id,
        PROVIDER_EXCEPTION_SET,
        &ProviderExceptionSetPayload {
            id: provider_id.clone(),
            start_date,
            end_date,
            reason,
        },
    )?;
    do_rebuild(&state)?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    let row = proj.get_provider(&provider_id).map_err(|e| e.to_string())?.unwrap();
    build_provider_dto_from_proj(&proj, &row)
}

#[specta::specta]
#[tauri::command]
pub async fn remove_provider_exception(
    state: State<'_, AppState>,
    provider_id: String,
    start_date: String,
    end_date: String,
) -> Result<ProviderDto, String> {
    do_rebuild(&state)?;
    {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        let exceptions = proj.list_provider_exceptions(&provider_id).map_err(|e| e.to_string())?;
        let found = exceptions.iter().any(|e| e.start_date == start_date && e.end_date == end_date);
        if !found {
            return Err(format!(
                "No exception found for provider '{}' from {} to {}",
                provider_id, start_date, end_date
            ));
        }
    }
    let stream_id = format!("provider:{provider_id}");
    append_event(
        &state,
        &stream_id,
        PROVIDER_EXCEPTION_REMOVED,
        &ProviderExceptionRemovedPayload {
            id: provider_id.clone(),
            start_date,
            end_date,
        },
    )?;
    do_rebuild(&state)?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    let row = proj.get_provider(&provider_id).map_err(|e| e.to_string())?.unwrap();
    build_provider_dto_from_proj(&proj, &row)
}

#[specta::specta]
#[tauri::command]
pub async fn archive_provider(
    state: State<'_, AppState>,
    provider_id: String,
) -> Result<ProviderDto, String> {
    do_rebuild(&state)?;
    {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        let row = proj.get_provider(&provider_id).map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Provider '{}' not found", provider_id))?;
        if row.archived {
            return Err(format!("Provider '{}' is already archived", provider_id));
        }
    }
    let stream_id = format!("provider:{provider_id}");
    append_event(
        &state,
        &stream_id,
        PROVIDER_ARCHIVED,
        &ProviderArchivedPayload { id: provider_id.clone() },
    )?;
    do_rebuild(&state)?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    let row = proj.get_provider(&provider_id).map_err(|e| e.to_string())?.unwrap();
    build_provider_dto_from_proj(&proj, &row)
}

#[specta::specta]
#[tauri::command]
pub async fn unarchive_provider(
    state: State<'_, AppState>,
    provider_id: String,
) -> Result<ProviderDto, String> {
    do_rebuild(&state)?;
    {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        let row = proj.get_provider(&provider_id).map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Provider '{}' not found", provider_id))?;
        if !row.archived {
            return Err(format!("Provider '{}' is not archived", provider_id));
        }
    }
    let stream_id = format!("provider:{provider_id}");
    append_event(
        &state,
        &stream_id,
        PROVIDER_UNARCHIVED,
        &ProviderUnarchivedPayload { id: provider_id.clone() },
    )?;
    do_rebuild(&state)?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    let row = proj.get_provider(&provider_id).map_err(|e| e.to_string())?.unwrap();
    build_provider_dto_from_proj(&proj, &row)
}

#[specta::specta]
#[tauri::command]
pub async fn list_providers(state: State<'_, AppState>) -> Result<Vec<ProviderDto>, String> {
    do_rebuild(&state)?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    let rows = proj.list_providers().map_err(|e| e.to_string())?;
    let mut result = Vec::with_capacity(rows.len());
    for row in &rows {
        result.push(build_provider_dto_from_proj(&proj, row)?);
    }
    Ok(result)
}

#[specta::specta]
#[tauri::command]
pub async fn get_provider(
    state: State<'_, AppState>,
    provider_id: String,
) -> Result<ProviderDto, String> {
    do_rebuild(&state)?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    let row = proj.get_provider(&provider_id).map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Provider '{}' not found", provider_id))?;
    build_provider_dto_from_proj(&proj, &row)
}

fn build_provider_dto_from_proj(
    proj: &std::sync::MutexGuard<'_, crate::db::ProjectionStore>,
    row: &ProviderRow,
) -> Result<ProviderDto, String> {
    let office_ids = proj.list_provider_offices(&row.id).map_err(|e| e.to_string())?;
    let avail = proj.list_provider_availability(&row.id).map_err(|e| e.to_string())?;
    let exc = proj.list_provider_exceptions(&row.id).map_err(|e| e.to_string())?;
    Ok(ProviderDto {
        id: row.id.clone(),
        name: row.name.clone(),
        provider_type: row.provider_type.clone(),
        office_ids,
        availability: avail.into_iter().map(|a| AvailabilityWindowDto {
            office_id: a.office_id,
            day_of_week: a.day_of_week,
            start_time: a.start_time,
            end_time: a.end_time,
        }).collect(),
        exceptions: exc.into_iter().map(|e| AvailabilityExceptionDto {
            start_date: e.start_date,
            end_date: e.end_date,
            reason: e.reason,
        }).collect(),
        archived: row.archived,
    })
}

// ── ProcedureType commands ────────────────────────────────────────────────────

#[specta::specta]
#[tauri::command]
pub async fn define_procedure_type(
    state: State<'_, AppState>,
    name: String,
    category: String,
    default_duration_minutes: u32,
) -> Result<ProcedureTypeDto, String> {
    service::validate_name(&name)?;
    service::validate_category(&category)?;
    service::validate_duration(default_duration_minutes)?;
    let id = Uuid::new_v4().to_string();
    let stream_id = format!("procedure_type:{id}");
    append_event(
        &state,
        &stream_id,
        PROCEDURE_TYPE_DEFINED,
        &ProcedureTypeDefinedPayload {
            id: id.clone(),
            name,
            category,
            default_duration_minutes,
        },
    )?;
    do_rebuild(&state)?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    let row = proj.get_procedure_type(&id).map_err(|e| e.to_string())?.unwrap();
    Ok(procedure_type_to_dto(&row))
}

#[specta::specta]
#[tauri::command]
pub async fn update_procedure_type(
    state: State<'_, AppState>,
    id: String,
    name: Option<String>,
    category: Option<String>,
    default_duration_minutes: Option<u32>,
) -> Result<ProcedureTypeDto, String> {
    if name.is_none() && category.is_none() && default_duration_minutes.is_none() {
        return Err("At least one field must be provided to update".to_string());
    }
    if let Some(n) = &name {
        service::validate_name(n)?;
    }
    if let Some(c) = &category {
        service::validate_category(c)?;
    }
    if let Some(d) = default_duration_minutes {
        service::validate_duration(d)?;
    }
    do_rebuild(&state)?;
    {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        proj.get_procedure_type(&id).map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Procedure type '{}' not found", id))?;
    }
    let stream_id = format!("procedure_type:{id}");
    append_event(
        &state,
        &stream_id,
        PROCEDURE_TYPE_UPDATED,
        &ProcedureTypeUpdatedPayload { id: id.clone(), name, category, default_duration_minutes },
    )?;
    do_rebuild(&state)?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    let row = proj.get_procedure_type(&id).map_err(|e| e.to_string())?.unwrap();
    Ok(procedure_type_to_dto(&row))
}

#[specta::specta]
#[tauri::command]
pub async fn deactivate_procedure_type(
    state: State<'_, AppState>,
    id: String,
) -> Result<ProcedureTypeDto, String> {
    do_rebuild(&state)?;
    {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        let row = proj.get_procedure_type(&id).map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Procedure type '{}' not found", id))?;
        if !row.is_active {
            return Err(format!("Procedure type '{}' is already inactive", id));
        }
    }
    let stream_id = format!("procedure_type:{id}");
    append_event(&state, &stream_id, PROCEDURE_TYPE_DEACTIVATED, &ProcedureTypeDeactivatedPayload { id: id.clone() })?;
    do_rebuild(&state)?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    let row = proj.get_procedure_type(&id).map_err(|e| e.to_string())?.unwrap();
    Ok(procedure_type_to_dto(&row))
}

#[specta::specta]
#[tauri::command]
pub async fn reactivate_procedure_type(
    state: State<'_, AppState>,
    id: String,
) -> Result<ProcedureTypeDto, String> {
    do_rebuild(&state)?;
    {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        let row = proj.get_procedure_type(&id).map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Procedure type '{}' not found", id))?;
        if row.is_active {
            return Err(format!("Procedure type '{}' is already active", id));
        }
    }
    let stream_id = format!("procedure_type:{id}");
    append_event(&state, &stream_id, PROCEDURE_TYPE_REACTIVATED, &ProcedureTypeReactivatedPayload { id: id.clone() })?;
    do_rebuild(&state)?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    let row = proj.get_procedure_type(&id).map_err(|e| e.to_string())?.unwrap();
    Ok(procedure_type_to_dto(&row))
}

#[specta::specta]
#[tauri::command]
pub async fn seed_default_procedure_types(state: State<'_, AppState>) -> Result<Vec<ProcedureTypeDto>, String> {
    let defaults = [
        ("Consultation", "Consult", 30u32),
        ("Cleaning", "Preventive", 30),
        ("Fluoride Treatment", "Preventive", 15),
        ("Exam", "Diagnostic", 15),
        ("X-Ray", "Diagnostic", 15),
        ("Filling", "Restorative", 45),
        ("Crown", "Restorative", 60),
        ("Extraction", "Invasive", 30),
        ("Root Canal", "Invasive", 90),
        ("Whitening", "Cosmetic", 60),
    ];
    let events_guard = state.events.lock().map_err(|e| e.to_string())?;
    for (name, category, duration) in &defaults {
        let id = Uuid::new_v4().to_string();
        let stream_id = format!("procedure_type:{id}");
        let json = serde_json::to_string(&ProcedureTypeDefinedPayload {
            id,
            name: name.to_string(),
            category: category.to_string(),
            default_duration_minutes: *duration,
        }).map_err(|e| e.to_string())?;
        events_guard.append(&stream_id, 0, PROCEDURE_TYPE_DEFINED, &json).map_err(|e| e.to_string())?;
    }
    drop(events_guard);
    do_rebuild(&state)?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    let rows = proj.list_procedure_types().map_err(|e| e.to_string())?;
    Ok(rows.iter().map(procedure_type_to_dto).collect())
}

#[specta::specta]
#[tauri::command]
pub async fn list_procedure_types(state: State<'_, AppState>) -> Result<Vec<ProcedureTypeDto>, String> {
    do_rebuild(&state)?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    let rows = proj.list_procedure_types().map_err(|e| e.to_string())?;
    Ok(rows.iter().map(procedure_type_to_dto).collect())
}
