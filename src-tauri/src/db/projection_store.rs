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

// ── Practice Setup row types ──────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PracticeSettingsRow {
    pub name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub address_line_1: Option<String>,
    pub address_line_2: Option<String>,
    pub city_town: Option<String>,
    pub subdivision: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, Clone)]
pub struct OfficeRow {
    pub id: String,
    pub name: String,
    pub chair_count: u32,
    pub archived: bool,
}

#[derive(Debug, Clone)]
pub struct OfficeHoursRow {
    pub office_id: String,
    pub day_of_week: String,
    pub open_time: String,
    pub close_time: String,
}

#[derive(Debug, Clone)]
pub struct ProviderRow {
    pub id: String,
    pub name: String,
    pub provider_type: String,
    pub archived: bool,
}

#[derive(Debug, Clone)]
pub struct ProviderAvailabilityRow {
    pub provider_id: String,
    pub office_id: String,
    pub day_of_week: String,
    pub start_time: String,
    pub end_time: String,
}

#[derive(Debug, Clone)]
pub struct ProviderExceptionRow {
    pub provider_id: String,
    pub start_date: String,
    pub end_date: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ProcedureTypeRow {
    pub id: String,
    pub name: String,
    pub category: String,
    pub default_duration_minutes: u32,
    pub is_active: bool,
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
            CREATE TABLE IF NOT EXISTS practice_settings (
                id INTEGER PRIMARY KEY CHECK(id = 1),
                name TEXT NOT NULL,
                phone TEXT,
                email TEXT,
                website TEXT,
                address_line_1 TEXT,
                address_line_2 TEXT,
                city_town TEXT,
                subdivision TEXT,
                country TEXT
            );
            CREATE TABLE IF NOT EXISTS offices (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                chair_count INTEGER NOT NULL,
                archived INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE IF NOT EXISTS office_hours (
                office_id TEXT NOT NULL,
                day_of_week TEXT NOT NULL,
                open_time TEXT NOT NULL,
                close_time TEXT NOT NULL,
                PRIMARY KEY (office_id, day_of_week)
            );
            CREATE TABLE IF NOT EXISTS providers (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                provider_type TEXT NOT NULL,
                archived INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE IF NOT EXISTS provider_office_assignments (
                provider_id TEXT NOT NULL,
                office_id TEXT NOT NULL,
                PRIMARY KEY (provider_id, office_id)
            );
            CREATE TABLE IF NOT EXISTS provider_availability (
                provider_id TEXT NOT NULL,
                office_id TEXT NOT NULL,
                day_of_week TEXT NOT NULL,
                start_time TEXT NOT NULL,
                end_time TEXT NOT NULL,
                PRIMARY KEY (provider_id, office_id, day_of_week)
            );
            CREATE TABLE IF NOT EXISTS provider_exceptions (
                provider_id TEXT NOT NULL,
                start_date TEXT NOT NULL,
                end_date TEXT NOT NULL,
                reason TEXT,
                PRIMARY KEY (provider_id, start_date, end_date)
            );
            CREATE TABLE IF NOT EXISTS procedure_types (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                category TEXT NOT NULL,
                default_duration_minutes INTEGER NOT NULL,
                is_active INTEGER NOT NULL DEFAULT 1
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

    // ── Practice Setup rows ───────────────────────────────────────────────────

    pub fn upsert_practice_settings(&self, row: &PracticeSettingsRow) -> Result<()> {
        self.conn.execute(
            "INSERT INTO practice_settings
             (id, name, phone, email, website, address_line_1, address_line_2, city_town, subdivision, country)
             VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET
                 name = excluded.name, phone = excluded.phone, email = excluded.email,
                 website = excluded.website, address_line_1 = excluded.address_line_1,
                 address_line_2 = excluded.address_line_2, city_town = excluded.city_town,
                 subdivision = excluded.subdivision, country = excluded.country",
            params![row.name, row.phone, row.email, row.website,
                    row.address_line_1, row.address_line_2, row.city_town,
                    row.subdivision, row.country],
        )?;
        Ok(())
    }

    pub fn get_practice_settings(&self) -> Result<Option<PracticeSettingsRow>> {
        let r: SqlResult<PracticeSettingsRow> = self.conn.query_row(
            "SELECT name, phone, email, website, address_line_1, address_line_2,
                    city_town, subdivision, country
             FROM practice_settings WHERE id = 1",
            [],
            |row| Ok(PracticeSettingsRow {
                name: row.get(0)?,
                phone: row.get(1)?,
                email: row.get(2)?,
                website: row.get(3)?,
                address_line_1: row.get(4)?,
                address_line_2: row.get(5)?,
                city_town: row.get(6)?,
                subdivision: row.get(7)?,
                country: row.get(8)?,
            }),
        );
        match r {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn upsert_office(&self, row: &OfficeRow) -> Result<()> {
        self.conn.execute(
            "INSERT INTO offices (id, name, chair_count, archived)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(id) DO UPDATE SET
                 name = excluded.name, chair_count = excluded.chair_count,
                 archived = excluded.archived",
            params![row.id, row.name, row.chair_count, row.archived as i32],
        )?;
        Ok(())
    }

    pub fn rename_office(&self, id: &str, new_name: &str) -> Result<()> {
        self.conn.execute("UPDATE offices SET name = ?2 WHERE id = ?1", params![id, new_name])?;
        Ok(())
    }

    pub fn update_office_chair_count(&self, id: &str, new_chair_count: u32) -> Result<()> {
        self.conn.execute(
            "UPDATE offices SET chair_count = ?2 WHERE id = ?1",
            params![id, new_chair_count],
        )?;
        Ok(())
    }

    pub fn archive_office(&self, id: &str) -> Result<()> {
        self.conn.execute("UPDATE offices SET archived = 1 WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_office(&self, id: &str) -> Result<Option<OfficeRow>> {
        let r: SqlResult<OfficeRow> = self.conn.query_row(
            "SELECT id, name, chair_count, archived FROM offices WHERE id = ?1",
            params![id],
            |row| Ok(OfficeRow {
                id: row.get(0)?,
                name: row.get(1)?,
                chair_count: row.get::<_, i64>(2)? as u32,
                archived: row.get::<_, i32>(3)? != 0,
            }),
        );
        match r {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn list_offices(&self) -> Result<Vec<OfficeRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, chair_count, archived FROM offices ORDER BY name ASC"
        )?;
        let rows = stmt.query_map([], |row| Ok(OfficeRow {
            id: row.get(0)?,
            name: row.get(1)?,
            chair_count: row.get::<_, i64>(2)? as u32,
            archived: row.get::<_, i32>(3)? != 0,
        }))?.collect::<SqlResult<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn set_office_hours(&self, office_id: &str, day: &str, open: &str, close: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO office_hours (office_id, day_of_week, open_time, close_time)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(office_id, day_of_week) DO UPDATE SET
                 open_time = excluded.open_time, close_time = excluded.close_time",
            params![office_id, day, open, close],
        )?;
        Ok(())
    }

    pub fn delete_office_hours(&self, office_id: &str, day: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM office_hours WHERE office_id = ?1 AND day_of_week = ?2",
            params![office_id, day],
        )?;
        Ok(())
    }

    pub fn list_office_hours(&self, office_id: &str) -> Result<Vec<OfficeHoursRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT office_id, day_of_week, open_time, close_time
             FROM office_hours WHERE office_id = ?1"
        )?;
        let rows = stmt.query_map(params![office_id], |row| Ok(OfficeHoursRow {
            office_id: row.get(0)?,
            day_of_week: row.get(1)?,
            open_time: row.get(2)?,
            close_time: row.get(3)?,
        }))?.collect::<SqlResult<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn upsert_provider(&self, row: &ProviderRow) -> Result<()> {
        self.conn.execute(
            "INSERT INTO providers (id, name, provider_type, archived)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(id) DO UPDATE SET
                 name = excluded.name, provider_type = excluded.provider_type,
                 archived = excluded.archived",
            params![row.id, row.name, row.provider_type, row.archived as i32],
        )?;
        Ok(())
    }

    pub fn rename_provider(&self, id: &str, new_name: &str) -> Result<()> {
        self.conn.execute("UPDATE providers SET name = ?2 WHERE id = ?1", params![id, new_name])?;
        Ok(())
    }

    pub fn update_provider_type(&self, id: &str, new_type: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE providers SET provider_type = ?2 WHERE id = ?1",
            params![id, new_type],
        )?;
        Ok(())
    }

    pub fn archive_provider(&self, id: &str) -> Result<()> {
        self.conn.execute("UPDATE providers SET archived = 1 WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn unarchive_provider(&self, id: &str) -> Result<()> {
        self.conn.execute("UPDATE providers SET archived = 0 WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_provider(&self, id: &str) -> Result<Option<ProviderRow>> {
        let r: SqlResult<ProviderRow> = self.conn.query_row(
            "SELECT id, name, provider_type, archived FROM providers WHERE id = ?1",
            params![id],
            |row| Ok(ProviderRow {
                id: row.get(0)?,
                name: row.get(1)?,
                provider_type: row.get(2)?,
                archived: row.get::<_, i32>(3)? != 0,
            }),
        );
        match r {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn list_providers(&self) -> Result<Vec<ProviderRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, provider_type, archived FROM providers ORDER BY name ASC"
        )?;
        let rows = stmt.query_map([], |row| Ok(ProviderRow {
            id: row.get(0)?,
            name: row.get(1)?,
            provider_type: row.get(2)?,
            archived: row.get::<_, i32>(3)? != 0,
        }))?.collect::<SqlResult<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn add_provider_office_assignment(&self, provider_id: &str, office_id: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO provider_office_assignments (provider_id, office_id)
             VALUES (?1, ?2)",
            params![provider_id, office_id],
        )?;
        Ok(())
    }

    pub fn remove_provider_office_assignment(&self, provider_id: &str, office_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM provider_office_assignments WHERE provider_id = ?1 AND office_id = ?2",
            params![provider_id, office_id],
        )?;
        Ok(())
    }

    pub fn list_provider_offices(&self, provider_id: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT office_id FROM provider_office_assignments WHERE provider_id = ?1"
        )?;
        let ids = stmt.query_map(params![provider_id], |row| row.get(0))?
            .collect::<SqlResult<Vec<String>>>()?;
        Ok(ids)
    }

    pub fn set_provider_availability(&self, row: &ProviderAvailabilityRow) -> Result<()> {
        self.conn.execute(
            "INSERT INTO provider_availability
             (provider_id, office_id, day_of_week, start_time, end_time)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(provider_id, office_id, day_of_week) DO UPDATE SET
                 start_time = excluded.start_time, end_time = excluded.end_time",
            params![row.provider_id, row.office_id, row.day_of_week, row.start_time, row.end_time],
        )?;
        Ok(())
    }

    pub fn delete_provider_availability(&self, provider_id: &str, office_id: &str, day: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM provider_availability
             WHERE provider_id = ?1 AND office_id = ?2 AND day_of_week = ?3",
            params![provider_id, office_id, day],
        )?;
        Ok(())
    }

    pub fn delete_provider_availability_for_office(&self, provider_id: &str, office_id: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT day_of_week FROM provider_availability
             WHERE provider_id = ?1 AND office_id = ?2"
        )?;
        let days: Vec<String> = stmt.query_map(params![provider_id, office_id], |row| row.get(0))?
            .collect::<SqlResult<Vec<String>>>()?;
        // Note: actual deletion happens via event projection (ProviderAvailabilityCleared)
        Ok(days)
    }

    pub fn list_provider_availability(&self, provider_id: &str) -> Result<Vec<ProviderAvailabilityRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT provider_id, office_id, day_of_week, start_time, end_time
             FROM provider_availability WHERE provider_id = ?1"
        )?;
        let rows = stmt.query_map(params![provider_id], |row| Ok(ProviderAvailabilityRow {
            provider_id: row.get(0)?,
            office_id: row.get(1)?,
            day_of_week: row.get(2)?,
            start_time: row.get(3)?,
            end_time: row.get(4)?,
        }))?.collect::<SqlResult<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn add_provider_exception(&self, row: &ProviderExceptionRow) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO provider_exceptions
             (provider_id, start_date, end_date, reason)
             VALUES (?1, ?2, ?3, ?4)",
            params![row.provider_id, row.start_date, row.end_date, row.reason],
        )?;
        Ok(())
    }

    pub fn remove_provider_exception(&self, provider_id: &str, start: &str, end: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM provider_exceptions
             WHERE provider_id = ?1 AND start_date = ?2 AND end_date = ?3",
            params![provider_id, start, end],
        )?;
        Ok(())
    }

    pub fn list_provider_exceptions(&self, provider_id: &str) -> Result<Vec<ProviderExceptionRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT provider_id, start_date, end_date, reason
             FROM provider_exceptions WHERE provider_id = ?1
             ORDER BY start_date ASC"
        )?;
        let rows = stmt.query_map(params![provider_id], |row| Ok(ProviderExceptionRow {
            provider_id: row.get(0)?,
            start_date: row.get(1)?,
            end_date: row.get(2)?,
            reason: row.get(3)?,
        }))?.collect::<SqlResult<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn upsert_procedure_type(&self, row: &ProcedureTypeRow) -> Result<()> {
        self.conn.execute(
            "INSERT INTO procedure_types (id, name, category, default_duration_minutes, is_active)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET
                 name = excluded.name, category = excluded.category,
                 default_duration_minutes = excluded.default_duration_minutes,
                 is_active = excluded.is_active",
            params![row.id, row.name, row.category, row.default_duration_minutes, row.is_active as i32],
        )?;
        Ok(())
    }

    pub fn apply_procedure_type_update(
        &self,
        id: &str,
        name: Option<&str>,
        category: Option<&str>,
        duration: Option<u32>,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE procedure_types SET
                 name = COALESCE(?2, name),
                 category = COALESCE(?3, category),
                 default_duration_minutes = COALESCE(?4, default_duration_minutes)
             WHERE id = ?1",
            params![id, name, category, duration],
        )?;
        Ok(())
    }

    pub fn set_procedure_type_active(&self, id: &str, active: bool) -> Result<()> {
        self.conn.execute(
            "UPDATE procedure_types SET is_active = ?2 WHERE id = ?1",
            params![id, active as i32],
        )?;
        Ok(())
    }

    pub fn get_procedure_type(&self, id: &str) -> Result<Option<ProcedureTypeRow>> {
        let r: SqlResult<ProcedureTypeRow> = self.conn.query_row(
            "SELECT id, name, category, default_duration_minutes, is_active
             FROM procedure_types WHERE id = ?1",
            params![id],
            |row| Ok(ProcedureTypeRow {
                id: row.get(0)?,
                name: row.get(1)?,
                category: row.get(2)?,
                default_duration_minutes: row.get::<_, i64>(3)? as u32,
                is_active: row.get::<_, i32>(4)? != 0,
            }),
        );
        match r {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn list_procedure_types(&self) -> Result<Vec<ProcedureTypeRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, category, default_duration_minutes, is_active
             FROM procedure_types ORDER BY name ASC"
        )?;
        let rows = stmt.query_map([], |row| Ok(ProcedureTypeRow {
            id: row.get(0)?,
            name: row.get(1)?,
            category: row.get(2)?,
            default_duration_minutes: row.get::<_, i64>(3)? as u32,
            is_active: row.get::<_, i32>(4)? != 0,
        }))?.collect::<SqlResult<Vec<_>>>()?;
        Ok(rows)
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
