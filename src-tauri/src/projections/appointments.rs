use crate::db::{EventStore, ProjectionStore, AppointmentRow, AppointmentNoteRow};
use crate::events::appointments::*;
use serde::de::DeserializeOwned;

const PROJECTION_NAME: &str = "appointments";

/// Incremental rebuild: reads all new appointment events and applies them.
pub fn rebuild(events: &EventStore, proj: &ProjectionStore) -> Result<(), String> {
    let last_id = proj.get_position(PROJECTION_NAME).map_err(|e| e.to_string())?;
    let new_events = events
        .read_events_since(last_id, ALL_EVENT_TYPES)
        .map_err(|e| e.to_string())?;

    if new_events.is_empty() {
        return Ok(());
    }

    let mut max_id = last_id;
    for event in &new_events {
        apply_event(proj, event)?;
        if event.id > max_id {
            max_id = event.id;
        }
    }

    proj.set_position(PROJECTION_NAME, max_id).map_err(|e| e.to_string())?;
    Ok(())
}

fn apply_event(proj: &ProjectionStore, event: &crate::db::StoredEvent) -> Result<(), String> {
    match event.event_type.as_str() {
        APPOINTMENT_BOOKED => {
            let p: AppointmentBookedPayload = parse(&event.payload, APPOINTMENT_BOOKED)?;

            // Denormalize names from other projections
            let patient = proj.get_patient(&p.patient_id).map_err(|e| e.to_string())?
                .ok_or_else(|| format!("Patient {} not found for appointment projection", p.patient_id))?;
            let procedure = proj.get_procedure_type(&p.procedure_type_id).map_err(|e| e.to_string())?
                .ok_or_else(|| format!("ProcedureType {} not found for appointment projection", p.procedure_type_id))?;
            let provider = proj.get_staff_member(&p.staff_member_id).map_err(|e| e.to_string())?
                .ok_or_else(|| format!("Staff member {} not found for appointment projection", p.staff_member_id))?;

            proj.insert_appointment(&AppointmentRow {
                appointment_id: p.appointment_id,
                office_id: p.office_id,
                patient_id: p.patient_id,
                patient_name: patient.full_name_display,
                patient_phone: patient.phone,
                patient_email: patient.email,
                preferred_contact_channel: patient.preferred_contact_channel,
                procedure_type_id: p.procedure_type_id,
                procedure_name: procedure.name,
                procedure_category: procedure.category,
                staff_member_id: p.staff_member_id,
                provider_name: provider.name,
                start_time: p.start_time,
                end_time: p.end_time,
                duration_minutes: p.duration_minutes,
                status: "Booked".to_string(),
                rescheduled_to_id: None,
                rescheduled_from_id: p.rescheduled_from_id,
                booked_by: p.booked_by,
            }).map_err(|e| e.to_string())?;
        }
        APPOINTMENT_RESCHEDULED => {
            let p: AppointmentRescheduledPayload = parse(&event.payload, APPOINTMENT_RESCHEDULED)?;
            proj.update_appointment_status(
                &p.appointment_id,
                "Rescheduled",
                Some(&p.rescheduled_to_id),
            ).map_err(|e| e.to_string())?;
        }
        APPOINTMENT_CANCELLED => {
            let p: AppointmentCancelledPayload = parse(&event.payload, APPOINTMENT_CANCELLED)?;
            proj.update_appointment_status(&p.appointment_id, "Cancelled", None)
                .map_err(|e| e.to_string())?;
        }
        APPOINTMENT_COMPLETED => {
            let p: AppointmentCompletedPayload = parse(&event.payload, APPOINTMENT_COMPLETED)?;
            proj.update_appointment_status(&p.appointment_id, "Completed", None)
                .map_err(|e| e.to_string())?;
        }
        APPOINTMENT_MARKED_NO_SHOW => {
            let p: AppointmentMarkedNoShowPayload = parse(&event.payload, APPOINTMENT_MARKED_NO_SHOW)?;
            proj.update_appointment_status(&p.appointment_id, "NoShow", None)
                .map_err(|e| e.to_string())?;
        }
        APPOINTMENT_NOTE_ADDED => {
            let p: AppointmentNoteAddedPayload = parse(&event.payload, APPOINTMENT_NOTE_ADDED)?;
            proj.add_appointment_note(&AppointmentNoteRow {
                note_id: p.note_id,
                appointment_id: p.appointment_id,
                text: p.text,
                recorded_by: p.recorded_by,
                recorded_at: p.recorded_at,
            }).map_err(|e| e.to_string())?;
        }
        _ => {}
    }
    Ok(())
}

fn parse<T: DeserializeOwned>(payload: &str, event_type: &str) -> Result<T, String> {
    serde_json::from_str(payload)
        .map_err(|e| format!("Failed to parse {} payload: {}", event_type, e))
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{EventStore, ProjectionStore};
    // We also need patient, staff member, procedure type in projections.db for the booking projection.
    use crate::db::{PatientRow, ProcedureTypeRow, StaffMemberRow, StaffAvailabilityRow};

    fn setup() -> (EventStore, ProjectionStore) {
        let events = EventStore::new_in_memory().unwrap();
        let proj = ProjectionStore::new_in_memory().unwrap();
        (events, proj)
    }

    fn seed_lookup_data(proj: &ProjectionStore) -> (String, String, String, String) {
        let office_id = "office-1".to_string();
        let patient_id = "patient-1".to_string();
        let procedure_id = "procedure-1".to_string();
        let staff_member_id = "sm-1".to_string();

        // seed patient
        proj.upsert_patient(&PatientRow {
            patient_id: patient_id.clone(),
            first_name: "Maria".to_string(),
            last_name: "Brown".to_string(),
            full_name_display: "Brown, Maria".to_string(),
            phone: Some("+18765551234".to_string()),
            email: Some("maria@example.com".to_string()),
            preferred_contact_channel: Some("Phone".to_string()),
            preferred_office_id: None,
            date_of_birth: None,
            address_line_1: None,
            city_town: None,
            subdivision: None,
            country: None,
            registered_by: "staff-1".to_string(),
            registered_at: "2026-01-01T09:00:00".to_string(),
            archived: false,
        }).unwrap();

        // seed procedure type
        proj.upsert_procedure_type(&ProcedureTypeRow {
            id: procedure_id.clone(),
            name: "Cleaning".to_string(),
            category: "Preventive".to_string(),
            default_duration_minutes: 60,
            is_active: true,
            required_provider_type: None,
        }).unwrap();

        // seed provider as StaffMember with Provider role + office assignment
        proj.upsert_staff_member(&StaffMemberRow {
            staff_member_id: staff_member_id.clone(),
            name: "Dr. Spence".to_string(),
            phone: None,
            email: None,
            preferred_contact_channel: None,
            pin_hash: None,
            clinical_specialization: Some("Dentist".to_string()),
            archived: false,
        }).unwrap();
        proj.add_staff_role(&staff_member_id, "Provider").unwrap();
        proj.add_staff_office_assignment(&staff_member_id, &office_id).unwrap();
        proj.set_staff_availability(&StaffAvailabilityRow {
            staff_member_id: staff_member_id.clone(),
            office_id: office_id.clone(),
            day_of_week: "Monday".to_string(),
            start_time: "08:00".to_string(),
            end_time: "17:00".to_string(),
        }).unwrap();

        (office_id, patient_id, procedure_id, staff_member_id)
    }

    fn append_for(
        events: &EventStore,
        stream_id: &str,
        event_type: &str,
        payload: &impl serde::Serialize,
    ) {
        let ver = events.current_version(stream_id).unwrap();
        let json = serde_json::to_string(payload).unwrap();
        events.append(stream_id, ver, event_type, &json).unwrap();
    }

    #[test]
    fn test_appointment_booked_creates_row() {
        let (events, proj) = setup();
        let (office_id, patient_id, procedure_id, staff_member_id) = seed_lookup_data(&proj);
        let appt_id = "appt-1".to_string();
        let stream = format!("appointment:{}", appt_id);

        append_for(&events, &stream, APPOINTMENT_BOOKED, &AppointmentBookedPayload {
            appointment_id: appt_id.clone(),
            office_id: office_id.clone(),
            patient_id: patient_id.clone(),
            procedure_type_id: procedure_id.clone(),
            staff_member_id: staff_member_id.clone(),
            start_time: "2026-03-09T10:00:00".to_string(),
            end_time: "2026-03-09T11:00:00".to_string(),
            duration_minutes: 60,
            booked_by: "staff-1".to_string(),
            rescheduled_from_id: None,
        });

        rebuild(&events, &proj).unwrap();

        let row = proj.get_appointment(&appt_id).unwrap().expect("appointment should exist");
        assert_eq!(row.status, "Booked");
        assert_eq!(row.patient_name, "Brown, Maria");
        assert_eq!(row.procedure_name, "Cleaning");
        assert_eq!(row.provider_name, "Dr. Spence");
        assert_eq!(row.duration_minutes, 60);
        assert_eq!(row.patient_phone, Some("+18765551234".to_string()));
    }

    #[test]
    fn test_appointment_cancelled() {
        let (events, proj) = setup();
        let (office_id, patient_id, procedure_id, staff_member_id) = seed_lookup_data(&proj);
        let appt_id = "appt-1".to_string();
        let stream = format!("appointment:{}", appt_id);

        append_for(&events, &stream, APPOINTMENT_BOOKED, &AppointmentBookedPayload {
            appointment_id: appt_id.clone(),
            office_id, patient_id, procedure_type_id: procedure_id,
            staff_member_id, start_time: "2026-03-09T10:00:00".to_string(),
            end_time: "2026-03-09T11:00:00".to_string(), duration_minutes: 60,
            booked_by: "staff-1".to_string(), rescheduled_from_id: None,
        });
        append_for(&events, &stream, APPOINTMENT_CANCELLED, &AppointmentCancelledPayload {
            appointment_id: appt_id.clone(),
            cancelled_by: "staff-1".to_string(),
            reason: Some("Patient request".to_string()),
        });

        rebuild(&events, &proj).unwrap();

        let row = proj.get_appointment(&appt_id).unwrap().unwrap();
        assert_eq!(row.status, "Cancelled");
    }

    #[test]
    fn test_appointment_completed() {
        let (events, proj) = setup();
        let (office_id, patient_id, procedure_id, staff_member_id) = seed_lookup_data(&proj);
        let appt_id = "appt-1".to_string();
        let stream = format!("appointment:{}", appt_id);

        append_for(&events, &stream, APPOINTMENT_BOOKED, &AppointmentBookedPayload {
            appointment_id: appt_id.clone(),
            office_id, patient_id, procedure_type_id: procedure_id,
            staff_member_id, start_time: "2026-03-09T10:00:00".to_string(),
            end_time: "2026-03-09T11:00:00".to_string(), duration_minutes: 60,
            booked_by: "staff-1".to_string(), rescheduled_from_id: None,
        });
        append_for(&events, &stream, APPOINTMENT_COMPLETED, &AppointmentCompletedPayload {
            appointment_id: appt_id.clone(),
            completed_by: "staff-1".to_string(),
        });

        rebuild(&events, &proj).unwrap();

        let row = proj.get_appointment(&appt_id).unwrap().unwrap();
        assert_eq!(row.status, "Completed");
    }

    #[test]
    fn test_appointment_no_show() {
        let (events, proj) = setup();
        let (office_id, patient_id, procedure_id, staff_member_id) = seed_lookup_data(&proj);
        let appt_id = "appt-1".to_string();
        let stream = format!("appointment:{}", appt_id);

        append_for(&events, &stream, APPOINTMENT_BOOKED, &AppointmentBookedPayload {
            appointment_id: appt_id.clone(),
            office_id, patient_id, procedure_type_id: procedure_id,
            staff_member_id, start_time: "2026-03-09T10:00:00".to_string(),
            end_time: "2026-03-09T11:00:00".to_string(), duration_minutes: 60,
            booked_by: "staff-1".to_string(), rescheduled_from_id: None,
        });
        append_for(&events, &stream, APPOINTMENT_MARKED_NO_SHOW, &AppointmentMarkedNoShowPayload {
            appointment_id: appt_id.clone(),
            recorded_by: "staff-1".to_string(),
        });

        rebuild(&events, &proj).unwrap();

        let row = proj.get_appointment(&appt_id).unwrap().unwrap();
        assert_eq!(row.status, "NoShow");
    }

    #[test]
    fn test_appointment_rescheduled() {
        let (events, proj) = setup();
        let (office_id, patient_id, procedure_id, staff_member_id) = seed_lookup_data(&proj);
        let orig_id = "appt-1".to_string();
        let new_id = "appt-2".to_string();
        let orig_stream = format!("appointment:{}", orig_id);
        let new_stream = format!("appointment:{}", new_id);

        append_for(&events, &orig_stream, APPOINTMENT_BOOKED, &AppointmentBookedPayload {
            appointment_id: orig_id.clone(),
            office_id: office_id.clone(), patient_id: patient_id.clone(),
            procedure_type_id: procedure_id.clone(),
            staff_member_id: staff_member_id.clone(),
            start_time: "2026-03-09T10:00:00".to_string(),
            end_time: "2026-03-09T11:00:00".to_string(), duration_minutes: 60,
            booked_by: "staff-1".to_string(), rescheduled_from_id: None,
        });
        append_for(&events, &orig_stream, APPOINTMENT_RESCHEDULED, &AppointmentRescheduledPayload {
            appointment_id: orig_id.clone(),
            rescheduled_to_id: new_id.clone(),
            rescheduled_by: "staff-1".to_string(),
        });
        append_for(&events, &new_stream, APPOINTMENT_BOOKED, &AppointmentBookedPayload {
            appointment_id: new_id.clone(),
            office_id: office_id.clone(), patient_id: patient_id.clone(),
            procedure_type_id: procedure_id.clone(),
            staff_member_id: staff_member_id.clone(),
            start_time: "2026-03-11T14:00:00".to_string(),
            end_time: "2026-03-11T15:00:00".to_string(), duration_minutes: 60,
            booked_by: "staff-1".to_string(),
            rescheduled_from_id: Some(orig_id.clone()),
        });

        rebuild(&events, &proj).unwrap();

        let orig = proj.get_appointment(&orig_id).unwrap().unwrap();
        assert_eq!(orig.status, "Rescheduled");
        assert_eq!(orig.rescheduled_to_id, Some(new_id.clone()));

        let new_appt = proj.get_appointment(&new_id).unwrap().unwrap();
        assert_eq!(new_appt.status, "Booked");
        assert_eq!(new_appt.rescheduled_from_id, Some(orig_id.clone()));
    }

    #[test]
    fn test_appointment_note_added() {
        let (events, proj) = setup();
        let (office_id, patient_id, procedure_id, staff_member_id) = seed_lookup_data(&proj);
        let appt_id = "appt-1".to_string();
        let stream = format!("appointment:{}", appt_id);

        append_for(&events, &stream, APPOINTMENT_BOOKED, &AppointmentBookedPayload {
            appointment_id: appt_id.clone(),
            office_id, patient_id, procedure_type_id: procedure_id,
            staff_member_id, start_time: "2026-03-09T10:00:00".to_string(),
            end_time: "2026-03-09T11:00:00".to_string(), duration_minutes: 60,
            booked_by: "staff-1".to_string(), rescheduled_from_id: None,
        });
        append_for(&events, &stream, APPOINTMENT_NOTE_ADDED, &AppointmentNoteAddedPayload {
            appointment_id: appt_id.clone(),
            note_id: "note-1".to_string(),
            text: "Patient arrived late".to_string(),
            recorded_by: "staff-1".to_string(),
            recorded_at: "2026-03-09T10:05:00".to_string(),
        });

        rebuild(&events, &proj).unwrap();

        let notes = proj.list_appointment_notes(&appt_id).unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].text, "Patient arrived late");
    }

    #[test]
    fn test_overlapping_booked_count() {
        let (events, proj) = setup();
        let (office_id, patient_id, procedure_id, staff_member_id) = seed_lookup_data(&proj);

        // Book 2 appointments at 10:00-11:00
        for i in 1..=2u32 {
            let appt_id = format!("appt-{}", i);
            let stream = format!("appointment:{}", appt_id);
            append_for(&events, &stream, APPOINTMENT_BOOKED, &AppointmentBookedPayload {
                appointment_id: appt_id.clone(),
                office_id: office_id.clone(),
                patient_id: patient_id.clone(),
                procedure_type_id: procedure_id.clone(),
                staff_member_id: staff_member_id.clone(),
                start_time: "2026-03-09T10:00:00".to_string(),
                end_time: "2026-03-09T11:00:00".to_string(),
                duration_minutes: 60,
                booked_by: "staff-1".to_string(),
                rescheduled_from_id: None,
            });
        }

        rebuild(&events, &proj).unwrap();

        // 2 appointments overlap with 10:00-11:00
        let count = proj.count_overlapping_booked(&office_id, "2026-03-09T10:00:00", "2026-03-09T11:00:00", None).unwrap();
        assert_eq!(count, 2);

        // Adjacent appointment at 11:00 does NOT overlap
        let count = proj.count_overlapping_booked(&office_id, "2026-03-09T11:00:00", "2026-03-09T12:00:00", None).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_rebuild_is_incremental() {
        let (events, proj) = setup();
        let (office_id, patient_id, procedure_id, staff_member_id) = seed_lookup_data(&proj);
        let appt_id = "appt-1".to_string();
        let stream = format!("appointment:{}", appt_id);

        append_for(&events, &stream, APPOINTMENT_BOOKED, &AppointmentBookedPayload {
            appointment_id: appt_id.clone(),
            office_id, patient_id, procedure_type_id: procedure_id,
            staff_member_id, start_time: "2026-03-09T10:00:00".to_string(),
            end_time: "2026-03-09T11:00:00".to_string(), duration_minutes: 60,
            booked_by: "staff-1".to_string(), rescheduled_from_id: None,
        });

        rebuild(&events, &proj).unwrap();
        let pos1 = proj.get_position(PROJECTION_NAME).unwrap();
        assert!(pos1 > 0);

        // Second rebuild should not move position if no new events
        rebuild(&events, &proj).unwrap();
        let pos2 = proj.get_position(PROJECTION_NAME).unwrap();
        assert_eq!(pos1, pos2);
    }
}
