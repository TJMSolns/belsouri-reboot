pub mod event_store;
pub mod projection_store;

pub use event_store::{EventStore, StoredEvent, EventStoreError};
pub use projection_store::{
    ProjectionStore, ProjectionStoreError,
    PracticeIdentityRow, LicenseStatusRow,
    PracticeSettingsRow, OfficeRow, OfficeHoursRow,
    ProviderRow, ProviderAvailabilityRow, ProviderExceptionRow,
    ProcedureTypeRow,
    StaffMemberRow, StaffRoleRow,
    PatientRow, PatientNoteRow,
    AppointmentRow, AppointmentNoteRow,
};
