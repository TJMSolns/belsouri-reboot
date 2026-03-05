use chrono::{DateTime, Duration, Utc};
use tauri::State;

use crate::app_state::AppState;
use crate::events::licensing::{
    ClockRollbackDetectedPayload, EvalStartedPayload, LicenseIssuedPayload,
    LicenseModuleEntryPayload, LicenseValidationSucceededPayload, PracticeIdentityEstablishedPayload,
    CLOCK_ROLLBACK_DETECTED, EVAL_STARTED, LICENSE_ISSUED, LICENSE_VALIDATION_SUCCEEDED,
    STREAM_LICENSE, STREAM_PRACTICE_IDENTITY,
};
use crate::licensing::crypto::{decode_and_verify, EMBEDDED_PUBLIC_KEY};
use crate::licensing::eval_token::EVAL_TOKEN;
use crate::licensing::service::{
    derive_machine_id_hash, derive_practice_id, get_machine_id,
    parse_license_payload, today_as_install_date,
};
use crate::licensing::types::LicenseStatusDto;
use crate::projections;

/// Runs on every app startup. Idempotent.
/// - Establishes practice identity if not yet done (first run)
/// - Starts eval period if no license events exist (first run)
/// - Detects clock rollback (appends ClockRollbackDetected if detected)
/// - Records a successful validation timestamp
/// - Rebuilds projections and returns current license status
#[specta::specta]
#[tauri::command]
pub fn startup_license_check(state: State<'_, AppState>) -> Result<LicenseStatusDto, String> {
    let events = state.events.lock().map_err(|_| "event store lock error".to_string())?;
    let projections = state.projections.lock().map_err(|_| "projection store lock error".to_string())?;
    let now = Utc::now();

    // --- Step 1: Establish practice identity (first run only) ---
    let identity_version = events
        .current_version(STREAM_PRACTICE_IDENTITY)
        .map_err(|e| e.to_string())?;

    if identity_version == 0 {
        let machine_id = get_machine_id()?;
        let install_date = today_as_install_date();
        let practice_id = derive_practice_id(&machine_id, &install_date);
        let machine_id_hash = derive_machine_id_hash(&machine_id);
        let established_at = fmt_dt(now);

        let payload = serde_json::to_string(&PracticeIdentityEstablishedPayload {
            practice_id,
            machine_id_hash,
            install_date,
            established_at,
        })
        .map_err(|e| e.to_string())?;

        events
            .append(STREAM_PRACTICE_IDENTITY, 0, PRACTICE_IDENTITY_ESTABLISHED, &payload)
            .map_err(|e| e.to_string())?;
    }

    // --- Step 2: Start eval if no license events yet (first run only) ---
    let license_version = events
        .current_version(STREAM_LICENSE)
        .map_err(|e| e.to_string())?;

    if license_version == 0 {
        let payload_bytes = decode_and_verify(EVAL_TOKEN, &EMBEDDED_PUBLIC_KEY)
            .map_err(|e| e.to_string())?;
        let license_payload = parse_license_payload(&payload_bytes)?;

        // Rebuild identity projection to get the practice_id we just established
        projections::practice_identity::rebuild(&events, &projections)?;
        let identity = projections::practice_identity::query(&projections)?
            .ok_or("practice identity missing after establish")?;

        let eval_duration = license_payload.max_duration_days.unwrap_or(30);
        let eval_expires = now + Duration::days(eval_duration as i64);
        let modules: Vec<String> = license_payload.modules.iter().map(|m| m.name.clone()).collect();

        let eval_payload = serde_json::to_string(&EvalStartedPayload {
            practice_id: identity.practice_id,
            started_at: fmt_dt(now),
            eval_expires_at: fmt_dt(eval_expires),
            modules,
        })
        .map_err(|e| e.to_string())?;

        events
            .append(STREAM_LICENSE, 0, EVAL_STARTED, &eval_payload)
            .map_err(|e| e.to_string())?;
    }

    // --- Step 3: Anti-rollback check (skip if already detected) ---
    let all_license_events = events.read_stream(STREAM_LICENSE).map_err(|e| e.to_string())?;
    let already_detected = all_license_events
        .iter()
        .any(|e| e.event_type == CLOCK_ROLLBACK_DETECTED);

    if !already_detected {
        let last_validated_at: Option<DateTime<Utc>> = all_license_events
            .iter()
            .filter(|e| e.event_type == LICENSE_VALIDATION_SUCCEEDED)
            .last()
            .and_then(|e| {
                serde_json::from_str::<LicenseValidationSucceededPayload>(&e.payload).ok()
            })
            .and_then(|p| DateTime::parse_from_rfc3339(&p.validated_at).ok())
            .map(|dt| dt.with_timezone(&Utc));

        if let Some(last) = last_validated_at {
            if now < last - Duration::hours(24) {
                // Clock rolled back — flag it permanently
                let version = events.current_version(STREAM_LICENSE).map_err(|e| e.to_string())?;
                let rollback_payload = serde_json::to_string(&ClockRollbackDetectedPayload {
                    detected_at: fmt_dt(now),
                    last_seen_at: fmt_dt(last),
                })
                .map_err(|e| e.to_string())?;
                events
                    .append(STREAM_LICENSE, version, CLOCK_ROLLBACK_DETECTED, &rollback_payload)
                    .map_err(|e| e.to_string())?;
            } else {
                append_validation_succeeded(&events, now, &all_license_events)?;
            }
        } else {
            append_validation_succeeded(&events, now, &all_license_events)?;
        }
    }

    // --- Step 4: Rebuild projections and return ---
    projections::practice_identity::rebuild(&events, &projections)?;
    projections::license_status::rebuild(&events, &projections, now)?;
    projections::license_status::query(&projections)
}

/// Returns the current license status (rebuilds projection with current time).
#[specta::specta]
#[tauri::command]
pub fn get_license_status(state: State<'_, AppState>) -> Result<LicenseStatusDto, String> {
    let events = state.events.lock().map_err(|_| "event store lock error".to_string())?;
    let projections = state.projections.lock().map_err(|_| "projection store lock error".to_string())?;
    let now = Utc::now();
    projections::license_status::rebuild(&events, &projections, now)?;
    projections::license_status::query(&projections)
}

/// Returns the current practice_id, or None if identity not yet established.
#[specta::specta]
#[tauri::command]
pub fn get_practice_id(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let events = state.events.lock().map_err(|_| "event store lock error".to_string())?;
    let projections = state.projections.lock().map_err(|_| "projection store lock error".to_string())?;
    projections::practice_identity::rebuild(&events, &projections)?;
    Ok(projections::practice_identity::query(&projections)?.map(|r| r.practice_id))
}

/// Activates a paid license key. Validates the key, checks practice_id match, appends LicenseIssued.
#[specta::specta]
#[tauri::command]
pub fn activate_license(
    state: State<'_, AppState>,
    license_key: String,
) -> Result<LicenseStatusDto, String> {
    let events = state.events.lock().map_err(|_| "event store lock error".to_string())?;
    let projections = state.projections.lock().map_err(|_| "projection store lock error".to_string())?;
    let now = Utc::now();

    // Verify and parse the license key
    let payload_bytes = decode_and_verify(&license_key, &EMBEDDED_PUBLIC_KEY)
        .map_err(|e| e.to_string())?;
    let license_payload = parse_license_payload(&payload_bytes)?;

    if license_payload.license_type != "paid" {
        return Err("License key is not a paid license".to_string());
    }

    // Verify practice_id matches this machine
    projections::practice_identity::rebuild(&events, &projections)?;
    let identity = projections::practice_identity::query(&projections)?
        .ok_or("Practice identity not established — run startup check first")?;

    let key_practice_id = license_payload
        .practice_id
        .as_deref()
        .ok_or("License key missing practice_id")?;

    if key_practice_id != identity.practice_id {
        return Err("License key was issued for a different machine".to_string());
    }

    // Build the license issued event
    let modules: Vec<LicenseModuleEntryPayload> = license_payload
        .modules
        .iter()
        .map(|m| {
            let expires_at = m
                .expires_at
                .clone()
                .ok_or_else(|| "Module missing expires_at in paid license".to_string())?;
            Ok(LicenseModuleEntryPayload {
                name: m.name.clone(),
                expires_at,
                grace_period_days: m.grace_period_days,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    let event_payload = serde_json::to_string(&LicenseIssuedPayload {
        practice_id: identity.practice_id,
        license_type: "paid".to_string(),
        issued_at: fmt_dt(now),
        modules,
        schema_version: 2,
    })
    .map_err(|e| e.to_string())?;

    let version = events
        .current_version(STREAM_LICENSE)
        .map_err(|e| e.to_string())?;
    events
        .append(STREAM_LICENSE, version, LICENSE_ISSUED, &event_payload)
        .map_err(|e| e.to_string())?;

    projections::license_status::rebuild(&events, &projections, now)?;
    projections::license_status::query(&projections)
}

// --- helpers ---

fn fmt_dt(dt: DateTime<Utc>) -> String {
    dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}

fn append_validation_succeeded(
    events: &crate::db::EventStore,
    now: DateTime<Utc>,
    all_license_events: &[crate::db::StoredEvent],
) -> Result<(), String> {
    let next_check_at = fmt_dt(now + Duration::hours(48));
    let payload = serde_json::to_string(&LicenseValidationSucceededPayload {
        validated_at: fmt_dt(now),
        next_check_at,
    })
    .map_err(|e| e.to_string())?;

    let version = all_license_events.len() as u64;
    events
        .append(STREAM_LICENSE, version, LICENSE_VALIDATION_SUCCEEDED, &payload)
        .map_err(|e| e.to_string())?;
    Ok(())
}

// Pull in the constant defined alongside get_machine_id
use crate::events::licensing::PRACTICE_IDENTITY_ESTABLISHED;
