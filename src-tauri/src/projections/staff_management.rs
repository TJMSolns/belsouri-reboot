use crate::db::{EventStore, ProjectionStore, StaffMemberRow};
use crate::events::staff_management::*;
use serde::de::DeserializeOwned;

const PROJECTION_NAME: &str = "staff_management";

/// Incremental rebuild: reads all new staff_management events and applies them.
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
        STAFF_MEMBER_REGISTERED => {
            let p: StaffMemberRegisteredPayload =
                parse(&event.payload, STAFF_MEMBER_REGISTERED)?;
            proj.upsert_staff_member(&StaffMemberRow {
                staff_member_id: p.staff_member_id,
                name: p.name,
                phone: p.phone,
                email: p.email,
                preferred_contact_channel: p.preferred_contact_channel,
                pin_hash: None,
                archived: false,
            }).map_err(|e| e.to_string())?;
        }
        PRACTICE_MANAGER_CLAIMED => {
            // The staff member row was created by the preceding StaffMemberRegistered event.
            // PracticeManagerClaimed signals that this was a bootstrap — no additional
            // projection change needed (role is applied by the following RoleAssigned event).
        }
        ROLE_ASSIGNED => {
            let p: RoleAssignedPayload = parse(&event.payload, ROLE_ASSIGNED)?;
            proj.add_staff_role(&p.staff_member_id, &p.role)
                .map_err(|e| e.to_string())?;
        }
        ROLE_REMOVED => {
            let p: RoleRemovedPayload = parse(&event.payload, ROLE_REMOVED)?;
            proj.remove_staff_role(&p.staff_member_id, &p.role)
                .map_err(|e| e.to_string())?;
        }
        PIN_SET | PIN_CHANGED => {
            let p: PINSetPayload = parse(&event.payload, PIN_SET)?;
            proj.set_staff_member_pin(&p.staff_member_id, Some(&p.pin_hash))
                .map_err(|e| e.to_string())?;
        }
        PIN_RESET => {
            let p: PINResetPayload = parse(&event.payload, PIN_RESET)?;
            proj.set_staff_member_pin(&p.staff_member_id, None)
                .map_err(|e| e.to_string())?;
        }
        STAFF_MEMBER_ARCHIVED => {
            let p: StaffMemberArchivedPayload = parse(&event.payload, STAFF_MEMBER_ARCHIVED)?;
            proj.set_staff_member_archived(&p.staff_member_id, true)
                .map_err(|e| e.to_string())?;
        }
        STAFF_MEMBER_UNARCHIVED => {
            let p: StaffMemberUnarchivedPayload = parse(&event.payload, STAFF_MEMBER_UNARCHIVED)?;
            proj.set_staff_member_archived(&p.staff_member_id, false)
                .map_err(|e| e.to_string())?;
        }
        _ => {}
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
        (EventStore::new_in_memory().unwrap(), ProjectionStore::new_in_memory().unwrap())
    }

    fn append(events: &EventStore, stream_id: &str, event_type: &str, payload: serde_json::Value) {
        let ver = events.current_version(stream_id).unwrap();
        events.append(stream_id, ver, event_type, &payload.to_string()).unwrap();
    }

    fn staff_stream(id: &str) -> String {
        format!("staff:{}", id)
    }

    #[test]
    fn test_claim_practice_manager_creates_staff_member() {
        let (events, proj) = stores();
        let id = "sm-001";
        append(&events, &staff_stream(id), STAFF_MEMBER_REGISTERED, serde_json::json!({
            "staff_member_id": id, "name": "Dr. Spence", "phone": null,
            "email": null, "preferred_contact_channel": null
        }));
        append(&events, &staff_stream(id), PRACTICE_MANAGER_CLAIMED, serde_json::json!({
            "staff_member_id": id
        }));
        append(&events, &staff_stream(id), ROLE_ASSIGNED, serde_json::json!({
            "staff_member_id": id, "role": "PracticeManager"
        }));

        rebuild(&events, &proj).unwrap();

        let row = proj.get_staff_member(id).unwrap().unwrap();
        assert_eq!(row.name, "Dr. Spence");
        assert!(!row.archived);
        assert!(row.pin_hash.is_none());
        let roles = proj.list_staff_roles(id).unwrap();
        assert!(roles.contains(&"PracticeManager".to_string()));
    }

    #[test]
    fn test_pin_set_and_reset() {
        let (events, proj) = stores();
        let id = "sm-002";
        append(&events, &staff_stream(id), STAFF_MEMBER_REGISTERED, serde_json::json!({
            "staff_member_id": id, "name": "Maria", "phone": null,
            "email": null, "preferred_contact_channel": null
        }));
        append(&events, &staff_stream(id), ROLE_ASSIGNED, serde_json::json!({
            "staff_member_id": id, "role": "Staff"
        }));
        append(&events, &staff_stream(id), PIN_SET, serde_json::json!({
            "staff_member_id": id, "pin_hash": "$2b$10$fakehash"
        }));

        rebuild(&events, &proj).unwrap();
        assert!(proj.get_staff_member(id).unwrap().unwrap().pin_hash.is_some());

        append(&events, &staff_stream(id), PIN_RESET, serde_json::json!({
            "staff_member_id": id, "reset_by_staff_member_id": "sm-001"
        }));
        rebuild(&events, &proj).unwrap();
        assert!(proj.get_staff_member(id).unwrap().unwrap().pin_hash.is_none());
    }

    #[test]
    fn test_role_assigned_and_removed() {
        let (events, proj) = stores();
        let id = "sm-003";
        append(&events, &staff_stream(id), STAFF_MEMBER_REGISTERED, serde_json::json!({
            "staff_member_id": id, "name": "Dr. Brown", "phone": null,
            "email": null, "preferred_contact_channel": null
        }));
        append(&events, &staff_stream(id), ROLE_ASSIGNED, serde_json::json!({
            "staff_member_id": id, "role": "Provider"
        }));
        append(&events, &staff_stream(id), ROLE_ASSIGNED, serde_json::json!({
            "staff_member_id": id, "role": "PracticeManager"
        }));
        rebuild(&events, &proj).unwrap();

        let roles = proj.list_staff_roles(id).unwrap();
        assert!(roles.contains(&"Provider".to_string()));
        assert!(roles.contains(&"PracticeManager".to_string()));

        append(&events, &staff_stream(id), ROLE_REMOVED, serde_json::json!({
            "staff_member_id": id, "role": "Provider"
        }));
        rebuild(&events, &proj).unwrap();
        let roles = proj.list_staff_roles(id).unwrap();
        assert!(!roles.contains(&"Provider".to_string()));
        assert!(roles.contains(&"PracticeManager".to_string()));
    }

    #[test]
    fn test_archive_unarchive() {
        let (events, proj) = stores();
        let id = "sm-004";
        append(&events, &staff_stream(id), STAFF_MEMBER_REGISTERED, serde_json::json!({
            "staff_member_id": id, "name": "Carlos", "phone": null,
            "email": null, "preferred_contact_channel": null
        }));
        append(&events, &staff_stream(id), STAFF_MEMBER_ARCHIVED, serde_json::json!({
            "staff_member_id": id
        }));
        rebuild(&events, &proj).unwrap();
        assert!(proj.get_staff_member(id).unwrap().unwrap().archived);

        append(&events, &staff_stream(id), STAFF_MEMBER_UNARCHIVED, serde_json::json!({
            "staff_member_id": id
        }));
        rebuild(&events, &proj).unwrap();
        assert!(!proj.get_staff_member(id).unwrap().unwrap().archived);
    }

    #[test]
    fn test_count_active_practice_managers() {
        let (events, proj) = stores();
        for (id, name) in [("pm1", "Dr. A"), ("pm2", "Dr. B")] {
            append(&events, &staff_stream(id), STAFF_MEMBER_REGISTERED, serde_json::json!({
                "staff_member_id": id, "name": name, "phone": null, "email": null, "preferred_contact_channel": null
            }));
            append(&events, &staff_stream(id), ROLE_ASSIGNED, serde_json::json!({
                "staff_member_id": id, "role": "PracticeManager"
            }));
        }
        rebuild(&events, &proj).unwrap();
        assert_eq!(proj.count_active_practice_managers().unwrap(), 2);

        append(&events, &staff_stream("pm2"), STAFF_MEMBER_ARCHIVED, serde_json::json!({
            "staff_member_id": "pm2"
        }));
        rebuild(&events, &proj).unwrap();
        assert_eq!(proj.count_active_practice_managers().unwrap(), 1);
    }

    #[test]
    fn test_has_active_pm_with_pin() {
        let (events, proj) = stores();
        let id = "pm-pin";
        append(&events, &staff_stream(id), STAFF_MEMBER_REGISTERED, serde_json::json!({
            "staff_member_id": id, "name": "Dr. X", "phone": null, "email": null, "preferred_contact_channel": null
        }));
        append(&events, &staff_stream(id), ROLE_ASSIGNED, serde_json::json!({
            "staff_member_id": id, "role": "PracticeManager"
        }));
        rebuild(&events, &proj).unwrap();
        assert!(!proj.has_active_pm_with_pin().unwrap());

        append(&events, &staff_stream(id), PIN_SET, serde_json::json!({
            "staff_member_id": id, "pin_hash": "$2b$10$fakehash"
        }));
        rebuild(&events, &proj).unwrap();
        assert!(proj.has_active_pm_with_pin().unwrap());
    }

    #[test]
    fn test_rebuild_is_incremental() {
        let (events, proj) = stores();
        let id = "sm-inc";
        append(&events, &staff_stream(id), STAFF_MEMBER_REGISTERED, serde_json::json!({
            "staff_member_id": id, "name": "Inc", "phone": null, "email": null, "preferred_contact_channel": null
        }));
        rebuild(&events, &proj).unwrap();
        let pos1 = proj.get_position(PROJECTION_NAME).unwrap();
        rebuild(&events, &proj).unwrap();
        assert_eq!(proj.get_position(PROJECTION_NAME).unwrap(), pos1);
    }
}
