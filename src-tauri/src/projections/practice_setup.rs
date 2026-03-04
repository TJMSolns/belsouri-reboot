use crate::db::{
    EventStore, ProjectionStore,
    PracticeSettingsRow, OfficeRow, ProviderRow, ProviderAvailabilityRow,
    ProviderExceptionRow, ProcedureTypeRow,
};
use crate::events::practice_setup::*;

const PROJECTION_NAME: &str = "practice_setup";

/// Incremental rebuild: reads all new practice_setup events and applies them.
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
        PRACTICE_DETAILS_UPDATED => {
            let p: PracticeDetailsUpdatedPayload = parse(&event.payload, PRACTICE_DETAILS_UPDATED)?;
            proj.upsert_practice_settings(&PracticeSettingsRow {
                name: p.name,
                phone: p.phone,
                email: p.email,
                website: p.website,
                address_line_1: p.address_line_1,
                address_line_2: p.address_line_2,
                city_town: p.city_town,
                subdivision: p.subdivision,
                country: p.country,
            }).map_err(|e| e.to_string())?;
        }
        OFFICE_CREATED => {
            let p: OfficeCreatedPayload = parse(&event.payload, OFFICE_CREATED)?;
            proj.upsert_office(&OfficeRow {
                id: p.id,
                name: p.name,
                chair_count: p.chair_count,
                archived: false,
            }).map_err(|e| e.to_string())?;
        }
        OFFICE_RENAMED => {
            let p: OfficeRenamedPayload = parse(&event.payload, OFFICE_RENAMED)?;
            proj.rename_office(&p.id, &p.new_name).map_err(|e| e.to_string())?;
        }
        OFFICE_CHAIR_COUNT_UPDATED => {
            let p: OfficeChairCountUpdatedPayload = parse(&event.payload, OFFICE_CHAIR_COUNT_UPDATED)?;
            proj.update_office_chair_count(&p.id, p.new_chair_count).map_err(|e| e.to_string())?;
        }
        OFFICE_HOURS_SET => {
            let p: OfficeHoursSetPayload = parse(&event.payload, OFFICE_HOURS_SET)?;
            proj.set_office_hours(&p.id, &p.day_of_week, &p.open_time, &p.close_time)
                .map_err(|e| e.to_string())?;
        }
        OFFICE_DAY_CLOSED => {
            let p: OfficeDayClosedPayload = parse(&event.payload, OFFICE_DAY_CLOSED)?;
            proj.delete_office_hours(&p.id, &p.day_of_week).map_err(|e| e.to_string())?;
        }
        OFFICE_ARCHIVED => {
            let p: OfficeArchivedPayload = parse(&event.payload, OFFICE_ARCHIVED)?;
            proj.archive_office(&p.id).map_err(|e| e.to_string())?;
        }
        PROVIDER_REGISTERED => {
            let p: ProviderRegisteredPayload = parse(&event.payload, PROVIDER_REGISTERED)?;
            proj.upsert_provider(&ProviderRow {
                id: p.id,
                name: p.name,
                provider_type: p.provider_type,
                archived: false,
            }).map_err(|e| e.to_string())?;
        }
        PROVIDER_RENAMED => {
            let p: ProviderRenamedPayload = parse(&event.payload, PROVIDER_RENAMED)?;
            proj.rename_provider(&p.id, &p.new_name).map_err(|e| e.to_string())?;
        }
        PROVIDER_TYPE_CHANGED => {
            let p: ProviderTypeChangedPayload = parse(&event.payload, PROVIDER_TYPE_CHANGED)?;
            proj.update_provider_type(&p.id, &p.new_provider_type).map_err(|e| e.to_string())?;
        }
        PROVIDER_ASSIGNED_TO_OFFICE => {
            let p: ProviderAssignedToOfficePayload = parse(&event.payload, PROVIDER_ASSIGNED_TO_OFFICE)?;
            proj.add_provider_office_assignment(&p.id, &p.office_id).map_err(|e| e.to_string())?;
        }
        PROVIDER_REMOVED_FROM_OFFICE => {
            let p: ProviderRemovedFromOfficePayload = parse(&event.payload, PROVIDER_REMOVED_FROM_OFFICE)?;
            proj.remove_provider_office_assignment(&p.id, &p.office_id).map_err(|e| e.to_string())?;
        }
        PROVIDER_AVAILABILITY_SET => {
            let p: ProviderAvailabilitySetPayload = parse(&event.payload, PROVIDER_AVAILABILITY_SET)?;
            proj.set_provider_availability(&ProviderAvailabilityRow {
                provider_id: p.id,
                office_id: p.office_id,
                day_of_week: p.day_of_week,
                start_time: p.start_time,
                end_time: p.end_time,
            }).map_err(|e| e.to_string())?;
        }
        PROVIDER_AVAILABILITY_CLEARED => {
            let p: ProviderAvailabilityClearedPayload = parse(&event.payload, PROVIDER_AVAILABILITY_CLEARED)?;
            proj.delete_provider_availability(&p.id, &p.office_id, &p.day_of_week)
                .map_err(|e| e.to_string())?;
        }
        PROVIDER_EXCEPTION_SET => {
            let p: ProviderExceptionSetPayload = parse(&event.payload, PROVIDER_EXCEPTION_SET)?;
            proj.add_provider_exception(&ProviderExceptionRow {
                provider_id: p.id,
                start_date: p.start_date,
                end_date: p.end_date,
                reason: p.reason,
            }).map_err(|e| e.to_string())?;
        }
        PROVIDER_EXCEPTION_REMOVED => {
            let p: ProviderExceptionRemovedPayload = parse(&event.payload, PROVIDER_EXCEPTION_REMOVED)?;
            proj.remove_provider_exception(&p.id, &p.start_date, &p.end_date)
                .map_err(|e| e.to_string())?;
        }
        PROVIDER_ARCHIVED => {
            let p: ProviderArchivedPayload = parse(&event.payload, PROVIDER_ARCHIVED)?;
            proj.archive_provider(&p.id).map_err(|e| e.to_string())?;
        }
        PROVIDER_UNARCHIVED => {
            let p: ProviderUnarchivedPayload = parse(&event.payload, PROVIDER_UNARCHIVED)?;
            proj.unarchive_provider(&p.id).map_err(|e| e.to_string())?;
        }
        PROCEDURE_TYPE_DEFINED => {
            let p: ProcedureTypeDefinedPayload = parse(&event.payload, PROCEDURE_TYPE_DEFINED)?;
            proj.upsert_procedure_type(&ProcedureTypeRow {
                id: p.id,
                name: p.name,
                category: p.category,
                default_duration_minutes: p.default_duration_minutes,
                is_active: true,
            }).map_err(|e| e.to_string())?;
        }
        PROCEDURE_TYPE_UPDATED => {
            let p: ProcedureTypeUpdatedPayload = parse(&event.payload, PROCEDURE_TYPE_UPDATED)?;
            proj.apply_procedure_type_update(
                &p.id,
                p.name.as_deref(),
                p.category.as_deref(),
                p.default_duration_minutes,
            ).map_err(|e| e.to_string())?;
        }
        PROCEDURE_TYPE_DEACTIVATED => {
            let p: ProcedureTypeDeactivatedPayload = parse(&event.payload, PROCEDURE_TYPE_DEACTIVATED)?;
            proj.set_procedure_type_active(&p.id, false).map_err(|e| e.to_string())?;
        }
        PROCEDURE_TYPE_REACTIVATED => {
            let p: ProcedureTypeReactivatedPayload = parse(&event.payload, PROCEDURE_TYPE_REACTIVATED)?;
            proj.set_procedure_type_active(&p.id, true).map_err(|e| e.to_string())?;
        }
        _ => {} // unknown event types are ignored (forward compatibility)
    }
    Ok(())
}

fn parse<'a, T: serde::de::DeserializeOwned>(json: &'a str, event_type: &str) -> Result<T, String> {
    serde_json::from_str(json)
        .map_err(|e| format!("Failed to parse {event_type}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{EventStore, ProjectionStore};
    use serde_json::json;

    fn stores() -> (EventStore, ProjectionStore) {
        (EventStore::new_in_memory().unwrap(), ProjectionStore::new_in_memory().unwrap())
    }

    fn append(es: &EventStore, stream_id: &str, event_type: &str, payload: serde_json::Value) {
        let ver = es.current_version(stream_id).unwrap();
        es.append(stream_id, ver, event_type, &payload.to_string()).unwrap();
    }

    #[test]
    fn test_rebuild_idempotent_no_events() {
        let (es, ps) = stores();
        rebuild(&es, &ps).unwrap();
        rebuild(&es, &ps).unwrap(); // second call is a no-op
        assert_eq!(ps.get_position(PROJECTION_NAME).unwrap(), 0);
    }

    #[test]
    fn test_office_created_and_renamed() {
        let (es, ps) = stores();
        let office_id = "aaa-111";
        append(&es, &format!("office:{office_id}"), OFFICE_CREATED,
            json!({"id": office_id, "name": "Kingston", "chair_count": 4}));
        rebuild(&es, &ps).unwrap();

        let office = ps.get_office(office_id).unwrap().unwrap();
        assert_eq!(office.name, "Kingston");
        assert_eq!(office.chair_count, 4);
        assert!(!office.archived);

        append(&es, &format!("office:{office_id}"), OFFICE_RENAMED,
            json!({"id": office_id, "new_name": "Kingston Main"}));
        rebuild(&es, &ps).unwrap();

        let office = ps.get_office(office_id).unwrap().unwrap();
        assert_eq!(office.name, "Kingston Main");
    }

    #[test]
    fn test_provider_registered_and_assigned() {
        let (es, ps) = stores();
        let pid = "prov-001";
        let oid = "off-001";
        append(&es, &format!("provider:{pid}"), PROVIDER_REGISTERED,
            json!({"id": pid, "name": "Dr. Brown", "provider_type": "Dentist"}));
        append(&es, &format!("provider:{pid}"), PROVIDER_ASSIGNED_TO_OFFICE,
            json!({"id": pid, "office_id": oid}));
        rebuild(&es, &ps).unwrap();

        let provider = ps.get_provider(pid).unwrap().unwrap();
        assert_eq!(provider.name, "Dr. Brown");
        assert_eq!(provider.provider_type, "Dentist");

        let offices = ps.list_provider_offices(pid).unwrap();
        assert_eq!(offices, vec![oid.to_string()]);
    }

    #[test]
    fn test_procedure_type_lifecycle() {
        let (es, ps) = stores();
        let id = "proc-001";
        append(&es, &format!("procedure_type:{id}"), PROCEDURE_TYPE_DEFINED,
            json!({"id": id, "name": "Cleaning", "category": "Preventive", "default_duration_minutes": 30}));
        rebuild(&es, &ps).unwrap();

        let pt = ps.get_procedure_type(id).unwrap().unwrap();
        assert_eq!(pt.name, "Cleaning");
        assert!(pt.is_active);

        append(&es, &format!("procedure_type:{id}"), PROCEDURE_TYPE_DEACTIVATED,
            json!({"id": id}));
        rebuild(&es, &ps).unwrap();

        let pt = ps.get_procedure_type(id).unwrap().unwrap();
        assert!(!pt.is_active);
    }

    #[test]
    fn test_practice_details_updated() {
        let (es, ps) = stores();
        append(&es, "practice", PRACTICE_DETAILS_UPDATED,
            json!({"name": "Smile Dental", "phone": "+1-876-555-0100",
                   "email": null, "website": null, "address_line_1": null,
                   "address_line_2": null, "city_town": "Kingston",
                   "subdivision": "Kingston", "country": "Jamaica"}));
        rebuild(&es, &ps).unwrap();

        let practice = ps.get_practice_settings().unwrap().unwrap();
        assert_eq!(practice.name, "Smile Dental");
        assert_eq!(practice.phone, Some("+1-876-555-0100".to_string()));
        assert_eq!(practice.city_town, Some("Kingston".to_string()));
    }

    #[test]
    fn test_office_hours_set_and_closed() {
        let (es, ps) = stores();
        let oid = "off-002";
        append(&es, &format!("office:{oid}"), OFFICE_CREATED,
            json!({"id": oid, "name": "Montego Bay", "chair_count": 2}));
        append(&es, &format!("office:{oid}"), OFFICE_HOURS_SET,
            json!({"id": oid, "day_of_week": "Monday", "open_time": "08:00", "close_time": "17:00"}));
        rebuild(&es, &ps).unwrap();

        let hours = ps.list_office_hours(oid).unwrap();
        assert_eq!(hours.len(), 1);
        assert_eq!(hours[0].day_of_week, "Monday");

        append(&es, &format!("office:{oid}"), OFFICE_DAY_CLOSED,
            json!({"id": oid, "day_of_week": "Monday"}));
        rebuild(&es, &ps).unwrap();

        let hours = ps.list_office_hours(oid).unwrap();
        assert!(hours.is_empty());
    }
}
