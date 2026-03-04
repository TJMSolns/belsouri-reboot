use crate::db::{EventStore, ProjectionStore, PatientRow, PatientNoteRow};
use crate::events::patient_management::*;
use serde::de::DeserializeOwned;

const PROJECTION_NAME: &str = "patient_management";

/// Incremental rebuild: reads all new patient_management events and applies them.
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
        PATIENT_REGISTERED => {
            let p: PatientRegisteredPayload = parse(&event.payload, PATIENT_REGISTERED)?;
            let full_name_display = format!("{}, {}", p.last_name, p.first_name);
            proj.upsert_patient(&PatientRow {
                patient_id: p.patient_id,
                first_name: p.first_name,
                last_name: p.last_name,
                full_name_display,
                phone: p.phone,
                email: p.email,
                preferred_contact_channel: p.preferred_contact_channel,
                preferred_office_id: p.preferred_office_id,
                date_of_birth: p.date_of_birth,
                address_line_1: None,
                city_town: None,
                subdivision: None,
                country: None,
                registered_by: p.registered_by.clone(),
                registered_at: event.created_at.clone(),
                archived: false,
            }).map_err(|e| e.to_string())?;
        }
        PATIENT_DEMOGRAPHICS_UPDATED => {
            let p: PatientDemographicsUpdatedPayload =
                parse(&event.payload, PATIENT_DEMOGRAPHICS_UPDATED)?;
            let full_name_display = format!("{}, {}", p.last_name, p.first_name);
            proj.update_patient_demographics(
                &p.patient_id,
                &p.first_name,
                &p.last_name,
                &full_name_display,
                p.date_of_birth.as_deref(),
                p.address_line_1.as_deref(),
                p.city_town.as_deref(),
                p.subdivision.as_deref(),
                p.country.as_deref(),
            ).map_err(|e| e.to_string())?;
        }
        PATIENT_CONTACT_INFO_UPDATED => {
            let p: PatientContactInfoUpdatedPayload =
                parse(&event.payload, PATIENT_CONTACT_INFO_UPDATED)?;
            proj.update_patient_contact_info(
                &p.patient_id,
                p.phone.as_deref(),
                p.email.as_deref(),
                p.preferred_contact_channel.as_deref(),
            ).map_err(|e| e.to_string())?;
        }
        PATIENT_NOTE_ADDED => {
            let p: PatientNoteAddedPayload = parse(&event.payload, PATIENT_NOTE_ADDED)?;
            proj.add_patient_note(&PatientNoteRow {
                note_id: p.note_id,
                patient_id: p.patient_id,
                text: p.text,
                recorded_by: p.recorded_by,
                recorded_at: p.recorded_at,
            }).map_err(|e| e.to_string())?;
        }
        PATIENT_ARCHIVED => {
            let p: PatientArchivedPayload = parse(&event.payload, PATIENT_ARCHIVED)?;
            proj.set_patient_archived(&p.patient_id, true).map_err(|e| e.to_string())?;
        }
        PATIENT_UNARCHIVED => {
            let p: PatientUnarchivedPayload = parse(&event.payload, PATIENT_UNARCHIVED)?;
            proj.set_patient_archived(&p.patient_id, false).map_err(|e| e.to_string())?;
        }
        _ => {} // ignore unknown event types
    }
    Ok(())
}

fn parse<T: DeserializeOwned>(json: &str, event_type: &str) -> Result<T, String> {
    serde_json::from_str(json)
        .map_err(|e| format!("Failed to parse {}: {}", event_type, e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{EventStore, ProjectionStore};

    fn stores() -> (EventStore, ProjectionStore) {
        let events = EventStore::new_in_memory().unwrap();
        let proj = ProjectionStore::new_in_memory().unwrap();
        (events, proj)
    }

    fn append_for(events: &EventStore, patient_id: &str, event_type: &str, payload: serde_json::Value) {
        let stream_id = format!("patient:{}", patient_id);
        let ver = events.current_version(&stream_id).unwrap();
        events.append(&stream_id, ver, event_type, &payload.to_string()).unwrap();
    }

    #[test]
    fn test_patient_registered_creates_row() {
        let (events, proj) = stores();
        let patient_id = "pid-001";
        append_for(&events, patient_id, PATIENT_REGISTERED, serde_json::json!({
            "patient_id": patient_id,
            "first_name": "Maria",
            "last_name": "Brown",
            "phone": "+18765551234",
            "email": null,
            "preferred_contact_channel": null,
            "preferred_office_id": null,
            "date_of_birth": null,
            "registered_by": "staff-001"
        }));

        rebuild(&events, &proj).unwrap();

        let row = proj.get_patient(patient_id).unwrap().unwrap();
        assert_eq!(row.first_name, "Maria");
        assert_eq!(row.last_name, "Brown");
        assert_eq!(row.full_name_display, "Brown, Maria");
        assert_eq!(row.phone, Some("+18765551234".to_string()));
        assert!(!row.archived);
    }

    #[test]
    fn test_demographics_update() {
        let (events, proj) = stores();
        let id = "pid-002";
        append_for(&events, id, PATIENT_REGISTERED, serde_json::json!({
            "patient_id": id, "first_name": "John", "last_name": "Smith",
            "phone": null, "email": "j@x.com", "preferred_contact_channel": null,
            "preferred_office_id": null, "date_of_birth": null, "registered_by": "s1"
        }));
        append_for(&events, id, PATIENT_DEMOGRAPHICS_UPDATED, serde_json::json!({
            "patient_id": id, "first_name": "Jonathan", "last_name": "Smith",
            "date_of_birth": "1985-06-15", "address_line_1": "10 Duke St",
            "city_town": "Kingston", "subdivision": "Kingston", "country": "Jamaica",
            "updated_by": "s1"
        }));

        rebuild(&events, &proj).unwrap();
        let row = proj.get_patient(id).unwrap().unwrap();
        assert_eq!(row.first_name, "Jonathan");
        assert_eq!(row.full_name_display, "Smith, Jonathan");
        assert_eq!(row.date_of_birth, Some("1985-06-15".to_string()));
        assert_eq!(row.city_town, Some("Kingston".to_string()));
    }

    #[test]
    fn test_contact_info_update() {
        let (events, proj) = stores();
        let id = "pid-003";
        append_for(&events, id, PATIENT_REGISTERED, serde_json::json!({
            "patient_id": id, "first_name": "Ann", "last_name": "Lee",
            "phone": "8765550001", "email": null, "preferred_contact_channel": null,
            "preferred_office_id": null, "date_of_birth": null, "registered_by": "s1"
        }));
        append_for(&events, id, PATIENT_CONTACT_INFO_UPDATED, serde_json::json!({
            "patient_id": id, "phone": "8765559999", "email": "ann@x.com",
            "preferred_contact_channel": "Email", "updated_by": "s1"
        }));

        rebuild(&events, &proj).unwrap();
        let row = proj.get_patient(id).unwrap().unwrap();
        assert_eq!(row.phone, Some("8765559999".to_string()));
        assert_eq!(row.email, Some("ann@x.com".to_string()));
        assert_eq!(row.preferred_contact_channel, Some("Email".to_string()));
    }

    #[test]
    fn test_archive_unarchive() {
        let (events, proj) = stores();
        let id = "pid-004";
        append_for(&events, id, PATIENT_REGISTERED, serde_json::json!({
            "patient_id": id, "first_name": "Tom", "last_name": "Reid",
            "phone": "8005550001", "email": null, "preferred_contact_channel": null,
            "preferred_office_id": null, "date_of_birth": null, "registered_by": "s1"
        }));
        append_for(&events, id, PATIENT_ARCHIVED, serde_json::json!({
            "patient_id": id, "archived_by": "s1"
        }));
        rebuild(&events, &proj).unwrap();
        assert!(proj.get_patient(id).unwrap().unwrap().archived);

        append_for(&events, id, PATIENT_UNARCHIVED, serde_json::json!({
            "patient_id": id, "unarchived_by": "s1"
        }));
        rebuild(&events, &proj).unwrap();
        assert!(!proj.get_patient(id).unwrap().unwrap().archived);
    }

    #[test]
    fn test_note_added() {
        let (events, proj) = stores();
        let id = "pid-005";
        append_for(&events, id, PATIENT_REGISTERED, serde_json::json!({
            "patient_id": id, "first_name": "Sue", "last_name": "Park",
            "phone": null, "email": "sue@x.com", "preferred_contact_channel": null,
            "preferred_office_id": null, "date_of_birth": null, "registered_by": "s1"
        }));
        append_for(&events, id, PATIENT_NOTE_ADDED, serde_json::json!({
            "patient_id": id, "note_id": "note-001", "text": "Allergic to penicillin",
            "recorded_by": "s1", "recorded_at": "2026-03-04T10:00:00Z"
        }));

        rebuild(&events, &proj).unwrap();
        let notes = proj.list_patient_notes(id).unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].text, "Allergic to penicillin");
    }

    #[test]
    fn test_search_by_name_prefix() {
        let (events, proj) = stores();
        for (pid, first, last, phone) in [
            ("p1", "Maria", "Brown", "+18761111111"),
            ("p2", "Michael", "Brown", "+18762222222"),
            ("p3", "Anna", "Smith", "+18763333333"),
        ] {
            append_for(&events, pid, PATIENT_REGISTERED, serde_json::json!({
                "patient_id": pid, "first_name": first, "last_name": last,
                "phone": phone, "email": null, "preferred_contact_channel": null,
                "preferred_office_id": null, "date_of_birth": null, "registered_by": "s1"
            }));
        }
        rebuild(&events, &proj).unwrap();

        // Search by last name prefix
        let results = proj.search_patients(Some("bro"), None, None, false).unwrap();
        assert_eq!(results.len(), 2);

        // Search by first name prefix
        let results = proj.search_patients(Some("mar"), None, None, false).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].first_name, "Maria");
    }

    #[test]
    fn test_rebuild_is_incremental() {
        let (events, proj) = stores();
        let id = "pid-inc";
        append_for(&events, id, PATIENT_REGISTERED, serde_json::json!({
            "patient_id": id, "first_name": "First", "last_name": "Last",
            "phone": "0001", "email": null, "preferred_contact_channel": null,
            "preferred_office_id": null, "date_of_birth": null, "registered_by": "s1"
        }));
        rebuild(&events, &proj).unwrap();
        let pos1 = proj.get_position(PROJECTION_NAME).unwrap();

        // Second rebuild with no new events: position unchanged
        rebuild(&events, &proj).unwrap();
        assert_eq!(proj.get_position(PROJECTION_NAME).unwrap(), pos1);
    }
}
