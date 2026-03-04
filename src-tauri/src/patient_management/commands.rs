use tauri::State;
use uuid::Uuid;
use chrono::Utc;

use crate::app_state::AppState;
use crate::db::PatientNoteRow;
use crate::events::patient_management::*;
use crate::projections::patient_management::rebuild;
use super::service::*;
use super::types::*;

fn do_rebuild(state: &AppState) -> Result<(), String> {
    let events = state.events.lock().map_err(|e| e.to_string())?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    rebuild(&events, &proj)
}

fn row_to_dto(row: crate::db::PatientRow) -> PatientDto {
    PatientDto {
        patient_id: row.patient_id,
        first_name: row.first_name,
        last_name: row.last_name,
        full_name_display: row.full_name_display,
        phone: row.phone,
        email: row.email,
        preferred_contact_channel: row.preferred_contact_channel,
        preferred_office_id: row.preferred_office_id,
        date_of_birth: row.date_of_birth,
        address_line_1: row.address_line_1,
        city_town: row.city_town,
        subdivision: row.subdivision,
        country: row.country,
        registered_by: row.registered_by,
        registered_at: row.registered_at,
        archived: row.archived,
    }
}

fn note_row_to_dto(row: PatientNoteRow) -> PatientNoteDto {
    PatientNoteDto {
        note_id: row.note_id,
        patient_id: row.patient_id,
        text: row.text,
        recorded_by: row.recorded_by,
        recorded_at: row.recorded_at,
    }
}

// ── Commands ──────────────────────────────────────────────────────────────────

#[tauri::command(rename_all = "snake_case")]
#[specta::specta]
pub fn register_patient(
    state: State<'_, AppState>,
    first_name: String,
    last_name: String,
    phone: Option<String>,
    email: Option<String>,
    preferred_contact_channel: Option<String>,
    preferred_office_id: Option<String>,
    date_of_birth: Option<String>,
    registered_by: String,
) -> Result<RegisterPatientResult, String> {
    validate_name(&first_name, "First name")?;
    validate_name(&last_name, "Last name")?;
    validate_contact_required(phone.as_deref(), email.as_deref())?;
    validate_preferred_channel(preferred_contact_channel.as_deref())?;
    validate_date_ymd_opt(date_of_birth.as_deref(), "Date of birth")?;

    do_rebuild(&state)?;

    // Soft duplicate check (only when phone is provided)
    let duplicate_warning = if let Some(ref ph) = phone {
        if !ph.trim().is_empty() {
            let proj = state.projections.lock().map_err(|e| e.to_string())?;
            let is_dup = proj.check_duplicate_patient(&first_name, &last_name, ph)
                .map_err(|e| e.to_string())?;
            if is_dup {
                Some(format!(
                    "A patient named {} {} with this phone number already exists.",
                    first_name, last_name
                ))
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let patient_id = Uuid::new_v4().to_string();
    let payload = PatientRegisteredPayload {
        patient_id: patient_id.clone(),
        first_name: first_name.clone(),
        last_name: last_name.clone(),
        phone: phone.clone(),
        email: email.clone(),
        preferred_contact_channel: preferred_contact_channel.clone(),
        preferred_office_id: preferred_office_id.clone(),
        date_of_birth: date_of_birth.clone(),
        registered_by: registered_by.clone(),
    };
    let payload_json = serde_json::to_string(&payload).map_err(|e| e.to_string())?;
    let stream_id = format!("patient:{}", patient_id);

    {
        let events = state.events.lock().map_err(|e| e.to_string())?;
        events.append(&stream_id, 0, PATIENT_REGISTERED, &payload_json)
            .map_err(|e| e.to_string())?;
    }

    do_rebuild(&state)?;

    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    let row = proj.get_patient(&patient_id).map_err(|e| e.to_string())?
        .ok_or_else(|| "Patient not found after registration".to_string())?;

    Ok(RegisterPatientResult {
        patient: row_to_dto(row),
        duplicate_warning,
    })
}

#[tauri::command(rename_all = "snake_case")]
#[specta::specta]
pub fn update_patient_demographics(
    state: State<'_, AppState>,
    patient_id: String,
    first_name: String,
    last_name: String,
    date_of_birth: Option<String>,
    address_line_1: Option<String>,
    city_town: Option<String>,
    subdivision: Option<String>,
    country: Option<String>,
    updated_by: String,
) -> Result<PatientDto, String> {
    validate_name(&first_name, "First name")?;
    validate_name(&last_name, "Last name")?;
    validate_date_ymd_opt(date_of_birth.as_deref(), "Date of birth")?;

    do_rebuild(&state)?;

    {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        proj.get_patient(&patient_id).map_err(|e| e.to_string())?
            .ok_or_else(|| "Patient not found".to_string())?;
    }

    let payload = PatientDemographicsUpdatedPayload {
        patient_id: patient_id.clone(),
        first_name,
        last_name,
        date_of_birth,
        address_line_1,
        city_town,
        subdivision,
        country,
        updated_by,
    };
    let payload_json = serde_json::to_string(&payload).map_err(|e| e.to_string())?;
    let stream_id = format!("patient:{}", patient_id);

    let version = {
        let events = state.events.lock().map_err(|e| e.to_string())?;
        let current = events.current_version(&stream_id).map_err(|e| e.to_string())?;
        events.append(&stream_id, current, PATIENT_DEMOGRAPHICS_UPDATED, &payload_json)
            .map_err(|e| e.to_string())?;
        current + 1
    };
    let _ = version;

    do_rebuild(&state)?;

    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    let row = proj.get_patient(&patient_id).map_err(|e| e.to_string())?
        .ok_or_else(|| "Patient not found".to_string())?;
    Ok(row_to_dto(row))
}

#[tauri::command(rename_all = "snake_case")]
#[specta::specta]
pub fn update_patient_contact_info(
    state: State<'_, AppState>,
    patient_id: String,
    phone: Option<String>,
    email: Option<String>,
    preferred_contact_channel: Option<String>,
    updated_by: String,
) -> Result<PatientDto, String> {
    validate_contact_required(phone.as_deref(), email.as_deref())?;
    validate_preferred_channel(preferred_contact_channel.as_deref())?;

    do_rebuild(&state)?;

    {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        proj.get_patient(&patient_id).map_err(|e| e.to_string())?
            .ok_or_else(|| "Patient not found".to_string())?;
    }

    let payload = PatientContactInfoUpdatedPayload {
        patient_id: patient_id.clone(),
        phone,
        email,
        preferred_contact_channel,
        updated_by,
    };
    let payload_json = serde_json::to_string(&payload).map_err(|e| e.to_string())?;
    let stream_id = format!("patient:{}", patient_id);

    {
        let events = state.events.lock().map_err(|e| e.to_string())?;
        let current = events.current_version(&stream_id).map_err(|e| e.to_string())?;
        events.append(&stream_id, current, PATIENT_CONTACT_INFO_UPDATED, &payload_json)
            .map_err(|e| e.to_string())?;
    }

    do_rebuild(&state)?;

    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    let row = proj.get_patient(&patient_id).map_err(|e| e.to_string())?
        .ok_or_else(|| "Patient not found".to_string())?;
    Ok(row_to_dto(row))
}

#[tauri::command(rename_all = "snake_case")]
#[specta::specta]
pub fn add_patient_note(
    state: State<'_, AppState>,
    patient_id: String,
    text: String,
    recorded_by: String,
) -> Result<PatientNoteDto, String> {
    validate_note_text(&text)?;

    do_rebuild(&state)?;

    // Notes allowed on archived patients (PM-Rule-6) — no archive check here
    {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        proj.get_patient(&patient_id).map_err(|e| e.to_string())?
            .ok_or_else(|| "Patient not found".to_string())?;
    }

    let note_id = Uuid::new_v4().to_string();
    let recorded_at = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
    let payload = PatientNoteAddedPayload {
        patient_id: patient_id.clone(),
        note_id: note_id.clone(),
        text: text.trim().to_string(),
        recorded_by,
        recorded_at: recorded_at.clone(),
    };
    let payload_json = serde_json::to_string(&payload).map_err(|e| e.to_string())?;
    let stream_id = format!("patient:{}", patient_id);

    {
        let events = state.events.lock().map_err(|e| e.to_string())?;
        let current = events.current_version(&stream_id).map_err(|e| e.to_string())?;
        events.append(&stream_id, current, PATIENT_NOTE_ADDED, &payload_json)
            .map_err(|e| e.to_string())?;
    }

    do_rebuild(&state)?;

    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    let notes = proj.list_patient_notes(&patient_id).map_err(|e| e.to_string())?;
    let note = notes.into_iter()
        .find(|n| n.note_id == note_id)
        .ok_or_else(|| "Note not found after insertion".to_string())?;
    Ok(note_row_to_dto(note))
}

#[tauri::command(rename_all = "snake_case")]
#[specta::specta]
pub fn archive_patient(
    state: State<'_, AppState>,
    patient_id: String,
    archived_by: String,
) -> Result<PatientDto, String> {
    do_rebuild(&state)?;

    {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        let row = proj.get_patient(&patient_id).map_err(|e| e.to_string())?
            .ok_or_else(|| "Patient not found".to_string())?;
        if row.archived {
            return Err("Patient is already archived".to_string());
        }
    }

    let payload = PatientArchivedPayload { patient_id: patient_id.clone(), archived_by };
    let payload_json = serde_json::to_string(&payload).map_err(|e| e.to_string())?;
    let stream_id = format!("patient:{}", patient_id);

    {
        let events = state.events.lock().map_err(|e| e.to_string())?;
        let current = events.current_version(&stream_id).map_err(|e| e.to_string())?;
        events.append(&stream_id, current, PATIENT_ARCHIVED, &payload_json)
            .map_err(|e| e.to_string())?;
    }

    do_rebuild(&state)?;

    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    let row = proj.get_patient(&patient_id).map_err(|e| e.to_string())?
        .ok_or_else(|| "Patient not found".to_string())?;
    Ok(row_to_dto(row))
}

#[tauri::command(rename_all = "snake_case")]
#[specta::specta]
pub fn unarchive_patient(
    state: State<'_, AppState>,
    patient_id: String,
    unarchived_by: String,
) -> Result<PatientDto, String> {
    do_rebuild(&state)?;

    {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        let row = proj.get_patient(&patient_id).map_err(|e| e.to_string())?
            .ok_or_else(|| "Patient not found".to_string())?;
        if !row.archived {
            return Err("Patient is not archived".to_string());
        }
    }

    let payload = PatientUnarchivedPayload { patient_id: patient_id.clone(), unarchived_by };
    let payload_json = serde_json::to_string(&payload).map_err(|e| e.to_string())?;
    let stream_id = format!("patient:{}", patient_id);

    {
        let events = state.events.lock().map_err(|e| e.to_string())?;
        let current = events.current_version(&stream_id).map_err(|e| e.to_string())?;
        events.append(&stream_id, current, PATIENT_UNARCHIVED, &payload_json)
            .map_err(|e| e.to_string())?;
    }

    do_rebuild(&state)?;

    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    let row = proj.get_patient(&patient_id).map_err(|e| e.to_string())?
        .ok_or_else(|| "Patient not found".to_string())?;
    Ok(row_to_dto(row))
}

#[tauri::command(rename_all = "snake_case")]
#[specta::specta]
pub fn search_patients(
    state: State<'_, AppState>,
    name_prefix: Option<String>,
    phone_fragment: Option<String>,
    preferred_office_id: Option<String>,
    include_archived: bool,
) -> Result<Vec<PatientDto>, String> {
    do_rebuild(&state)?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    let rows = proj.search_patients(
        name_prefix.as_deref(),
        phone_fragment.as_deref(),
        preferred_office_id.as_deref(),
        include_archived,
    ).map_err(|e| e.to_string())?;
    Ok(rows.into_iter().map(row_to_dto).collect())
}

#[tauri::command(rename_all = "snake_case")]
#[specta::specta]
pub fn get_patient(
    state: State<'_, AppState>,
    patient_id: String,
) -> Result<PatientWithNotesDto, String> {
    do_rebuild(&state)?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    let row = proj.get_patient(&patient_id).map_err(|e| e.to_string())?
        .ok_or_else(|| "Patient not found".to_string())?;
    let notes = proj.list_patient_notes(&patient_id).map_err(|e| e.to_string())?;
    Ok(PatientWithNotesDto {
        patient: row_to_dto(row),
        notes: notes.into_iter().map(note_row_to_dto).collect(),
    })
}
