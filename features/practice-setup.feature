# Feature: Practice Setup
#
# Phase 2.4 BDD Scenarios — Practice Setup context
# Date: 2026-03-03
# All open questions resolved or flagged — ready for Tony review before implementation.
#
# Covers:
#   Practice identity (4 rules)
#   Office lifecycle (6 rules)
#   Provider lifecycle — RETIRED (DM-1, 2026-03-06) — see features/staff-management.feature
#   Procedure Type lifecycle (5 rules)
#   Setup checklist (4 rules)
#
# Context: Practice Setup owns offices, procedure types, and practice identity.
# Provider/clinical data moved to Staff Management context (DM-1, 2026-03-06).
# All scheduling depends on offices, providers (in Staff Management), and procedure types.

Feature: Practice Setup

  # ─────────────────────────────────────────────────────────────
  # PRACTICE IDENTITY
  # ─────────────────────────────────────────────────────────────

  # Rule P1: Practice must have a name before it is considered configured

  Scenario: Setting practice name for the first time
    Given the practice has no name configured
    When the Practice Manager submits UpdatePracticeDetails with name "Spence Dental"
    Then a PracticeDetailsUpdated event is recorded
    And the practice name is "Spence Dental"

  Scenario: Updating practice name after it is already set
    Given the practice name is "Spence Dental"
    When the Practice Manager submits UpdatePracticeDetails with name "Spence Dental Group"
    Then a PracticeDetailsUpdated event is recorded
    And the practice name is "Spence Dental Group"

  Scenario: Submitting an empty name is rejected
    Given the practice has no name configured
    When the Practice Manager submits UpdatePracticeDetails with name ""
    Then no PracticeDetailsUpdated event is recorded
    And an error is shown: "Name is required"

  Scenario: Submitting a whitespace-only name is rejected
    When the Practice Manager submits UpdatePracticeDetails with name "   "
    Then no PracticeDetailsUpdated event is recorded
    And an error is shown: "Name is required"

  # Rule P2: Contact fields other than name are optional

  Scenario: Setting only the practice name with no contact details
    When the Practice Manager submits UpdatePracticeDetails with name "Spence Dental" and no other fields
    Then a PracticeDetailsUpdated event is recorded
    And phone and email are not set

  Scenario: Setting name with phone and preferred contact channel
    When the Practice Manager submits UpdatePracticeDetails with name "Spence Dental", phone "876-555-0100", and preferred channel "WhatsApp"
    Then a PracticeDetailsUpdated event is recorded
    And the practice phone is "876-555-0100"
    And the preferred contact channel is "WhatsApp"

  Scenario: Setting a full structured address
    When the Practice Manager submits UpdatePracticeDetails with name "Spence Dental", address_line_1 "123 Main St", city_town "Kingston", subdivision "St. Andrew", country "Jamaica"
    Then a PracticeDetailsUpdated event is recorded
    And the address subdivision is "St. Andrew"

  Scenario: Updating only phone preserves existing name
    Given the practice name is "Spence Dental"
    When the Practice Manager submits UpdatePracticeDetails with name "Spence Dental" and phone "876-555-0200"
    Then a PracticeDetailsUpdated event is recorded
    And the practice name remains "Spence Dental"
    And the practice phone is "876-555-0200"

  # Rule P4: Practice is a singleton

  Scenario: Practice exists in unconfigured state on fresh install
    Given no PracticeDetailsUpdated events exist in the event store
    When the Practice Manager views the practice settings screen
    Then the practice setup form is shown with empty fields
    And no create or delete controls are visible

  # ─────────────────────────────────────────────────────────────
  # OFFICE LIFECYCLE
  # ─────────────────────────────────────────────────────────────

  # Rule O1: Office requires name and at least one chair

  Scenario: Creating an office with name and chairs
    When the Practice Manager creates an office with name "Kingston" and chair_count 3
    Then an OfficeCreated event is recorded
    And the office "Kingston" is active with 3 chairs

  Scenario: Creating an office with the minimum of one chair
    When the Practice Manager creates an office with name "Portmore" and chair_count 1
    Then an OfficeCreated event is recorded

  Scenario: Creating an office with zero chairs is rejected
    When the Practice Manager creates an office with name "Kingston" and chair_count 0
    Then no OfficeCreated event is recorded
    And an error is shown: "At least one chair is required"

  Scenario: Creating an office with an empty name is rejected
    When the Practice Manager creates an office with name "" and chair_count 2
    Then no OfficeCreated event is recorded
    And an error is shown: "Name is required"

  Scenario: Creating two offices with the same name shows a soft warning
    Given an active office named "Kingston" exists
    When the Practice Manager creates another office with name "Kingston" and chair_count 2
    Then an OfficeCreated event is recorded
    And a soft warning is shown: "Another office named 'Kingston' already exists"

  # Rule O2: Operating hours are per day; unconfigured days are closed

  Scenario: Setting operating hours for a single day
    Given an active office "Kingston" exists
    When the Practice Manager sets Monday hours to 08:00-17:00 for office "Kingston"
    Then an OfficeHoursSet event is recorded with day Monday, open 08:00, close 17:00

  Scenario: A day with no hours configured is treated as closed
    Given an active office "Kingston" exists with no hours configured for Tuesday
    When Scheduling queries whether "Kingston" is open on Tuesday
    Then the office is considered closed on Tuesday

  Scenario: Setting hours where close time equals open time is rejected
    Given an active office "Kingston" exists
    When the Practice Manager sets Monday hours to 09:00-09:00 for office "Kingston"
    Then no OfficeHoursSet event is recorded
    And an error is shown: "Close time must be after open time"

  Scenario: Setting hours where close time is before open time is rejected
    Given an active office "Kingston" exists
    When the Practice Manager sets Monday hours to 17:00-08:00 for office "Kingston"
    Then no OfficeHoursSet event is recorded
    And an error is shown: "Close time must be after open time"

  Scenario: Closing a previously configured day
    Given an active office "Kingston" has Monday hours 08:00-17:00
    When the Practice Manager closes Monday for office "Kingston"
    Then an OfficeDayClosed event is recorded for Monday
    And Monday is treated as closed at office "Kingston"

  Scenario: Re-setting hours for an already-configured day replaces the previous hours
    Given an active office "Kingston" has Monday hours 08:00-17:00
    When the Practice Manager sets Monday hours to 09:00-16:00 for office "Kingston"
    Then an OfficeHoursSet event is recorded with open 09:00 and close 16:00
    And Monday hours are 09:00-16:00 at office "Kingston"

  # Rule O3: Chair count can be changed; reducing below concurrent appointments warns, not blocks

  Scenario: Increasing chair count
    Given an active office "Kingston" has 2 chairs
    When the Practice Manager updates chair count to 4 for office "Kingston"
    Then an OfficeChairCountUpdated event is recorded with new_chair_count 4

  Scenario: Reducing chair count with no capacity conflicts
    Given an active office "Kingston" has 4 chairs and no concurrent appointments
    When the Practice Manager updates chair count to 2 for office "Kingston"
    Then an OfficeChairCountUpdated event is recorded with new_chair_count 2

  Scenario: Reducing chair count below concurrent appointment count warns but proceeds
    Given an active office "Kingston" has 4 chairs
    And 3 appointments are booked concurrently at "Kingston"
    When the Practice Manager updates chair count to 1 for office "Kingston"
    Then an OfficeChairCountUpdated event is recorded
    And a warning is shown: "Reducing chairs may conflict with existing appointments"

  Scenario: Setting chair count to zero is rejected
    Given an active office "Kingston" exists
    When the Practice Manager updates chair count to 0 for office "Kingston"
    Then no OfficeChairCountUpdated event is recorded
    And an error is shown: "At least one chair is required"

  # Rule O4: Office can be renamed while active

  Scenario: Renaming an active office
    Given an active office "Kingston" exists
    When the Practice Manager renames office "Kingston" to "Kingston Main"
    Then an OfficeRenamed event is recorded with new_name "Kingston Main"

  Scenario: Renaming an office to an empty name is rejected
    Given an active office "Kingston" exists
    When the Practice Manager renames office "Kingston" to ""
    Then no OfficeRenamed event is recorded
    And an error is shown: "Name is required"

  Scenario: Renaming an archived office is rejected
    Given an archived office "Old Location" exists
    When the Practice Manager attempts to rename office "Old Location" to "New Name"
    Then no OfficeRenamed event is recorded
    And an error is shown: "Cannot modify an archived office"

  # Rule O5: Archiving an office is permanent

  Scenario: Archiving an active office
    Given an active office "Montego Bay" exists
    When the Practice Manager archives office "Montego Bay"
    Then an OfficeArchived event is recorded
    And office "Montego Bay" does not appear in the active office list

  Scenario: Archived office data is preserved in history
    Given an archived office "Old Office" exists
    When the Practice Manager views historical appointment records
    Then appointments associated with "Old Office" are still visible

  Scenario: Archiving an already-archived office is rejected
    Given an archived office "Old Office" exists
    When the Practice Manager attempts to archive office "Old Office"
    Then no OfficeArchived event is recorded
    And an error is shown: "Office is already archived"

  Scenario: Setting hours on an archived office is rejected
    Given an archived office "Old Office" exists
    When the Practice Manager attempts to set hours on office "Old Office"
    Then no OfficeHoursSet event is recorded
    And an error is shown: "Cannot modify an archived office"

  # Rule O6: Setup checklist — office step completion

  Scenario: Office setup step is incomplete when no active office has hours
    Given an active office "Kingston" exists with 2 chairs but no operating hours
    When the setup checklist is evaluated
    Then the office setup step is "incomplete"

  Scenario: Office setup step is complete when one active office has hours
    Given an active office "Kingston" exists with 2 chairs and Monday hours 08:00-17:00
    When the setup checklist is evaluated
    Then the office setup step is "complete"

  Scenario: Office setup step becomes incomplete when the only active office is archived
    Given the office setup step is "complete" with one qualifying active office
    When the Practice Manager archives that office
    Then the office setup step is "incomplete"

  # ─────────────────────────────────────────────────────────────
  # PROVIDER LIFECYCLE — RETIRED (DM-1, 2026-03-06)
  # ─────────────────────────────────────────────────────────────
  #
  # Provider IS A StaffMember. Clinical configuration (provider type, office
  # assignments, availability, exceptions) has moved to the Staff Management
  # context. See features/staff-management.feature — CLINICAL CONFIGURATION section.
  #
  # The setup checklist Provider step now reads from Staff Management:
  # active StaffMember + Provider role + ClinicalSpecialization set + ≥1 office + ≥1 availability day.
  #
  # Rules PR1–PR8 are retired. No scenarios listed here.
  # ─────────────────────────────────────────────────────────────

  # ─────────────────────────────────────────────────────────────
  # PROCEDURE TYPE LIFECYCLE
  # ─────────────────────────────────────────────────────────────

  # Rule PT1: Define procedure type with valid fields

  Scenario: Defining a procedure type with all required fields
    When the Practice Manager defines procedure type "Cleaning" with category "Preventive" and duration 30 minutes
    Then a ProcedureTypeDefined event is recorded with name "Cleaning", category "Preventive", default_duration_minutes 30

  Scenario: Defining a procedure type with minimum valid duration
    When the Practice Manager defines procedure type "Quick Exam" with category "Diagnostic" and duration 15 minutes
    Then a ProcedureTypeDefined event is recorded with default_duration_minutes 15

  Scenario: Defining a procedure type with maximum valid duration
    When the Practice Manager defines procedure type "Complex Surgery" with category "Invasive" and duration 240 minutes
    Then a ProcedureTypeDefined event is recorded with default_duration_minutes 240

  Scenario: Defining a procedure type with duration below 15 minutes is rejected
    When the Practice Manager defines procedure type "Quick Look" with category "Diagnostic" and duration 14 minutes
    Then no ProcedureTypeDefined event is recorded
    And an error is shown: "Duration must be between 15 and 240 minutes"

  Scenario: Defining a procedure type with duration above 240 minutes is rejected
    When the Practice Manager defines procedure type "Very Long Procedure" with category "Invasive" and duration 241 minutes
    Then no ProcedureTypeDefined event is recorded
    And an error is shown: "Duration must be between 15 and 240 minutes"

  Scenario: Defining a procedure type with an empty name is rejected
    When the Practice Manager defines procedure type "" with category "Preventive" and duration 30 minutes
    Then no ProcedureTypeDefined event is recorded
    And an error is shown: "Name is required"

  Scenario: Defining a procedure type with an invalid category is rejected
    When the Practice Manager defines procedure type "Special Service" with category "Administrative" and duration 30 minutes
    Then no ProcedureTypeDefined event is recorded
    And an error is shown: "Category must be one of: Consult, Preventive, Restorative, Invasive, Cosmetic, Diagnostic"

  # Rule PT2: Update procedure type

  Scenario: Updating a procedure type name
    Given an active procedure type "Cleaning" exists
    When the Practice Manager updates the name to "Deep Cleaning"
    Then a ProcedureTypeUpdated event is recorded with name "Deep Cleaning"

  Scenario: Updating a procedure type duration
    Given an active procedure type "Filling" with duration 45 minutes exists
    When the Practice Manager updates the duration to 60 minutes
    Then a ProcedureTypeUpdated event is recorded with default_duration_minutes 60

  Scenario: Updating duration to an invalid value is rejected
    Given an active procedure type "Quick Exam" with duration 15 minutes exists
    When the Practice Manager updates the duration to 10 minutes
    Then no ProcedureTypeUpdated event is recorded
    And an error is shown: "Duration must be between 15 and 240 minutes"

  # Rule PT3: Deactivate and reactivate

  Scenario: Deactivating an active procedure type
    Given an active procedure type "Whitening" exists
    When the Practice Manager deactivates procedure type "Whitening"
    Then a ProcedureTypeDeactivated event is recorded
    And "Whitening" does not appear in the scheduling procedure list

  Scenario: Deactivating an already-deactivated procedure type is rejected
    Given procedure type "Whitening" is already deactivated
    When the Practice Manager attempts to deactivate procedure type "Whitening"
    Then no ProcedureTypeDeactivated event is recorded
    And an error is shown: "Procedure type is already deactivated"

  Scenario: Reactivating a deactivated procedure type
    Given procedure type "Whitening" is deactivated
    When the Practice Manager reactivates procedure type "Whitening"
    Then a ProcedureTypeReactivated event is recorded
    And "Whitening" appears in the scheduling procedure list

  Scenario: Reactivating an active procedure type is rejected
    Given an active procedure type "Whitening" exists
    When the Practice Manager attempts to reactivate procedure type "Whitening"
    Then no ProcedureTypeReactivated event is recorded
    And an error is shown: "Procedure type is not deactivated"

  Scenario: Deactivated procedure type referenced in past appointments is preserved
    Given procedure type "Root Canal" is referenced in historical appointments
    When the Practice Manager deactivates procedure type "Root Canal"
    Then a ProcedureTypeDeactivated event is recorded
    And historical appointments still display "Root Canal" by name

  # Rule PT4: Seed defaults

  Scenario: Accepting seed defaults when no procedures exist
    Given no procedure types exist in the system
    When the Practice Manager accepts the seed defaults
    Then 10 ProcedureTypeDefined events are recorded
    And the procedure list includes "Cleaning" (Preventive, 30 min), "Root Canal" (Invasive, 90 min), and "Consultation" (Consult, 30 min)
    And all seeded procedures are active

  # Rule PT5: Procedure type setup checklist step

  Scenario: Procedure type step complete when at least one active procedure exists
    Given one active procedure type "Cleaning" exists
    When the setup checklist is evaluated
    Then the procedure type setup step is "complete"

  Scenario: Procedure type step satisfied by seeded defaults
    Given seed defaults have been accepted
    When the setup checklist is evaluated
    Then the procedure type setup step is "complete"

  Scenario: Procedure type step incomplete when all types are deactivated
    Given all procedure types are deactivated
    When the setup checklist is evaluated
    Then the procedure type setup step is "incomplete"

  # ─────────────────────────────────────────────────────────────
  # SETUP CHECKLIST
  # ─────────────────────────────────────────────────────────────

  # Rule SC1: Five independent checklist steps

  Scenario: All five setup steps complete — practice is ready to schedule
    Given the Staff Management step is complete
    And the Practice identity step is complete
    And the Office step is complete
    And the Provider step is complete
    And the Procedure Type step is complete
    When the setup checklist is evaluated
    Then the checklist shows "5 of 5 complete"
    And the "ready to schedule" indicator is shown

  Scenario: Four of five steps complete — not ready to schedule
    Given the Staff Management step is incomplete
    And the Practice identity step is complete
    And the Office step is complete
    And the Provider step is complete
    And the Procedure Type step is complete
    When the setup checklist is evaluated
    Then the checklist shows "4 of 5 complete"
    And the "ready to schedule" indicator is not shown
    And the incomplete step is identified as "Staff Management"

  # Rule SC2: Checklist is non-blocking

  Scenario: User can navigate to office setup regardless of practice name
    Given the practice has no name configured
    When the user navigates to the Office setup screen
    Then the Office setup screen is displayed
    And no blocking prompt is shown

  Scenario: Checklist is shown as a dashboard widget, not a modal gate
    Given the application is launched
    When the user views the dashboard
    Then the setup checklist widget is displayed
    And all other navigation remains accessible

  # Rule SC3: Checklist reflects current state including reversals

  Scenario: Provider step reverts to incomplete when only qualifying staff member is archived
    Given the provider setup step is "complete" because one StaffMember with Provider role satisfies all requirements
    When the Practice Manager archives that staff member
    Then the provider setup step is "incomplete"

  Scenario: Office step reverts to incomplete when only qualifying office is archived
    Given the office setup step is "complete" because one active office has all required configuration
    When the Practice Manager archives that office
    Then the office setup step is "incomplete"

  Scenario: Procedure type step reverts to incomplete when all types are deactivated
    Given the procedure type setup step is "complete" with several active procedure types
    When the Practice Manager deactivates all procedure types
    Then the procedure type setup step is "incomplete"
