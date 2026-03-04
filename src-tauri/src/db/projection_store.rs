use rusqlite::{Connection, Result as SqlResult, params};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProjectionStoreError {
    #[error("Database error: {0}")]
    Db(#[from] rusqlite::Error),
}

pub type Result<T> = std::result::Result<T, ProjectionStoreError>;

pub struct ProjectionStore {
    conn: Connection,
}

#[derive(Debug, Clone)]
pub struct PracticeIdentityRow {
    pub practice_id: String,
    pub machine_id_hash: String,
    pub install_date: String,
    pub established_at: String,
}

#[derive(Debug, Clone)]
pub struct LicenseStatusRow {
    pub overall_validity: String,
    pub license_type: Option<String>,
    pub eval_expires_at: Option<String>,
    pub last_validated_at: Option<String>,
    pub modules_json: String,
    pub computed_at: String,
}

impl ProjectionStore {
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
            CREATE TABLE IF NOT EXISTS projection_metadata (
                projection_name TEXT PRIMARY KEY,
                last_event_id INTEGER NOT NULL DEFAULT 0,
                updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
            );
            CREATE TABLE IF NOT EXISTS practice_identity (
                id INTEGER PRIMARY KEY CHECK(id = 1),
                practice_id TEXT NOT NULL,
                machine_id_hash TEXT NOT NULL,
                install_date TEXT NOT NULL,
                established_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS license_status (
                id INTEGER PRIMARY KEY CHECK(id = 1),
                overall_validity TEXT NOT NULL,
                license_type TEXT,
                eval_expires_at TEXT,
                last_validated_at TEXT,
                modules_json TEXT NOT NULL DEFAULT '[]',
                computed_at TEXT NOT NULL
            );
        ")?;
        Ok(())
    }

    pub fn upsert_practice_identity(&self, row: &PracticeIdentityRow) -> Result<()> {
        self.conn.execute(
            "INSERT INTO practice_identity (id, practice_id, machine_id_hash, install_date, established_at)
             VALUES (1, ?1, ?2, ?3, ?4)
             ON CONFLICT(id) DO UPDATE SET
                 practice_id = excluded.practice_id,
                 machine_id_hash = excluded.machine_id_hash,
                 install_date = excluded.install_date,
                 established_at = excluded.established_at",
            params![row.practice_id, row.machine_id_hash, row.install_date, row.established_at],
        )?;
        Ok(())
    }

    pub fn get_practice_identity(&self) -> Result<Option<PracticeIdentityRow>> {
        let result: SqlResult<PracticeIdentityRow> = self.conn.query_row(
            "SELECT practice_id, machine_id_hash, install_date, established_at FROM practice_identity WHERE id = 1",
            [],
            |row| Ok(PracticeIdentityRow {
                practice_id: row.get(0)?,
                machine_id_hash: row.get(1)?,
                install_date: row.get(2)?,
                established_at: row.get(3)?,
            }),
        );
        match result {
            Ok(r) => Ok(Some(r)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn upsert_license_status(&self, row: &LicenseStatusRow) -> Result<()> {
        self.conn.execute(
            "INSERT INTO license_status (id, overall_validity, license_type, eval_expires_at, last_validated_at, modules_json, computed_at)
             VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(id) DO UPDATE SET
                 overall_validity = excluded.overall_validity,
                 license_type = excluded.license_type,
                 eval_expires_at = excluded.eval_expires_at,
                 last_validated_at = excluded.last_validated_at,
                 modules_json = excluded.modules_json,
                 computed_at = excluded.computed_at",
            params![
                row.overall_validity,
                row.license_type,
                row.eval_expires_at,
                row.last_validated_at,
                row.modules_json,
                row.computed_at
            ],
        )?;
        Ok(())
    }

    pub fn get_license_status(&self) -> Result<Option<LicenseStatusRow>> {
        let result: SqlResult<LicenseStatusRow> = self.conn.query_row(
            "SELECT overall_validity, license_type, eval_expires_at, last_validated_at, modules_json, computed_at
             FROM license_status WHERE id = 1",
            [],
            |row| Ok(LicenseStatusRow {
                overall_validity: row.get(0)?,
                license_type: row.get(1)?,
                eval_expires_at: row.get(2)?,
                last_validated_at: row.get(3)?,
                modules_json: row.get(4)?,
                computed_at: row.get(5)?,
            }),
        );
        match result {
            Ok(r) => Ok(Some(r)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Returns the last processed event id for a projection (0 if never set).
    pub fn get_position(&self, projection_name: &str) -> Result<i64> {
        let result: SqlResult<i64> = self.conn.query_row(
            "SELECT last_event_id FROM projection_metadata WHERE projection_name = ?1",
            params![projection_name],
            |row| row.get(0),
        );
        match result {
            Ok(id) => Ok(id),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(0),
            Err(e) => Err(e.into()),
        }
    }

    /// Updates the last processed event id for a projection. Idempotent.
    pub fn set_position(&self, projection_name: &str, event_id: i64) -> Result<()> {
        self.conn.execute(
            "INSERT INTO projection_metadata (projection_name, last_event_id, updated_at)
             VALUES (?1, ?2, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
             ON CONFLICT(projection_name) DO UPDATE SET
                 last_event_id = excluded.last_event_id,
                 updated_at = excluded.updated_at",
            params![projection_name, event_id],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> ProjectionStore {
        ProjectionStore::new_in_memory().unwrap()
    }

    #[test]
    fn test_get_position_returns_zero_when_not_set() {
        let s = store();
        assert_eq!(s.get_position("practice_identity").unwrap(), 0);
    }

    #[test]
    fn test_set_and_get_position() {
        let s = store();
        s.set_position("practice_identity", 42).unwrap();
        assert_eq!(s.get_position("practice_identity").unwrap(), 42);
    }

    #[test]
    fn test_set_position_is_idempotent() {
        let s = store();
        s.set_position("practice_identity", 10).unwrap();
        s.set_position("practice_identity", 10).unwrap();
        assert_eq!(s.get_position("practice_identity").unwrap(), 10);
    }

    #[test]
    fn test_set_position_updates() {
        let s = store();
        s.set_position("license_status", 5).unwrap();
        s.set_position("license_status", 20).unwrap();
        assert_eq!(s.get_position("license_status").unwrap(), 20);
    }

    #[test]
    fn test_multiple_projections_independent() {
        let s = store();
        s.set_position("practice_identity", 10).unwrap();
        s.set_position("license_status", 7).unwrap();
        assert_eq!(s.get_position("practice_identity").unwrap(), 10);
        assert_eq!(s.get_position("license_status").unwrap(), 7);
    }
}
