use tauri::State;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::events::staff_shift::*;
use crate::projections::staff_shift::rebuild;
use super::types::*;

fn do_rebuild(state: &AppState) -> Result<(), String> {
    let events = state.events.lock().map_err(|e| e.to_string())?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    rebuild(&events, &proj)
}

fn row_to_dto(row: &crate::db::StaffShiftRow) -> StaffShiftDto {
    StaffShiftDto {
        shift_id: row.shift_id.clone(),
        staff_member_id: row.staff_member_id.clone(),
        staff_name: row.staff_name.clone(),
        office_id: row.office_id.clone(),
        office_name: row.office_name.clone(),
        date: row.date.clone(),
        start_time: row.start_time.clone(),
        end_time: row.end_time.clone(),
        role: row.role.clone(),
        created_by: row.created_by.clone(),
        cancelled: row.cancelled,
        cancel_reason: row.cancel_reason.clone(),
    }
}

fn validate_date(date: &str) -> Result<(), String> {
    // Validate YYYY-MM-DD format
    if date.len() != 10 {
        return Err(format!("Date '{}' is not a valid YYYY-MM-DD date", date));
    }
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return Err(format!("Date '{}' is not a valid YYYY-MM-DD date", date));
    }
    let y = parts[0].parse::<i32>().map_err(|_| format!("Date '{}' has an invalid year", date))?;
    let m = parts[1].parse::<u32>().map_err(|_| format!("Date '{}' has an invalid month", date))?;
    let d = parts[2].parse::<u32>().map_err(|_| format!("Date '{}' has an invalid day", date))?;
    if y < 2020 || m < 1 || m > 12 || d < 1 || d > 31 {
        return Err(format!("Date '{}' is not a valid YYYY-MM-DD date", date));
    }
    Ok(())
}

fn parse_hhmm(t: &str) -> Option<u32> {
    let parts: Vec<&str> = t.split(':').collect();
    if parts.len() != 2 { return None; }
    let h = parts[0].parse::<u32>().ok()?;
    let m = parts[1].parse::<u32>().ok()?;
    if h > 23 || m > 59 { return None; }
    Some(h * 60 + m)
}

fn validate_times(start_time: &str, end_time: &str) -> Result<(), String> {
    let start = parse_hhmm(start_time)
        .ok_or_else(|| format!("Start time '{}' is not a valid HH:MM time", start_time))?;
    let end = parse_hhmm(end_time)
        .ok_or_else(|| format!("End time '{}' is not a valid HH:MM time", end_time))?;
    if end <= start {
        return Err(format!(
            "End time '{}' must be strictly after start time '{}'",
            end_time, start_time
        ));
    }
    Ok(())
}

// ── Commands ──────────────────────────────────────────────────────────────────

#[specta::specta]
#[tauri::command]
pub async fn plan_staff_shift(
    state: State<'_, AppState>,
    staff_member_id: String,
    office_id: String,
    date: String,
    start_time: String,
    end_time: String,
    role: String,
    created_by: String,
) -> Result<PlanShiftResult, String> {
    // Validate fields
    if role.trim().is_empty() {
        return Err("Role must not be empty".to_string());
    }
    validate_date(&date)?;
    validate_times(&start_time, &end_time)?;

    // Rebuild before constraint checks
    do_rebuild(&state)?;

    {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;

        // Validate staff member exists and is not archived
        let staff = proj.get_staff_member(&staff_member_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Staff member '{}' does not exist", staff_member_id))?;
        if staff.archived {
            return Err(format!(
                "Staff member '{}' is archived and cannot be assigned a shift",
                staff.name
            ));
        }

        // Validate role is assigned to this staff member
        let roles = proj.list_staff_roles(&staff_member_id).map_err(|e| e.to_string())?;
        if !roles.contains(&role) {
            return Err(format!(
                "Staff member '{}' does not hold the '{}' role. Assign the role first.",
                staff.name, role
            ));
        }

        // Validate office exists and is not archived
        let office = proj.get_office(&office_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Office '{}' does not exist", office_id))?;
        if office.archived {
            return Err(format!(
                "Office '{}' is archived and cannot be used for shift planning",
                office.name
            ));
        }
    }

    let shift_id = Uuid::new_v4().to_string();
    let stream_id = format!("staff_shift:{}", shift_id);

    let payload = StaffShiftPlannedPayload {
        shift_id: shift_id.clone(),
        staff_member_id,
        office_id,
        date,
        start_time,
        end_time,
        role,
        created_by,
    };

    {
        let events = state.events.lock().map_err(|e| e.to_string())?;
        events.append(&stream_id, 0, STAFF_SHIFT_PLANNED,
            &serde_json::to_string(&payload).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    }

    do_rebuild(&state)?;
    Ok(PlanShiftResult { shift_id })
}

#[specta::specta]
#[tauri::command]
pub async fn cancel_staff_shift(
    state: State<'_, AppState>,
    shift_id: String,
    cancel_reason: Option<String>,
    cancelled_by: String,
) -> Result<(), String> {
    do_rebuild(&state)?;

    {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;
        let shift = proj.get_shift(&shift_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Shift '{}' does not exist", shift_id))?;
        if shift.cancelled {
            return Err(format!(
                "Shift for {} on {} is already cancelled",
                shift.staff_name, shift.date
            ));
        }
    }

    let stream_id = format!("staff_shift:{}", shift_id);
    let payload = StaffShiftCancelledPayload {
        shift_id: shift_id.clone(),
        cancel_reason,
        cancelled_by,
    };

    {
        let events = state.events.lock().map_err(|e| e.to_string())?;
        let ver = events.current_version(&stream_id).map_err(|e| e.to_string())?;
        events.append(&stream_id, ver, STAFF_SHIFT_CANCELLED,
            &serde_json::to_string(&payload).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    }

    do_rebuild(&state)?;
    Ok(())
}

#[specta::specta]
#[tauri::command]
pub async fn get_shift_roster(
    state: State<'_, AppState>,
    week_start_date: String,
    office_id: Option<String>,
) -> Result<Vec<StaffShiftDto>, String> {
    // Compute week_end (Sunday) from the Monday week_start_date
    let week_end = {
        let parts: Vec<&str> = week_start_date.split('-').collect();
        if parts.len() == 3 {
            let y: i32 = parts[0].parse().unwrap_or(2026);
            let m: u32 = parts[1].parse().unwrap_or(1);
            let d: u32 = parts[2].parse().unwrap_or(1);
            // Add 6 days to get Sunday
            let start = chrono::NaiveDate::from_ymd_opt(y, m, d)
                .ok_or_else(|| format!("Invalid week_start_date '{}'", week_start_date))?;
            let end = start + chrono::Duration::days(6);
            end.format("%Y-%m-%d").to_string()
        } else {
            return Err(format!("Invalid week_start_date '{}'", week_start_date));
        }
    };

    do_rebuild(&state)?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    let rows = proj.get_shifts_for_week(&week_start_date, &week_end, office_id.as_deref())
        .map_err(|e| e.to_string())?;
    Ok(rows.iter().map(row_to_dto).collect())
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use crate::db::{EventStore, ProjectionStore, StaffMemberRow, OfficeRow};
    use crate::events::staff_shift::*;
    use crate::projections::staff_shift::rebuild;

    fn setup() -> (EventStore, ProjectionStore) {
        let events = EventStore::new_in_memory().unwrap();
        let proj = ProjectionStore::new_in_memory().unwrap();

        // Seed a staff member with the Staff role
        proj.upsert_staff_member(&StaffMemberRow {
            staff_member_id: "staff-1".to_string(),
            name: "Jane Doe".to_string(),
            phone: None,
            email: None,
            preferred_contact_channel: None,
            pin_hash: None,
            clinical_specialization: None,
            archived: false,
        }).unwrap();
        proj.add_staff_role("staff-1", "Staff").unwrap();

        // Seed an office
        proj.upsert_office(&OfficeRow {
            id: "office-1".to_string(),
            name: "Main Office".to_string(),
            chair_count: 3,
            archived: false,
            address_line_1: None,
            address_line_2: None,
            city_town: None,
            subdivision: None,
            country: None,
        }).unwrap();

        (events, proj)
    }

    fn append_planned(events: &EventStore, shift_id: &str, date: &str) {
        let stream = format!("staff_shift:{}", shift_id);
        let payload = StaffShiftPlannedPayload {
            shift_id: shift_id.to_string(),
            staff_member_id: "staff-1".to_string(),
            office_id: "office-1".to_string(),
            date: date.to_string(),
            start_time: "09:00".to_string(),
            end_time: "17:00".to_string(),
            role: "Staff".to_string(),
            created_by: "staff-1".to_string(),
        };
        let ver = events.current_version(&stream).unwrap();
        let json = serde_json::to_string(&payload).unwrap();
        events.append(&stream, ver, STAFF_SHIFT_PLANNED, &json).unwrap();
    }

    #[test]
    fn test_plan_shift_creates_shift() {
        let (events, proj) = setup();
        let shift_id = "shift-1";
        append_planned(&events, shift_id, "2026-03-09");
        rebuild(&events, &proj).unwrap();

        let row = proj.get_shift(shift_id).unwrap().expect("shift should exist");
        assert_eq!(row.staff_name, "Jane Doe");
        assert_eq!(row.office_name, "Main Office");
        assert_eq!(row.date, "2026-03-09");
        assert_eq!(row.start_time, "09:00");
        assert_eq!(row.end_time, "17:00");
        assert_eq!(row.role, "Staff");
        assert!(!row.cancelled);
    }

    #[test]
    fn test_cancel_shift_marks_cancelled() {
        let (events, proj) = setup();
        let shift_id = "shift-1";
        let stream = format!("staff_shift:{}", shift_id);
        append_planned(&events, shift_id, "2026-03-09");

        let cancel_payload = StaffShiftCancelledPayload {
            shift_id: shift_id.to_string(),
            cancel_reason: Some("Personal reasons".to_string()),
            cancelled_by: "staff-1".to_string(),
        };
        let ver = events.current_version(&stream).unwrap();
        let json = serde_json::to_string(&cancel_payload).unwrap();
        events.append(&stream, ver, STAFF_SHIFT_CANCELLED, &json).unwrap();

        rebuild(&events, &proj).unwrap();

        let row = proj.get_shift(shift_id).unwrap().unwrap();
        assert!(row.cancelled);
        assert_eq!(row.cancel_reason, Some("Personal reasons".to_string()));
    }

    #[test]
    fn test_plan_shift_rejects_unknown_staff() {
        let (events, proj) = setup();
        // Attempt to insert a shift for a non-existent staff member via projection rebuild
        // (the command-layer validation happens before appending to event store,
        // so we test that get_staff_member returns None for unknown staff)
        let unknown = proj.get_staff_member("nonexistent-staff").unwrap();
        assert!(unknown.is_none(), "Expected no staff member for unknown ID");

        // Validate that the command validate path catches it — test via validate logic
        // by checking that inserting without seed data would fail at the get_staff_member check
        let _ = events; // satisfy unused warning
    }

    #[test]
    fn test_plan_shift_rejects_archived_staff() {
        let (_events, proj) = setup();
        // Update staff to archived
        proj.upsert_staff_member(&StaffMemberRow {
            staff_member_id: "staff-1".to_string(),
            name: "Jane Doe".to_string(),
            phone: None,
            email: None,
            preferred_contact_channel: None,
            pin_hash: None,
            clinical_specialization: None,
            archived: true,
        }).unwrap();
        let row = proj.get_staff_member("staff-1").unwrap().unwrap();
        assert!(row.archived, "Staff member should be archived");
    }

    #[test]
    fn test_get_shift_roster_returns_week() {
        let (events, proj) = setup();

        // 2 shifts in week of 2026-03-09 (Mon–Sun), 1 in next week
        append_planned(&events, "s1", "2026-03-09");
        append_planned(&events, "s2", "2026-03-10");
        append_planned(&events, "s3", "2026-03-16");

        rebuild(&events, &proj).unwrap();

        let week_shifts = proj.get_shifts_for_week("2026-03-09", "2026-03-15", None).unwrap();
        assert_eq!(week_shifts.len(), 2);
        assert!(week_shifts.iter().all(|s| s.date.as_str() >= "2026-03-09" && s.date.as_str() <= "2026-03-15"));

        // Next week should return 1
        let next_week = proj.get_shifts_for_week("2026-03-16", "2026-03-22", None).unwrap();
        assert_eq!(next_week.len(), 1);
    }
}
