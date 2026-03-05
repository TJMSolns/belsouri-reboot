use crate::db::{EventStore, ProjectionStore, StaffShiftRow};
use crate::events::staff_shift::*;
use serde::de::DeserializeOwned;

const PROJECTION_NAME: &str = "staff_shift";

/// Incremental rebuild: reads new staff_shift events and applies them.
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
        STAFF_SHIFT_PLANNED => {
            let p: StaffShiftPlannedPayload = parse(&event.payload, STAFF_SHIFT_PLANNED)?;

            // Denormalize staff_name from staff_members table
            let staff_row = proj.get_staff_member(&p.staff_member_id)
                .map_err(|e| e.to_string())?
                .ok_or_else(|| format!("StaffMember {} not found for staff_shift projection", p.staff_member_id))?;

            // Denormalize office_name from offices table
            let office_row = proj.get_office(&p.office_id)
                .map_err(|e| e.to_string())?
                .ok_or_else(|| format!("Office {} not found for staff_shift projection", p.office_id))?;

            proj.insert_staff_shift(&StaffShiftRow {
                shift_id: p.shift_id,
                staff_member_id: p.staff_member_id,
                staff_name: staff_row.name,
                office_id: p.office_id,
                office_name: office_row.name,
                date: p.date,
                start_time: p.start_time,
                end_time: p.end_time,
                role: p.role,
                created_by: p.created_by,
                cancelled: false,
                cancel_reason: None,
            }).map_err(|e| e.to_string())?;
        }
        STAFF_SHIFT_CANCELLED => {
            let p: StaffShiftCancelledPayload = parse(&event.payload, STAFF_SHIFT_CANCELLED)?;
            proj.cancel_staff_shift(&p.shift_id, p.cancel_reason.as_deref())
                .map_err(|e| e.to_string())?;
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
    use crate::db::{StaffMemberRow, OfficeRow};

    fn setup() -> (EventStore, ProjectionStore) {
        let events = EventStore::new_in_memory().unwrap();
        let proj = ProjectionStore::new_in_memory().unwrap();
        (events, proj)
    }

    fn seed_staff_and_office(proj: &ProjectionStore) -> (String, String) {
        let staff_id = "staff-1".to_string();
        let office_id = "office-1".to_string();

        proj.upsert_staff_member(&StaffMemberRow {
            staff_member_id: staff_id.clone(),
            name: "Jane Doe".to_string(),
            phone: None,
            email: None,
            preferred_contact_channel: None,
            pin_hash: None,
            archived: false,
        }).unwrap();

        proj.upsert_office(&OfficeRow {
            id: office_id.clone(),
            name: "Main Office".to_string(),
            chair_count: 3,
            archived: false,
            address_line_1: None,
            address_line_2: None,
            city_town: None,
            subdivision: None,
            country: None,
        }).unwrap();

        (staff_id, office_id)
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
    fn test_shift_planned_creates_row() {
        let (events, proj) = setup();
        let (staff_id, office_id) = seed_staff_and_office(&proj);
        let shift_id = "shift-1".to_string();
        let stream = format!("staff_shift:{}", shift_id);

        append_for(&events, &stream, STAFF_SHIFT_PLANNED, &StaffShiftPlannedPayload {
            shift_id: shift_id.clone(),
            staff_member_id: staff_id.clone(),
            office_id: office_id.clone(),
            date: "2026-03-09".to_string(),
            start_time: "09:00".to_string(),
            end_time: "17:00".to_string(),
            role: "Staff".to_string(),
            created_by: staff_id.clone(),
        });

        rebuild(&events, &proj).unwrap();

        let row = proj.get_shift(&shift_id).unwrap().expect("shift should exist");
        assert_eq!(row.staff_name, "Jane Doe");
        assert_eq!(row.office_name, "Main Office");
        assert_eq!(row.date, "2026-03-09");
        assert_eq!(row.start_time, "09:00");
        assert_eq!(row.end_time, "17:00");
        assert!(!row.cancelled);
        assert!(row.cancel_reason.is_none());
    }

    #[test]
    fn test_shift_cancelled_updates_row() {
        let (events, proj) = setup();
        let (staff_id, office_id) = seed_staff_and_office(&proj);
        let shift_id = "shift-1".to_string();
        let stream = format!("staff_shift:{}", shift_id);

        append_for(&events, &stream, STAFF_SHIFT_PLANNED, &StaffShiftPlannedPayload {
            shift_id: shift_id.clone(),
            staff_member_id: staff_id.clone(),
            office_id: office_id.clone(),
            date: "2026-03-09".to_string(),
            start_time: "09:00".to_string(),
            end_time: "17:00".to_string(),
            role: "Staff".to_string(),
            created_by: staff_id.clone(),
        });
        append_for(&events, &stream, STAFF_SHIFT_CANCELLED, &StaffShiftCancelledPayload {
            shift_id: shift_id.clone(),
            cancel_reason: Some("Sick day".to_string()),
            cancelled_by: staff_id.clone(),
        });

        rebuild(&events, &proj).unwrap();

        let row = proj.get_shift(&shift_id).unwrap().unwrap();
        assert!(row.cancelled);
        assert_eq!(row.cancel_reason, Some("Sick day".to_string()));
    }

    #[test]
    fn test_get_shifts_for_week() {
        let (events, proj) = setup();
        let (staff_id, office_id) = seed_staff_and_office(&proj);

        for (shift_id, date) in [("s1", "2026-03-09"), ("s2", "2026-03-10"), ("s3", "2026-03-16")] {
            let stream = format!("staff_shift:{}", shift_id);
            append_for(&events, &stream, STAFF_SHIFT_PLANNED, &StaffShiftPlannedPayload {
                shift_id: shift_id.to_string(),
                staff_member_id: staff_id.clone(),
                office_id: office_id.clone(),
                date: date.to_string(),
                start_time: "09:00".to_string(),
                end_time: "17:00".to_string(),
                role: "Staff".to_string(),
                created_by: staff_id.clone(),
            });
        }

        rebuild(&events, &proj).unwrap();

        let week_shifts = proj.get_shifts_for_week("2026-03-09", "2026-03-15", None).unwrap();
        assert_eq!(week_shifts.len(), 2);
        assert!(week_shifts.iter().all(|s| s.date.as_str() >= "2026-03-09" && s.date.as_str() <= "2026-03-15"));
    }

    #[test]
    fn test_rebuild_is_incremental() {
        let (events, proj) = setup();
        let (staff_id, office_id) = seed_staff_and_office(&proj);
        let shift_id = "shift-1".to_string();
        let stream = format!("staff_shift:{}", shift_id);

        append_for(&events, &stream, STAFF_SHIFT_PLANNED, &StaffShiftPlannedPayload {
            shift_id: shift_id.clone(),
            staff_member_id: staff_id.clone(),
            office_id: office_id.clone(),
            date: "2026-03-09".to_string(),
            start_time: "09:00".to_string(),
            end_time: "17:00".to_string(),
            role: "Staff".to_string(),
            created_by: staff_id.clone(),
        });

        rebuild(&events, &proj).unwrap();
        let pos1 = proj.get_position(PROJECTION_NAME).unwrap();
        assert!(pos1 > 0);

        rebuild(&events, &proj).unwrap();
        let pos2 = proj.get_position(PROJECTION_NAME).unwrap();
        assert_eq!(pos1, pos2);
    }
}
