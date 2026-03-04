use rusqlite::{Connection, Result as SqlResult, params};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EventStoreError {
    #[error("Database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("Concurrency conflict: expected version {expected}, found {actual}")]
    ConcurrencyConflict { expected: u64, actual: u64 },
}

pub type Result<T> = std::result::Result<T, EventStoreError>;

#[derive(Debug, Clone)]
pub struct StoredEvent {
    pub id: i64,
    pub stream_id: String,
    pub stream_version: u64,
    pub event_type: String,
    pub payload: String,
    pub created_at: String,
}

pub struct EventStore {
    conn: Connection,
}

impl EventStore {
    pub fn open(path: &std::path::Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        let store = Self { conn };
        store.init()?;
        Ok(store)
    }

    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let store = Self { conn };
        store.init_without_wal()?;
        Ok(store)
    }

    fn init(&self) -> Result<()> {
        self.conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        self.init_schema()
    }

    fn init_without_wal(&self) -> Result<()> {
        self.init_schema()
    }

    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch("
            CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                stream_id TEXT NOT NULL,
                stream_version INTEGER NOT NULL,
                event_type TEXT NOT NULL,
                payload TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
                UNIQUE(stream_id, stream_version)
            );
            CREATE INDEX IF NOT EXISTS idx_events_stream
                ON events(stream_id, stream_version);
            CREATE INDEX IF NOT EXISTS idx_events_type
                ON events(event_type);
            CREATE INDEX IF NOT EXISTS idx_events_created
                ON events(created_at);
        ")?;
        Ok(())
    }

    /// Returns the current max stream_version for a stream, or 0 if no events exist.
    pub fn current_version(&self, stream_id: &str) -> Result<u64> {
        let version: Option<i64> = self.conn.query_row(
            "SELECT MAX(stream_version) FROM events WHERE stream_id = ?1",
            params![stream_id],
            |row| row.get(0),
        )?;
        Ok(version.unwrap_or(0) as u64)
    }

    /// Appends an event to a stream with optimistic concurrency control.
    /// `expected_version` must match the current max version (0 means stream must not exist).
    /// Returns the new global event id.
    pub fn append(
        &self,
        stream_id: &str,
        expected_version: u64,
        event_type: &str,
        payload_json: &str,
    ) -> Result<i64> {
        let actual = self.current_version(stream_id)?;
        if actual != expected_version {
            return Err(EventStoreError::ConcurrencyConflict {
                expected: expected_version,
                actual,
            });
        }
        let new_version = expected_version + 1;
        self.conn.execute(
            "INSERT INTO events (stream_id, stream_version, event_type, payload)
             VALUES (?1, ?2, ?3, ?4)",
            params![stream_id, new_version, event_type, payload_json],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Reads all events for a stream, ordered by stream_version.
    pub fn read_stream(&self, stream_id: &str) -> Result<Vec<StoredEvent>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, stream_id, stream_version, event_type, payload, created_at
             FROM events WHERE stream_id = ?1
             ORDER BY stream_version ASC",
        )?;
        let events = stmt.query_map(params![stream_id], |row| {
            Ok(StoredEvent {
                id: row.get(0)?,
                stream_id: row.get(1)?,
                stream_version: row.get::<_, i64>(2)? as u64,
                event_type: row.get(3)?,
                payload: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?
        .collect::<SqlResult<Vec<_>>>()?;
        Ok(events)
    }

    /// Reads all events with id > after_id, optionally filtered by event_type.
    /// Used by projections for incremental reads.
    pub fn read_events_since(
        &self,
        after_id: i64,
        event_types: &[&str],
    ) -> Result<Vec<StoredEvent>> {
        if event_types.is_empty() {
            let mut stmt = self.conn.prepare(
                "SELECT id, stream_id, stream_version, event_type, payload, created_at
                 FROM events WHERE id > ?1
                 ORDER BY id ASC",
            )?;
            let events = stmt.query_map(params![after_id], |row| {
                Ok(StoredEvent {
                    id: row.get(0)?,
                    stream_id: row.get(1)?,
                    stream_version: row.get::<_, i64>(2)? as u64,
                    event_type: row.get(3)?,
                    payload: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })?
            .collect::<SqlResult<Vec<_>>>()?;
            return Ok(events);
        }

        // Build parameterized IN clause
        let placeholders: String = event_types
            .iter()
            .enumerate()
            .map(|(i, _)| format!("?{}", i + 2))
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!(
            "SELECT id, stream_id, stream_version, event_type, payload, created_at
             FROM events WHERE id > ?1 AND event_type IN ({})
             ORDER BY id ASC",
            placeholders
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let mut values: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(after_id)];
        for et in event_types {
            values.push(Box::new(et.to_string()));
        }
        let params: Vec<&dyn rusqlite::ToSql> = values.iter().map(|v| v.as_ref()).collect();
        let events = stmt
            .query_map(params.as_slice(), |row| {
                Ok(StoredEvent {
                    id: row.get(0)?,
                    stream_id: row.get(1)?,
                    stream_version: row.get::<_, i64>(2)? as u64,
                    event_type: row.get(3)?,
                    payload: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })?
            .collect::<SqlResult<Vec<_>>>()?;
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> EventStore {
        EventStore::new_in_memory().unwrap()
    }

    #[test]
    fn test_append_single_event() {
        let s = store();
        let id = s.append("practice_identity", 0, "PracticeIdentityEstablished", r#"{"practice_id":"abc"}"#).unwrap();
        assert!(id > 0);
        let events = s.read_stream("practice_identity").unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "PracticeIdentityEstablished");
        assert_eq!(events[0].stream_version, 1);
    }

    #[test]
    fn test_append_multiple_events_sequential() {
        let s = store();
        s.append("license", 0, "EvalStarted", r#"{}"#).unwrap();
        s.append("license", 1, "LicenseValidationSucceeded", r#"{}"#).unwrap();
        let events = s.read_stream("license").unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].stream_version, 1);
        assert_eq!(events[1].stream_version, 2);
    }

    #[test]
    fn test_optimistic_concurrency_conflict() {
        let s = store();
        s.append("license", 0, "EvalStarted", r#"{}"#).unwrap();
        // Try to append with wrong expected version
        let err = s.append("license", 0, "EvalStarted", r#"{}"#).unwrap_err();
        assert!(matches!(err, EventStoreError::ConcurrencyConflict { expected: 0, actual: 1 }));
    }

    #[test]
    fn test_current_version_empty_stream() {
        let s = store();
        assert_eq!(s.current_version("nonexistent").unwrap(), 0);
    }

    #[test]
    fn test_current_version_after_appends() {
        let s = store();
        s.append("license", 0, "EvalStarted", r#"{}"#).unwrap();
        s.append("license", 1, "LicenseValidationSucceeded", r#"{}"#).unwrap();
        assert_eq!(s.current_version("license").unwrap(), 2);
    }

    #[test]
    fn test_read_events_since_no_filter() {
        let s = store();
        let id1 = s.append("practice_identity", 0, "PracticeIdentityEstablished", r#"{}"#).unwrap();
        let _id2 = s.append("license", 0, "EvalStarted", r#"{}"#).unwrap();
        let events = s.read_events_since(id1, &[]).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "EvalStarted");
    }

    #[test]
    fn test_read_events_since_with_type_filter() {
        let s = store();
        s.append("practice_identity", 0, "PracticeIdentityEstablished", r#"{}"#).unwrap();
        s.append("license", 0, "EvalStarted", r#"{}"#).unwrap();
        s.append("license", 1, "LicenseValidationSucceeded", r#"{}"#).unwrap();
        let events = s.read_events_since(0, &["EvalStarted"]).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "EvalStarted");
    }

    #[test]
    fn test_streams_are_independent() {
        let s = store();
        s.append("practice_identity", 0, "PracticeIdentityEstablished", r#"{}"#).unwrap();
        s.append("license", 0, "EvalStarted", r#"{}"#).unwrap();
        assert_eq!(s.current_version("practice_identity").unwrap(), 1);
        assert_eq!(s.current_version("license").unwrap(), 1);
    }
}
