# Feature: Patient Management
#
# Phase 2.4 BDD Scenarios — Patient Management context
# Date: 2026-03-04
# All open questions resolved — Tony confirmed PM-1, PM-2, PM-3, PM-6 on 2026-03-04.
# PM-4 (NIS/insurance) and PM-5 (phone format) are ASSUMED (Post-MVP / Free-text).
#
# Covers:
#   RegisterPatient — name + contact validation (PM-Rule-1)
#   Single-word name dot placeholder (PM-Rule-2)
#   Soft duplicate warning on same name + phone (PM-Rule-3)
#   Practice-wide patients — preferred_office_id is optional hint (PM-Rule-4)
#   UpdatePatientDemographics and UpdatePatientContactInfo (PM-Rule-5)
#   AddPatientNote — append-only, attributed (PM-Rule-6)
#   ArchivePatient / UnarchivePatient — PM-only (PM-Rule-7)
#   PatientList search — name prefix, phone, preferred_office_id, archived filter (PM-Rule-8)
#
# Domain language: Patient, Register, Archive, PatientNote, Practice Manager, active staff member
# Banned terms in use: NONE.
# (No "client", "customer", "medical record", "EMR", "EHR", "delete")

Feature: Patient Management — Patient Registration and Lifecycle

  Background:
    Given an active Practice Manager "Dr. Spence" is the current user

  # ─────────────────────────────────────────────────────────────
  # RULE PM-Rule-1: RegisterPatient requires first name, last name, and at least one contact method
  # ─────────────────────────────────────────────────────────────

  Rule: RegisterPatient requires first name, last name, and at least one contact method

    Scenario: Registering a patient with full name and phone
      When the active staff member registers patient "Maria" "Brown" with phone "876-555-0100"
      Then a PatientRegistered event is recorded with first_name "Maria", last_name "Brown", phone "876-555-0100"
      And the PatientRegistered event carries registered_by for the active staff member

    Scenario: Registering a patient with email only and no phone
      When the active staff member registers patient "James" "Reid" with email "james.reid@example.com" and no phone
      Then a PatientRegistered event is recorded with email "james.reid@example.com" and no phone

    Scenario: Registering a patient with both phone and email
      When the active staff member registers patient "Rosa" "Williams" with phone "876-555-0200" and email "rosa@example.com"
      Then a PatientRegistered event is recorded with both phone "876-555-0200" and email "rosa@example.com"

    Scenario: Registering a patient without any contact method is rejected
      When the active staff member attempts to register patient "James" "Reid" with no phone and no email
      Then registration is rejected with "At least one contact method (phone or email) is required"

    Scenario: Registering a patient with empty first name is rejected
      When the active staff member attempts to register patient "" "Brown" with phone "876-555-0100"
      Then registration is rejected with "First name is required"

    Scenario: Registering a patient with empty last name is rejected
      When the active staff member attempts to register patient "Maria" "" with phone "876-555-0100"
      Then registration is rejected with "Last name is required"

    Scenario: Registering a patient with whitespace-only first name is rejected
      When the active staff member attempts to register patient "   " "Brown" with phone "876-555-0100"
      Then registration is rejected with "First name is required"

    Scenario: Registering a patient with whitespace-only last name is rejected
      When the active staff member attempts to register patient "Maria" "   " with phone "876-555-0100"
      Then registration is rejected with "Last name is required"

    Scenario: Registering a patient with full optional details
      When the active staff member registers patient "Rosa" "Williams" with phone "876-555-0200", email "rosa@example.com", date_of_birth "1990-06-15", address "14 Hope Road", preferred_office_id for "Kingston", preferred_contact_channel "WhatsApp"
      Then a PatientRegistered event is recorded with all supplied fields

  # ─────────────────────────────────────────────────────────────
  # RULE PM-Rule-2: Single-word name patients use dot placeholder for last name
  # ─────────────────────────────────────────────────────────────

  Rule: Single-word name patients use dot placeholder "." for last name

    Scenario: Registering a patient who uses only one name
      When the active staff member registers patient "Delroy" "." with phone "876-555-0300"
      Then a PatientRegistered event is recorded with first_name "Delroy" and last_name "."

    Scenario: Dot placeholder in first name position is accepted
      When the active staff member registers patient "." "Brown" with phone "876-555-0301"
      Then a PatientRegistered event is recorded with first_name "." and last_name "Brown"

    Scenario: Updating last name to dot placeholder when patient adopts single-name usage
      Given an active patient "Maria" "Brown" is registered with phone "876-555-0100"
      When the active staff member updates patient demographics with last_name "."
      Then a PatientDemographicsUpdated event is recorded with last_name "."

    Scenario: Empty last name is rejected even when single-word name is intended
      When the active staff member attempts to register patient "Delroy" "" with phone "876-555-0300"
      Then registration is rejected with "Last name is required"

  # ─────────────────────────────────────────────────────────────
  # RULE PM-Rule-3: Soft duplicate warning on same full name + phone — no hard block
  # ─────────────────────────────────────────────────────────────

  Rule: Registering a patient with the same full name and phone as an existing patient produces a soft warning but is not blocked

    Scenario: Soft duplicate warning when same name and phone already registered
      Given an active patient "Maria" "Brown" with phone "876-555-0100" is already registered
      When the active staff member registers patient "Maria" "Brown" with phone "876-555-0100"
      Then a PatientRegistered event is recorded
      And a duplicate_warning is returned: "A patient with this name and phone number already exists"

    Scenario: Staff member proceeds through duplicate warning — second registration persists
      Given an active patient "Maria" "Brown" with phone "876-555-0100" is already registered
      When the active staff member registers patient "Maria" "Brown" with phone "876-555-0100" and confirms the duplicate warning
      Then a PatientRegistered event is recorded
      And two patient records exist with the name "Maria Brown" and phone "876-555-0100"

    Scenario: No duplicate warning when phone differs
      Given an active patient "Maria" "Brown" with phone "876-555-0100" is already registered
      When the active staff member registers patient "Maria" "Brown" with phone "876-555-0199"
      Then a PatientRegistered event is recorded with no duplicate_warning

    Scenario: No duplicate warning when no phone supplied on new registration
      Given an active patient "Maria" "Brown" with phone "876-555-0100" is already registered
      When the active staff member registers patient "Maria" "Brown" with email "maria@example.com" and no phone
      Then a PatientRegistered event is recorded with no duplicate_warning

    Scenario: Archived patient with same name and phone does not trigger duplicate warning
      Given an archived patient "Maria" "Brown" with phone "876-555-0100" exists
      When the active staff member registers patient "Maria" "Brown" with phone "876-555-0100"
      Then a PatientRegistered event is recorded with no duplicate_warning

  # ─────────────────────────────────────────────────────────────
  # RULE PM-Rule-4: Patients are practice-wide — preferred_office_id is optional
  # ─────────────────────────────────────────────────────────────

  Rule: Patients are practice-wide; preferred_office_id is an optional filtering hint, not a booking constraint

    Scenario: Patient registered with a preferred office appears in that office's filter
      When the active staff member registers patient "Devon" "Brooks" with phone "876-555-0400" and preferred_office_id for "Kingston"
      And the PatientList is queried with filter preferred_office_id = Kingston
      Then "Devon Brooks" appears in the results

    Scenario: Patient registered without a preferred office is visible across all offices
      When the active staff member registers patient "Devon" "Brooks" with phone "876-555-0400" and no preferred_office_id
      And the PatientList is queried with no office filter
      Then "Devon Brooks" appears in the results

    Scenario: Patient with preferred office for Kingston can still be booked at Montego Bay
      Given an active patient "Devon" "Brooks" with preferred_office_id for "Kingston" is registered
      When Patient Scheduling attempts to book "Devon Brooks" at the Montego Bay office
      Then the booking is not rejected on account of preferred_office_id
      And the preferred_office_id constraint does not appear in the booking constraint check

  # ─────────────────────────────────────────────────────────────
  # RULE PM-Rule-5: UpdatePatientDemographics and UpdatePatientContactInfo
  # ─────────────────────────────────────────────────────────────

  Rule: UpdatePatientDemographics updates name, DOB, and address; both names must remain non-empty

    Scenario: Adding a date of birth to an existing patient
      Given an active patient "Maria" "Brown" is registered with phone "876-555-0100"
      When the active staff member updates patient demographics with date_of_birth "1985-04-12"
      Then a PatientDemographicsUpdated event is recorded with date_of_birth "1985-04-12"
      And the event carries updated_by for the active staff member

    Scenario: Correcting a misspelled first name
      Given an active patient "Marria" "Brown" is registered with phone "876-555-0100"
      When the active staff member updates patient demographics with first_name "Maria"
      Then a PatientDemographicsUpdated event is recorded with first_name "Maria"

    Scenario: Adding a full address
      Given an active patient "Maria" "Brown" is registered with phone "876-555-0100"
      When the active staff member updates patient demographics with address_line_1 "14 Hope Road", city_town "Kingston", subdivision "St. Andrew", country "Jamaica"
      Then a PatientDemographicsUpdated event is recorded with all address fields

    Scenario: Clearing date of birth is permitted
      Given an active patient "Maria" "Brown" with date_of_birth "1985-04-12" is registered
      When the active staff member updates patient demographics with date_of_birth null
      Then a PatientDemographicsUpdated event is recorded with date_of_birth null

    Scenario: Updating demographics on archived patient is rejected
      Given an archived patient "Maria" "Brown" exists
      When the active staff member attempts to update demographics for the archived patient
      Then the update is rejected with "Cannot update demographics for an archived patient"

    Scenario: Updating first name to empty string is rejected
      Given an active patient "Maria" "Brown" is registered with phone "876-555-0100"
      When the active staff member attempts to update patient demographics with first_name ""
      Then the update is rejected with "First name is required"

    Scenario: Updating last name to empty string is rejected
      Given an active patient "Maria" "Brown" is registered with phone "876-555-0100"
      When the active staff member attempts to update patient demographics with last_name ""
      Then the update is rejected with "Last name is required"

  Rule: UpdatePatientContactInfo requires at least one contact method to remain after the update

    Scenario: Adding an email to a patient who has only a phone
      Given an active patient "Maria" "Brown" with phone "876-555-0100" and no email is registered
      When the active staff member updates contact info to add email "maria@example.com"
      Then a PatientContactInfoUpdated event is recorded with phone "876-555-0100" and email "maria@example.com"

    Scenario: Removing phone when email is already present
      Given an active patient "Maria" "Brown" with phone "876-555-0100" and email "maria@example.com" is registered
      When the active staff member updates contact info to remove phone (set to null)
      Then a PatientContactInfoUpdated event is recorded with phone null and email "maria@example.com"

    Scenario: Removing the only contact method is rejected
      Given an active patient "Maria" "Brown" with phone "876-555-0100" and no email is registered
      When the active staff member attempts to update contact info to remove phone (set to null) with no email
      Then the update is rejected with "At least one contact method (phone or email) is required"

    Scenario: Updating preferred contact channel
      Given an active patient "Maria" "Brown" is registered with phone "876-555-0100"
      When the active staff member updates contact info with preferred_contact_channel "SMS"
      Then a PatientContactInfoUpdated event is recorded with preferred_contact_channel "SMS"

    Scenario: Updating contact info on archived patient is rejected
      Given an archived patient "Maria" "Brown" exists
      When the active staff member attempts to update contact info for the archived patient
      Then the update is rejected with "Cannot update contact info for an archived patient"

  # ─────────────────────────────────────────────────────────────
  # RULE PM-Rule-6: AddPatientNote — append-only, non-empty text, recorded_by required
  # ─────────────────────────────────────────────────────────────

  Rule: AddPatientNote appends an immutable note with full staff attribution; text must be non-empty

    Scenario: Active staff member adds a note to a patient
      Given an active patient "Maria" "Brown" is registered with phone "876-555-0100"
      When the active staff member adds note "Patient prefers morning appointments" to "Maria Brown"
      Then a PatientNoteAdded event is recorded with text "Patient prefers morning appointments"
      And the event carries a system-generated note_id, recorded_by for the staff member, and recorded_at timestamp

    Scenario: Multiple notes accumulate on one patient
      Given an active patient "Maria" "Brown" is registered with phone "876-555-0100"
      When the active staff member adds note "Prefers mornings" to "Maria Brown"
      And another staff member adds note "Allergic to latex gloves" to "Maria Brown"
      Then two PatientNoteAdded events are recorded on "Maria Brown"
      And both notes are visible in the patient's note list in recorded_at order

    Scenario: Adding a note with empty text is rejected
      Given an active patient "Maria" "Brown" is registered with phone "876-555-0100"
      When the active staff member attempts to add an empty note to "Maria Brown"
      Then the note is rejected with "Note text is required"

    Scenario: Adding a note with whitespace-only text is rejected
      Given an active patient "Maria" "Brown" is registered with phone "876-555-0100"
      When the active staff member attempts to add a note with text "   " to "Maria Brown"
      Then the note is rejected with "Note text is required"

    Scenario: A note cannot be edited once recorded
      Given a PatientNoteAdded event has been recorded on "Maria Brown"
      Then no EditPatientNote command exists to modify that note

    Scenario: A note cannot be removed once recorded
      Given a PatientNoteAdded event has been recorded on "Maria Brown"
      Then no RemovePatientNote command exists to remove that note

    Scenario: A note can be added to an archived patient
      Given an archived patient "Maria" "Brown" exists
      When the active staff member adds note "Contact re: reactivation" to "Maria Brown"
      Then a PatientNoteAdded event is recorded on the archived patient

  # ─────────────────────────────────────────────────────────────
  # RULE PM-Rule-7: ArchivePatient — PM-only; UnarchivePatient restores all data
  # ─────────────────────────────────────────────────────────────

  Rule: ArchivePatient and UnarchivePatient require Practice Manager authority; all patient data is preserved

    Scenario: Practice Manager archives an active patient
      Given an active patient "Maria" "Brown" is registered with phone "876-555-0100"
      When the Practice Manager archives patient "Maria Brown"
      Then a PatientArchived event is recorded with patient_id and archived_by for the Practice Manager
      And "Maria Brown" no longer appears in the active PatientList

    Scenario: Non-PM staff member cannot archive a patient
      Given an active patient "Maria" "Brown" is registered with phone "876-555-0100"
      And the current user is a staff member without the Practice Manager role
      When the staff member attempts to archive patient "Maria Brown"
      Then the archive is rejected with "Only a Practice Manager can archive a patient"

    Scenario: Archiving an already-archived patient is rejected
      Given an archived patient "Maria" "Brown" exists
      When the Practice Manager attempts to archive patient "Maria Brown" again
      Then the archive is rejected with "Patient is already archived"

    Scenario: Archived patient's data is fully preserved
      Given an active patient "Maria" "Brown" with date_of_birth "1985-04-12" and note "Prefers mornings" is registered
      When the Practice Manager archives patient "Maria Brown"
      Then the PatientDetail for "Maria Brown" still contains date_of_birth "1985-04-12"
      And the PatientNoteList for "Maria Brown" still contains "Prefers mornings"

    Scenario: Archived patient cannot be found in default active PatientList query
      Given an archived patient "Maria" "Brown" exists
      When the active PatientList is queried with no archived filter
      Then "Maria Brown" does not appear in the results

    Scenario: Practice Manager unarchives an archived patient
      Given an archived patient "Maria" "Brown" exists
      When the Practice Manager unarchives patient "Maria Brown"
      Then a PatientUnarchived event is recorded with patient_id and unarchived_by for the Practice Manager
      And "Maria Brown" is visible in the active PatientList again

    Scenario: Non-PM staff member cannot unarchive a patient
      Given an archived patient "Maria" "Brown" exists
      And the current user is a staff member without the Practice Manager role
      When the staff member attempts to unarchive patient "Maria Brown"
      Then the unarchive is rejected with "Only a Practice Manager can unarchive a patient"

    Scenario: Unarchiving an active patient is rejected
      Given an active patient "Maria" "Brown" is registered with phone "876-555-0100"
      When the Practice Manager attempts to unarchive patient "Maria Brown"
      Then the unarchive is rejected with "Patient is not archived"

    Scenario: Unarchived patient retains all data from before archival
      Given a patient "Maria" "Brown" was registered with notes and demographics, then archived
      When the Practice Manager unarchives patient "Maria Brown"
      Then all notes, demographics, and contact info are intact

    Scenario: Patient with future appointments can be archived without auto-cancellation
      Given an active patient "Maria" "Brown" has a future appointment booked
      When the Practice Manager archives patient "Maria Brown"
      Then a PatientArchived event is recorded
      And no appointment cancellation event is emitted by Patient Management

  # ─────────────────────────────────────────────────────────────
  # RULE PM-Rule-8: PatientList search — name prefix, phone, preferred_office_id, archived
  # ─────────────────────────────────────────────────────────────

  Rule: PatientList supports prefix search by name, filter by phone, filter by preferred_office_id, and filter by archived status

    Scenario: Searching patients by name prefix matching last name
      Given active patients "Maria" "Brown", "Devon" "Brooks", and "James" "Reid" are registered
      When the PatientList is queried with name prefix "Bro"
      Then the results include "Brown, Maria" and "Brooks, Devon"
      And "Reid, James" is not in the results

    Scenario: Searching patients by name prefix matching first name
      Given active patients "Maria" "Brown" and "Marcus" "James" are registered
      When the PatientList is queried with name prefix "Mar"
      Then the results include "Brown, Maria" and "James, Marcus"

    Scenario: Searching patients by phone substring
      Given active patients "Maria" "Brown" with phone "876-555-0100" and "James" "Reid" with phone "876-777-0200" are registered
      When the PatientList is queried with phone "876-555"
      Then the results include "Brown, Maria"
      And "Reid, James" is not in the results

    Scenario: Searching patients by preferred office
      Given active patient "Maria" "Brown" with preferred_office_id for "Kingston" and "James" "Reid" with preferred_office_id for "Montego Bay" are registered
      When the PatientList is queried with filter preferred_office_id = Kingston
      Then the results include "Brown, Maria"
      And "Reid, James" is not in the results

    Scenario: Querying with no filters returns all active patients
      Given three active patients are registered
      When the PatientList is queried with no filters
      Then all three active patients are returned

    Scenario: Combining name prefix and office filter
      Given active patients "Maria" "Brown" with preferred_office for "Kingston" and "Monica" "Reid" with preferred_office for "Montego Bay" are registered
      When the PatientList is queried with name prefix "M" and preferred_office_id = Kingston
      Then only "Brown, Maria" is returned

    Scenario: Searching for archived patients
      Given an archived patient "Maria" "Brown" and an active patient "James" "Reid" exist
      When the PatientList is queried with archived = true
      Then "Brown, Maria" appears in the results
      And "Reid, James" does not appear in the results

    Scenario: Name prefix with no matching patients returns empty list
      Given active patients "Maria" "Brown" and "James" "Reid" are registered
      When the PatientList is queried with name prefix "XYZ"
      Then the results are empty
      And no error is returned

    Scenario: Patient with dot placeholder last name is searchable
      Given an active patient "Delroy" "." with phone "876-555-0300" is registered
      When the PatientList is queried with name prefix "."
      Then "., Delroy" appears in the results

    Scenario: PatientList displays names in Last, First format
      Given active patients "Maria" "Brown" and "James" "Reid" are registered
      When the PatientList is queried with no filters
      Then results are displayed with full_name_display as "Brown, Maria" and "Reid, James"

    Scenario: Archived patients do not appear in default PatientList query
      Given active patient "James" "Reid" and archived patient "Maria" "Brown" exist
      When the PatientList is queried with no filters
      Then "Reid, James" appears in the results
      And "Brown, Maria" does not appear in the results
