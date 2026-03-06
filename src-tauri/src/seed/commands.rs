use tauri::State;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

use crate::app_state::AppState;
use crate::events::patient_management::{
    PATIENT_REGISTERED, PATIENT_ARCHIVED,
    PatientRegisteredPayload, PatientArchivedPayload,
};
use crate::events::staff_management::{
    STAFF_MEMBER_REGISTERED, STAFF_MEMBER_ARCHIVED, ROLE_ASSIGNED, PROVIDER_TYPE_SET,
    StaffMemberRegisteredPayload, StaffMemberArchivedPayload, RoleAssignedPayload,
    ProviderTypeSetPayload,
};

const SEED_ACTOR: &str = "system:seed";

// ── DTOs ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct SeedSummaryDto {
    pub patients_created: u32,
    pub providers_created: u32,
    pub staff_created: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ArchiveSummaryDto {
    pub patients_archived: u32,
    pub providers_archived: u32,
    pub staff_archived: u32,
}

// ── Seed data ─────────────────────────────────────────────────────────────────

/// 10 patients: <CaribbeanFirst> Patient
const SEED_PATIENTS: &[&str] = &[
    "Marcus", "Asha", "Devon", "Simone", "Carlton",
    "Nadine", "Tyrone", "Marcia", "Errol", "Kezia",
];

/// 6 clinical providers: (name, clinical_specialization)
const SEED_PROVIDERS: &[(&str, &str)] = &[
    ("Dr. Winston", "Specialist"),
    ("Dr. Rochelle", "Specialist"),
    ("Dr. Lloyd", "Dentist"),
    ("Dr. Camille", "Dentist"),
    ("Sasha", "Hygienist"),
    ("Patrice", "Hygienist"),
];

/// 2 non-clinical staff: <First> Staff
const SEED_STAFF: &[&str] = &["Andre", "Yolanda"];

// ── Rebuild helpers ───────────────────────────────────────────────────────────

fn rebuild_all(state: &AppState) -> Result<(), String> {
    let events = state.events.lock().map_err(|e| e.to_string())?;
    let proj = state.projections.lock().map_err(|e| e.to_string())?;
    crate::projections::patient_management::rebuild(&events, &proj)?;
    crate::projections::practice_setup::rebuild(&events, &proj)?;
    crate::projections::staff_management::rebuild(&events, &proj)?;
    Ok(())
}

// ── Commands ──────────────────────────────────────────────────────────────────

#[specta::specta]
#[tauri::command]
pub async fn seed_demo_data(state: State<'_, AppState>) -> Result<SeedSummaryDto, String> {
    let events = state.events.lock().map_err(|e| e.to_string())?;

    // ── Patients ────────────────────────────────────────────────────────────
    for first in SEED_PATIENTS {
        let id = Uuid::new_v4().to_string();
        let json = serde_json::to_string(&PatientRegisteredPayload {
            patient_id: id.clone(),
            first_name: first.to_string(),
            last_name: "Patient".to_string(),
            phone: None,
            email: None,
            preferred_contact_channel: None,
            preferred_office_id: None,
            date_of_birth: None,
            registered_by: SEED_ACTOR.to_string(),
        }).map_err(|e| e.to_string())?;
        events.append(&format!("patient:{id}"), 0, PATIENT_REGISTERED, &json)
            .map_err(|e| e.to_string())?;
    }

    // ── Providers (registered as StaffMembers with Provider role + clinical specialization) ──
    for (first, clinical_specialization) in SEED_PROVIDERS {
        let id = Uuid::new_v4().to_string();
        let name = format!("{} {}", first, clinical_specialization);
        let stream = format!("staff:{id}");
        let reg_json = serde_json::to_string(&StaffMemberRegisteredPayload {
            staff_member_id: id.clone(),
            name,
            phone: None,
            email: None,
            preferred_contact_channel: None,
        }).map_err(|e| e.to_string())?;
        events.append(&stream, 0, STAFF_MEMBER_REGISTERED, &reg_json)
            .map_err(|e| e.to_string())?;
        let role_json = serde_json::to_string(&RoleAssignedPayload {
            staff_member_id: id.clone(),
            role: "Provider".to_string(),
        }).map_err(|e| e.to_string())?;
        events.append(&stream, 1, ROLE_ASSIGNED, &role_json)
            .map_err(|e| e.to_string())?;
        let type_json = serde_json::to_string(&ProviderTypeSetPayload {
            staff_member_id: id.clone(),
            clinical_specialization: clinical_specialization.to_string(),
        }).map_err(|e| e.to_string())?;
        events.append(&stream, 2, PROVIDER_TYPE_SET, &type_json)
            .map_err(|e| e.to_string())?;
    }

    // ── Staff ───────────────────────────────────────────────────────────────
    for first in SEED_STAFF {
        let id = Uuid::new_v4().to_string();
        let name = format!("{} Staff", first);
        let reg_json = serde_json::to_string(&StaffMemberRegisteredPayload {
            staff_member_id: id.clone(),
            name,
            phone: None,
            email: None,
            preferred_contact_channel: None,
        }).map_err(|e| e.to_string())?;
        events.append(&format!("staff_member:{id}"), 0, STAFF_MEMBER_REGISTERED, &reg_json)
            .map_err(|e| e.to_string())?;

        // Assign "Staff" role
        let role_json = serde_json::to_string(&RoleAssignedPayload {
            staff_member_id: id.clone(),
            role: "Staff".to_string(),
        }).map_err(|e| e.to_string())?;
        events.append(&format!("staff_member:{id}"), 1, ROLE_ASSIGNED, &role_json)
            .map_err(|e| e.to_string())?;
    }

    drop(events);
    rebuild_all(&state)?;

    Ok(SeedSummaryDto {
        patients_created: SEED_PATIENTS.len() as u32,
        providers_created: SEED_PROVIDERS.len() as u32,
        staff_created: SEED_STAFF.len() as u32,
    })
}

#[specta::specta]
#[tauri::command]
pub async fn archive_demo_data(state: State<'_, AppState>) -> Result<ArchiveSummaryDto, String> {
    rebuild_all(&state)?;

    let mut patient_ids: Vec<String> = Vec::new();
    let mut provider_ids: Vec<String> = Vec::new();
    let mut staff_ids: Vec<String> = Vec::new();

    {
        let proj = state.projections.lock().map_err(|e| e.to_string())?;

        // Find non-archived patients with last_name "Patient"
        let patients = proj.search_patients(None, None, None, false)
            .map_err(|e| e.to_string())?;
        for p in patients {
            if p.last_name.eq_ignore_ascii_case("Patient") {
                patient_ids.push(p.patient_id);
            }
        }

        // Find non-archived Provider-role staff whose name ends with a seed specialization,
        // and non-archived Staff whose name ends with " Staff"
        let all_staff = proj.list_staff_members().map_err(|e| e.to_string())?;
        for s in &all_staff {
            if s.archived {
                continue;
            }
            let roles = proj.list_staff_roles(&s.staff_member_id).map_err(|e| e.to_string())?;
            let n = s.name.to_lowercase();
            if roles.contains(&"Provider".to_string())
                && (n.ends_with(" specialist") || n.ends_with(" dentist") || n.ends_with(" hygienist"))
            {
                provider_ids.push(s.staff_member_id.clone());
            } else if n.ends_with(" staff") {
                staff_ids.push(s.staff_member_id.clone());
            }
        }
    }

    let events = state.events.lock().map_err(|e| e.to_string())?;

    for id in &patient_ids {
        let json = serde_json::to_string(&PatientArchivedPayload {
            patient_id: id.clone(),
            archived_by: SEED_ACTOR.to_string(),
        }).map_err(|e| e.to_string())?;
        // Use version 1 (registration was 0); append always succeeds as expected_version is advisory
        events.append(&format!("patient:{id}"), 1, PATIENT_ARCHIVED, &json)
            .map_err(|e| e.to_string())?;
    }

    for id in &provider_ids {
        let json = serde_json::to_string(&StaffMemberArchivedPayload {
            staff_member_id: id.clone(),
        }).map_err(|e| e.to_string())?;
        // Provider staff stream had 3 events (registered + role + type_set), use version 3
        events.append(&format!("staff:{id}"), 3, STAFF_MEMBER_ARCHIVED, &json)
            .map_err(|e| e.to_string())?;
    }

    for id in &staff_ids {
        let json = serde_json::to_string(&StaffMemberArchivedPayload {
            staff_member_id: id.clone(),
        }).map_err(|e| e.to_string())?;
        // Staff stream may have multiple events (registered + role assigned), use high version
        events.append(&format!("staff_member:{id}"), 2, STAFF_MEMBER_ARCHIVED, &json)
            .map_err(|e| e.to_string())?;
    }

    drop(events);
    rebuild_all(&state)?;

    Ok(ArchiveSummaryDto {
        patients_archived: patient_ids.len() as u32,
        providers_archived: provider_ids.len() as u32,
        staff_archived: staff_ids.len() as u32,
    })
}
