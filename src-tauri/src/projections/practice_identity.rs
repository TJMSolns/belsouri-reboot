use crate::db::{EventStore, ProjectionStore, PracticeIdentityRow};
use crate::events::licensing::{PRACTICE_IDENTITY_ESTABLISHED, PracticeIdentityEstablishedPayload};

const PROJECTION_NAME: &str = "practice_identity";

/// Incrementally processes new PracticeIdentityEstablished events and upserts into the projection.
/// Safe to call repeatedly — only processes events since last position.
pub fn rebuild(event_store: &EventStore, proj_store: &ProjectionStore) -> Result<(), String> {
    let position = proj_store.get_position(PROJECTION_NAME).map_err(|e| e.to_string())?;

    let events = event_store
        .read_events_since(position, &[PRACTICE_IDENTITY_ESTABLISHED])
        .map_err(|e| e.to_string())?;

    if events.is_empty() {
        return Ok(());
    }

    let mut last_id = position;
    for event in &events {
        if let Ok(p) = serde_json::from_str::<PracticeIdentityEstablishedPayload>(&event.payload) {
            let row = PracticeIdentityRow {
                practice_id: p.practice_id,
                machine_id_hash: p.machine_id_hash,
                install_date: p.install_date,
                established_at: p.established_at,
            };
            proj_store.upsert_practice_identity(&row).map_err(|e| e.to_string())?;
        }
        last_id = event.id;
    }

    proj_store.set_position(PROJECTION_NAME, last_id).map_err(|e| e.to_string())?;
    Ok(())
}

/// Returns the current practice identity, or None if not yet established.
pub fn query(proj_store: &ProjectionStore) -> Result<Option<PracticeIdentityRow>, String> {
    proj_store.get_practice_identity().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::EventStore;
    use crate::events::licensing::{PracticeIdentityEstablishedPayload, STREAM_PRACTICE_IDENTITY};

    fn stores() -> (EventStore, ProjectionStore) {
        (EventStore::new_in_memory().unwrap(), ProjectionStore::new_in_memory().unwrap())
    }

    fn append_identity(es: &EventStore, practice_id: &str) {
        let payload = serde_json::to_string(&PracticeIdentityEstablishedPayload {
            practice_id: practice_id.to_string(),
            machine_id_hash: "b".repeat(64),
            install_date: "2026-03-04".to_string(),
            established_at: "2026-03-04T10:00:00.000Z".to_string(),
        }).unwrap();
        es.append(STREAM_PRACTICE_IDENTITY, 0, PRACTICE_IDENTITY_ESTABLISHED, &payload).unwrap();
    }

    #[test]
    fn test_query_returns_none_before_any_event() {
        let (es, ps) = stores();
        rebuild(&es, &ps).unwrap();
        assert!(query(&ps).unwrap().is_none());
    }

    #[test]
    fn test_rebuild_persists_identity() {
        let (es, ps) = stores();
        append_identity(&es, &"a".repeat(64));
        rebuild(&es, &ps).unwrap();
        let row = query(&ps).unwrap().expect("identity should exist");
        assert_eq!(row.practice_id, "a".repeat(64));
        assert_eq!(row.install_date, "2026-03-04");
    }

    #[test]
    fn test_rebuild_is_idempotent() {
        let (es, ps) = stores();
        append_identity(&es, &"a".repeat(64));
        rebuild(&es, &ps).unwrap();
        rebuild(&es, &ps).unwrap(); // second call — no-op
        let row = query(&ps).unwrap().expect("should still exist");
        assert_eq!(row.practice_id, "a".repeat(64));
    }

    #[test]
    fn test_position_advances() {
        let (es, ps) = stores();
        assert_eq!(ps.get_position("practice_identity").unwrap(), 0);
        append_identity(&es, &"a".repeat(64));
        rebuild(&es, &ps).unwrap();
        assert!(ps.get_position("practice_identity").unwrap() > 0);
    }

    #[test]
    fn test_second_rebuild_skips_already_processed() {
        let (es, ps) = stores();
        append_identity(&es, &"a".repeat(64));
        rebuild(&es, &ps).unwrap();
        let pos_after_first = ps.get_position("practice_identity").unwrap();

        // Append another identity event (reinstall scenario)
        let payload2 = serde_json::to_string(&PracticeIdentityEstablishedPayload {
            practice_id: "c".repeat(64),
            machine_id_hash: "d".repeat(64),
            install_date: "2026-03-05".to_string(),
            established_at: "2026-03-05T10:00:00.000Z".to_string(),
        }).unwrap();
        es.append(STREAM_PRACTICE_IDENTITY, 1, PRACTICE_IDENTITY_ESTABLISHED, &payload2).unwrap();
        rebuild(&es, &ps).unwrap();

        let pos_after_second = ps.get_position("practice_identity").unwrap();
        assert!(pos_after_second > pos_after_first);
        // Latest identity wins (upsert)
        let row = query(&ps).unwrap().unwrap();
        assert_eq!(row.practice_id, "c".repeat(64));
    }
}
