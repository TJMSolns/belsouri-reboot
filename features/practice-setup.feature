# Feature: Practice Setup
#
# Phase 2.4 BDD Scenarios — Practice Setup context
# Date: 2026-03-03
# All open questions resolved or flagged — ready for Tony review before implementation.
#
# Covers:
#   Practice identity (4 rules)
#   Office lifecycle (6 rules)
#   Provider lifecycle and availability (8 rules)
#   Procedure Type lifecycle (5 rules)
#   Setup checklist (4 rules)
#
# Context: Practice Setup is the foundational bounded context.
# All scheduling depends on offices, providers, and procedure types being configured.

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
  # PROVIDER LIFECYCLE
  # ─────────────────────────────────────────────────────────────

  # Rule PR1: Provider registration

  Scenario: Registering a provider with name and type
    When the Practice Manager registers provider "Dr. Smith" as "Dentist"
    Then a ProviderRegistered event is recorded with name "Dr. Smith" and provider_type "Dentist"

  Scenario: Registering a provider with an empty name is rejected
    When the Practice Manager registers provider "" as "Dentist"
    Then no ProviderRegistered event is recorded
    And an error is shown: "Name is required"

  Scenario: Registering a provider with an invalid type is rejected
    When the Practice Manager registers provider "Dr. Smith" as "Receptionist"
    Then no ProviderRegistered event is recorded
    And an error is shown: "Provider type must be Dentist, Hygienist, or Specialist"

  # Rule PR2: Assignment required before setting availability

  Scenario: Assigning a provider to an office
    Given an active provider "Dr. Smith" exists
    And an active office "Kingston" exists
    When the Practice Manager assigns provider "Dr. Smith" to office "Kingston"
    Then a ProviderAssignedToOffice event is recorded

  Scenario: Setting availability before assignment is rejected
    Given an active provider "Dr. Smith" exists
    And an active office "Kingston" exists
    And provider "Dr. Smith" is not assigned to office "Kingston"
    When the Practice Manager sets Monday 08:00-17:00 availability for "Dr. Smith" at "Kingston"
    Then no ProviderAvailabilitySet event is recorded
    And an error is shown: "Provider must be assigned to this office first"

  Scenario: Setting availability after assignment succeeds
    Given an active provider "Dr. Smith" is assigned to office "Kingston"
    When the Practice Manager sets Monday 08:00-17:00 availability for "Dr. Smith" at "Kingston"
    Then a ProviderAvailabilitySet event is recorded with office "Kingston", day Monday, start 08:00, end 17:00

  Scenario: Assigning a provider already assigned to that office is rejected
    Given provider "Dr. Smith" is already assigned to office "Kingston"
    When the Practice Manager assigns provider "Dr. Smith" to office "Kingston"
    Then no ProviderAssignedToOffice event is recorded
    And an error is shown: "Provider is already assigned to this office"

  # Rule PR3: Removing from office clears availability

  Scenario: Removing provider from office clears all availability at that office
    Given provider "Dr. Smith" is assigned to office "Kingston"
    And provider "Dr. Smith" has availability Mon, Wed, Fri at office "Kingston"
    When the Practice Manager removes provider "Dr. Smith" from office "Kingston"
    Then a ProviderRemovedFromOffice event is recorded
    And 3 ProviderAvailabilityCleared events are recorded for Monday, Wednesday, Friday at "Kingston"

  Scenario: Removing provider from office with no availability only emits removal event
    Given provider "Dr. Smith" is assigned to office "Kingston"
    And provider "Dr. Smith" has no availability set at office "Kingston"
    When the Practice Manager removes provider "Dr. Smith" from office "Kingston"
    Then a ProviderRemovedFromOffice event is recorded
    And no ProviderAvailabilityCleared events are recorded

  Scenario: Removing provider from office they are not assigned to is rejected
    Given provider "Dr. Smith" is not assigned to office "Montego Bay"
    When the Practice Manager removes provider "Dr. Smith" from office "Montego Bay"
    Then no ProviderRemovedFromOffice event is recorded
    And an error is shown: "Provider is not assigned to this office"

  # Rule PR4: No cross-office availability overlap on same day

  Scenario: Setting non-overlapping availability at two offices on the same day
    Given provider "Dr. Smith" is assigned to offices "Kingston" and "Montego Bay"
    And provider "Dr. Smith" has availability Monday 08:00-12:00 at "Kingston"
    When the Practice Manager sets Monday 13:00-17:00 availability for "Dr. Smith" at "Montego Bay"
    Then a ProviderAvailabilitySet event is recorded for "Montego Bay" Monday 13:00-17:00

  Scenario: Setting overlapping availability at a second office on the same day is rejected
    Given provider "Dr. Smith" is assigned to offices "Kingston" and "Montego Bay"
    And provider "Dr. Smith" has availability Monday 08:00-14:00 at "Kingston"
    When the Practice Manager sets Monday 12:00-17:00 availability for "Dr. Smith" at "Montego Bay"
    Then no ProviderAvailabilitySet event is recorded
    And an error is shown: "Provider has overlapping availability at Kingston on Monday (08:00-14:00)"

  Scenario: Adjacent availability windows at two offices are allowed
    Given provider "Dr. Smith" is assigned to offices "Kingston" and "Montego Bay"
    And provider "Dr. Smith" has availability Monday 08:00-12:00 at "Kingston"
    When the Practice Manager sets Monday 12:00-17:00 availability for "Dr. Smith" at "Montego Bay"
    Then a ProviderAvailabilitySet event is recorded for "Montego Bay" Monday 12:00-17:00

  # Rule PR5: Availability outside office hours warns but proceeds

  Scenario: Setting provider availability within office hours has no warning
    Given an active office "Kingston" has Monday hours 08:00-17:00
    And provider "Dr. Smith" is assigned to office "Kingston"
    When the Practice Manager sets Monday 08:00-17:00 availability for "Dr. Smith" at "Kingston"
    Then a ProviderAvailabilitySet event is recorded
    And no warning is shown

  Scenario: Setting provider availability starting before office opens warns but proceeds
    Given an active office "Kingston" has Monday hours 08:00-17:00
    And provider "Dr. Smith" is assigned to office "Kingston"
    When the Practice Manager sets Monday 07:00-17:00 availability for "Dr. Smith" at "Kingston"
    Then a ProviderAvailabilitySet event is recorded
    And a warning is shown: "Availability starts before office opens at 08:00"

  Scenario: Setting availability on a day the office is closed warns but proceeds
    Given an active office "Kingston" has no hours configured for Sunday
    And provider "Dr. Smith" is assigned to office "Kingston"
    When the Practice Manager sets Sunday 10:00-14:00 availability for "Dr. Smith" at "Kingston"
    Then a ProviderAvailabilitySet event is recorded
    And a warning is shown: "Office is closed on Sunday"

  # Rule PR6: Exceptions are provider-wide and override availability

  Scenario: Setting a provider exception blocks them at all offices for the date range
    Given provider "Dr. Smith" is assigned to offices "Kingston" and "Montego Bay"
    When the Practice Manager sets an exception for "Dr. Smith" from 2026-12-20 to 2026-12-31 with reason "Holiday vacation"
    Then a ProviderExceptionSet event is recorded with start 2026-12-20, end 2026-12-31, reason "Holiday vacation"
    And "Dr. Smith" is unavailable at both "Kingston" and "Montego Bay" from 2026-12-20 to 2026-12-31

  Scenario: Setting a single-day exception
    When the Practice Manager sets an exception for "Dr. Smith" from 2026-11-05 to 2026-11-05
    Then a ProviderExceptionSet event is recorded with start 2026-11-05 and end 2026-11-05

  Scenario: Setting an exception where end date is before start date is rejected
    When the Practice Manager sets an exception for "Dr. Smith" from 2026-12-31 to 2026-12-20
    Then no ProviderExceptionSet event is recorded
    And an error is shown: "End date must be on or after start date"

  Scenario: Setting an exception over dates with existing appointments warns but proceeds
    Given provider "Dr. Smith" has 3 appointments booked between 2026-12-20 and 2026-12-31
    When the Practice Manager sets an exception for "Dr. Smith" from 2026-12-20 to 2026-12-31
    Then a ProviderExceptionSet event is recorded
    And a warning is shown: "3 appointments exist in this date range — they will not be cancelled"

  Scenario: Removing a provider exception restores availability
    Given provider "Dr. Smith" has an exception from 2026-12-20 to 2026-12-31
    When the Practice Manager removes the exception from 2026-12-20 to 2026-12-31 for "Dr. Smith"
    Then a ProviderExceptionRemoved event is recorded
    And "Dr. Smith" is available again per their weekly availability

  Scenario: Removing an exception that does not exist is rejected
    Given provider "Dr. Smith" has no exception for 2026-12-20 to 2026-12-31
    When the Practice Manager attempts to remove exception from 2026-12-20 to 2026-12-31 for "Dr. Smith"
    Then no ProviderExceptionRemoved event is recorded
    And an error is shown: "No exception found for that date range"

  # Rule PR7: Archive and unarchive

  Scenario: Archiving an active provider
    Given an active provider "Dr. Smith" exists
    When the Practice Manager archives provider "Dr. Smith"
    Then a ProviderArchived event is recorded
    And provider "Dr. Smith" does not appear in the active provider list

  Scenario: Archiving an already-archived provider is rejected
    Given provider "Dr. Smith" is already archived
    When the Practice Manager attempts to archive provider "Dr. Smith"
    Then no ProviderArchived event is recorded
    And an error is shown: "Provider is already archived"

  Scenario: Unarchiving an archived provider
    Given provider "Dr. Smith" is archived
    When the Practice Manager unarchives provider "Dr. Smith"
    Then a ProviderUnarchived event is recorded
    And provider "Dr. Smith" appears in the active provider list

  Scenario: Unarchiving an active provider is rejected
    Given an active provider "Dr. Smith" exists
    When the Practice Manager attempts to unarchive provider "Dr. Smith"
    Then no ProviderUnarchived event is recorded
    And an error is shown: "Provider is not archived"

  Scenario: Archived provider appointments remain visible in history
    Given provider "Dr. Smith" is archived
    And historical appointments for "Dr. Smith" exist
    When the Practice Manager views historical appointments
    Then the appointments for "Dr. Smith" are visible

  # Rule PR8: Setup checklist — provider step completion

  Scenario: Provider step is incomplete when provider is registered but not assigned
    Given provider "Dr. Smith" is registered but not assigned to any office
    When the setup checklist is evaluated
    Then the provider setup step is "incomplete"

  Scenario: Provider step is incomplete when provider is assigned but has no availability
    Given provider "Dr. Smith" is assigned to office "Kingston" with no availability set
    When the setup checklist is evaluated
    Then the provider setup step is "incomplete"

  Scenario: Provider step is complete when provider is assigned and has availability for one day
    Given provider "Dr. Smith" is assigned to office "Kingston" with Monday availability set
    When the setup checklist is evaluated
    Then the provider setup step is "complete"

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

  Scenario: Provider step reverts to incomplete when only active provider is archived
    Given the provider setup step is "complete" because one provider satisfies all requirements
    When the Practice Manager archives that provider
    Then the provider setup step is "incomplete"

  Scenario: Office step reverts to incomplete when only qualifying office is archived
    Given the office setup step is "complete" because one active office has all required configuration
    When the Practice Manager archives that office
    Then the office setup step is "incomplete"

  Scenario: Procedure type step reverts to incomplete when all types are deactivated
    Given the procedure type setup step is "complete" with several active procedure types
    When the Practice Manager deactivates all procedure types
    Then the procedure type setup step is "incomplete"
