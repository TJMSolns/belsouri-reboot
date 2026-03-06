pub mod event_store;
pub mod projection_store;

pub use event_store::{EventStore, StoredEvent, EventStoreError};
pub use projection_store::{
    ProjectionStore, ProjectionStoreError,
    PracticeIdentityRow, LicenseStatusRow,
    PracticeSettingsRow, OfficeRow, OfficeHoursRow,
    ProcedureTypeRow,
    StaffMemberRow, StaffRoleRow,
    StaffOfficeAssignmentRow, StaffAvailabilityRow, StaffExceptionRow,
    PatientRow, PatientNoteRow,
    AppointmentRow, AppointmentNoteRow,
    StaffShiftRow,
};
