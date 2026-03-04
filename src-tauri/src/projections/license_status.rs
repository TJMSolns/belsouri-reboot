use chrono::{DateTime, Utc};

use crate::db::{EventStore, ProjectionStore, LicenseStatusRow};
use crate::events::licensing::{
    EVAL_STARTED, LICENSE_ISSUED, LICENSE_RENEWED, LICENSE_VALIDATION_SUCCEEDED,
    LICENSE_DEGRADED, LICENSE_EXPIRED, CLOCK_ROLLBACK_DETECTED, STREAM_LICENSE,
};
use crate::licensing::service::compute_license_status;
use crate::licensing::types::{LicenseStatusDto, ModuleStatusDto};

const PROJECTION_NAME: &str = "license_status";

const LICENSE_EVENT_TYPES: &[&str] = &[
    EVAL_STARTED,
    LICENSE_ISSUED,
    LICENSE_RENEWED,
    LICENSE_VALIDATION_SUCCEEDED,
    LICENSE_DEGRADED,
    LICENSE_EXPIRED,
    CLOCK_ROLLBACK_DETECTED,
];

/// Incrementally rebuilds the license status projection.
/// Skips processing if no new license events exist since last position.
/// When new events exist, reads the full license stream and recomputes status.
pub fn rebuild(
    event_store: &EventStore,
    proj_store: &ProjectionStore,
    now: DateTime<Utc>,
) -> Result<(), String> {
    let position = proj_store.get_position(PROJECTION_NAME).map_err(|e| e.to_string())?;

    let new_events = event_store
        .read_events_since(position, LICENSE_EVENT_TYPES)
        .map_err(|e| e.to_string())?;

    if new_events.is_empty() {
        return Ok(());
    }

    let all_events = event_store
        .read_stream(STREAM_LICENSE)
        .map_err(|e| e.to_string())?;

    let status = compute_license_status(&all_events, now);
    let modules_json = serde_json::to_string(&status.modules).map_err(|e| e.to_string())?;

    let row = LicenseStatusRow {
        overall_validity: status.overall_validity,
        license_type: status.license_type,
        eval_expires_at: status.eval_expires_at,
        last_validated_at: status.last_validated_at,
        modules_json,
        computed_at: now.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
    };

    proj_store.upsert_license_status(&row).map_err(|e| e.to_string())?;

    let last_id = new_events.last().unwrap().id;
    proj_store.set_position(PROJECTION_NAME, last_id).map_err(|e| e.to_string())?;

    Ok(())
}

/// Returns the current license status from the projection, or a not-activated status.
pub fn query(proj_store: &ProjectionStore) -> Result<LicenseStatusDto, String> {
    let row = proj_store.get_license_status().map_err(|e| e.to_string())?;
    match row {
        None => Ok(LicenseStatusDto::not_activated()),
        Some(r) => {
            let modules: Vec<ModuleStatusDto> =
                serde_json::from_str(&r.modules_json).map_err(|e| e.to_string())?;
            Ok(LicenseStatusDto {
                overall_validity: r.overall_validity,
                license_type: r.license_type,
                eval_expires_at: r.eval_expires_at,
                last_validated_at: r.last_validated_at,
                modules,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::EventStore;
    use crate::events::licensing::{EvalStartedPayload, STREAM_LICENSE};

    fn stores() -> (EventStore, ProjectionStore) {
        (EventStore::new_in_memory().unwrap(), ProjectionStore::new_in_memory().unwrap())
    }

    fn now() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-03-04T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn append_eval_started(es: &EventStore) {
        let payload = serde_json::to_string(&EvalStartedPayload {
            practice_id: "a".repeat(64),
            started_at: "2026-03-04T00:00:00Z".to_string(),
            eval_expires_at: "2026-04-03T00:00:00Z".to_string(),
            modules: vec!["scheduling".to_string()],
        }).unwrap();
        es.append(STREAM_LICENSE, 0, EVAL_STARTED, &payload).unwrap();
    }

    #[test]
    fn test_query_returns_not_activated_before_any_events() {
        let (es, ps) = stores();
        rebuild(&es, &ps, now()).unwrap();
        let status = query(&ps).unwrap();
        assert_eq!(status.overall_validity, "Invalid");
        assert!(status.license_type.is_none());
    }

    #[test]
    fn test_rebuild_with_eval_started() {
        let (es, ps) = stores();
        append_eval_started(&es);
        rebuild(&es, &ps, now()).unwrap();
        let status = query(&ps).unwrap();
        assert_eq!(status.license_type, Some("eval".to_string()));
        assert_eq!(status.overall_validity, "Valid");
        assert_eq!(status.modules.len(), 1);
        assert_eq!(status.modules[0].status, "Active");
    }

    #[test]
    fn test_rebuild_is_idempotent() {
        let (es, ps) = stores();
        append_eval_started(&es);
        rebuild(&es, &ps, now()).unwrap();
        rebuild(&es, &ps, now()).unwrap();
        let status = query(&ps).unwrap();
        assert_eq!(status.modules.len(), 1);
    }

    #[test]
    fn test_rebuild_skips_when_no_new_events() {
        let (es, ps) = stores();
        append_eval_started(&es);
        rebuild(&es, &ps, now()).unwrap();
        let pos = ps.get_position("license_status").unwrap();

        // Second rebuild — no new events
        rebuild(&es, &ps, now()).unwrap();
        assert_eq!(ps.get_position("license_status").unwrap(), pos);
    }

    #[test]
    fn test_position_advances_after_rebuild() {
        let (es, ps) = stores();
        assert_eq!(ps.get_position("license_status").unwrap(), 0);
        append_eval_started(&es);
        rebuild(&es, &ps, now()).unwrap();
        assert!(ps.get_position("license_status").unwrap() > 0);
    }
}
