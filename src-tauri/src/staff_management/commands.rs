use tauri::State;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::events::staff_management::*;
use crate::projections::staff_management::rebuild;
use super::service::*;
use super::types::*;

fn do_rebuild(state: &AppState) -> Result<(), String> {
    let events = state.events.lock().map_err(|e| e.to_string())?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    rebuild(&events, &proj)
}

fn build_dto(state: &AppState, staff_member_id: &str) -> Result<StaffMemberDto, String> {
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    build_dto_from_proj(&proj, staff_member_id)
}

fn build_dto_from_proj(
    proj: &std::sync::MutexGuard<'_, crate::db::ProjectionStore>,
    staff_member_id: &str,
) -> Result<StaffMemberDto, String> {
    let row = proj.get_staff_member(staff_member_id).map_err(|e| e.to_string())?
        .ok_or_else(|| "Staff member not found".to_string())?;
    let roles = proj.list_staff_roles(staff_member_id).map_err(|e| e.to_string())?;
    let office_ids = proj.list_staff_offices(staff_member_id).map_err(|e| e.to_string())?;
    let avail = proj.list_staff_availability(staff_member_id).map_err(|e| e.to_string())?;
    let exceptions = proj.list_staff_exceptions(staff_member_id).map_err(|e| e.to_string())?;
    Ok(StaffMemberDto {
        staff_member_id: row.staff_member_id,
        name: row.name,
        phone: row.phone,
        email: row.email,
        preferred_contact_channel: row.preferred_contact_channel,
        has_pin: row.pin_hash.is_some(),
        roles,
        archived: row.archived,
        clinical_specialization: row.clinical_specialization,
        office_ids,
        availability: avail.into_iter().map(|a| AvailabilityWindowDto {
            office_id: a.office_id,
            day_of_week: a.day_of_week,
            start_time: a.start_time,
            end_time: a.end_time,
        }).collect(),
        exceptions: exceptions.into_iter().map(|e| AvailabilityExceptionDto {
            start_date: e.start_date,
            end_date: e.end_date,
            reason: e.reason,
        }).collect(),
    })
}

fn staff_stream(staff_member_id: &str) -> String {
    format!("staff:{}", staff_member_id)
}

// ── Commands ──────────────────────────────────────────────────────────────────

/// First-run bootstrap: first person claims the PracticeManager role.
/// Rejected if any active PracticeManager already exists.
#[tauri::command]
#[specta::specta]
pub fn claim_practice_manager_role(
    state: State<'_, AppState>,
    name: String,
) -> Result<StaffMemberDto, String> {
    validate_name(&name)?;
    do_rebuild(&state)?;

    {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        let count = proj.count_active_practice_managers().map_err(|e| e.to_string())?;
        if count > 0 {
            return Err("A Practice Manager already exists".to_string());
        }
    }

    let staff_member_id = Uuid::new_v4().to_string();
    let stream_id = staff_stream(&staff_member_id);

    let registered = StaffMemberRegisteredPayload {
        staff_member_id: staff_member_id.clone(),
        name: name.trim().to_string(),
        phone: None,
        email: None,
        preferred_contact_channel: None,
    };
    let claimed = PracticeManagerClaimedPayload { staff_member_id: staff_member_id.clone() };
    let role_assigned = RoleAssignedPayload {
        staff_member_id: staff_member_id.clone(),
        role: "PracticeManager".to_string(),
    };

    {
        let events = state.events.lock().map_err(|e| e.to_string())?;
        events.append(&stream_id, 0, STAFF_MEMBER_REGISTERED,
            &serde_json::to_string(&registered).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        events.append(&stream_id, 1, PRACTICE_MANAGER_CLAIMED,
            &serde_json::to_string(&claimed).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        events.append(&stream_id, 2, ROLE_ASSIGNED,
            &serde_json::to_string(&role_assigned).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    }

    do_rebuild(&state)?;
    build_dto(&state, &staff_member_id)
}

/// Register a new staff member. Requires at least one active PracticeManager.
#[tauri::command]
#[specta::specta]
pub fn register_staff_member(
    state: State<'_, AppState>,
    name: String,
    phone: Option<String>,
    email: Option<String>,
    preferred_contact_channel: Option<String>,
    initial_role: String,
) -> Result<StaffMemberDto, String> {
    validate_name(&name)?;
    validate_role(&initial_role)?;
    validate_preferred_channel(preferred_contact_channel.as_deref())?;

    do_rebuild(&state)?;

    {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        if proj.count_active_practice_managers().map_err(|e| e.to_string())? == 0 {
            return Err("No active Practice Manager exists. Use ClaimPracticeManagerRole first.".to_string());
        }
    }

    let staff_member_id = Uuid::new_v4().to_string();
    let stream_id = staff_stream(&staff_member_id);

    let registered = StaffMemberRegisteredPayload {
        staff_member_id: staff_member_id.clone(),
        name: name.trim().to_string(),
        phone: phone.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
        email: email.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
        preferred_contact_channel: preferred_contact_channel.clone(),
    };
    let role_assigned = RoleAssignedPayload {
        staff_member_id: staff_member_id.clone(),
        role: initial_role,
    };

    {
        let events = state.events.lock().map_err(|e| e.to_string())?;
        events.append(&stream_id, 0, STAFF_MEMBER_REGISTERED,
            &serde_json::to_string(&registered).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        events.append(&stream_id, 1, ROLE_ASSIGNED,
            &serde_json::to_string(&role_assigned).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    }

    do_rebuild(&state)?;
    build_dto(&state, &staff_member_id)
}

#[tauri::command]
#[specta::specta]
pub fn assign_role(
    state: State<'_, AppState>,
    staff_member_id: String,
    role: String,
) -> Result<StaffMemberDto, String> {
    validate_role(&role)?;
    do_rebuild(&state)?;

    {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        let row = proj.get_staff_member(&staff_member_id).map_err(|e| e.to_string())?
            .ok_or_else(|| "Staff member not found".to_string())?;
        if row.archived {
            return Err("Cannot modify an archived staff member".to_string());
        }
        let roles = proj.list_staff_roles(&staff_member_id).map_err(|e| e.to_string())?;
        if roles.contains(&role) {
            return Err(format!("Staff member already holds the {} role", role));
        }
    }

    let payload = RoleAssignedPayload { staff_member_id: staff_member_id.clone(), role };
    let stream_id = staff_stream(&staff_member_id);
    {
        let events = state.events.lock().map_err(|e| e.to_string())?;
        let ver = events.current_version(&stream_id).map_err(|e| e.to_string())?;
        events.append(&stream_id, ver, ROLE_ASSIGNED,
            &serde_json::to_string(&payload).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    }

    do_rebuild(&state)?;
    build_dto(&state, &staff_member_id)
}

#[tauri::command]
#[specta::specta]
pub fn remove_role(
    state: State<'_, AppState>,
    staff_member_id: String,
    role: String,
) -> Result<StaffMemberDto, String> {
    validate_role(&role)?;
    do_rebuild(&state)?;

    {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        let row = proj.get_staff_member(&staff_member_id).map_err(|e| e.to_string())?
            .ok_or_else(|| "Staff member not found".to_string())?;
        if row.archived {
            return Err("Cannot modify an archived staff member".to_string());
        }
        let roles = proj.list_staff_roles(&staff_member_id).map_err(|e| e.to_string())?;
        if !roles.contains(&role) {
            return Err(format!("Staff member does not hold the {} role", role));
        }
        if roles.len() == 1 {
            return Err("Cannot remove the last role from a staff member".to_string());
        }
        if role == "PracticeManager" {
            let active_pm_count = proj.count_active_practice_managers().map_err(|e| e.to_string())?;
            if active_pm_count <= 1 {
                return Err("Cannot remove the PracticeManager role from the last active Practice Manager".to_string());
            }
        }
    }

    let payload = RoleRemovedPayload { staff_member_id: staff_member_id.clone(), role };
    let stream_id = staff_stream(&staff_member_id);
    {
        let events = state.events.lock().map_err(|e| e.to_string())?;
        let ver = events.current_version(&stream_id).map_err(|e| e.to_string())?;
        events.append(&stream_id, ver, ROLE_REMOVED,
            &serde_json::to_string(&payload).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    }

    do_rebuild(&state)?;
    build_dto(&state, &staff_member_id)
}

#[tauri::command]
#[specta::specta]
pub fn set_pin(
    state: State<'_, AppState>,
    staff_member_id: String,
    new_pin: String,
) -> Result<StaffMemberDto, String> {
    validate_pin(&new_pin)?;
    do_rebuild(&state)?;

    {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        let row = proj.get_staff_member(&staff_member_id).map_err(|e| e.to_string())?
            .ok_or_else(|| "Staff member not found".to_string())?;
        if row.archived {
            return Err("Cannot modify an archived staff member".to_string());
        }
        if row.pin_hash.is_some() {
            return Err("PIN already set — use ChangePIN to update it".to_string());
        }
    }

    let pin_hash = hash_pin(&new_pin)?;
    let payload = PINSetPayload { staff_member_id: staff_member_id.clone(), pin_hash };
    let stream_id = staff_stream(&staff_member_id);
    {
        let events = state.events.lock().map_err(|e| e.to_string())?;
        let ver = events.current_version(&stream_id).map_err(|e| e.to_string())?;
        events.append(&stream_id, ver, PIN_SET,
            &serde_json::to_string(&payload).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    }

    do_rebuild(&state)?;
    build_dto(&state, &staff_member_id)
}

#[tauri::command]
#[specta::specta]
pub fn change_pin(
    state: State<'_, AppState>,
    staff_member_id: String,
    current_pin: String,
    new_pin: String,
) -> Result<StaffMemberDto, String> {
    validate_pin(&new_pin)?;
    do_rebuild(&state)?;

    {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        let row = proj.get_staff_member(&staff_member_id).map_err(|e| e.to_string())?
            .ok_or_else(|| "Staff member not found".to_string())?;
        if row.archived {
            return Err("Cannot modify an archived staff member".to_string());
        }
        let stored_hash = row.pin_hash
            .ok_or_else(|| "No PIN set — use SetPIN to establish a PIN first".to_string())?;
        if !verify_pin(&current_pin, &stored_hash) {
            return Err("Current PIN does not match".to_string());
        }
    }

    let pin_hash = hash_pin(&new_pin)?;
    let payload = PINChangedPayload { staff_member_id: staff_member_id.clone(), pin_hash };
    let stream_id = staff_stream(&staff_member_id);
    {
        let events = state.events.lock().map_err(|e| e.to_string())?;
        let ver = events.current_version(&stream_id).map_err(|e| e.to_string())?;
        events.append(&stream_id, ver, PIN_CHANGED,
            &serde_json::to_string(&payload).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    }

    do_rebuild(&state)?;
    build_dto(&state, &staff_member_id)
}

/// Practice Manager resets another staff member's PIN (forgotten PIN recovery).
#[tauri::command]
#[specta::specta]
pub fn reset_pin(
    state: State<'_, AppState>,
    staff_member_id: String,
    reset_by_staff_member_id: String,
) -> Result<StaffMemberDto, String> {
    if staff_member_id == reset_by_staff_member_id {
        return Err("Use ChangePIN to update your own PIN".to_string());
    }
    do_rebuild(&state)?;

    {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        // Verify issuer is an active PracticeManager
        let issuer = proj.get_staff_member(&reset_by_staff_member_id).map_err(|e| e.to_string())?
            .ok_or_else(|| "Issuer staff member not found".to_string())?;
        if issuer.archived {
            return Err("Only a Practice Manager can reset a PIN".to_string());
        }
        let issuer_roles = proj.list_staff_roles(&reset_by_staff_member_id).map_err(|e| e.to_string())?;
        if !issuer_roles.contains(&"PracticeManager".to_string()) {
            return Err("Only a Practice Manager can reset a PIN".to_string());
        }
        // Verify target exists and is active
        let target = proj.get_staff_member(&staff_member_id).map_err(|e| e.to_string())?
            .ok_or_else(|| "Staff member not found".to_string())?;
        if target.archived {
            return Err("Cannot modify an archived staff member".to_string());
        }
    }

    let payload = PINResetPayload {
        staff_member_id: staff_member_id.clone(),
        reset_by_staff_member_id,
    };
    let stream_id = staff_stream(&staff_member_id);
    {
        let events = state.events.lock().map_err(|e| e.to_string())?;
        let ver = events.current_version(&stream_id).map_err(|e| e.to_string())?;
        events.append(&stream_id, ver, PIN_RESET,
            &serde_json::to_string(&payload).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    }

    do_rebuild(&state)?;
    build_dto(&state, &staff_member_id)
}

#[tauri::command]
#[specta::specta]
pub fn archive_staff_member(
    state: State<'_, AppState>,
    staff_member_id: String,
) -> Result<StaffMemberDto, String> {
    do_rebuild(&state)?;

    {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        let row = proj.get_staff_member(&staff_member_id).map_err(|e| e.to_string())?
            .ok_or_else(|| "Staff member not found".to_string())?;
        if row.archived {
            return Err("Staff member is already archived".to_string());
        }
        let roles = proj.list_staff_roles(&staff_member_id).map_err(|e| e.to_string())?;
        if roles.contains(&"PracticeManager".to_string()) {
            let count = proj.count_active_practice_managers().map_err(|e| e.to_string())?;
            if count <= 1 {
                return Err("Cannot archive the last active Practice Manager. Assign the Practice Manager role to another staff member first.".to_string());
            }
        }
    }

    let payload = StaffMemberArchivedPayload { staff_member_id: staff_member_id.clone() };
    let stream_id = staff_stream(&staff_member_id);
    {
        let events = state.events.lock().map_err(|e| e.to_string())?;
        let ver = events.current_version(&stream_id).map_err(|e| e.to_string())?;
        events.append(&stream_id, ver, STAFF_MEMBER_ARCHIVED,
            &serde_json::to_string(&payload).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    }

    do_rebuild(&state)?;
    build_dto(&state, &staff_member_id)
}

#[tauri::command]
#[specta::specta]
pub fn unarchive_staff_member(
    state: State<'_, AppState>,
    staff_member_id: String,
) -> Result<StaffMemberDto, String> {
    do_rebuild(&state)?;

    {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        let row = proj.get_staff_member(&staff_member_id).map_err(|e| e.to_string())?
            .ok_or_else(|| "Staff member not found".to_string())?;
        if !row.archived {
            return Err("Staff member is not archived".to_string());
        }
    }

    let payload = StaffMemberUnarchivedPayload { staff_member_id: staff_member_id.clone() };
    let stream_id = staff_stream(&staff_member_id);
    {
        let events = state.events.lock().map_err(|e| e.to_string())?;
        let ver = events.current_version(&stream_id).map_err(|e| e.to_string())?;
        events.append(&stream_id, ver, STAFF_MEMBER_UNARCHIVED,
            &serde_json::to_string(&payload).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    }

    do_rebuild(&state)?;
    build_dto(&state, &staff_member_id)
}

/// Verify a PIN without creating any domain event. Returns true if correct.
/// This is a session concern — no event is emitted.
#[tauri::command]
#[specta::specta]
pub fn verify_staff_pin(
    state: State<'_, AppState>,
    staff_member_id: String,
    pin: String,
) -> Result<bool, String> {
    do_rebuild(&state)?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    let row = proj.get_staff_member(&staff_member_id).map_err(|e| e.to_string())?
        .ok_or_else(|| "Staff member not found".to_string())?;
    if row.archived {
        return Ok(false);
    }
    let Some(hash) = row.pin_hash else {
        return Ok(false);
    };
    Ok(verify_pin(&pin, &hash))
}

#[tauri::command]
#[specta::specta]
pub fn list_staff_members(
    state: State<'_, AppState>,
) -> Result<Vec<StaffMemberDto>, String> {
    do_rebuild(&state)?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    let rows = proj.list_staff_members().map_err(|e| e.to_string())?;
    let mut dtos = Vec::with_capacity(rows.len());
    for row in &rows {
        dtos.push(build_dto_from_proj(&proj, &row.staff_member_id)?);
    }
    Ok(dtos)
}

#[tauri::command]
#[specta::specta]
pub fn get_staff_member_dto(
    state: State<'_, AppState>,
    staff_member_id: String,
) -> Result<StaffMemberDto, String> {
    do_rebuild(&state)?;
    build_dto(&state, &staff_member_id)
}

/// Staff Management setup step: complete when at least one active PM has a PIN.
#[tauri::command]
#[specta::specta]
pub fn get_staff_setup_status(
    state: State<'_, AppState>,
) -> Result<StaffSetupStatusDto, String> {
    do_rebuild(&state)?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    let complete = proj.has_active_pm_with_pin().map_err(|e| e.to_string())?;
    Ok(StaffSetupStatusDto { complete })
}

// ── Provider clinical commands ─────────────────────────────────────────────────

/// Set the clinical specialization of a Provider-role staff member.
#[tauri::command]
#[specta::specta]
pub fn set_provider_type(
    state: State<'_, AppState>,
    staff_member_id: String,
    clinical_specialization: String,
) -> Result<StaffMemberDto, String> {
    validate_clinical_specialization(&clinical_specialization)?;
    do_rebuild(&state)?;

    {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        let row = proj.get_staff_member(&staff_member_id).map_err(|e| e.to_string())?
            .ok_or_else(|| "Staff member not found".to_string())?;
        if row.archived {
            return Err("Cannot modify an archived staff member".to_string());
        }
        let roles = proj.list_staff_roles(&staff_member_id).map_err(|e| e.to_string())?;
        if !roles.contains(&"Provider".to_string()) {
            return Err("Staff member does not hold the Provider role".to_string());
        }
    }

    let payload = crate::events::staff_management::ProviderTypeSetPayload {
        staff_member_id: staff_member_id.clone(),
        clinical_specialization,
    };
    let stream_id = staff_stream(&staff_member_id);
    {
        let events = state.events.lock().map_err(|e| e.to_string())?;
        let ver = events.current_version(&stream_id).map_err(|e| e.to_string())?;
        events.append(&stream_id, ver, PROVIDER_TYPE_SET,
            &serde_json::to_string(&payload).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    }

    do_rebuild(&state)?;
    build_dto(&state, &staff_member_id)
}

/// Assign a Provider-role staff member to an office.
#[tauri::command]
#[specta::specta]
pub fn assign_provider_to_office(
    state: State<'_, AppState>,
    staff_member_id: String,
    office_id: String,
) -> Result<StaffMemberDto, String> {
    do_rebuild(&state)?;

    {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        let row = proj.get_staff_member(&staff_member_id).map_err(|e| e.to_string())?
            .ok_or_else(|| "Staff member not found".to_string())?;
        if row.archived {
            return Err("Cannot modify an archived staff member".to_string());
        }
        let roles = proj.list_staff_roles(&staff_member_id).map_err(|e| e.to_string())?;
        if !roles.contains(&"Provider".to_string()) {
            return Err("Staff member does not hold the Provider role".to_string());
        }
        let office = proj.get_office(&office_id).map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Office '{}' not found", office_id))?;
        if office.archived {
            return Err(format!("Office '{}' is archived", office_id));
        }
        let current_offices = proj.list_staff_offices(&staff_member_id).map_err(|e| e.to_string())?;
        if current_offices.contains(&office_id) {
            return Err(format!("Provider is already assigned to office '{}'", office_id));
        }
    }

    let payload = crate::events::staff_management::ProviderAssignedToOfficePayload {
        staff_member_id: staff_member_id.clone(),
        office_id,
    };
    let stream_id = staff_stream(&staff_member_id);
    {
        let events = state.events.lock().map_err(|e| e.to_string())?;
        let ver = events.current_version(&stream_id).map_err(|e| e.to_string())?;
        events.append(&stream_id, ver, PROVIDER_ASSIGNED_TO_OFFICE,
            &serde_json::to_string(&payload).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    }

    do_rebuild(&state)?;
    build_dto(&state, &staff_member_id)
}

/// Remove a Provider-role staff member from an office (also clears their availability for that office).
#[tauri::command]
#[specta::specta]
pub fn remove_provider_from_office(
    state: State<'_, AppState>,
    staff_member_id: String,
    office_id: String,
) -> Result<StaffMemberDto, String> {
    do_rebuild(&state)?;

    let days_to_clear: Vec<String> = {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        let current_offices = proj.list_staff_offices(&staff_member_id).map_err(|e| e.to_string())?;
        if !current_offices.contains(&office_id) {
            return Err(format!("Provider is not assigned to office '{}'", office_id));
        }
        proj.list_staff_availability_for_office(&staff_member_id, &office_id)
            .map_err(|e| e.to_string())?
    };

    let stream_id = staff_stream(&staff_member_id);
    {
        let events = state.events.lock().map_err(|e| e.to_string())?;
        let mut ver = events.current_version(&stream_id).map_err(|e| e.to_string())?;
        let json = serde_json::to_string(&crate::events::staff_management::ProviderRemovedFromOfficePayload {
            staff_member_id: staff_member_id.clone(),
            office_id: office_id.clone(),
        }).map_err(|e| e.to_string())?;
        events.append(&stream_id, ver, PROVIDER_REMOVED_FROM_OFFICE, &json)
            .map_err(|e| e.to_string())?;
        ver += 1;
        for day in &days_to_clear {
            let json = serde_json::to_string(&crate::events::staff_management::ProviderAvailabilityClearedPayload {
                staff_member_id: staff_member_id.clone(),
                office_id: office_id.clone(),
                day_of_week: day.clone(),
            }).map_err(|e| e.to_string())?;
            events.append(&stream_id, ver, PROVIDER_AVAILABILITY_CLEARED, &json)
                .map_err(|e| e.to_string())?;
            ver += 1;
        }
    }

    do_rebuild(&state)?;
    build_dto(&state, &staff_member_id)
}

/// Set a provider's availability window for a specific office and day.
#[tauri::command]
#[specta::specta]
pub fn set_provider_availability(
    state: State<'_, AppState>,
    staff_member_id: String,
    office_id: String,
    day_of_week: String,
    start_time: String,
    end_time: String,
) -> Result<StaffMemberDto, String> {
    validate_hhmm(&start_time)?;
    validate_hhmm(&end_time)?;
    validate_time_range(&start_time, &end_time)?;
    do_rebuild(&state)?;

    {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        let row = proj.get_staff_member(&staff_member_id).map_err(|e| e.to_string())?
            .ok_or_else(|| "Staff member not found".to_string())?;
        if row.archived {
            return Err("Cannot modify an archived staff member".to_string());
        }
        let roles = proj.list_staff_roles(&staff_member_id).map_err(|e| e.to_string())?;
        if !roles.contains(&"Provider".to_string()) {
            return Err("Staff member does not hold the Provider role".to_string());
        }
        let current_offices = proj.list_staff_offices(&staff_member_id).map_err(|e| e.to_string())?;
        if !current_offices.contains(&office_id) {
            return Err(format!("Provider is not assigned to office '{}'", office_id));
        }
        let avail = proj.list_staff_availability(&staff_member_id).map_err(|e| e.to_string())?;
        check_no_cross_office_overlap(&avail, &office_id, &day_of_week, &start_time, &end_time)?;
    }

    let payload = crate::events::staff_management::ProviderAvailabilitySetPayload {
        staff_member_id: staff_member_id.clone(),
        office_id,
        day_of_week,
        start_time,
        end_time,
    };
    let stream_id = staff_stream(&staff_member_id);
    {
        let events = state.events.lock().map_err(|e| e.to_string())?;
        let ver = events.current_version(&stream_id).map_err(|e| e.to_string())?;
        events.append(&stream_id, ver, PROVIDER_AVAILABILITY_SET,
            &serde_json::to_string(&payload).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    }

    do_rebuild(&state)?;
    build_dto(&state, &staff_member_id)
}

/// Clear a provider's availability window for a specific office and day.
#[tauri::command]
#[specta::specta]
pub fn clear_provider_availability(
    state: State<'_, AppState>,
    staff_member_id: String,
    office_id: String,
    day_of_week: String,
) -> Result<StaffMemberDto, String> {
    do_rebuild(&state)?;

    {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        let avail = proj.list_staff_availability(&staff_member_id).map_err(|e| e.to_string())?;
        let has = avail.iter().any(|a| a.office_id == office_id && a.day_of_week == day_of_week);
        if !has {
            return Err(format!(
                "Provider has no availability for office '{}' on {}",
                office_id, day_of_week
            ));
        }
    }

    let payload = crate::events::staff_management::ProviderAvailabilityClearedPayload {
        staff_member_id: staff_member_id.clone(),
        office_id,
        day_of_week,
    };
    let stream_id = staff_stream(&staff_member_id);
    {
        let events = state.events.lock().map_err(|e| e.to_string())?;
        let ver = events.current_version(&stream_id).map_err(|e| e.to_string())?;
        events.append(&stream_id, ver, PROVIDER_AVAILABILITY_CLEARED,
            &serde_json::to_string(&payload).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    }

    do_rebuild(&state)?;
    build_dto(&state, &staff_member_id)
}

/// Set a provider availability exception (time-off block).
#[tauri::command]
#[specta::specta]
pub fn set_provider_exception(
    state: State<'_, AppState>,
    staff_member_id: String,
    start_date: String,
    end_date: String,
    reason: Option<String>,
) -> Result<StaffMemberDto, String> {
    validate_date_ymd(&start_date)?;
    validate_date_ymd(&end_date)?;
    validate_date_range(&start_date, &end_date)?;
    do_rebuild(&state)?;

    {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        let row = proj.get_staff_member(&staff_member_id).map_err(|e| e.to_string())?
            .ok_or_else(|| "Staff member not found".to_string())?;
        if row.archived {
            return Err("Cannot modify an archived staff member".to_string());
        }
        let roles = proj.list_staff_roles(&staff_member_id).map_err(|e| e.to_string())?;
        if !roles.contains(&"Provider".to_string()) {
            return Err("Staff member does not hold the Provider role".to_string());
        }
    }

    let payload = crate::events::staff_management::ProviderExceptionSetPayload {
        staff_member_id: staff_member_id.clone(),
        start_date,
        end_date,
        reason,
    };
    let stream_id = staff_stream(&staff_member_id);
    {
        let events = state.events.lock().map_err(|e| e.to_string())?;
        let ver = events.current_version(&stream_id).map_err(|e| e.to_string())?;
        events.append(&stream_id, ver, PROVIDER_EXCEPTION_SET,
            &serde_json::to_string(&payload).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    }

    do_rebuild(&state)?;
    build_dto(&state, &staff_member_id)
}

/// Remove a provider availability exception.
#[tauri::command]
#[specta::specta]
pub fn remove_provider_exception(
    state: State<'_, AppState>,
    staff_member_id: String,
    start_date: String,
    end_date: String,
) -> Result<StaffMemberDto, String> {
    do_rebuild(&state)?;

    {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        let exceptions = proj.list_staff_exceptions(&staff_member_id).map_err(|e| e.to_string())?;
        let found = exceptions.iter().any(|e| e.start_date == start_date && e.end_date == end_date);
        if !found {
            return Err(format!(
                "No exception found for provider from {} to {}",
                start_date, end_date
            ));
        }
    }

    let payload = crate::events::staff_management::ProviderExceptionRemovedPayload {
        staff_member_id: staff_member_id.clone(),
        start_date,
        end_date,
    };
    let stream_id = staff_stream(&staff_member_id);
    {
        let events = state.events.lock().map_err(|e| e.to_string())?;
        let ver = events.current_version(&stream_id).map_err(|e| e.to_string())?;
        events.append(&stream_id, ver, PROVIDER_EXCEPTION_REMOVED,
            &serde_json::to_string(&payload).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    }

    do_rebuild(&state)?;
    build_dto(&state, &staff_member_id)
}

/// List all active (non-archived) Provider-role staff members.
#[tauri::command]
#[specta::specta]
pub fn list_providers(
    state: State<'_, AppState>,
) -> Result<Vec<StaffMemberDto>, String> {
    do_rebuild(&state)?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    let rows = proj.list_staff_members().map_err(|e| e.to_string())?;
    let mut dtos = Vec::new();
    for row in &rows {
        let roles = proj.list_staff_roles(&row.staff_member_id).map_err(|e| e.to_string())?;
        if roles.contains(&"Provider".to_string()) {
            dtos.push(build_dto_from_proj(&proj, &row.staff_member_id)?);
        }
    }
    Ok(dtos)
}
