use tauri::State;
use uuid::Uuid;
use chrono::Utc;
use crate::app_state::AppState;
use crate::events::appointments::*;
use crate::appointments::service;
use crate::appointments::types::*;
use crate::projections::appointments::rebuild;

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

fn row_to_dto(row: &crate::db::AppointmentRow) -> AppointmentDto {
    AppointmentDto {
        appointment_id: row.appointment_id.clone(),
        office_id: row.office_id.clone(),
        patient_id: row.patient_id.clone(),
        patient_name: row.patient_name.clone(),
        patient_phone: row.patient_phone.clone(),
        procedure_type_id: row.procedure_type_id.clone(),
        procedure_name: row.procedure_name.clone(),
        procedure_category: row.procedure_category.clone(),
        provider_id: row.provider_id.clone(),
        provider_name: row.provider_name.clone(),
        start_time: row.start_time.clone(),
        end_time: row.end_time.clone(),
        duration_minutes: row.duration_minutes,
        status: row.status.clone(),
        rescheduled_to_id: row.rescheduled_to_id.clone(),
        rescheduled_from_id: row.rescheduled_from_id.clone(),
        booked_by: row.booked_by.clone(),
    }
}

fn note_row_to_dto(row: &crate::db::AppointmentNoteRow) -> AppointmentNoteDto {
    AppointmentNoteDto {
        note_id: row.note_id.clone(),
        appointment_id: row.appointment_id.clone(),
        text: row.text.clone(),
        recorded_by: row.recorded_by.clone(),
        recorded_at: row.recorded_at.clone(),
    }
}

/// Run all 5 booking constraints. Returns Err if any fail.
/// Reads constraint data from the projection store, then drops the lock before returning.
fn check_booking_constraints(
    state: &State<'_, AppState>,
    office_id: &str,
    patient_id: &str,
    procedure_type_id: &str,
    provider_id: &str,
    start_time: &str,
    duration_minutes: u32,
    exclude_appointment_id: Option<&str>,
) -> Result<String, String> {
    // Parse datetime and compute end_time
    let start_dt = service::parse_datetime(start_time)?;
    let end_time = service::compute_end_time(&start_dt, duration_minutes);
    let end_dt = service::parse_datetime(&end_time)?;

    // Gather all constraint data from projections (hold lock, read, drop)
    let (office, office_hours, provider, avail, exceptions, patient, procedure, overlapping) = {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;

        let office = proj.get_office(office_id).map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Office {} not found", office_id))?;
        let office_hours = proj.list_office_hours(office_id).map_err(|e| e.to_string())?;
        let provider = proj.get_provider(provider_id).map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Provider {} not found", provider_id))?;
        let avail = proj.list_provider_availability(provider_id).map_err(|e| e.to_string())?;
        let exceptions = proj.list_provider_exceptions(provider_id).map_err(|e| e.to_string())?;
        let patient = proj.get_patient(patient_id).map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Patient {} not found", patient_id))?;
        let procedure = proj.get_procedure_type(procedure_type_id).map_err(|e| e.to_string())?
            .ok_or_else(|| format!("ProcedureType {} not found", procedure_type_id))?;
        let overlapping = proj.count_overlapping_booked(
            office_id, start_time, &end_time, exclude_appointment_id,
        ).map_err(|e| e.to_string())?;

        (office, office_hours, provider, avail, exceptions, patient, procedure, overlapping)
    };

    // C1
    service::check_c1_office_open(&office_hours, &office.name, &start_dt, &end_dt)?;
    // C2
    service::check_c2_provider_available(
        &avail, &exceptions, &provider.name, office_id, &office.name, &start_dt, &end_dt,
    )?;
    // C3
    service::check_c3_chair_capacity(overlapping, office.chair_count, &office.name, &start_dt)?;
    // C4
    service::check_c4_patient_active(patient.archived)?;
    // C5
    service::check_c5_procedure_active(procedure.is_active, &procedure.name)?;

    Ok(end_time)
}

// ── Commands ──────────────────────────────────────────────────────────────────

#[specta::specta]
#[tauri::command]
pub async fn book_appointment(
    state: State<'_, AppState>,
    office_id: String,
    patient_id: String,
    procedure_type_id: String,
    provider_id: String,
    start_time: String,
    duration_minutes: Option<u32>,
    booked_by: String,
) -> Result<BookAppointmentResult, String> {
    // Resolve duration: use provided override or fall back to procedure default
    let final_duration = match duration_minutes {
        Some(d) => {
            service::validate_duration(d)?;
            d
        }
        None => {
            let proj = state.projections.lock().map_err(|e| e.to_string())?;
            let proc = proj.get_procedure_type(&procedure_type_id).map_err(|e| e.to_string())?
                .ok_or_else(|| format!("ProcedureType {} not found", procedure_type_id))?;
            proc.default_duration_minutes
        }
    };

    // Run all 5 constraints — returns end_time
    let end_time = check_booking_constraints(
        &state, &office_id, &patient_id, &procedure_type_id, &provider_id,
        &start_time, final_duration, None,
    )?;

    let appointment_id = Uuid::new_v4().to_string();
    let stream_id = format!("appointment:{}", appointment_id);

    append_event(&state, &stream_id, APPOINTMENT_BOOKED, &AppointmentBookedPayload {
        appointment_id: appointment_id.clone(),
        office_id,
        patient_id,
        procedure_type_id,
        provider_id,
        start_time,
        end_time,
        duration_minutes: final_duration,
        booked_by,
        rescheduled_from_id: None,
    })?;

    do_rebuild(&state)?;
    Ok(BookAppointmentResult { appointment_id })
}

#[specta::specta]
#[tauri::command]
pub async fn reschedule_appointment(
    state: State<'_, AppState>,
    appointment_id: String,
    new_office_id: String,
    new_provider_id: String,
    new_start_time: String,
    new_duration_minutes: Option<u32>,
    rescheduled_by: String,
) -> Result<RescheduleAppointmentResult, String> {
    // Get original appointment
    let (patient_id, procedure_type_id, original_duration) = {
        do_rebuild(&state)?;
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        let orig = proj.get_appointment(&appointment_id).map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Appointment {} not found", appointment_id))?;
        if orig.status != "Booked" {
            return Err("Only Booked appointments can be rescheduled".to_string());
        }
        (orig.patient_id.clone(), orig.procedure_type_id.clone(), orig.duration_minutes)
    };

    let final_duration = match new_duration_minutes {
        Some(d) => { service::validate_duration(d)?; d }
        None => original_duration,
    };

    // Run all 5 constraints on the new slot
    let new_end_time = check_booking_constraints(
        &state, &new_office_id, &patient_id, &procedure_type_id, &new_provider_id,
        &new_start_time, final_duration, None,
    )?;

    let new_appointment_id = Uuid::new_v4().to_string();
    let orig_stream = format!("appointment:{}", appointment_id);
    let new_stream  = format!("appointment:{}", new_appointment_id);

    // AppointmentRescheduled on original stream
    append_event(&state, &orig_stream, APPOINTMENT_RESCHEDULED, &AppointmentRescheduledPayload {
        appointment_id: appointment_id.clone(),
        rescheduled_to_id: new_appointment_id.clone(),
        rescheduled_by: rescheduled_by.clone(),
    })?;

    // AppointmentBooked on new stream
    append_event(&state, &new_stream, APPOINTMENT_BOOKED, &AppointmentBookedPayload {
        appointment_id: new_appointment_id.clone(),
        office_id: new_office_id,
        patient_id,
        procedure_type_id,
        provider_id: new_provider_id,
        start_time: new_start_time,
        end_time: new_end_time,
        duration_minutes: final_duration,
        booked_by: rescheduled_by,
        rescheduled_from_id: Some(appointment_id),
    })?;

    do_rebuild(&state)?;
    Ok(RescheduleAppointmentResult { new_appointment_id })
}

#[specta::specta]
#[tauri::command]
pub async fn cancel_appointment(
    state: State<'_, AppState>,
    appointment_id: String,
    cancelled_by: String,
    reason: Option<String>,
) -> Result<(), String> {
    do_rebuild(&state)?;
    let status = {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        let appt = proj.get_appointment(&appointment_id).map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Appointment {} not found", appointment_id))?;
        appt.status.clone()
    };

    match status.as_str() {
        "Booked" => {}
        "Cancelled" => return Err("Appointment is already cancelled".to_string()),
        "Completed" => return Err("Appointment cannot be cancelled — it has already been completed".to_string()),
        "NoShow" => return Err("Appointment cannot be cancelled — it has been marked no-show".to_string()),
        "Rescheduled" => return Err("Appointment cannot be cancelled — it has already been rescheduled".to_string()),
        other => return Err(format!("Cannot cancel appointment with status {}", other)),
    }

    let stream_id = format!("appointment:{}", appointment_id);
    append_event(&state, &stream_id, APPOINTMENT_CANCELLED, &AppointmentCancelledPayload {
        appointment_id,
        cancelled_by,
        reason,
    })?;
    do_rebuild(&state)?;
    Ok(())
}

#[specta::specta]
#[tauri::command]
pub async fn complete_appointment(
    state: State<'_, AppState>,
    appointment_id: String,
    completed_by: String,
) -> Result<(), String> {
    do_rebuild(&state)?;
    let status = {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        let appt = proj.get_appointment(&appointment_id).map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Appointment {} not found", appointment_id))?;
        appt.status.clone()
    };

    match status.as_str() {
        "Booked" => {}
        "Completed" => return Err("Appointment is already completed".to_string()),
        "Cancelled" => return Err("Appointment cannot be completed — it has been cancelled".to_string()),
        "NoShow" => return Err("Appointment cannot be completed — it has been marked no-show".to_string()),
        "Rescheduled" => return Err("Appointment cannot be completed — it has been rescheduled".to_string()),
        other => return Err(format!("Cannot complete appointment with status {}", other)),
    }

    let stream_id = format!("appointment:{}", appointment_id);
    append_event(&state, &stream_id, APPOINTMENT_COMPLETED, &AppointmentCompletedPayload {
        appointment_id,
        completed_by,
    })?;
    do_rebuild(&state)?;
    Ok(())
}

#[specta::specta]
#[tauri::command]
pub async fn mark_appointment_no_show(
    state: State<'_, AppState>,
    appointment_id: String,
    recorded_by: String,
) -> Result<(), String> {
    do_rebuild(&state)?;
    let status = {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        let appt = proj.get_appointment(&appointment_id).map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Appointment {} not found", appointment_id))?;
        appt.status.clone()
    };

    match status.as_str() {
        "Booked" => {}
        "NoShow" => return Err("Appointment is already marked no-show".to_string()),
        "Completed" => return Err("Appointment cannot be marked no-show — it has already been completed".to_string()),
        "Cancelled" => return Err("Appointment cannot be marked no-show — it has been cancelled".to_string()),
        "Rescheduled" => return Err("Appointment cannot be marked no-show — it has been rescheduled".to_string()),
        other => return Err(format!("Cannot mark appointment with status {} as no-show", other)),
    }

    let stream_id = format!("appointment:{}", appointment_id);
    append_event(&state, &stream_id, APPOINTMENT_MARKED_NO_SHOW, &AppointmentMarkedNoShowPayload {
        appointment_id,
        recorded_by,
    })?;
    do_rebuild(&state)?;
    Ok(())
}

#[specta::specta]
#[tauri::command]
pub async fn add_appointment_note(
    state: State<'_, AppState>,
    appointment_id: String,
    text: String,
    recorded_by: String,
) -> Result<AppointmentNoteDto, String> {
    service::validate_note_text(&text)?;
    service::validate_recorded_by(&recorded_by)?;

    // Appointment must exist (any status)
    do_rebuild(&state)?;
    {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        proj.get_appointment(&appointment_id).map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Appointment {} not found", appointment_id))?;
    }

    let note_id = Uuid::new_v4().to_string();
    let recorded_at = Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let stream_id = format!("appointment:{}", appointment_id);

    append_event(&state, &stream_id, APPOINTMENT_NOTE_ADDED, &AppointmentNoteAddedPayload {
        appointment_id: appointment_id.clone(),
        note_id: note_id.clone(),
        text: text.clone(),
        recorded_by: recorded_by.clone(),
        recorded_at: recorded_at.clone(),
    })?;
    do_rebuild(&state)?;

    Ok(AppointmentNoteDto {
        note_id,
        appointment_id,
        text,
        recorded_by,
        recorded_at,
    })
}

#[specta::specta]
#[tauri::command]
pub async fn get_schedule(
    state: State<'_, AppState>,
    office_id: String,
    date: String,
) -> Result<Vec<AppointmentDto>, String> {
    do_rebuild(&state)?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    let rows = proj.list_appointments_for_office_on_date(&office_id, &date)
        .map_err(|e| e.to_string())?;
    Ok(rows.iter().map(row_to_dto).collect())
}

#[specta::specta]
#[tauri::command]
pub async fn get_appointment(
    state: State<'_, AppState>,
    appointment_id: String,
) -> Result<Option<AppointmentWithNotesDto>, String> {
    do_rebuild(&state)?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    let row = proj.get_appointment(&appointment_id).map_err(|e| e.to_string())?;
    match row {
        None => Ok(None),
        Some(r) => {
            let notes = proj.list_appointment_notes(&appointment_id)
                .map_err(|e| e.to_string())?;
            Ok(Some(AppointmentWithNotesDto {
                appointment: row_to_dto(&r),
                notes: notes.iter().map(note_row_to_dto).collect(),
            }))
        }
    }
}

#[specta::specta]
#[tauri::command]
pub async fn get_provider_schedule(
    state: State<'_, AppState>,
    provider_id: String,
    start_date: String,
    end_date: String,
) -> Result<Vec<AppointmentDto>, String> {
    let start_dt = format!("{}T00:00:00", start_date);
    let end_dt   = format!("{}T23:59:59", end_date);
    do_rebuild(&state)?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    let rows = proj.list_appointments_for_provider_in_range(&provider_id, &start_dt, &end_dt)
        .map_err(|e| e.to_string())?;
    Ok(rows.iter().map(row_to_dto).collect())
}

#[specta::specta]
#[tauri::command]
pub async fn get_tomorrows_call_list(
    state: State<'_, AppState>,
    office_id: String,
    date: String,
) -> Result<Vec<CallListEntryDto>, String> {
    do_rebuild(&state)?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    let rows = proj.get_call_list(&office_id, &date).map_err(|e| e.to_string())?;
    let entries = rows.iter().map(|r| CallListEntryDto {
        appointment_id: r.appointment_id.clone(),
        office_id: r.office_id.clone(),
        patient_name: r.patient_name.clone(),
        patient_phone: r.patient_phone.clone(),
        patient_email: r.patient_email.clone(),
        preferred_contact_channel: r.preferred_contact_channel.clone(),
        procedure_name: r.procedure_name.clone(),
        provider_name: r.provider_name.clone(),
        start_time: r.start_time.clone(),
    }).collect();
    Ok(entries)
}
