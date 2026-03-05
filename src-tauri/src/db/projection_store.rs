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
    pub address_line_1: Option<String>,
    pub address_line_2: Option<String>,
    pub city_town: Option<String>,
    pub subdivision: Option<String>,
    pub country: Option<String>,
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
    pub required_provider_type: Option<String>,
}

// ── Staff Management row types ────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct StaffMemberRow {
    pub staff_member_id: String,
    pub name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub preferred_contact_channel: Option<String>,
    pub pin_hash: Option<String>,
    pub archived: bool,
}

#[derive(Debug, Clone)]
pub struct StaffRoleRow {
    pub staff_member_id: String,
    pub role: String,
}

// ── Staff Shift row types ─────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct StaffShiftRow {
    pub shift_id: String,
    pub staff_member_id: String,
    pub staff_name: String,
    pub office_id: String,
    pub office_name: String,
    pub date: String,
    pub start_time: String,
    pub end_time: String,
    pub role: String,
    pub created_by: String,
    pub cancelled: bool,
    pub cancel_reason: Option<String>,
}

// ── Appointment row types ─────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AppointmentRow {
    pub appointment_id: String,
    pub office_id: String,
    pub patient_id: String,
    pub patient_name: String,
    pub patient_phone: Option<String>,
    pub patient_email: Option<String>,
    pub preferred_contact_channel: Option<String>,
    pub procedure_type_id: String,
    pub procedure_name: String,
    pub procedure_category: String,
    pub provider_id: String,
    pub provider_name: String,
    /// Local datetime "YYYY-MM-DDTHH:MM:SS"
    pub start_time: String,
    /// Local datetime "YYYY-MM-DDTHH:MM:SS"
    pub end_time: String,
    pub duration_minutes: u32,
    /// Booked | Completed | Cancelled | NoShow | Rescheduled
    pub status: String,
    pub rescheduled_to_id: Option<String>,
    pub rescheduled_from_id: Option<String>,
    pub booked_by: String,
}

#[derive(Debug, Clone)]
pub struct AppointmentNoteRow {
    pub note_id: String,
    pub appointment_id: String,
    pub text: String,
    pub recorded_by: String,
    pub recorded_at: String,
}

// ── Patient Management row types ──────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PatientRow {
    pub patient_id: String,
    pub first_name: String,
    pub last_name: String,
    pub full_name_display: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub preferred_contact_channel: Option<String>,
    pub preferred_office_id: Option<String>,
    pub date_of_birth: Option<String>,
    pub address_line_1: Option<String>,
    pub city_town: Option<String>,
    pub subdivision: Option<String>,
    pub country: Option<String>,
    pub registered_by: String,
    pub registered_at: String,
    pub archived: bool,
}

#[derive(Debug, Clone)]
pub struct PatientNoteRow {
    pub note_id: String,
    pub patient_id: String,
    pub text: String,
    pub recorded_by: String,
    pub recorded_at: String,
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
                archived INTEGER NOT NULL DEFAULT 0,
                address_line_1 TEXT,
                address_line_2 TEXT,
                city_town TEXT,
                subdivision TEXT,
                country TEXT
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
                is_active INTEGER NOT NULL DEFAULT 1,
                required_provider_type TEXT
            );
            CREATE TABLE IF NOT EXISTS staff_members (
                staff_member_id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                phone TEXT,
                email TEXT,
                preferred_contact_channel TEXT,
                pin_hash TEXT,
                archived INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE IF NOT EXISTS staff_member_roles (
                staff_member_id TEXT NOT NULL,
                role TEXT NOT NULL,
                PRIMARY KEY (staff_member_id, role)
            );
            CREATE TABLE IF NOT EXISTS patients (
                patient_id TEXT PRIMARY KEY,
                first_name TEXT NOT NULL,
                last_name TEXT NOT NULL,
                full_name_display TEXT NOT NULL,
                phone TEXT,
                email TEXT,
                preferred_contact_channel TEXT,
                preferred_office_id TEXT,
                date_of_birth TEXT,
                address_line_1 TEXT,
                city_town TEXT,
                subdivision TEXT,
                country TEXT,
                registered_by TEXT NOT NULL,
                registered_at TEXT NOT NULL,
                archived INTEGER NOT NULL DEFAULT 0
            );
            CREATE INDEX IF NOT EXISTS idx_patients_name
                ON patients(lower(last_name), lower(first_name));
            CREATE INDEX IF NOT EXISTS idx_patients_phone ON patients(phone);
            CREATE TABLE IF NOT EXISTS patient_notes (
                note_id TEXT PRIMARY KEY,
                patient_id TEXT NOT NULL,
                text TEXT NOT NULL,
                recorded_by TEXT NOT NULL,
                recorded_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_patient_notes_patient_id
                ON patient_notes(patient_id);
            CREATE TABLE IF NOT EXISTS appointment_list (
                appointment_id TEXT PRIMARY KEY,
                office_id TEXT NOT NULL,
                patient_id TEXT NOT NULL,
                patient_name TEXT NOT NULL,
                patient_phone TEXT,
                patient_email TEXT,
                preferred_contact_channel TEXT,
                procedure_type_id TEXT NOT NULL,
                procedure_name TEXT NOT NULL,
                procedure_category TEXT NOT NULL,
                provider_id TEXT NOT NULL,
                provider_name TEXT NOT NULL,
                start_time TEXT NOT NULL,
                end_time TEXT NOT NULL,
                duration_minutes INTEGER NOT NULL,
                status TEXT NOT NULL DEFAULT 'Booked',
                rescheduled_to_id TEXT,
                rescheduled_from_id TEXT,
                booked_by TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_appointment_list_office_start
                ON appointment_list(office_id, start_time);
            CREATE INDEX IF NOT EXISTS idx_appointment_list_patient
                ON appointment_list(patient_id);
            CREATE TABLE IF NOT EXISTS appointment_notes (
                note_id TEXT PRIMARY KEY,
                appointment_id TEXT NOT NULL,
                text TEXT NOT NULL,
                recorded_by TEXT NOT NULL,
                recorded_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_appointment_notes_appt
                ON appointment_notes(appointment_id);
            CREATE TABLE IF NOT EXISTS staff_shift_roster (
                shift_id TEXT PRIMARY KEY,
                staff_member_id TEXT NOT NULL,
                staff_name TEXT NOT NULL,
                office_id TEXT NOT NULL,
                office_name TEXT NOT NULL,
                date TEXT NOT NULL,
                start_time TEXT NOT NULL,
                end_time TEXT NOT NULL,
                role TEXT NOT NULL,
                created_by TEXT NOT NULL,
                cancelled INTEGER NOT NULL DEFAULT 0,
                cancel_reason TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_staff_shift_roster_date
                ON staff_shift_roster(date);
        ")?;
        // Migrations for existing databases (ignore "duplicate column" errors)
        let _ = self.conn.execute_batch("ALTER TABLE offices ADD COLUMN address_line_1 TEXT;");
        let _ = self.conn.execute_batch("ALTER TABLE offices ADD COLUMN address_line_2 TEXT;");
        let _ = self.conn.execute_batch("ALTER TABLE offices ADD COLUMN city_town TEXT;");
        let _ = self.conn.execute_batch("ALTER TABLE offices ADD COLUMN subdivision TEXT;");
        let _ = self.conn.execute_batch("ALTER TABLE offices ADD COLUMN country TEXT;");
        let _ = self.conn.execute_batch("ALTER TABLE procedure_types ADD COLUMN required_provider_type TEXT;");
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
            "INSERT INTO offices (id, name, chair_count, archived,
                 address_line_1, address_line_2, city_town, subdivision, country)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET
                 name = excluded.name, chair_count = excluded.chair_count,
                 archived = excluded.archived",
            params![row.id, row.name, row.chair_count, row.archived as i32,
                    row.address_line_1, row.address_line_2, row.city_town,
                    row.subdivision, row.country],
        )?;
        Ok(())
    }

    pub fn set_office_address(
        &self,
        id: &str,
        address_line_1: Option<&str>,
        address_line_2: Option<&str>,
        city_town: Option<&str>,
        subdivision: Option<&str>,
        country: Option<&str>,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE offices SET address_line_1 = ?2, address_line_2 = ?3,
             city_town = ?4, subdivision = ?5, country = ?6 WHERE id = ?1",
            params![id, address_line_1, address_line_2, city_town, subdivision, country],
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
            "SELECT id, name, chair_count, archived,
                    address_line_1, address_line_2, city_town, subdivision, country
             FROM offices WHERE id = ?1",
            params![id],
            |row| Ok(OfficeRow {
                id: row.get(0)?,
                name: row.get(1)?,
                chair_count: row.get::<_, i64>(2)? as u32,
                archived: row.get::<_, i32>(3)? != 0,
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

    pub fn list_offices(&self) -> Result<Vec<OfficeRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, chair_count, archived,
                    address_line_1, address_line_2, city_town, subdivision, country
             FROM offices ORDER BY name ASC"
        )?;
        let rows = stmt.query_map([], |row| Ok(OfficeRow {
            id: row.get(0)?,
            name: row.get(1)?,
            chair_count: row.get::<_, i64>(2)? as u32,
            archived: row.get::<_, i32>(3)? != 0,
            address_line_1: row.get(4)?,
            address_line_2: row.get(5)?,
            city_town: row.get(6)?,
            subdivision: row.get(7)?,
            country: row.get(8)?,
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

    pub fn list_providers_for_office(&self, office_id: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT provider_id FROM provider_office_assignments WHERE office_id = ?1"
        )?;
        let ids = stmt.query_map(params![office_id], |row| row.get(0))?
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
            "INSERT INTO procedure_types (id, name, category, default_duration_minutes, is_active, required_provider_type)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(id) DO UPDATE SET
                 name = excluded.name, category = excluded.category,
                 default_duration_minutes = excluded.default_duration_minutes,
                 is_active = excluded.is_active",
            params![row.id, row.name, row.category, row.default_duration_minutes, row.is_active as i32, row.required_provider_type],
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

    pub fn set_procedure_type_required_provider_type(
        &self,
        id: &str,
        required_provider_type: Option<&str>,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE procedure_types SET required_provider_type = ?2 WHERE id = ?1",
            params![id, required_provider_type],
        )?;
        Ok(())
    }

    pub fn get_procedure_type(&self, id: &str) -> Result<Option<ProcedureTypeRow>> {
        let r: SqlResult<ProcedureTypeRow> = self.conn.query_row(
            "SELECT id, name, category, default_duration_minutes, is_active, required_provider_type
             FROM procedure_types WHERE id = ?1",
            params![id],
            |row| Ok(ProcedureTypeRow {
                id: row.get(0)?,
                name: row.get(1)?,
                category: row.get(2)?,
                default_duration_minutes: row.get::<_, i64>(3)? as u32,
                is_active: row.get::<_, i32>(4)? != 0,
                required_provider_type: row.get(5)?,
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
            "SELECT id, name, category, default_duration_minutes, is_active, required_provider_type
             FROM procedure_types ORDER BY name ASC"
        )?;
        let rows = stmt.query_map([], |row| Ok(ProcedureTypeRow {
            id: row.get(0)?,
            name: row.get(1)?,
            category: row.get(2)?,
            default_duration_minutes: row.get::<_, i64>(3)? as u32,
            is_active: row.get::<_, i32>(4)? != 0,
            required_provider_type: row.get(5)?,
        }))?.collect::<SqlResult<Vec<_>>>()?;
        Ok(rows)
    }

    // ── Staff Management rows ─────────────────────────────────────────────────

    pub fn upsert_staff_member(&self, row: &StaffMemberRow) -> Result<()> {
        self.conn.execute(
            "INSERT INTO staff_members
             (staff_member_id, name, phone, email, preferred_contact_channel, pin_hash, archived)
             VALUES (?1,?2,?3,?4,?5,?6,?7)
             ON CONFLICT(staff_member_id) DO UPDATE SET
                 name = excluded.name, phone = excluded.phone, email = excluded.email,
                 preferred_contact_channel = excluded.preferred_contact_channel,
                 pin_hash = excluded.pin_hash, archived = excluded.archived",
            params![row.staff_member_id, row.name, row.phone, row.email,
                    row.preferred_contact_channel, row.pin_hash, row.archived as i32],
        )?;
        Ok(())
    }

    pub fn set_staff_member_pin(&self, staff_member_id: &str, pin_hash: Option<&str>) -> Result<()> {
        self.conn.execute(
            "UPDATE staff_members SET pin_hash = ?2 WHERE staff_member_id = ?1",
            params![staff_member_id, pin_hash],
        )?;
        Ok(())
    }

    pub fn set_staff_member_archived(&self, staff_member_id: &str, archived: bool) -> Result<()> {
        self.conn.execute(
            "UPDATE staff_members SET archived = ?2 WHERE staff_member_id = ?1",
            params![staff_member_id, archived as i32],
        )?;
        Ok(())
    }

    pub fn add_staff_role(&self, staff_member_id: &str, role: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO staff_member_roles (staff_member_id, role) VALUES (?1, ?2)",
            params![staff_member_id, role],
        )?;
        Ok(())
    }

    pub fn remove_staff_role(&self, staff_member_id: &str, role: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM staff_member_roles WHERE staff_member_id = ?1 AND role = ?2",
            params![staff_member_id, role],
        )?;
        Ok(())
    }

    pub fn list_staff_roles(&self, staff_member_id: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT role FROM staff_member_roles WHERE staff_member_id = ?1"
        )?;
        let roles = stmt.query_map(params![staff_member_id], |row| row.get(0))?
            .collect::<SqlResult<Vec<String>>>()?;
        Ok(roles)
    }

    pub fn get_staff_member(&self, staff_member_id: &str) -> Result<Option<StaffMemberRow>> {
        let r: SqlResult<StaffMemberRow> = self.conn.query_row(
            "SELECT staff_member_id, name, phone, email, preferred_contact_channel, pin_hash, archived
             FROM staff_members WHERE staff_member_id = ?1",
            params![staff_member_id],
            |row| Ok(StaffMemberRow {
                staff_member_id: row.get(0)?,
                name: row.get(1)?,
                phone: row.get(2)?,
                email: row.get(3)?,
                preferred_contact_channel: row.get(4)?,
                pin_hash: row.get(5)?,
                archived: row.get::<_, i32>(6)? != 0,
            }),
        );
        match r {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn list_staff_members(&self) -> Result<Vec<StaffMemberRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT staff_member_id, name, phone, email, preferred_contact_channel, pin_hash, archived
             FROM staff_members ORDER BY name ASC"
        )?;
        let rows = stmt.query_map([], |row| Ok(StaffMemberRow {
            staff_member_id: row.get(0)?,
            name: row.get(1)?,
            phone: row.get(2)?,
            email: row.get(3)?,
            preferred_contact_channel: row.get(4)?,
            pin_hash: row.get(5)?,
            archived: row.get::<_, i32>(6)? != 0,
        }))?.collect::<SqlResult<Vec<_>>>()?;
        Ok(rows)
    }

    /// Returns count of active (non-archived) PracticeManagers.
    pub fn count_active_practice_managers(&self) -> Result<i64> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM staff_members sm
             JOIN staff_member_roles r ON sm.staff_member_id = r.staff_member_id
             WHERE r.role = 'PracticeManager' AND sm.archived = 0",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// Returns true if there is at least one active PracticeManager with a PIN set.
    pub fn has_active_pm_with_pin(&self) -> Result<bool> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM staff_members sm
             JOIN staff_member_roles r ON sm.staff_member_id = r.staff_member_id
             WHERE r.role = 'PracticeManager' AND sm.archived = 0 AND sm.pin_hash IS NOT NULL",
            [],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    // ── Patient Management rows ───────────────────────────────────────────────

    pub fn upsert_patient(&self, row: &PatientRow) -> Result<()> {
        self.conn.execute(
            "INSERT INTO patients (
                 patient_id, first_name, last_name, full_name_display, phone, email,
                 preferred_contact_channel, preferred_office_id, date_of_birth,
                 address_line_1, city_town, subdivision, country,
                 registered_by, registered_at, archived)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)
             ON CONFLICT(patient_id) DO UPDATE SET
                 first_name = excluded.first_name,
                 last_name = excluded.last_name,
                 full_name_display = excluded.full_name_display,
                 phone = excluded.phone,
                 email = excluded.email,
                 preferred_contact_channel = excluded.preferred_contact_channel,
                 preferred_office_id = excluded.preferred_office_id,
                 date_of_birth = excluded.date_of_birth,
                 address_line_1 = excluded.address_line_1,
                 city_town = excluded.city_town,
                 subdivision = excluded.subdivision,
                 country = excluded.country,
                 registered_by = excluded.registered_by,
                 registered_at = excluded.registered_at,
                 archived = excluded.archived",
            params![
                row.patient_id, row.first_name, row.last_name, row.full_name_display,
                row.phone, row.email, row.preferred_contact_channel, row.preferred_office_id,
                row.date_of_birth, row.address_line_1, row.city_town, row.subdivision,
                row.country, row.registered_by, row.registered_at, row.archived as i32
            ],
        )?;
        Ok(())
    }

    pub fn update_patient_demographics(
        &self,
        patient_id: &str,
        first_name: &str,
        last_name: &str,
        full_name_display: &str,
        date_of_birth: Option<&str>,
        address_line_1: Option<&str>,
        city_town: Option<&str>,
        subdivision: Option<&str>,
        country: Option<&str>,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE patients SET
                 first_name = ?2, last_name = ?3, full_name_display = ?4,
                 date_of_birth = ?5, address_line_1 = ?6, city_town = ?7,
                 subdivision = ?8, country = ?9
             WHERE patient_id = ?1",
            params![patient_id, first_name, last_name, full_name_display,
                    date_of_birth, address_line_1, city_town, subdivision, country],
        )?;
        Ok(())
    }

    pub fn update_patient_contact_info(
        &self,
        patient_id: &str,
        phone: Option<&str>,
        email: Option<&str>,
        preferred_contact_channel: Option<&str>,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE patients SET phone = ?2, email = ?3, preferred_contact_channel = ?4
             WHERE patient_id = ?1",
            params![patient_id, phone, email, preferred_contact_channel],
        )?;
        Ok(())
    }

    pub fn set_patient_archived(&self, patient_id: &str, archived: bool) -> Result<()> {
        self.conn.execute(
            "UPDATE patients SET archived = ?2 WHERE patient_id = ?1",
            params![patient_id, archived as i32],
        )?;
        Ok(())
    }

    pub fn get_patient(&self, patient_id: &str) -> Result<Option<PatientRow>> {
        let r: SqlResult<PatientRow> = self.conn.query_row(
            "SELECT patient_id, first_name, last_name, full_name_display, phone, email,
                    preferred_contact_channel, preferred_office_id, date_of_birth,
                    address_line_1, city_town, subdivision, country,
                    registered_by, registered_at, archived
             FROM patients WHERE patient_id = ?1",
            params![patient_id],
            |row| Ok(PatientRow {
                patient_id: row.get(0)?,
                first_name: row.get(1)?,
                last_name: row.get(2)?,
                full_name_display: row.get(3)?,
                phone: row.get(4)?,
                email: row.get(5)?,
                preferred_contact_channel: row.get(6)?,
                preferred_office_id: row.get(7)?,
                date_of_birth: row.get(8)?,
                address_line_1: row.get(9)?,
                city_town: row.get(10)?,
                subdivision: row.get(11)?,
                country: row.get(12)?,
                registered_by: row.get(13)?,
                registered_at: row.get(14)?,
                archived: row.get::<_, i32>(15)? != 0,
            }),
        );
        match r {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Search patients. All filter params are optional (None = no filter).
    /// name_prefix matches start of first_name OR last_name (case-insensitive).
    /// phone_fragment is a substring match.
    pub fn search_patients(
        &self,
        name_prefix: Option<&str>,
        phone_fragment: Option<&str>,
        preferred_office_id: Option<&str>,
        include_archived: bool,
    ) -> Result<Vec<PatientRow>> {
        let name_pat = name_prefix.map(|s| format!("{}%", s.to_lowercase()));
        let phone_pat = phone_fragment.map(|s| format!("%{}%", s));
        let mut stmt = self.conn.prepare(
            "SELECT patient_id, first_name, last_name, full_name_display, phone, email,
                    preferred_contact_channel, preferred_office_id, date_of_birth,
                    address_line_1, city_town, subdivision, country,
                    registered_by, registered_at, archived
             FROM patients
             WHERE
                 (?1 IS NULL OR lower(first_name) LIKE ?1 OR lower(last_name) LIKE ?1)
                 AND (?2 IS NULL OR phone LIKE ?2)
                 AND (?3 IS NULL OR preferred_office_id = ?3)
                 AND (archived = 0 OR ?4 = 1)
             ORDER BY last_name ASC, first_name ASC
             LIMIT 200",
        )?;
        let rows = stmt.query_map(
            params![name_pat, phone_pat, preferred_office_id, include_archived as i32],
            |row| Ok(PatientRow {
                patient_id: row.get(0)?,
                first_name: row.get(1)?,
                last_name: row.get(2)?,
                full_name_display: row.get(3)?,
                phone: row.get(4)?,
                email: row.get(5)?,
                preferred_contact_channel: row.get(6)?,
                preferred_office_id: row.get(7)?,
                date_of_birth: row.get(8)?,
                address_line_1: row.get(9)?,
                city_town: row.get(10)?,
                subdivision: row.get(11)?,
                country: row.get(12)?,
                registered_by: row.get(13)?,
                registered_at: row.get(14)?,
                archived: row.get::<_, i32>(15)? != 0,
            }),
        )?.collect::<SqlResult<Vec<_>>>()?;
        Ok(rows)
    }

    /// Returns true if a non-archived patient with the same first+last name AND phone exists.
    pub fn check_duplicate_patient(
        &self,
        first_name: &str,
        last_name: &str,
        phone: &str,
    ) -> Result<bool> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM patients
             WHERE lower(first_name) = lower(?1)
               AND lower(last_name) = lower(?2)
               AND phone = ?3
               AND archived = 0",
            params![first_name, last_name, phone],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn add_patient_note(&self, row: &PatientNoteRow) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO patient_notes (note_id, patient_id, text, recorded_by, recorded_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![row.note_id, row.patient_id, row.text, row.recorded_by, row.recorded_at],
        )?;
        Ok(())
    }

    pub fn list_patient_notes(&self, patient_id: &str) -> Result<Vec<PatientNoteRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT note_id, patient_id, text, recorded_by, recorded_at
             FROM patient_notes WHERE patient_id = ?1
             ORDER BY recorded_at ASC",
        )?;
        let rows = stmt.query_map(params![patient_id], |row| Ok(PatientNoteRow {
            note_id: row.get(0)?,
            patient_id: row.get(1)?,
            text: row.get(2)?,
            recorded_by: row.get(3)?,
            recorded_at: row.get(4)?,
        }))?.collect::<SqlResult<Vec<_>>>()?;
        Ok(rows)
    }

    // ── Appointment rows ──────────────────────────────────────────────────────

    pub fn insert_appointment(&self, row: &AppointmentRow) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO appointment_list (
                 appointment_id, office_id, patient_id, patient_name, patient_phone,
                 patient_email, preferred_contact_channel,
                 procedure_type_id, procedure_name, procedure_category,
                 provider_id, provider_name,
                 start_time, end_time, duration_minutes,
                 status, rescheduled_to_id, rescheduled_from_id, booked_by)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19)",
            params![
                row.appointment_id, row.office_id, row.patient_id, row.patient_name,
                row.patient_phone, row.patient_email, row.preferred_contact_channel,
                row.procedure_type_id, row.procedure_name, row.procedure_category,
                row.provider_id, row.provider_name,
                row.start_time, row.end_time, row.duration_minutes,
                row.status, row.rescheduled_to_id, row.rescheduled_from_id, row.booked_by
            ],
        )?;
        Ok(())
    }

    pub fn update_appointment_status(
        &self,
        appointment_id: &str,
        status: &str,
        rescheduled_to_id: Option<&str>,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE appointment_list SET status = ?2, rescheduled_to_id = COALESCE(?3, rescheduled_to_id)
             WHERE appointment_id = ?1",
            params![appointment_id, status, rescheduled_to_id],
        )?;
        Ok(())
    }

    pub fn get_appointment(&self, appointment_id: &str) -> Result<Option<AppointmentRow>> {
        let r: SqlResult<AppointmentRow> = self.conn.query_row(
            "SELECT appointment_id, office_id, patient_id, patient_name, patient_phone,
                    patient_email, preferred_contact_channel,
                    procedure_type_id, procedure_name, procedure_category,
                    provider_id, provider_name,
                    start_time, end_time, duration_minutes,
                    status, rescheduled_to_id, rescheduled_from_id, booked_by
             FROM appointment_list WHERE appointment_id = ?1",
            params![appointment_id],
            |row| Ok(AppointmentRow {
                appointment_id: row.get(0)?,
                office_id: row.get(1)?,
                patient_id: row.get(2)?,
                patient_name: row.get(3)?,
                patient_phone: row.get(4)?,
                patient_email: row.get(5)?,
                preferred_contact_channel: row.get(6)?,
                procedure_type_id: row.get(7)?,
                procedure_name: row.get(8)?,
                procedure_category: row.get(9)?,
                provider_id: row.get(10)?,
                provider_name: row.get(11)?,
                start_time: row.get(12)?,
                end_time: row.get(13)?,
                duration_minutes: row.get::<_, i64>(14)? as u32,
                status: row.get(15)?,
                rescheduled_to_id: row.get(16)?,
                rescheduled_from_id: row.get(17)?,
                booked_by: row.get(18)?,
            }),
        );
        match r {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn map_appointment_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<AppointmentRow> {
        Ok(AppointmentRow {
            appointment_id: row.get(0)?,
            office_id: row.get(1)?,
            patient_id: row.get(2)?,
            patient_name: row.get(3)?,
            patient_phone: row.get(4)?,
            patient_email: row.get(5)?,
            preferred_contact_channel: row.get(6)?,
            procedure_type_id: row.get(7)?,
            procedure_name: row.get(8)?,
            procedure_category: row.get(9)?,
            provider_id: row.get(10)?,
            provider_name: row.get(11)?,
            start_time: row.get(12)?,
            end_time: row.get(13)?,
            duration_minutes: row.get::<_, i64>(14)? as u32,
            status: row.get(15)?,
            rescheduled_to_id: row.get(16)?,
            rescheduled_from_id: row.get(17)?,
            booked_by: row.get(18)?,
        })
    }

    /// Returns all appointments for an office on a given date (YYYY-MM-DD), all statuses.
    pub fn list_appointments_for_office_on_date(
        &self,
        office_id: &str,
        date: &str,
    ) -> Result<Vec<AppointmentRow>> {
        let start = format!("{}T00:00:00", date);
        let end   = format!("{}T23:59:59", date);
        let mut stmt = self.conn.prepare(
            "SELECT appointment_id, office_id, patient_id, patient_name, patient_phone,
                    patient_email, preferred_contact_channel,
                    procedure_type_id, procedure_name, procedure_category,
                    provider_id, provider_name,
                    start_time, end_time, duration_minutes,
                    status, rescheduled_to_id, rescheduled_from_id, booked_by
             FROM appointment_list
             WHERE office_id = ?1 AND start_time >= ?2 AND start_time <= ?3
             ORDER BY start_time ASC",
        )?;
        let rows = stmt.query_map(params![office_id, start, end], Self::map_appointment_row)?
            .collect::<SqlResult<Vec<_>>>()?;
        Ok(rows)
    }

    /// Count of Booked appointments at an office whose time window overlaps [start_time, end_time).
    /// Overlap = existing.start < proposed.end AND existing.end > proposed.start.
    /// Optionally exclude a specific appointment_id (used when rescheduling from same office).
    pub fn count_overlapping_booked(
        &self,
        office_id: &str,
        start_time: &str,
        end_time: &str,
        exclude_id: Option<&str>,
    ) -> Result<i64> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM appointment_list
             WHERE office_id = ?1
               AND status = 'Booked'
               AND start_time < ?3
               AND end_time > ?2
               AND (?4 IS NULL OR appointment_id != ?4)",
            params![office_id, start_time, end_time, exclude_id],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    pub fn add_appointment_note(&self, row: &AppointmentNoteRow) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO appointment_notes (note_id, appointment_id, text, recorded_by, recorded_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![row.note_id, row.appointment_id, row.text, row.recorded_by, row.recorded_at],
        )?;
        Ok(())
    }

    pub fn list_appointment_notes(&self, appointment_id: &str) -> Result<Vec<AppointmentNoteRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT note_id, appointment_id, text, recorded_by, recorded_at
             FROM appointment_notes WHERE appointment_id = ?1
             ORDER BY recorded_at ASC",
        )?;
        let rows = stmt.query_map(params![appointment_id], |row| Ok(AppointmentNoteRow {
            note_id: row.get(0)?,
            appointment_id: row.get(1)?,
            text: row.get(2)?,
            recorded_by: row.get(3)?,
            recorded_at: row.get(4)?,
        }))?.collect::<SqlResult<Vec<_>>>()?;
        Ok(rows)
    }

    /// Returns Booked appointments at an office for a specific date (YYYY-MM-DD), ordered by start_time.
    pub fn get_call_list(
        &self,
        office_id: &str,
        date: &str,
    ) -> Result<Vec<AppointmentRow>> {
        let start = format!("{}T00:00:00", date);
        let end   = format!("{}T23:59:59", date);
        let mut stmt = self.conn.prepare(
            "SELECT appointment_id, office_id, patient_id, patient_name, patient_phone,
                    patient_email, preferred_contact_channel,
                    procedure_type_id, procedure_name, procedure_category,
                    provider_id, provider_name,
                    start_time, end_time, duration_minutes,
                    status, rescheduled_to_id, rescheduled_from_id, booked_by
             FROM appointment_list
             WHERE office_id = ?1 AND start_time >= ?2 AND start_time <= ?3 AND status = 'Booked'
             ORDER BY start_time ASC",
        )?;
        let rows = stmt.query_map(params![office_id, start, end], Self::map_appointment_row)?
            .collect::<SqlResult<Vec<_>>>()?;
        Ok(rows)
    }

    /// Returns all appointments for a provider within a datetime range (inclusive).
    /// `start_dt` and `end_dt` are full datetime strings: "YYYY-MM-DDTHH:MM:SS".
    pub fn list_appointments_for_provider_in_range(
        &self,
        provider_id: &str,
        start_dt: &str,
        end_dt: &str,
    ) -> Result<Vec<AppointmentRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT appointment_id, office_id, patient_id, patient_name, patient_phone,
                    patient_email, preferred_contact_channel,
                    procedure_type_id, procedure_name, procedure_category,
                    provider_id, provider_name,
                    start_time, end_time, duration_minutes,
                    status, rescheduled_to_id, rescheduled_from_id, booked_by
             FROM appointment_list
             WHERE provider_id = ?1 AND start_time >= ?2 AND start_time <= ?3
             ORDER BY start_time ASC",
        )?;
        let rows = stmt.query_map(params![provider_id, start_dt, end_dt], Self::map_appointment_row)?
            .collect::<SqlResult<Vec<_>>>()?;
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

    // ── Staff Shift roster methods ────────────────────────────────────────────

    pub fn insert_staff_shift(&self, row: &StaffShiftRow) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO staff_shift_roster
             (shift_id, staff_member_id, staff_name, office_id, office_name,
              date, start_time, end_time, role, created_by, cancelled, cancel_reason)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                row.shift_id, row.staff_member_id, row.staff_name,
                row.office_id, row.office_name, row.date,
                row.start_time, row.end_time, row.role, row.created_by,
                row.cancelled as i32, row.cancel_reason
            ],
        )?;
        Ok(())
    }

    pub fn cancel_staff_shift(&self, shift_id: &str, cancel_reason: Option<&str>) -> Result<()> {
        self.conn.execute(
            "UPDATE staff_shift_roster SET cancelled = 1, cancel_reason = ?2 WHERE shift_id = ?1",
            params![shift_id, cancel_reason],
        )?;
        Ok(())
    }

    pub fn get_shift(&self, shift_id: &str) -> Result<Option<StaffShiftRow>> {
        let r: SqlResult<StaffShiftRow> = self.conn.query_row(
            "SELECT shift_id, staff_member_id, staff_name, office_id, office_name,
                    date, start_time, end_time, role, created_by, cancelled, cancel_reason
             FROM staff_shift_roster WHERE shift_id = ?1",
            params![shift_id],
            |row| Ok(StaffShiftRow {
                shift_id: row.get(0)?,
                staff_member_id: row.get(1)?,
                staff_name: row.get(2)?,
                office_id: row.get(3)?,
                office_name: row.get(4)?,
                date: row.get(5)?,
                start_time: row.get(6)?,
                end_time: row.get(7)?,
                role: row.get(8)?,
                created_by: row.get(9)?,
                cancelled: row.get::<_, i32>(10)? != 0,
                cancel_reason: row.get(11)?,
            }),
        );
        match r {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Returns shifts where date BETWEEN week_start AND week_end,
    /// optionally filtered by office_id.
    pub fn get_shifts_for_week(
        &self,
        week_start: &str,
        week_end: &str,
        office_id: Option<&str>,
    ) -> Result<Vec<StaffShiftRow>> {
        let map_row = |row: &rusqlite::Row<'_>| -> rusqlite::Result<StaffShiftRow> {
            Ok(StaffShiftRow {
                shift_id: row.get(0)?,
                staff_member_id: row.get(1)?,
                staff_name: row.get(2)?,
                office_id: row.get(3)?,
                office_name: row.get(4)?,
                date: row.get(5)?,
                start_time: row.get(6)?,
                end_time: row.get(7)?,
                role: row.get(8)?,
                created_by: row.get(9)?,
                cancelled: row.get::<_, i32>(10)? != 0,
                cancel_reason: row.get(11)?,
            })
        };

        let cols = "shift_id, staff_member_id, staff_name, office_id, office_name,
                    date, start_time, end_time, role, created_by, cancelled, cancel_reason";

        if let Some(oid) = office_id {
            let mut stmt = self.conn.prepare(&format!(
                "SELECT {} FROM staff_shift_roster
                 WHERE date >= ?1 AND date <= ?2 AND office_id = ?3
                 ORDER BY date ASC, start_time ASC",
                cols
            ))?;
            let rows = stmt.query_map(params![week_start, week_end, oid], map_row)?
                .collect::<SqlResult<Vec<_>>>()?;
            Ok(rows)
        } else {
            let mut stmt = self.conn.prepare(&format!(
                "SELECT {} FROM staff_shift_roster
                 WHERE date >= ?1 AND date <= ?2
                 ORDER BY date ASC, start_time ASC",
                cols
            ))?;
            let rows = stmt.query_map(params![week_start, week_end], map_row)?
                .collect::<SqlResult<Vec<_>>>()?;
            Ok(rows)
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
