# Example Map: Patient Management

**Date**: 2026-03-04
**Participants**: Tony (Product Owner / Business), Claude (Developer / Tester)
**Feature**: Patient Management context — Patient registration, demographics, contact info, notes, archive/unarchive, search
**Status**: Phase 2.2 + 2.3 complete — all open questions resolved; ready for Phase 2.4 BDD scenarios

---

## Three Amigos Summary (Phase 2.1)

Decisions carried in from the Three Amigos session (Tony confirmed 2026-03-04):

| Item | Decision |
|------|----------|
| Patient scope | Practice-wide — not bound to one office. preferred_office_id is an optional hint for filtering only. |
| Name structure | first_name + last_name both required. Single-word names use dot placeholder (last_name = "."). |
| Duplicate detection | Soft warning when registering a patient with the same full name + phone as an existing patient. No hard block. No merge workflow at MVP. |
| Archive authority | Practice Manager only. Front desk cannot archive patients. |
| Note model | Append-only. Non-empty text. Full audit trail — recorded_by (staff_member_id) required. Notes travel with the patient to all downstream contexts. |
| NIS / insurance identifiers | Post-MVP. Not in this aggregate at MVP. |
| Phone format | Free-text string. No format enforcement at MVP. |
| Contact requirement | At registration and after any UpdatePatientContactInfo, at least one of phone or email must be present. |

### Open Questions — All Resolved

| ID | Question | Status |
|----|----------|--------|
| PM-1 | Is a patient bound to one office or practice-wide? | [CONFIRMED — Tony 2026-03-04] Practice-wide with optional preferred_office_id |
| PM-2 | Single-word patient names — first/last structure or single field? | [CONFIRMED — Tony 2026-03-04] First + last both required; single-word names use dot placeholder (last_name = ".") |
| PM-3 | Deduplication at MVP — soft warning or hard block? | [CONFIRMED — Tony 2026-03-04] Soft warning on same full name + phone; not blocked |
| PM-4 | NIS/insurance numbers at MVP? | [ASSUMED] Post-MVP |
| PM-5 | Phone format validation? | [ASSUMED] Free-text, no enforcement at MVP |
| PM-6 | Who can archive a patient? | [CONFIRMED — Tony 2026-03-04] Practice Manager only |

---

## Rule Cards

---

## Patient Registration

---

### Rule PM-Rule-1: RegisterPatient requires non-empty first_name, non-empty last_name, and at least one of phone or email

**Rule**: RegisterPatient is rejected if first_name is empty, last_name is empty, or neither phone nor email is provided. All other fields are optional. On success, a PatientRegistered event is recorded with the supplied fields and the registered_by staff_member_id.

| # | Example | Type |
|---|---------|------|
| PM1a | Staff member registers "Maria" "Brown" with phone "876-555-0100" → PatientRegistered with first_name "Maria", last_name "Brown", phone "876-555-0100" | ✅ Happy path |
| PM1b | Staff member registers "James" "Reid" with email "james.reid@example.com" (no phone) → PatientRegistered with email only | ✅ Happy path |
| PM1c | Staff member registers "Rosa" "Williams" with both phone "876-555-0200" and email "rosa@example.com" → PatientRegistered with both contact fields | ✅ Happy path |
| PM1d | Staff member registers "James" "Reid" with no phone and no email → Rejected: "At least one contact method (phone or email) is required" | ❌ Negative path |
| PM1e | Staff member registers with empty first_name → Rejected: "First name is required" | ❌ Negative path |
| PM1f | Staff member registers with empty last_name → Rejected: "Last name is required" | ❌ Negative path |
| PM1g | Staff member registers with whitespace-only first_name → Rejected: "First name is required" | ❌ Edge case |
| PM1h | Staff member registers with whitespace-only last_name → Rejected: "Last name is required" | ❌ Edge case |
| PM1i | Staff member registers full record with first_name, last_name, phone, email, date_of_birth, address fields, preferred_office_id, preferred_contact_channel → PatientRegistered with all supplied fields | ✅ Happy path |
| PM1j | Registration includes registered_by = active staff_member_id → PatientRegistered carries registered_by for audit | ✅ Happy path |

---

### Rule PM-Rule-2: Single-word name patients use dot placeholder for last_name

**Rule**: When a patient has only a single-word name, the front desk enters that name as first_name and uses "." as last_name. The dot is the agreed placeholder for unknown or absent last names. Both fields must still be non-empty — the dot is the explicit convention, not a blank.

| # | Example | Type |
|---|---------|------|
| PM2a | Staff member registers patient with first_name "Delroy" and last_name "." with phone "876-555-0300" → PatientRegistered with last_name "." | ✅ Happy path |
| PM2b | Staff member registers with first_name "." and last_name "Brown" → PatientRegistered; dot in first_name position is also valid if that is the only name | ✅ Edge case |
| PM2c | Staff member registers with both first_name and last_name = "." → PatientRegistered; technically valid per invariant (both non-empty); UI may warn but does not block | ✅ Edge case |
| PM2d | Staff member attempts to register with last_name = "" (truly empty, not dot) → Rejected: "Last name is required" | ❌ Negative path |

---

### Rule PM-Rule-3: Soft duplicate warning on same full name and phone — no hard block

**Rule**: Before accepting RegisterPatient, the system checks the PatientList projection for an existing active patient with the same first_name + last_name + phone combination. If a match exists, a duplicate warning is returned alongside registration proceeding (or the UI prompts for confirmation). The command is NOT rejected — the user may proceed. No merge workflow exists at MVP.

| # | Example | Type |
|---|---------|------|
| PM3a | Active patient "Maria Brown" with phone "876-555-0100" already registered → Staff member registers "Maria" "Brown" with phone "876-555-0100" → PatientRegistered emitted with duplicate_warning flag = true; warning shown to user | ✅ Happy path (with warning) |
| PM3b | Staff member proceeds through the duplicate warning → PatientRegistered is persisted; two patient records with same name + phone exist in system | ✅ Happy path (duplicate accepted) |
| PM3c | Active patient "Maria Brown" exists with phone "876-555-0100" → New patient "Maria" "Brown" registered with different phone "876-555-0199" → No duplicate warning; PatientRegistered with no warning | ✅ Edge case |
| PM3d | Active patient "Maria Brown" exists with phone "876-555-0100" → New patient "Maria" "Brown" registered with email only (no phone) → No duplicate warning (phone is absent, name+phone check does not trigger) | ✅ Edge case |
| PM3e | Archived patient "Maria Brown" with phone "876-555-0100" exists → New "Maria" "Brown" registered with same phone → No duplicate warning (archived patients are excluded from duplicate check) | ✅ Edge case |

---

### Rule PM-Rule-4: Patients are practice-wide — preferred_office_id is optional, not binding

**Rule**: A patient record belongs to the practice, not to a specific office. The preferred_office_id field is an optional hint that enables filtering ("show patients who prefer the Kingston office") but does not prevent the patient from being booked at any office. A patient without a preferred_office_id appears in all office views.

| # | Example | Type |
|---|---------|------|
| PM4a | Patient registered with preferred_office_id pointing to the Kingston office → PatientRegistered with preferred_office_id; patient appears in Kingston office filter | ✅ Happy path |
| PM4b | Patient registered with no preferred_office_id → PatientRegistered with preferred_office_id = null; patient visible across all offices | ✅ Happy path |
| PM4c | Patient with preferred_office_id = Kingston is booked at Montego Bay office → Booking is valid; preferred_office_id is not a booking constraint | ✅ Edge case |
| PM4d | PatientList queried with filter preferred_office_id = Kingston → Returns only patients whose preferred_office_id = Kingston | ✅ Happy path |
| PM4e | PatientList queried with no office filter → Returns all active patients across all offices | ✅ Happy path |

---

## Patient Demographics and Contact

---

### Rule PM-Rule-5: UpdatePatientDemographics can update name, DOB, and address; contact constraint is separate

**Rule**: UpdatePatientDemographics accepts updates to first_name, last_name, date_of_birth, and address fields. Both first_name and last_name must remain non-empty after the update. DOB and address fields are optional and may be cleared. This command does NOT update contact info (phone/email) — that is handled by UpdatePatientContactInfo. Patient must be active.

| # | Example | Type |
|---|---------|------|
| PM5a | Staff member updates "Maria Brown" to add date_of_birth "1985-04-12" → PatientDemographicsUpdated with updated date_of_birth | ✅ Happy path |
| PM5b | Staff member corrects spelling: first_name "Marria" → "Maria" → PatientDemographicsUpdated with corrected first_name | ✅ Happy path |
| PM5c | Staff member adds full address (line_1, city_town, subdivision "St. Andrew", country "Jamaica") → PatientDemographicsUpdated with address fields | ✅ Happy path |
| PM5d | Staff member clears date_of_birth (sets to null) → PatientDemographicsUpdated with date_of_birth = null | ✅ Happy path |
| PM5e | Staff member updates demographics on archived patient → Rejected: "Cannot update demographics for an archived patient" | ❌ Negative path |
| PM5f | Staff member updates first_name to "" (empty) → Rejected: "First name is required" | ❌ Negative path |
| PM5g | Staff member updates last_name to "" (empty) → Rejected: "Last name is required" | ❌ Negative path |
| PM5h | Staff member updates last_name to "." (dot placeholder) for a patient who now uses only a single name → PatientDemographicsUpdated with last_name "." | ✅ Edge case |
| PM5i | UpdatePatientDemographics carries updated_by (staff_member_id) → Event records who made the change for audit trail | ✅ Happy path |

**UpdatePatientContactInfo sub-rule**: After any UpdatePatientContactInfo, at least one of phone or email must remain on the patient record.

| # | Example | Type |
|---|---------|------|
| PM5j | Patient has phone only → Staff member adds email → PatientContactInfoUpdated with phone + email | ✅ Happy path |
| PM5k | Patient has phone only → Staff member attempts to remove phone (set to null) with no email → Rejected: "At least one contact method (phone or email) is required" | ❌ Negative path |
| PM5l | Patient has phone + email → Staff member removes phone (sets to null) keeping email → PatientContactInfoUpdated; contact still valid | ✅ Happy path |
| PM5m | Staff member updates preferred_contact_channel to "SMS" → PatientContactInfoUpdated with new preferred channel | ✅ Happy path |
| PM5n | UpdatePatientContactInfo on archived patient → Rejected: "Cannot update contact info for an archived patient" | ❌ Negative path |

---

## Patient Notes

---

### Rule PM-Rule-6: AddPatientNote — append-only, non-empty text, recorded_by required

**Rule**: AddPatientNote appends a new PatientNote to the patient's note list. The text must be non-empty. The recorded_by staff_member_id is required — no anonymous notes. The note is immutable once recorded. Notes are indexed by note_id and ordered by recorded_at. Any active staff member (any role) may add a note.

| # | Example | Type |
|---|---------|------|
| PM6a | Active staff member adds note "Patient prefers morning appointments" → PatientNoteAdded with note_id (system-generated), text, recorded_by, recorded_at | ✅ Happy path |
| PM6b | Active staff member adds a second note on the same patient → Two PatientNoteAdded events on the patient; both notes preserved in order | ✅ Happy path |
| PM6c | Staff member attempts to add a note with empty text → Rejected: "Note text is required" | ❌ Negative path |
| PM6d | Staff member attempts to add a note with whitespace-only text → Rejected: "Note text is required" | ❌ Edge case |
| PM6e | AddPatientNote without a recorded_by staff_member_id → Rejected: "Note must be attributed to a staff member" | ❌ Negative path |
| PM6f | Notes cannot be edited once added → No EditPatientNote command exists | ✅ Invariant verification |
| PM6g | Notes cannot be removed → No RemovePatientNote command exists | ✅ Invariant verification |
| PM6h | AddPatientNote on an archived patient → Per aggregate doc (HS-6 note in commands table), notes can also be added to archived patients — PatientNoteAdded recorded | ✅ Edge case |
| PM6i | Multiple staff members each add a note over time → Each note carries its own recorded_by and recorded_at; full attribution trail preserved | ✅ Happy path |

---

## Patient Archive / Unarchive

---

### Rule PM-Rule-7: ArchivePatient — PM-only; archived patient cannot be booked; UnarchivePatient restores all data

**Rule**: ArchivePatient requires the archived_by actor to hold the PracticeManager role. An archived patient is hidden from the active PatientList and cannot be booked in Patient Scheduling. All historical data (demographics, contact info, notes) is fully preserved. UnarchivePatient (also PM-only) restores the patient to Active status; all data is intact.

| # | Example | Type |
|---|---------|------|
| PM7a | Practice Manager archives patient "Maria Brown" → PatientArchived with patient_id, archived_by; patient hidden from active list | ✅ Happy path |
| PM7b | Staff member (not PM) attempts to archive a patient → Rejected: "Only a Practice Manager can archive a patient" | ❌ Negative path |
| PM7c | Practice Manager attempts to archive an already-archived patient → Rejected: "Patient is already archived" | ❌ Negative path |
| PM7d | Archived patient "Maria Brown" is searched in active PatientList → Not returned in results | ✅ Happy path |
| PM7e | Archived patient's data (demographics, notes) is preserved and viewable by PM → All fields intact | ✅ Happy path |
| PM7f | Patient Scheduling attempts to book an appointment for an archived patient → Booking rejected (patient not active is a booking constraint) | ✅ Happy path |
| PM7g | Practice Manager unarchives "Maria Brown" → PatientUnarchived; patient active again, visible in PatientList | ✅ Happy path |
| PM7h | Practice Manager attempts to unarchive an active patient → Rejected: "Patient is not archived" | ❌ Negative path |
| PM7i | Staff member (not PM) attempts to unarchive a patient → Rejected: "Only a Practice Manager can unarchive a patient" | ❌ Negative path |
| PM7j | Archived patient is unarchived → All notes, demographics, and contact info are intact from before archival | ✅ Happy path |
| PM7k | Patient with future appointments is archived → PatientArchived recorded; no auto-cancellation of appointments (downstream Patient Scheduling handles constraint at booking time, not retrospectively) | ✅ Edge case |

---

## Patient Search

---

### Rule PM-Rule-8: PatientList search supports prefix search by name, filter by phone, filter by preferred_office_id, and filter by archived status

**Rule**: The PatientList projection is queryable with four independent filter parameters: name prefix (matches first_name or last_name), phone (exact or substring match), preferred_office_id, and archived status. Filters may be combined. Default query returns active (non-archived) patients only. The full_name_display format is "Last, First" for sorted list display.

| # | Example | Type |
|---|---------|------|
| PM8a | Search with name prefix "Bro" → Returns "Brown, Maria", "Brooks, Devon", etc. (prefix match on first_name or last_name) | ✅ Happy path |
| PM8b | Search with name prefix "Mar" → Returns patients with first_name starting "Mar" (e.g., "Maria") and patients with last_name starting "Mar" (e.g., "Martin") | ✅ Happy path |
| PM8c | Search with phone "876-555" → Returns patients whose phone contains "876-555" | ✅ Happy path |
| PM8d | Search with preferred_office_id = Kingston office UUID → Returns only active patients whose preferred_office_id = Kingston | ✅ Happy path |
| PM8e | Search with archived = true → Returns only archived patients | ✅ Happy path |
| PM8f | Search with no filters → Returns all active patients | ✅ Happy path |
| PM8g | Search with name prefix "Brown" + preferred_office_id = Kingston → Returns active patients named "Brown*" who prefer Kingston office | ✅ Happy path |
| PM8h | Search with name prefix that matches no patients → Returns empty list; not an error | ✅ Edge case |
| PM8i | Patient with last_name "." (dot placeholder) searched by name prefix "." → Returns that patient | ✅ Edge case |
| PM8j | PatientList displays full_name_display as "Brown, Maria" (last, first) for sorted display | ✅ Happy path |
| PM8k | Archived patient does not appear in default PatientList query (archived filter defaults to false) | ✅ Happy path |

---

## Open Questions Summary

All open questions are resolved. No outstanding items.

| # | Question | Status |
|---|----------|--------|
| PM-1 | Patient bound to one office or practice-wide? | [CONFIRMED — Tony 2026-03-04] |
| PM-2 | Single-word names — dot placeholder or single field? | [CONFIRMED — Tony 2026-03-04] |
| PM-3 | Duplicate detection — soft warning or hard block? | [CONFIRMED — Tony 2026-03-04] |
| PM-4 | NIS/insurance at MVP? | [ASSUMED — Post-MVP] |
| PM-5 | Phone format enforcement? | [ASSUMED — Free-text, no enforcement] |
| PM-6 | Archive authority? | [CONFIRMED — Tony 2026-03-04] |

---

**Phase 2.3 Acceptance Criteria Review**: All business rules validated against the Patient aggregate doc (`doc/domain/aggregates/patient-aggregate.md`). All commands and events match. All OQs are marked CONFIRMED or ASSUMED (none remain OPEN). Ubiquitous language used throughout. No banned terms (no "client", "customer", "medical record", "EMR", "EHR", "delete"). Ready for Phase 2.4 BDD scenarios and Phase 2.5 governance review.
