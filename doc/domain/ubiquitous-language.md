# Ubiquitous Language

Living glossary of domain terms. Updated during Event Storming and throughout development. These terms must be used consistently in code, tests, documentation, and conversation.

**Rule**: If a term isn't in this glossary, it shouldn't be in the codebase. If you need a new term, add it here first.

---

## Practice Setup Context

### Core Entities

| Term | Definition | Not This | Notes |
|------|-----------|----------|-------|
| **Practice** | The root organizational entity representing a dental clinic business. Singleton per installation. Holds identity (name, contact info, address). | "clinic", "company", "organization" | A practice may operate from multiple offices. Nico refers to "the practice" as the whole business. |
| **Office** | A physical location where dental services are provided. Each office has its own name, chair capacity, and operating hours. | "location", "branch", "site", "clinic" | Nico uses geographic names: "Kingston", "Montego Bay". Offices are independently configured. |
| **Provider** | Any dental professional who provides services to patients. Covers dentists, hygienists, and specialists. | "staff", "doctor", "employee", "clinician", "staff member" | Nico confirmed: "provider is a word we use and covers all staff roles." This is the single unified term for all clinical personnel. |
| **Procedure Type** | A named category of dental service that can be scheduled, with a default duration and color-coded category. | "service", "treatment", "appointment type" | Examples: Cleaning, Consultation, Root Canal. Each has a category and default duration. |

### Value Objects

| Term | Definition | Not This | Notes |
|------|-----------|----------|-------|
| **Chair** | A physical treatment station within an office. Chair count is the capacity constraint that limits concurrent appointments. | "room", "operatory", "station" | Nico: "how many chairs" = office capacity. Not a bookable resource itself -- just a count that constrains scheduling. |
| **Provider Type** | Classification of a provider's clinical role: Dentist, Hygienist, or Specialist. | "role", "job title", "staff type" | Nico confirmed these three types are sufficient. |
| **Procedure Category** | A color-coded grouping of procedure types for calendar display and workflow organization. | "procedure group", "service type" | Six categories: Consult (yellow), Preventive (blue), Restorative (green), Invasive (red), Cosmetic (purple), Diagnostic (gray). Nico confirmed all appropriate. |
| **Operating Hours** | The time range during which an office is open on a given day of the week. | "business hours", "opening times", "schedule" | Per-office, per-day. Days without configured hours are considered closed. |
| **Availability** | A time window during which a provider works at a specific office on a given day of the week. | "schedule", "shift", "working hours" | Office-scoped -- same provider can have different availability at different offices. Must not overlap across offices on the same day. |
| **Exception** | A date range during which a provider is unavailable, overriding normal weekly availability. | "time off", "leave", "absence", "vacation" | Used for vacations, holidays, off-season closures. Jamaica dental care is seasonal (insurance-driven). Existing appointments are warned about but not auto-cancelled. |
| **Subdivision** | The administrative region within a country used for addresses. Parish in Jamaica; varies by Caribbean country. | "state", "province", "county" | Country-aware label. MVP uses "Parish" for Jamaica. |
| **Default Duration** | The standard time in minutes allocated for a procedure type when scheduling. | "appointment length", "time slot" | Range: 15-240 minutes. Can be overridden per appointment. |

### Actions and Concepts

| Term | Definition | Not This | Notes |
|------|-----------|----------|-------|
| **Register** (a provider) | Add a new provider to the practice with name and type. | "create", "add", "onboard" | Domain language: providers are "registered", not "created". |
| **Assign** (provider to office) | Link a provider to an office, enabling availability to be set there. | "add to", "attach", "associate" | Must happen before setting availability. Removal auto-clears availability at that office. |
| **Archive** | Soft-delete an entity (office or provider), hiding it from active lists but preserving historical data. | "delete", "remove", "deactivate" | Event sourcing is append-only -- no hard deletes. Archived entities can be unarchived. |
| **Define** (a procedure type) | Create a new procedure type with name, category, and default duration. | "create", "add" | Domain language: procedure types are "defined", not "created". |
| **Deactivate** (a procedure type) | Remove a procedure type from the active list without deleting it. | "delete", "archive", "remove" | Procedure types use deactivate/reactivate, not archive/unarchive, because they are configuration items not people/places. |
| **Practice Manager** | Role with global authority over all practice configuration. | "admin", "super user", "owner" | Full Practice Setup access. Can delegate permissions to Office Managers (future enhancement). |
| **Office Manager** | Role with authority scoped to a single office. | "local admin", "branch manager" | Scope of delegated permissions controlled by Practice Manager. MVP: Practice Manager only; delegation is future. |

### Bounded Context Terms

| Term | Definition | Not This | Notes |
|------|-----------|----------|-------|
| **Practice Setup** | The bounded context responsible for configuring the static structure of a practice: offices, providers, procedure types, and practice identity. | "admin", "settings", "configuration" | Foundational context -- all other contexts depend on it. Purely upstream (emits events, does not consume from other contexts). |
| **Bounded Context** | A logical boundary around a cohesive set of domain concepts with their own language and rules. | "module", "service", "component" | Each context has its own aggregates, events, and invariants. |
| **Aggregate** | A cluster of domain objects treated as a single unit for data changes, with an aggregate root that enforces invariants. | "entity", "model", "record" | Practice Setup has 4 aggregates: Practice, Office, Provider, ProcedureType. |
| **Domain Event** | An immutable record of something that happened in the domain. Stored in the event store and used to build projections. | "log entry", "notification", "message" | Named in past tense: OfficeCreated, ProviderRegistered. Append-only -- never deleted or modified. |
| **Projection** | A queryable read model materialized from domain events. Built incrementally, never from full rebuild. | "view", "cache", "snapshot" | Must be deterministic and incremental. O(n^2) full rebuilds caused performance collapse in belsouri-old. |
| **Command** | A request to change domain state. Validated against invariants before producing events. | "action", "request", "mutation" | Named in imperative: CreateOffice, RegisterProvider. May be rejected if invariants are violated. |

---

## Licensing Context

### Core Entities

| Term | Definition | Not This | Notes |
|------|-----------|----------|-------|
| **PracticeIdentity** | The immutable binding between a specific hardware installation and the application. Established exactly once on first run; never changes. Answers "who is this installation?" | "installation record", "machine record" | Singleton aggregate. The `practiceId` it produces is embedded in every license key and verified on activation. |
| **License** | The aggregate representing the application's authorization to operate. Tracks status per module and enforces the principle: read access is always granted; write access depends on module status. | "subscription", "account", "activation" | Singleton aggregate per installation. One License; multiple modules within it. |

### Value Objects

| Term | Definition | Not This | Notes |
|------|-----------|----------|-------|
| **PracticeId** | A derived, machine-bound identifier: `lowercase_hex(SHA-256(machineId_utf8 \|\| ":" \|\| installDate_utf8))`. Embedded in every license key and verified on activation. | "tenant ID", "clinic ID", "site ID" | 64-character lowercase hex string. Computed from MachineId and InstallDate. Stable as long as the hardware and install date are unchanged. |
| **MachineId** | The raw hardware identifier read from the OS on first run via the `machine-uid` crate (MachineGuid on Windows). Never stored — only its SHA-256 hash is persisted. | "hardware ID", "device ID" | Read once on first run. Used to compute PracticeId and MachineIdHash. Never written to the event store. |
| **InstallDate** | The UTC date of the application's first launch. Fixed permanently at first run. Part of the PracticeId computation. | "activation date", "registration date" | YYYY-MM-DD format. Recorded in the PracticeIdentityEstablished event. |
| **LicenseKey** | A base64url-encoded string carrying a JSON payload and an Ed25519 signature. Issued by the License Server. Entered manually by the Practice Manager to activate or renew the license. | "activation code", "serial number", "product key" | Format: `base64url(payload_json \|\| signature_bytes)`. The payload embeds PracticeId, modules, expiry dates, and grace period per module. |
| **LicenseType** | Classifies a license as either `Eval` (30-day free trial) or `Paid` (purchased). | "tier", "plan", "edition" | Determines the initial state. Eval has no grace period — it expires directly to Expired. |
| **EvalPeriod** | The 30-day free trial period that begins on first run. All modules are included. No grace period after eval expires. | "trial period", "demo mode" | Starts with EvalStarted event. Ends when eval_expires_at is reached. |
| **GracePeriod** | The period after a paid module's expiry date during which the module enters Degraded mode. Write access is blocked; read access continues. | "buffer period", "leniency window" | Per-module. Length is set by GracePeriodDays embedded in the license payload. Not hardcoded in the application. |
| **GracePeriodDays** | The number of days after a module's `expires_at` during which the module is Degraded rather than Expired. Embedded in the license payload per module. | "grace days" | Application does not hardcode criticality. A module with `grace_period_days = 0` goes straight to Expired. |
| **LicenseStatus** | The overall validity state of the license installation: Eval, Active, Degraded, Expired, or Invalid. | "account status" | Invalid is session-only — triggered by ClockRollbackDetected; clears at next startup with correct clock. |
| **LicenseModule** | A named feature set within a license payload that has its own expiry date and grace period. Module statuses are independent. | "feature", "add-on", "plan feature" | MVP module: "scheduling" (covers Practice Setup, Patient Mgmt, Staff, Patient Scheduling). |
| **SchemaVersion** | A version number in the license payload that identifies the structure of the payload JSON. The application rejects unknown schema versions. | "version", "format version" | Current schema version: 2. |

### States

| Term | Definition | Not This | Notes |
|------|-----------|----------|-------|
| **Eval** (license state) | The application is in the 30-day free trial. Full read/write access to all modules. | "demo", "trial mode" | All modules included. No grace period after eval expires. |
| **Active** (module state) | The module is within its paid license period. Full read/write access. | "valid", "licensed" | Requires overall validity = Valid (no clock rollback). |
| **Degraded** (module state) | A paid module has passed its `expires_at` but is within its grace period. Read access continues; write access is blocked. | "limited", "restricted", "read-only mode" | The application surface is fully usable for viewing. New data cannot be entered. |
| **Expired** (license/module state) | The eval period or module grace period has been exhausted. Read access continues; write access blocked; renewal prompt always visible. | "inactive", "lapsed" | Read access is unconditional — data is always theirs. |
| **Invalid** (license state) | A clock rollback >24h before the last LicenseValidationSucceeded timestamp was detected on startup. All modules are read-only for the session. | "corrupted", "tampered" | Session-only state. Cleared at next startup when clock is corrected. |
| **Degraded mode** | The application operating mode when one or more modules are in Degraded or Expired state. The UI shows a status banner warning; writes to those modules are blocked. | "read-only mode", "offline mode" | Not a single flag — individual modules degrade independently. |

### Actions and Concepts

| Term | Definition | Not This | Notes |
|------|-----------|----------|-------|
| **License Server** | The external system (operated by Tony) that signs and issues LicenseKeys. The application never makes runtime API calls to it — users obtain a key and enter it manually. | "license service", "activation server" | Private Ed25519 key is on the License Server only. Application embeds the public key for offline verification. |
| **Activate** (a license) | Enter a new LicenseKey for the first time on an installation, transitioning from Eval to Active. | "register", "unlock" | Command: ActivateLicense. Emits LicenseIssued. Verifies Ed25519 signature and PracticeId match. |
| **Renew** (a license) | Enter a replacement LicenseKey to extend or restore module access. Accepted from any license status. | "reactivate", "extend" | Command: RenewLicense. Emits LicenseRenewed. Write access restored at next startup. |
| **Startup Validation** | The enforcement gate that runs on every application launch. Checks clock rollback, evaluates module statuses, emits LicenseDegraded/LicenseExpired events as needed, and records a validation timestamp for anti-rollback. | "license check" | The only point where write access restrictions change. 48h background check updates the banner only. |
| **Anti-Rollback Check** | The startup check that compares the current system clock to the last LicenseValidationSucceeded timestamp. If the clock is >24h behind, ClockRollbackDetected is emitted and all writes are blocked for the session. | "tamper check" | Prevents backdating the clock to avoid expiry. |

### Bounded Context Terms

| Term | Definition | Not This | Notes |
|------|-----------|----------|-------|
| **Licensing** | The bounded context that controls whether the application is in a valid operational state. Upstream of every other bounded context — all feature access depends on license status. | "auth", "billing", "subscription management" | Two aggregates: PracticeIdentity (immutable machine binding) and License (lifecycle state machine). |

---

## Staff Management Context

### Core Entities

| Term | Definition | Not This | Notes |
|------|-----------|----------|-------|
| **StaffMember** | A person who works at the practice in any capacity. Holds one or more roles and authenticates via a PIN for quick identity switching on a shared workstation. | "user", "employee", "user account" | Staff Management is thin: identity + roles + PIN only. Not an HR system. Does not track schedules, hours, or payroll. |

### Value Objects

| Term | Definition | Not This | Notes |
|------|-----------|----------|-------|
| **Role** | A classification of a StaffMember's authority within the application: PracticeManager, Provider, or Staff. A staff member may hold multiple roles simultaneously. | "permission", "user type", "access level" | Three roles at MVP. PracticeManager = full config authority. Provider = clinical scheduling resource. Staff = non-clinical (receptionist, administrator). |
| **PIN** | A short numeric code used for quick local identity switching on a shared workstation. Not a password. No remote authentication or session management. | "password", "passcode", "login credential" | Hashed before storage (bcrypt or Argon2). Raw PIN never stored. Required before a StaffMember can be set as the active identity. |
| **Active Identity** | The StaffMember currently operating the application, established by PIN verification. Provides attribution for all domain commands. | "current user", "logged-in user", "session user" | Not a domain event — identity switching is a session concern. Provides the `staff_member_id` required by all write commands. |
| **PreferredContactChannel** | The default communication channel for contacting a person: WhatsApp, SMS, Phone, or Email. WhatsApp is the default. | "contact method", "notification preference" | Shared value object used by StaffMember, Patient, and Practice. WhatsApp default reflects Caribbean communication norms. |

### Actions and Concepts

| Term | Definition | Not This | Notes |
|------|-----------|----------|-------|
| **Register** (a staff member) | Add a new StaffMember to the practice with a name and initial role. | "create", "add", "invite", "onboard" | Domain language: staff members are "registered". Requires an existing active Practice Manager (except first-run bootstrap). |
| **Claim** (Practice Manager role) | The first-run action by which the first person at the keyboard declares themselves Practice Manager before any PM exists. Bypasses the "requires existing PM" precondition. | "register", "self-assign" | Emits PracticeManagerClaimed — a distinct event from RoleAssigned, for clear audit trail. Can only happen once. |
| **Set PIN** | Establish a PIN for a StaffMember who has none yet. Required before the staff member can switch to active identity. | "create password", "set password" | Command: SetPIN. Emits PINSet. Raw PIN is hashed at the command layer before the event is stored. |
| **Change PIN** | Replace an existing PIN after verifying the current one. | "reset password", "update password" | Command: ChangePIN. Requires current PIN verification. |
| **Switch** (active identity) | Verify a staff member's PIN and make them the current active identity operating the application. | "log in", "sign in" | Not a domain event — a session concern. The identity established by switching provides attribution for all subsequent commands. |
| **Last Practice Manager Guard** | The invariant that prevents archiving a StaffMember or removing the PracticeManager role if that person is the last active Practice Manager. Ensures the role is never vacant. | "PM lock", "admin lock" | Enforced at the command layer on ArchiveStaffMember and RemoveRole(PracticeManager). |

### Bounded Context Terms

| Term | Definition | Not This | Notes |
|------|-----------|----------|-------|
| **Staff Management** | The bounded context that manages the identity, roles, and PIN-based quick switching for all people who work at the practice. Entry point for "who is acting right now." | "HR", "user management", "authentication" | Thin by design. Upstream of all other contexts that need actor identity. Not an HR system. |

### Staff Shift Terms (SCH-5 — confirmed by Tony 2026-03-05)

| Term | Definition | Not This | Notes |
|------|-----------|----------|-------|
| **Shift** | A planned working period for a non-clinical staff member at a specific office on a specific date. Contains: who (staff_member_id), when (date + start_time + end_time), where (office_id), and which role they are performing. Created ad-hoc by either the staff member themselves or a Practice Manager. | "schedule", "rota", "assignment", "availability", "timetable" | Domain language: shifts are "planned" (verb: Plan). Not "created" or "scheduled". StaffShift is an aggregate in the Staff Management context. |
| **Shift Roster** | A view of all planned shifts for a given time period (typically a week), showing who is working, when, and where across all non-clinical staff. Rendered on the Schedule page Roster tab. | "staff schedule", "weekly schedule", "timetable" | Query pattern: `WHERE date >= week_start AND date <= week_end`. Cancelled shifts are shown greyed out, not removed. |
| **Plan a Shift** | The domain action of creating a new planned working period for a non-clinical staff member. | "create a shift", "schedule a shift", "add a shift" | Command: PlanStaffShift. Produces: StaffShiftPlanned event. Domain verb is "Plan" — consistent with the practice manager planning their team's week. |
| **Cancel a Shift** | The domain action of marking a planned shift as no longer occurring. The shift record and its history are preserved. | "delete a shift", "remove a shift", "unschedule" | Command: CancelStaffShift. Produces: StaffShiftCancelled event. Soft cancel — event store is append-only. Canceller must be the shift owner or a Practice Manager. |

---

## Staff Scheduling Context

### Core Entities

| Term | Definition | Not This | Notes |
|------|-----------|----------|-------|
| **ResolvedSchedule** | The materialized projection that answers "Is provider X available at office Y on date D at time T?" Built from Practice Setup events; pre-materialized 90 days forward. | "schedule", "calendar", "availability matrix" | The authoritative answer to provider availability queries. Applies availability windows, exceptions, office hours, and assignment status. Consumed by Patient Scheduling to validate bookings. |
| **OfficeProviderView** | The "today's schedule" projection showing which providers are working at each office on a given date and during what hours. | "daily roster", "staff schedule" | Filtered view of ResolvedSchedule (available = true records only). Consumed by the UI for the daily schedule display. |

### Value Objects

| Term | Definition | Not This | Notes |
|------|-----------|----------|-------|
| **AvailabilityException** | A date range during which a provider is unavailable, overriding their normal weekly availability window. Applied during ResolvedSchedule computation to mark days as blocked. | "time off", "leave", "absence" | Configured in Practice Setup (ProviderExceptionSet event). Applied in Staff Scheduling's resolution algorithm. |
| **blocked_reason** | The field on a ResolvedSchedule row that explains why a provider is unavailable: "exception", "not assigned", "no availability", or null (available). | "unavailability reason", "block type" | Used to generate meaningful error messages when booking is rejected. |
| **ScheduleEntry** | A single provider's working window at an office on a date: provider_id, provider_name, start_time, end_time. Returned by the office schedule query. | "shift entry", "roster entry" | Value object in the OfficeProviderView query response. |
| **ScheduleDay** | A single day in a provider's week schedule: date, office, start_time, end_time, and whether it is an exception day. Returned by the provider week schedule query. | "work day", "shift day" | Value object in the provider week view query response. |

### Actions and Concepts

| Term | Definition | Not This | Notes |
|------|-----------|----------|-------|
| **Resolve** (schedule) | The algorithm that combines a provider's weekly availability, their exceptions, and office hours to compute whether they are available on a specific date and time. | "calculate", "compute schedule" | Step-by-step: (1) check weekly window, (2) apply office hours, (3) apply exceptions, (4) check assignment, (5) check archive status. |
| **Pre-materialization** | The strategy of building the ResolvedSchedule projection eagerly for a 90-day forward window, rather than computing availability on-demand per query. Avoids performance issues on old hardware. | "eager loading", "cache warming" | D2 design decision. Keeps booking validation fast on 4GB RAM, 8-year-old CPUs. |

### Bounded Context Terms

| Term | Definition | Not This | Notes |
|------|-----------|----------|-------|
| **Staff Scheduling** | The bounded context that materializes a queryable resolved schedule from Practice Setup events and provides availability answers to Patient Scheduling. At MVP, projection-first — no new aggregates. | "scheduling engine", "roster management" | Upstream: Practice Setup (availability data). Downstream: Patient Scheduling (availability queries), UI (daily schedule view). |
| **Projection-First Context** | A bounded context that owns no aggregates producing new events at MVP; instead, it subscribes to upstream events and materializes queryable views. Staff Scheduling is this pattern at MVP. | "read-only context", "view context" | Expected to grow into an aggregate-owning context when time-off request workflows are added in a future phase. |

---

## Patient Management Context

### Core Entities

| Term | Definition | Not This | Notes |
|------|-----------|----------|-------|
| **Patient** | A person who receives dental care at the practice. The Patient aggregate holds demographic identity and contact information. Not a clinical record — clinical history belongs to the future Clinical Records context. | "client", "customer", "visitor" | Practice-wide record (not bound to one office). Thin at MVP: name, contact info, date of birth, address, and notes. |

### Value Objects

| Term | Definition | Not This | Notes |
|------|-----------|----------|-------|
| **PatientId** | System-generated UUID that uniquely identifies a patient record within the practice. Referenced by Patient Scheduling when booking appointments. | "patient number", "chart number", "MRN" | UUID, system-generated at RegisterPatient. Used as a foreign key in Patient Scheduling (appointment.patient_id). |
| **PatientNote** | An append-only, staff-attributed text note attached to a patient record. Cannot be edited or deleted — audit trail by design. Travels with the patient across all contexts (Scheduling, Clinical Records). | "comment", "memo", "annotation" | Value object with: note_id, text, recorded_by (staff_member_id), recorded_at. Nico: "when staff edits patients we need to audit that who did it when." |
| **Single-Name Convention** | The practice convention for patients who have only one name (a Caribbean cultural norm). Both first_name and last_name are required fields; for single-name patients, last_name uses "." as a placeholder. | (no alias) | Open question — Tony to confirm the actual convention used at Nico's practice. The "." placeholder is the current assumption. |
| **Preferred Office** | An optional reference from a patient record to the office where they typically receive care. A hint for filtering, not a hard binding — a patient can receive care at any office. | "home office", "assigned office" | Field: preferred_office_id. Patient records are practice-wide. Preferred office enables the "show patients for Kingston" filter without restricting bookings. |
| **Soft Warning (duplicate)** | The user-facing alert shown when a patient is registered with the same full name and phone number as an existing patient. Not a hard block — front desk confirms intent. | "duplicate error", "conflict" | No merge workflow at MVP. The soft warning prevents accidental duplicates. |

### Actions and Concepts

| Term | Definition | Not This | Notes |
|------|-----------|----------|-------|
| **Register** (a patient) | Create a new patient record with at minimum a name and one contact method (phone or email). | "create", "add", "enroll" | Domain language: patients are "registered", not "created". Command: RegisterPatient. |
| **Full Audit Trail** | The requirement that every write command on the Patient aggregate carries the staff_member_id of who performed the action (registered_by, updated_by, recorded_by, archived_by). No anonymous writes. | "logging", "change history" | Explicitly requested by Nico: "when staff edits patients we need to audit that who did it when." |

### Bounded Context Terms

| Term | Definition | Not This | Notes |
|------|-----------|----------|-------|
| **Patient Management** | The bounded context responsible for knowing who the patients are: registering patients, maintaining demographics and contact info, and providing patient identity to Patient Scheduling for booking. | "EHR", "patient records", "CRM" | Thin at MVP: demographics, contact info, notes. Clinical records are post-MVP. Upstream of Patient Scheduling. |

---

## Patient Scheduling Context

### Core Entities

| Term | Definition | Not This | Notes |
|------|-----------|----------|-------|
| **Appointment** | A scheduled visit by a patient for a specific procedure with a specific provider at a specific office, within a defined time window. The Appointment aggregate enforces all five booking constraints. | "booking", "visit", "session" | Domain language: appointments are "booked" (not created). The most downstream aggregate — depends on Practice Setup, Staff Scheduling, Patient Management, Staff Management, and Licensing. |

### Value Objects

| Term | Definition | Not This | Notes |
|------|-----------|----------|-------|
| **AppointmentId** | System-generated UUID identifying a specific appointment. When an appointment is rescheduled, the original and the replacement are separate aggregates with separate IDs, linked by rescheduled_to_id / rescheduled_from_id. | "booking reference", "appointment number" | UUID, system-generated at BookAppointment. |
| **AppointmentStatus** | The lifecycle state of an appointment: Booked, Completed, Cancelled, NoShow, or Rescheduled. Transitions are one-way — terminal statuses cannot be reversed. | "booking status", "visit status" | Booked is the only non-terminal state. Completed, Cancelled, NoShow, and Rescheduled are terminal. |
| **TimeWindow** | The start_time + end_time + duration_minutes triple that defines when an appointment occupies a provider and a chair at an office. Duration defaults from the procedure type and is overridable within 15-240 minutes. | "time slot", "block", "appointment time" | Chair capacity is checked against overlapping time windows: `existing.start < proposed.end AND existing.end > proposed.start`. |
| **NoShow** | The terminal status applied when a patient did not arrive for their appointment. The appointment is closed; a new appointment must be booked if the patient reschedules. | "missed appointment", "did not attend", "DNA" | Marked by MarkAppointmentNoShow command. Terminal — cannot be reversed at MVP. |
| **Reschedule** | The operation that marks the original appointment as terminal (Rescheduled status) and creates a new appointment aggregate for the new time slot. Both aggregates link to each other via rescheduled_to_id / rescheduled_from_id. | "move appointment", "change time" | Two aggregates; preserves full history. The original appointment record is never modified. |
| **AppointmentNote** | An append-only, staff-attributed text note specific to a single appointment visit. Separate from PatientNote (which is general patient context). Both are visible in patient history. | "visit note", "appointment comment" | Value object with: note_id, text, recorded_by (staff_member_id), recorded_at. |
| **Booking Constraints** | The five hard-stop checks that must all pass for BookAppointment to succeed: (C1) office open, (C2) provider available, (C3) chair capacity not exceeded, (C4) patient active, (C5) procedure type active. | "booking rules", "validation rules" | All five are hard stops at MVP — no override flag. Failure returns a specific error message per constraint. |
| **Chair Capacity Check** | Constraint C3: the count of concurrent Booked appointments at the office whose time window overlaps the proposed slot must be less than the office's chair_count. | "room availability", "capacity check" | Chair count is an integer property of the Office (configured in Practice Setup). Concurrent = overlapping time windows. |
| **TomorrowsCallList** | A projection of next-day appointments with patient contact info, used by the front desk to manually call patients for reminders. The MVP approach to appointment reminders (no SMS/email integration). | "reminder list", "call sheet" | SMS/email reminders are post-MVP. Query: `get_tomorrows_appointments(office_id)`. |

### Actions and Concepts

| Term | Definition | Not This | Notes |
|------|-----------|----------|-------|
| **Book** (an appointment) | Create a new appointment after all five booking constraints pass. | "create appointment", "schedule" | Command: BookAppointment. Domain language: appointments are "booked". The MVP booking flow is: Office → Patient → Procedure → Provider → Time window. |
| **Complete** (an appointment) | Mark an appointment as concluded normally. Terminal. | "close", "finish", "check out" | Command: CompleteAppointment. Precondition: appointment in Booked status. |
| **Cancel** (an appointment) | Mark an appointment as cancelled before the scheduled time. Terminal. Can be cancelled from Booked status. | "void", "delete appointment" | Command: CancelAppointment. Accepts an optional reason. |
| **Mark No-Show** | Apply the NoShow terminal status to an appointment when the patient did not arrive. | "flag as absent", "mark absent" | Command: MarkAppointmentNoShow. Precondition: Booked status. |

### Bounded Context Terms

| Term | Definition | Not This | Notes |
|------|-----------|----------|-------|
| **Patient Scheduling** | The bounded context responsible for booking appointments and maintaining the appointment schedule. The most downstream MVP context — it consumes from all other contexts and produces no events consumed by other MVP contexts. | "calendar", "booking system" | Single aggregate: Appointment. The scheduling module licensed under the Licensing context gates all writes in this context. |

---

## Banned Terms

Terms that must NOT be used in code, tests, or documentation because they are ambiguous, generic, or conflict with domain language.

| Banned Term | Why | Use Instead |
|-------------|-----|-------------|
| staff / staff member | Ambiguous -- could mean clinical or administrative | **Provider** (for clinical staff in scheduling context) |
| clinic | Ambiguous -- could mean practice or office | **Practice** (the business) or **Office** (the location) |
| doctor | Too specific -- excludes hygienists and specialists | **Provider** |
| room | Implies bookable physical space; our constraint is chair count | **Chair** (as capacity unit) |
| slot | Implies fixed time blocks; our scheduling is continuous | **Availability** (provider's working window) |
| appointment type | Conflates scheduling concept with practice config | **Procedure Type** |
| DTO | Implementation jargon -- not domain language | Describe the actual concept |
| Manager / Handler / Service | Generic OOP terms that hide domain meaning | Use domain-specific names |
| delete | Implies permanent removal; event sourcing is append-only | **Archive** (office/provider/patient/staff member) or **Deactivate** (procedure type) |
| user | Ambiguous -- could mean patient, provider, or admin | Use the specific role: **Practice Manager**, **Provider**, **Patient**, **StaffMember** |
| login / log in | Implies password-based session authentication | **Switch** (active identity via PIN) |
| password | Implies remote authentication infrastructure | **PIN** (local quick-switching only) |
| user account | Implies a networked identity system | **StaffMember** (with roles and a PIN) |
| subscription | Implies recurring billing managed by the app | **License** (app-managed, key-based) |
| create appointment | Misleading verb for scheduling | **Book** (an appointment) |
| patient record (as a clinical chart) | Conflates scheduling identity with clinical history | **Patient** (demographic identity) or **Clinical Record** (post-MVP, for clinical data) |
| schedule (as a noun for provider's pattern) | Overloaded — means different things across contexts | **Availability** (Practice Setup), **ResolvedSchedule** (Staff Scheduling), **Appointment** list (Patient Scheduling) |

---

**Note**: All Phase 1 ceremony contexts are now documented: Practice Setup, Licensing, Staff Management, Staff Scheduling, Patient Management, and Patient Scheduling. Future additions will cover post-MVP contexts (Clinical Records, Jamaica EHR Integration, Billing/Insurance) as those domains are discovered.
