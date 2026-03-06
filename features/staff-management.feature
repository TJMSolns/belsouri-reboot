# Feature: Staff Management
#
# Phase 2.4 BDD Scenarios — Staff Management context
# Date: 2026-03-03
# All assumptions (SM-1 through SM-5, SM3e, SM4d, SM9e, SM9f, SM11d) confirmed by Tony 2026-03-04.
# Phase 2.5 governance: PASS (2026-03-04).
# DM-1 amendment (2026-03-06): Clinical configuration section added (SM-C1 through SM-C8).
# Provider lifecycle moved from Practice Setup to Staff Management.
#
# Covers:
#   First-run bootstrap (Rule SM1)
#   StaffMember registration (Rule SM2)
#   PIN setup — SetPIN (Rule SM3)
#   PIN change — ChangePIN (Rule SM4) [SM-1 CONFIRMED — Tony 2026-03-04: current PIN required]
#   Role assignment — additive model (Rule SM5)
#   Role removal — explicit command, last PM guard (Rule SM6)
#   Archive (Rule SM7) [SM-5 CONFIRMED — Tony 2026-03-04: no Provider cascade]
#   Unarchive (Rule SM8)
#   Active identity switching — session concern, no domain event (Rule SM9) [SM-4 CONFIRMED — Tony 2026-03-04]
#   Last Practice Manager invariant (Rule SM10)
#   Setup checklist — Staff Management step (Rule SM11)
#
# Domain language: StaffMember, PIN, PracticeManager, Provider, Staff, Register, Archive, Claim, Switch
# Banned terms in use: NONE. (No "user", "login", "password", "delete", "account".)

Feature: Staff Management

  # ─────────────────────────────────────────────────────────────
  # FIRST-RUN BOOTSTRAP
  # Rule SM1: First StaffMember always gets PracticeManager role via ClaimPracticeManagerRole
  # ─────────────────────────────────────────────────────────────

  Scenario: First person claims PracticeManager role on fresh install
    Given no active Practice Manager exists in the system
    When the first person submits ClaimPracticeManagerRole with name "Dr. Spence"
    Then a StaffMemberRegistered event is recorded with name "Dr. Spence"
    And a PracticeManagerClaimed event is recorded
    And a RoleAssigned event is recorded for role "PracticeManager"

  Scenario: ClaimPracticeManagerRole is rejected when a Practice Manager already exists
    Given an active Practice Manager "Dr. Spence" exists
    When a person submits ClaimPracticeManagerRole with name "Dr. Brown"
    Then no StaffMemberRegistered event is recorded
    And an error is shown: "A Practice Manager already exists"

  Scenario: ClaimPracticeManagerRole with empty name is rejected
    Given no active Practice Manager exists in the system
    When the first person submits ClaimPracticeManagerRole with name ""
    Then no StaffMemberRegistered event is recorded
    And an error is shown: "Name is required"

  Scenario: Setup checklist Staff Management step is incomplete after bootstrap until PIN is set
    Given no active Practice Manager exists in the system
    When the first person submits ClaimPracticeManagerRole with name "Dr. Spence"
    And the setup checklist is evaluated
    Then the staff management setup step is "incomplete"

  Scenario: Subsequent staff registration requires an existing Practice Manager after bootstrap
    Given an active Practice Manager "Dr. Spence" exists
    When the Practice Manager registers "Maria" with initial role "Staff"
    Then a StaffMemberRegistered event is recorded with name "Maria"
    And a RoleAssigned event is recorded for role "Staff"

  # ─────────────────────────────────────────────────────────────
  # STAFF MEMBER REGISTRATION
  # Rule SM2: Name required, contact info optional, at least one role
  # ─────────────────────────────────────────────────────────────

  Scenario: Registering a staff member with name and role only
    Given an active Practice Manager exists
    When the Practice Manager registers "Maria" with initial role "Staff"
    Then a StaffMemberRegistered event is recorded with name "Maria"
    And a RoleAssigned event is recorded for role "Staff"
    And the registered staff member has no phone or email set

  Scenario: Registering a staff member with full contact info
    Given an active Practice Manager exists
    When the Practice Manager registers "Dr. Brown" with initial role "Provider", phone "876-555-0100", email "brown@clinic.com", and preferred contact channel "WhatsApp"
    Then a StaffMemberRegistered event is recorded with name "Dr. Brown", phone "876-555-0100", email "brown@clinic.com", and preferred_contact_channel "WhatsApp"
    And a RoleAssigned event is recorded for role "Provider"

  Scenario: Registering a staff member with empty name is rejected
    Given an active Practice Manager exists
    When the Practice Manager registers "" with initial role "Staff"
    Then no StaffMemberRegistered event is recorded
    And an error is shown: "Name is required"

  Scenario: Registering a staff member with whitespace-only name is rejected
    Given an active Practice Manager exists
    When the Practice Manager registers "   " with initial role "Staff"
    Then no StaffMemberRegistered event is recorded
    And an error is shown: "Name is required"

  Scenario: Registering a staff member with an invalid role is rejected
    Given an active Practice Manager exists
    When the Practice Manager registers "Sandra" with initial role "Admin"
    Then no StaffMemberRegistered event is recorded
    And an error is shown: "Role must be PracticeManager, Provider, or Staff"

  Scenario: Newly registered staff member has no PIN and cannot switch to active identity
    Given an active Practice Manager exists
    When the Practice Manager registers "Maria" with initial role "Staff"
    Then "Maria" is active in the staff list
    And "Maria" cannot be selected as the active identity because no PIN is set

  Scenario: Practice Manager can register another Practice Manager
    Given an active Practice Manager "Dr. Spence" exists
    When the Practice Manager registers "Dr. Brown" with initial role "PracticeManager"
    Then a StaffMemberRegistered event is recorded with name "Dr. Brown"
    And a RoleAssigned event is recorded for role "PracticeManager"

  # ─────────────────────────────────────────────────────────────
  # PIN SETUP
  # Rule SM3: PIN required before StaffMember can switch to active identity
  # ─────────────────────────────────────────────────────────────

  Scenario: Staff member sets PIN for the first time
    Given an active staff member "Maria" exists with no PIN set
    When "Maria" submits SetPIN with new_pin "5678"
    Then a PINSet event is recorded for "Maria"
    And the stored value is a hash of "5678", not the raw PIN

  Scenario: Staff member with PIN set can switch to active identity
    Given an active staff member "Maria" has a PIN set
    When the application verifies "Maria" with her correct PIN
    Then "Maria" becomes the active identity

  Scenario: Staff member cannot switch to active identity without a PIN
    Given an active staff member "Maria" exists with no PIN set
    When the application attempts to switch to "Maria" as the active identity
    Then the switch is rejected: "PIN not set — please set your PIN before switching"

  Scenario: Setting PIN when one is already set is rejected
    Given an active staff member "Maria" has a PIN set
    When "Maria" submits SetPIN with new_pin "9999"
    Then no PINSet event is recorded
    And an error is shown: "PIN already set — use ChangePIN to update it"

  Scenario: Archived staff member cannot set a PIN
    Given staff member "Maria" is archived
    When "Maria" submits SetPIN with new_pin "5678"
    Then no PINSet event is recorded
    And an error is shown: "Cannot modify an archived staff member"

  Scenario: SetPIN with minimum length PIN of 4 digits succeeds
    Given an active staff member "Maria" exists with no PIN set
    When "Maria" submits SetPIN with new_pin "1234"
    Then a PINSet event is recorded for "Maria"

  Scenario: SetPIN with maximum length PIN of 6 digits succeeds
    Given an active staff member "Maria" exists with no PIN set
    When "Maria" submits SetPIN with new_pin "123456"
    Then a PINSet event is recorded for "Maria"

  # ─────────────────────────────────────────────────────────────
  # PIN CHANGE
  # Rule SM4: ChangePIN requires current PIN verification [SM-1 CONFIRMED — Tony 2026-03-04]
  # ─────────────────────────────────────────────────────────────

  Scenario: Staff member changes PIN by providing correct current PIN
    Given an active staff member "Maria" has PIN "5678" set
    When "Maria" submits ChangePIN with current_pin "5678" and new_pin "9999"
    Then a PINChanged event is recorded for "Maria"
    And "Maria" can subsequently switch identity using PIN "9999"

  Scenario: ChangePIN with incorrect current PIN is rejected
    Given an active staff member "Maria" has PIN "5678" set
    When "Maria" submits ChangePIN with current_pin "0000" and new_pin "9999"
    Then no PINChanged event is recorded
    And an error is shown: "Current PIN does not match"

  Scenario: ChangePIN when no PIN has been set is rejected
    Given an active staff member "Maria" exists with no PIN set
    When "Maria" submits ChangePIN with current_pin "5678" and new_pin "9999"
    Then no PINChanged event is recorded
    And an error is shown: "No PIN set — use SetPIN to establish a PIN first"

  Scenario: Archived staff member cannot change their PIN
    Given staff member "Maria" is archived
    When "Maria" submits ChangePIN with current_pin "5678" and new_pin "9999"
    Then no PINChanged event is recorded
    And an error is shown: "Cannot modify an archived staff member"

  # ─────────────────────────────────────────────────────────────
  # Rule SM4b: Practice Manager can reset any staff member's PIN
  # [SM11d CONFIRMED — Tony 2026-03-04: PM reset resolves forgotten PIN without support call]
  # ─────────────────────────────────────────────────────────────

  Scenario: Practice Manager resets a staff member's forgotten PIN
    Given an active staff member "Maria" has PIN "5678" set
    And an active Practice Manager "Dr. Brown" exists
    When "Dr. Brown" submits ResetPIN for "Maria"
    Then a PINReset event is recorded for "Maria" with reset_by "Dr. Brown"
    And "Maria" no longer has a PIN set
    And "Maria" must use SetPIN before switching to active identity

  Scenario: ResetPIN by a non-Practice-Manager is rejected
    Given an active staff member "Maria" has PIN "5678" set
    And an active staff member "Carlos" holds role "Staff"
    When "Carlos" submits ResetPIN for "Maria"
    Then no PINReset event is recorded
    And an error is shown: "Only a Practice Manager can reset a PIN"

  Scenario: Practice Manager cannot reset their own PIN via ResetPIN
    Given an active Practice Manager "Dr. Brown" has PIN "1234" set
    When "Dr. Brown" submits ResetPIN for "Dr. Brown"
    Then no PINReset event is recorded
    And an error is shown: "Use ChangePIN to update your own PIN"

  # ─────────────────────────────────────────────────────────────
  # ROLE ASSIGNMENT
  # Rule SM5: Roles are additive and not mutually exclusive
  # ─────────────────────────────────────────────────────────────

  Scenario: Practice Manager assigns an additional role to a staff member
    Given an active staff member "Dr. Brown" holds role "Provider"
    When the Practice Manager assigns role "PracticeManager" to "Dr. Brown"
    Then a RoleAssigned event is recorded for "Dr. Brown" with role "PracticeManager"
    And "Dr. Brown" now holds both "Provider" and "PracticeManager" roles

  Scenario: Assigning a role the staff member already holds is rejected
    Given an active staff member "Maria" holds role "Staff"
    When the Practice Manager assigns role "Staff" to "Maria"
    Then no RoleAssigned event is recorded
    And an error is shown: "Staff member already holds the Staff role"

  Scenario: Assigning Provider role to a Staff role holder
    Given an active staff member "Maria" holds role "Staff"
    When the Practice Manager assigns role "Provider" to "Maria"
    Then a RoleAssigned event is recorded for "Maria" with role "Provider"
    And "Maria" holds both "Staff" and "Provider" roles

  Scenario: Assigning a role to an archived staff member is rejected
    Given staff member "Maria" is archived
    When the Practice Manager assigns role "Provider" to "Maria"
    Then no RoleAssigned event is recorded
    And an error is shown: "Cannot modify an archived staff member"

  Scenario: A staff member can hold all three roles simultaneously
    Given an active staff member "Dr. Brown" holds roles "Provider" and "Staff"
    When the Practice Manager assigns role "PracticeManager" to "Dr. Brown"
    Then a RoleAssigned event is recorded for "Dr. Brown" with role "PracticeManager"
    And "Dr. Brown" holds roles "PracticeManager", "Provider", and "Staff"

  # ─────────────────────────────────────────────────────────────
  # ROLE REMOVAL
  # Rule SM6: Explicit command; cannot remove last role or last PM's PracticeManager role
  # ─────────────────────────────────────────────────────────────

  Scenario: Removing one role from a staff member who holds multiple roles
    Given an active staff member "Dr. Brown" holds roles "Provider" and "PracticeManager"
    And at least one other active Practice Manager exists
    When the Practice Manager removes role "Provider" from "Dr. Brown"
    Then a RoleRemoved event is recorded for "Dr. Brown" with role "Provider"
    And "Dr. Brown" now holds only the "PracticeManager" role

  Scenario: Removing the last role from a staff member is rejected
    Given an active staff member "Maria" holds only the "Staff" role
    When the Practice Manager removes role "Staff" from "Maria"
    Then no RoleRemoved event is recorded
    And an error is shown: "Cannot remove the last role from a staff member"

  Scenario: Removing PracticeManager role from the last active Practice Manager is rejected
    Given "Dr. Spence" is the only active Practice Manager
    When the Practice Manager attempts to remove role "PracticeManager" from "Dr. Spence"
    Then no RoleRemoved event is recorded
    And an error is shown: "Cannot remove the PracticeManager role from the last active Practice Manager"

  Scenario: Removing PracticeManager role is allowed when another Practice Manager exists
    Given "Dr. Spence" and "Dr. Brown" are both active Practice Managers
    When the Practice Manager removes role "PracticeManager" from "Dr. Brown"
    Then a RoleRemoved event is recorded for "Dr. Brown" with role "PracticeManager"
    And "Dr. Spence" remains the active Practice Manager

  Scenario: Removing a role the staff member does not hold is rejected
    Given an active staff member "Maria" holds only the "Staff" role
    When the Practice Manager removes role "Provider" from "Maria"
    Then no RoleRemoved event is recorded
    And an error is shown: "Staff member does not hold the Provider role"

  Scenario: Removing a role from an archived staff member is rejected
    Given staff member "Maria" is archived
    When the Practice Manager removes role "Staff" from "Maria"
    Then no RoleRemoved event is recorded
    And an error is shown: "Cannot modify an archived staff member"

  # ─────────────────────────────────────────────────────────────
  # ARCHIVE
  # Rule SM7: Soft-delete; blocked if last active PM; no auto-cascade to Provider [SM-5 CONFIRMED — Tony 2026-03-04]
  # ─────────────────────────────────────────────────────────────

  Scenario: Archiving an active staff member
    Given an active staff member "Maria" exists
    When the Practice Manager archives "Maria"
    Then a StaffMemberArchived event is recorded for "Maria"
    And "Maria" does not appear in the active staff list

  Scenario: Archiving the last active Practice Manager is rejected
    Given "Dr. Spence" is the only active Practice Manager
    When the Practice Manager archives "Dr. Spence"
    Then no StaffMemberArchived event is recorded
    And an error is shown: "Cannot archive the last active Practice Manager. Assign the Practice Manager role to another staff member first."

  Scenario: Archiving a Practice Manager when another Practice Manager exists is allowed
    Given "Dr. Spence" and "Dr. Brown" are both active Practice Managers
    When the Practice Manager archives "Dr. Brown"
    Then a StaffMemberArchived event is recorded for "Dr. Brown"
    And "Dr. Spence" remains active as Practice Manager

  Scenario: Archiving an already-archived staff member is rejected
    Given staff member "Maria" is archived
    When the Practice Manager attempts to archive "Maria"
    Then no StaffMemberArchived event is recorded
    And an error is shown: "Staff member is already archived"

  Scenario: Archived staff member historical attributions are preserved
    Given staff member "Maria" has been archived
    And historical records show "Maria" as the actor on past domain changes
    When the Practice Manager views those historical records
    Then the records still display "Maria" as the actor

  # ─────────────────────────────────────────────────────────────
  # UNARCHIVE
  # Rule SM8: Restores StaffMember to active with prior roles intact
  # ─────────────────────────────────────────────────────────────

  Scenario: Unarchiving an archived staff member
    Given staff member "Maria" is archived
    When the Practice Manager unarchives "Maria"
    Then a StaffMemberUnarchived event is recorded for "Maria"
    And "Maria" appears in the active staff list with her prior roles

  Scenario: Unarchiving an active staff member is rejected
    Given an active staff member "Maria" exists
    When the Practice Manager attempts to unarchive "Maria"
    Then no StaffMemberUnarchived event is recorded
    And an error is shown: "Staff member is not archived"

  Scenario: Unarchived staff member with PIN set can immediately switch to active identity
    Given staff member "Maria" had a PIN set before being archived
    When the Practice Manager unarchives "Maria"
    Then a StaffMemberUnarchived event is recorded for "Maria"
    And "Maria" can be selected as the active identity using her PIN

  Scenario: Unarchived staff member without PIN still cannot switch to active identity
    Given staff member "Maria" had no PIN set before being archived
    When the Practice Manager unarchives "Maria"
    Then a StaffMemberUnarchived event is recorded for "Maria"
    And "Maria" cannot be selected as the active identity because no PIN is set

  # ─────────────────────────────────────────────────────────────
  # ACTIVE IDENTITY SWITCHING
  # Rule SM9: Session concern, not a domain event [SM-4 CONFIRMED — Tony 2026-03-04]
  # ─────────────────────────────────────────────────────────────

  Scenario: Staff member switches to active identity with correct PIN
    Given an active staff member "Maria" has PIN "5678" set
    When "Maria" enters PIN "5678" at the identity switching screen
    Then "Maria" is recorded as the active identity in the application
    And no domain event is emitted for the identity switch

  Scenario: Identity switch rejected for incorrect PIN
    Given an active staff member "Maria" has PIN "5678" set
    When "Maria" enters PIN "0000" at the identity switching screen
    Then the identity switch is rejected: "Incorrect PIN"
    And the active identity is unchanged

  Scenario: Identity switch rejected when no PIN is set
    Given an active staff member "Maria" exists with no PIN set
    When the identity switching screen shows "Maria"
    Then no PIN entry field is shown for "Maria"
    And a "PIN not set" indicator is displayed for "Maria"

  Scenario: Archived staff member cannot be selected as the active identity
    Given staff member "Maria" is archived
    When the identity switching screen is displayed
    Then "Maria" does not appear on the identity switching screen

  # ─────────────────────────────────────────────────────────────
  # LAST PRACTICE MANAGER INVARIANT
  # Rule SM10: System must always have at least one active PracticeManager
  # ─────────────────────────────────────────────────────────────

  Scenario: Archiving the sole Practice Manager is rejected
    Given "Dr. Spence" is the only active Practice Manager
    When the Practice Manager archives "Dr. Spence"
    Then no StaffMemberArchived event is recorded
    And an error is shown: "Cannot archive the last active Practice Manager. Assign the Practice Manager role to another staff member first."

  Scenario: Removing PracticeManager role from the sole Practice Manager is rejected
    Given "Dr. Spence" is the only active Practice Manager
    When the Practice Manager removes role "PracticeManager" from "Dr. Spence"
    Then no RoleRemoved event is recorded
    And an error is shown: "Cannot remove the PracticeManager role from the last active Practice Manager"

  Scenario: Archiving a Practice Manager when two exist leaves one PM active
    Given "Dr. Spence" and "Dr. Brown" are both active Practice Managers
    When the Practice Manager archives "Dr. Brown"
    Then a StaffMemberArchived event is recorded for "Dr. Brown"
    And "Dr. Spence" is the remaining active Practice Manager

  Scenario: Removing PracticeManager from one PM when two exist is allowed
    Given "Dr. Spence" and "Dr. Brown" are both active Practice Managers
    When the Practice Manager removes role "PracticeManager" from "Dr. Brown"
    Then a RoleRemoved event is recorded for "Dr. Brown" with role "PracticeManager"
    And "Dr. Spence" is the remaining active Practice Manager

  Scenario: Last PM guard applies even when the last PM holds multiple roles
    Given "Dr. Spence" holds roles "PracticeManager", "Provider", and "Staff"
    And "Dr. Spence" is the only active Practice Manager
    When the Practice Manager archives "Dr. Spence"
    Then no StaffMemberArchived event is recorded
    And an error is shown: "Cannot archive the last active Practice Manager. Assign the Practice Manager role to another staff member first."

  # ─────────────────────────────────────────────────────────────
  # SETUP CHECKLIST — STAFF MANAGEMENT STEP
  # Rule SM11: Requires at least one active PM with a PIN set
  # ─────────────────────────────────────────────────────────────

  Scenario: Staff Management step is incomplete when PM has no PIN set
    Given "Dr. Spence" holds the "PracticeManager" role
    And "Dr. Spence" has no PIN set
    When the setup checklist is evaluated
    Then the staff management setup step is "incomplete"

  Scenario: Staff Management step is complete when one active PM has a PIN set
    Given "Dr. Spence" holds the "PracticeManager" role
    And "Dr. Spence" has a PIN set
    When the setup checklist is evaluated
    Then the staff management setup step is "complete"

  Scenario: Staff Management step remains complete when one of two PMs is archived
    Given "Dr. Spence" holds the "PracticeManager" role with PIN set
    And "Dr. Brown" holds the "PracticeManager" role with PIN set
    When the Practice Manager archives "Dr. Brown"
    And the setup checklist is evaluated
    Then the staff management setup step is "complete"

  Scenario: Staff Management step reverts to incomplete if the only qualifying PM has their PM role removed
    Given "Dr. Spence" is the only active Practice Manager with a PIN set
    And "Dr. Brown" is an active Practice Manager without a PIN set
    When the Practice Manager removes role "PracticeManager" from "Dr. Spence"
    And the setup checklist is evaluated
    Then the staff management setup step is "incomplete"

  # ─────────────────────────────────────────────────────────────
  # CLINICAL CONFIGURATION (DM-1, 2026-03-06)
  # Provider IS A StaffMember — clinical config lives on StaffMember aggregate
  # Rules SM-C1 through SM-C8 from dm1-staff-provider-merge-examples.md
  # ─────────────────────────────────────────────────────────────

  # Rule SM-C1: Provider role is required before clinical configuration commands are accepted

  Scenario: Setting ClinicalSpecialization is rejected when staff member does not hold Provider role
    Given an active staff member "Maria Brown" holds only the "Staff" role
    When the Practice Manager submits SetProviderType for "Maria Brown" with ClinicalSpecialization "Hygienist"
    Then no ProviderTypeSet event is recorded
    And an error is shown: "Maria Brown does not hold the Provider role. Assign it before configuring clinical details."

  Scenario: Setting ClinicalSpecialization succeeds when staff member holds Provider role
    Given an active staff member "Dr. Brown" holds the "Provider" role
    When the Practice Manager submits SetProviderType for "Dr. Brown" with ClinicalSpecialization "Dentist"
    Then a ProviderTypeSet event is recorded with staff_member_id "Dr. Brown" and clinical_specialization "Dentist"

  Scenario: Assigning provider to office succeeds when staff member holds Provider role
    Given an active staff member "Dr. Brown" holds the "Provider" role
    And an active office "Kingston" exists
    When the Practice Manager submits AssignProviderToOffice for "Dr. Brown" at "Kingston"
    Then a ProviderAssignedToOffice event is recorded with staff_member_id "Dr. Brown" and office "Kingston"

  Scenario: A StaffMember can hold Provider role without a ClinicalSpecialization and is excluded from scheduling
    Given an active staff member "Dr. Brown" holds the "Provider" role
    And no ClinicalSpecialization has been set for "Dr. Brown"
    When Scheduling queries available providers at any office
    Then "Dr. Brown" is not returned

  # Rule SM-C2: ClinicalSpecialization must be set before a provider appears in scheduling

  Scenario: Provider without ClinicalSpecialization does not appear in scheduling queries
    Given "Dr. Brown" holds the "Provider" role with no ClinicalSpecialization set
    And "Dr. Brown" is assigned to office "Kingston" with Monday availability set
    When Scheduling queries available providers at "Kingston"
    Then "Dr. Brown" is not returned

  Scenario: Provider with ClinicalSpecialization appears in scheduling after full configuration
    Given "Dr. Brown" holds the "Provider" role with ClinicalSpecialization "Dentist"
    And "Dr. Brown" is assigned to office "Kingston" with Monday 08:00-17:00 availability
    When Scheduling queries available providers at "Kingston" for Monday
    Then "Dr. Brown" is returned as a "Dentist"

  Scenario: Booking rejected when provider specialization does not match procedure requirement
    Given "Dr. Brown" holds the "Provider" role with ClinicalSpecialization "Hygienist"
    When Patient Scheduling attempts to book a "Root Canal" (requires Dentist) with "Dr. Brown"
    Then the booking is rejected: "Dr. Brown is a Hygienist; this procedure requires a Dentist."

  # Rule SM-C3: Office assignment required before setting availability

  Scenario: Setting availability at assigned office succeeds
    Given "Dr. Brown" holds the "Provider" role and is assigned to office "Kingston"
    When the Practice Manager sets Monday 08:00-17:00 availability for "Dr. Brown" at "Kingston"
    Then a ProviderAvailabilitySet event is recorded with staff_member_id "Dr. Brown", office "Kingston", day Monday

  Scenario: Setting availability at unassigned office is rejected
    Given "Dr. Brown" holds the "Provider" role
    And "Dr. Brown" is not assigned to office "Kingston"
    When the Practice Manager sets Monday 08:00-17:00 availability for "Dr. Brown" at "Kingston"
    Then no ProviderAvailabilitySet event is recorded
    And an error is shown: "Dr. Brown is not assigned to Kingston. Assign them to this office first."

  # Rule SM-C4: No cross-office availability overlap on the same day

  Scenario: Non-overlapping cross-office availability on the same day is accepted
    Given "Dr. Brown" is assigned to offices "Kingston" and "Montego Bay"
    And "Dr. Brown" has Monday 08:00-12:00 availability at "Kingston"
    When the Practice Manager sets Monday 13:00-17:00 availability for "Dr. Brown" at "Montego Bay"
    Then a ProviderAvailabilitySet event is recorded for "Montego Bay" Monday 13:00-17:00

  Scenario: Overlapping cross-office availability on the same day is rejected
    Given "Dr. Brown" is assigned to offices "Kingston" and "Montego Bay"
    And "Dr. Brown" has Monday 08:00-14:00 availability at "Kingston"
    When the Practice Manager sets Monday 12:00-17:00 availability for "Dr. Brown" at "Montego Bay"
    Then no ProviderAvailabilitySet event is recorded
    And an error is shown: "Dr. Brown has overlapping availability at Kingston on Monday (08:00-14:00)."

  Scenario: Adjacent availability windows at two offices on the same day are allowed
    Given "Dr. Brown" is assigned to offices "Kingston" and "Montego Bay"
    And "Dr. Brown" has Monday 08:00-12:00 availability at "Kingston"
    When the Practice Manager sets Monday 12:00-17:00 availability for "Dr. Brown" at "Montego Bay"
    Then a ProviderAvailabilitySet event is recorded for "Montego Bay" Monday 12:00-17:00

  # Rule SM-C5: Exceptions are provider-wide and override all availability

  Scenario: Setting an exception blocks provider at all offices for the date range
    Given "Dr. Brown" is assigned to offices "Kingston" and "Montego Bay" with availability configured
    When the Practice Manager sets an exception for "Dr. Brown" from 2026-12-20 to 2026-12-31 with reason "Holiday vacation"
    Then a ProviderExceptionSet event is recorded with start_date 2026-12-20, end_date 2026-12-31, reason "Holiday vacation"
    And "Dr. Brown" is unavailable at both "Kingston" and "Montego Bay" from 2026-12-20 to 2026-12-31

  Scenario: Setting an exception with existing appointments in range warns but proceeds
    Given "Dr. Brown" has 3 appointments booked between 2026-12-20 and 2026-12-31
    When the Practice Manager sets an exception for "Dr. Brown" from 2026-12-20 to 2026-12-31
    Then a ProviderExceptionSet event is recorded
    And a warning is shown: "3 appointments exist in this date range — they will not be cancelled."

  # Rule SM-C6: Archiving a StaffMember with Provider role removes them from scheduling

  Scenario: Archiving a provider removes them from scheduling queries
    Given "Dr. Brown" holds the "Provider" role with availability configured at "Kingston"
    When the Practice Manager archives "Dr. Brown"
    Then a StaffMemberArchived event is recorded for "Dr. Brown"
    And Scheduling queries for providers at "Kingston" do not return "Dr. Brown"

  Scenario: Booking an appointment with an archived StaffMember is rejected
    Given "Dr. Brown" is archived
    When Patient Scheduling attempts to book an appointment with "Dr. Brown"
    Then the booking is rejected: "Dr. Brown is not available (archived)."

  # Rule SM-C7: Provider role removal preserves clinical config but hides provider from scheduling

  Scenario: Removing Provider role hides provider from scheduling without clearing clinical config
    Given "Dr. Brown" holds the "Provider" role with Monday availability at "Kingston"
    When the Practice Manager removes the "Provider" role from "Dr. Brown"
    Then a RoleRemoved event is recorded for "Dr. Brown" with role "Provider"
    And Scheduling queries for providers at "Kingston" do not return "Dr. Brown"
    And "Dr. Brown"'s clinical configuration (office assignments and availability) is preserved

  Scenario: Re-adding Provider role restores provider to scheduling with prior config intact
    Given the "Provider" role was previously removed from "Dr. Brown"
    And "Dr. Brown"'s clinical configuration (Monday availability at "Kingston") was preserved
    When the Practice Manager assigns the "Provider" role to "Dr. Brown"
    Then a RoleAssigned event is recorded for "Dr. Brown" with role "Provider"
    And Scheduling queries for providers at "Kingston" for Monday return "Dr. Brown"

  Scenario: Clinical configuration commands are rejected after Provider role removal
    Given the "Provider" role was previously removed from "Dr. Brown"
    When the Practice Manager submits SetProviderType for "Dr. Brown" with ClinicalSpecialization "Specialist"
    Then no ProviderTypeSet event is recorded
    And an error is shown: "Dr. Brown does not hold the Provider role. Assign it before configuring clinical details."

  # Rule SM-C8: Setup checklist provider step reads from Staff Management

  Scenario: Provider step is incomplete when StaffMember holds Provider role but has no ClinicalSpecialization
    Given "Dr. Brown" holds the "Provider" role
    And "Dr. Brown" has no ClinicalSpecialization set
    When the setup checklist is evaluated
    Then the provider setup step is "incomplete"

  Scenario: Provider step is incomplete when ClinicalSpecialization is set but no office is assigned
    Given "Dr. Brown" holds the "Provider" role with ClinicalSpecialization "Dentist"
    And "Dr. Brown" is not assigned to any office
    When the setup checklist is evaluated
    Then the provider setup step is "incomplete"

  Scenario: Provider step is incomplete when assigned to office but no availability set
    Given "Dr. Brown" holds the "Provider" role with ClinicalSpecialization "Dentist"
    And "Dr. Brown" is assigned to office "Kingston" with no availability set
    When the setup checklist is evaluated
    Then the provider setup step is "incomplete"

  Scenario: Provider step is complete when all criteria are satisfied
    Given "Dr. Brown" holds the "Provider" role with ClinicalSpecialization "Dentist"
    And "Dr. Brown" is assigned to office "Kingston" with Monday 08:00-17:00 availability
    When the setup checklist is evaluated
    Then the provider setup step is "complete"

  Scenario: Provider step reverts to incomplete when qualifying StaffMember is archived
    Given the provider setup step is "complete" because "Dr. Brown" satisfies all criteria
    When the Practice Manager archives "Dr. Brown"
    Then the provider setup step is "incomplete"

  Scenario: Provider step reverts to incomplete when Provider role is removed from only qualifying StaffMember
    Given the provider setup step is "complete" because "Dr. Brown" satisfies all criteria
    When the Practice Manager removes the "Provider" role from "Dr. Brown"
    Then the provider setup step is "incomplete"

  # ─────────────────────────────────────────────────────────────
  # STAFF SHIFT ROSTER (SCH-5)
  # Phase 2.4 BDD Scenarios — StaffShift aggregate
  # Date: 2026-03-05
  # Confirmed by Tony (2026-03-05):
  #   - Planned (future-facing), ad-hoc (not fixed weekly)
  #   - Both PM and staff member themselves can plan shifts
  #   - A Staff role holder cannot plan shifts for other staff members
  #   - Role at shift time must be one of the staff member's assigned roles
  #   - end_time must be strictly after start_time
  #   - Cancelled shifts are soft-deleted — visible in history, greyed out in UI
  #   - UI: Roster tab on Schedule page + per-person upcoming view on Staff page
  #   - Shifts are informational — no booking constraints
  # ─────────────────────────────────────────────────────────────

  Background: Staff Shift Roster test data
    Given the practice has offices "Kingston" and "Montego Bay"
    And staff member "Maria Brown" exists with role "Staff" and is not archived
    And staff member "John Clarke" exists with role "Staff" and is not archived
    And "Dr. Reid" is an active Practice Manager

  # ─────────────────────────────────────────────────────────────
  # Rule SCH5-1: A Practice Manager can plan a shift for any active staff member
  # ─────────────────────────────────────────────────────────────

  Scenario: Practice Manager plans a shift for an active staff member
    Given "Dr. Reid" holds the "PracticeManager" role
    When "Dr. Reid" submits PlanStaffShift for "Maria Brown" at "Kingston" on "2026-03-09" from "09:00" to "17:00" with role "Staff"
    Then a StaffShiftPlanned event is recorded with staff_member_id "Maria Brown", office "Kingston", date "2026-03-09", start_time "09:00", end_time "17:00", role "Staff", created_by "Dr. Reid"

  Scenario: Practice Manager plans a shift for an archived staff member is rejected
    Given staff member "Sandra Lee" is archived
    When "Dr. Reid" submits PlanStaffShift for "Sandra Lee" at "Kingston" on "2026-03-09" from "09:00" to "17:00" with role "Staff"
    Then no StaffShiftPlanned event is recorded
    And an error is shown: "Staff member Sandra Lee is archived and cannot be assigned a shift"

  Scenario: Practice Manager plans a shift for themselves
    Given "Dr. Reid" holds role "PracticeManager"
    When "Dr. Reid" submits PlanStaffShift for "Dr. Reid" at "Montego Bay" on "2026-03-10" from "08:00" to "16:00" with role "PracticeManager"
    Then a StaffShiftPlanned event is recorded with staff_member_id "Dr. Reid" and created_by "Dr. Reid"

  # ─────────────────────────────────────────────────────────────
  # Rule SCH5-2: A staff member can plan their own shift
  # ─────────────────────────────────────────────────────────────

  Scenario: Staff member plans their own shift
    When "Maria Brown" submits PlanStaffShift for herself at "Montego Bay" on "2026-03-13" from "09:00" to "17:00" with role "Staff"
    Then a StaffShiftPlanned event is recorded with staff_member_id "Maria Brown" and created_by "Maria Brown"

  Scenario: Staff member cannot plan a shift for a different staff member
    When "Maria Brown" submits PlanStaffShift for "John Clarke" at "Kingston" on "2026-03-09" from "09:00" to "17:00" with role "Staff"
    Then no StaffShiftPlanned event is recorded
    And an error is shown: "Maria Brown does not have the Practice Manager role and cannot plan a shift for another staff member"

  # ─────────────────────────────────────────────────────────────
  # Rule SCH5-3: Role must be one assigned to the staff member
  # ─────────────────────────────────────────────────────────────

  Scenario: Shift with a role the staff member holds is accepted
    Given "Maria Brown" holds role "Staff"
    When "Dr. Reid" submits PlanStaffShift for "Maria Brown" at "Kingston" on "2026-03-09" from "09:00" to "17:00" with role "Staff"
    Then a StaffShiftPlanned event is recorded with role "Staff"

  Scenario: Shift with a role the staff member does not hold is rejected
    Given "Maria Brown" holds role "Staff" only
    When "Dr. Reid" submits PlanStaffShift for "Maria Brown" at "Kingston" on "2026-03-09" from "09:00" to "17:00" with role "PracticeManager"
    Then no StaffShiftPlanned event is recorded
    And an error is shown: "Maria Brown does not have the PracticeManager role and cannot plan a shift in that role"

  Scenario: Staff member with multiple roles can plan a shift in either role
    Given "Dr. Reid" holds roles "PracticeManager" and "Staff"
    When "Dr. Reid" submits PlanStaffShift for "Dr. Reid" at "Kingston" on "2026-03-09" from "09:00" to "17:00" with role "Staff"
    Then a StaffShiftPlanned event is recorded with role "Staff"

  # ─────────────────────────────────────────────────────────────
  # Rule SCH5-4: end_time must be strictly after start_time
  # ─────────────────────────────────────────────────────────────

  Scenario: Shift with valid time range is accepted
    When "Dr. Reid" submits PlanStaffShift for "Maria Brown" at "Kingston" on "2026-03-09" from "09:00" to "17:00" with role "Staff"
    Then a StaffShiftPlanned event is recorded with start_time "09:00" and end_time "17:00"

  Scenario: Shift with end_time equal to start_time is rejected
    When "Dr. Reid" submits PlanStaffShift for "Maria Brown" at "Kingston" on "2026-03-09" from "09:00" to "09:00" with role "Staff"
    Then no StaffShiftPlanned event is recorded
    And an error is shown: "Shift end time must be after start time"

  Scenario: Shift with end_time before start_time is rejected
    When "Dr. Reid" submits PlanStaffShift for "Maria Brown" at "Kingston" on "2026-03-09" from "17:00" to "09:00" with role "Staff"
    Then no StaffShiftPlanned event is recorded
    And an error is shown: "Shift end time must be after start time"

  # ─────────────────────────────────────────────────────────────
  # Rule SCH5-5: Shift can be cancelled by the shift owner or a Practice Manager
  # ─────────────────────────────────────────────────────────────

  Scenario: Practice Manager cancels a staff member's shift with a reason
    Given "Maria Brown" has a planned shift at "Kingston" on "2026-03-09"
    When "Dr. Reid" submits CancelStaffShift for that shift with reason "Office closed — public holiday"
    Then a StaffShiftCancelled event is recorded with cancelled_by "Dr. Reid" and reason "Office closed — public holiday"

  Scenario: Practice Manager cancels a shift with no reason
    Given "Maria Brown" has a planned shift at "Kingston" on "2026-03-09"
    When "Dr. Reid" submits CancelStaffShift for that shift with no reason
    Then a StaffShiftCancelled event is recorded with cancelled_by "Dr. Reid" and reason null

  Scenario: Staff member cancels their own shift
    Given "Maria Brown" has a planned shift at "Kingston" on "2026-03-09"
    When "Maria Brown" submits CancelStaffShift for her own shift
    Then a StaffShiftCancelled event is recorded with cancelled_by "Maria Brown"

  Scenario: Staff member cannot cancel another staff member's shift
    Given "Maria Brown" has a planned shift at "Kingston" on "2026-03-09"
    When "John Clarke" submits CancelStaffShift for Maria Brown's shift
    Then no StaffShiftCancelled event is recorded
    And an error is shown: "John Clarke does not have the Practice Manager role and cannot cancel another staff member's shift"

  Scenario: Cancelling an already-cancelled shift is rejected
    Given "Maria Brown" has a shift that has already been cancelled
    When "Dr. Reid" submits CancelStaffShift for that shift
    Then no StaffShiftCancelled event is recorded
    And an error is shown: "This shift has already been cancelled"

  Scenario: Cancelled shift remains visible in the roster with cancelled status
    Given "Maria Brown" has a planned shift at "Kingston" on "2026-03-09"
    When "Dr. Reid" submits CancelStaffShift for that shift with reason "Office closed — public holiday"
    Then a StaffShiftCancelled event is recorded
    And the staff_shift_roster projection contains the shift with cancelled "true" and cancel_reason "Office closed — public holiday"
    And the original StaffShiftPlanned event is preserved in the event store

  # ─────────────────────────────────────────────────────────────
  # Rule SCH5-6: Roster view shows all staff for the selected week
  # ─────────────────────────────────────────────────────────────

  Scenario: Week roster shows all planned shifts across staff members
    Given "Maria Brown" has planned shifts on "2026-03-09" (Monday) and "2026-03-11" (Wednesday) at "Kingston"
    And "John Clarke" has a planned shift on "2026-03-10" (Tuesday) at "Montego Bay"
    When the roster is queried for the week "2026-03-09" to "2026-03-15"
    Then the query returns 3 shift rows
    And one row shows "Maria Brown" at "Kingston" on "2026-03-09"
    And one row shows "Maria Brown" at "Kingston" on "2026-03-11"
    And one row shows "John Clarke" at "Montego Bay" on "2026-03-10"

  Scenario: Week roster shows cancelled shifts greyed out rather than removing them
    Given "Maria Brown" has planned shifts on "2026-03-09" and "2026-03-11" at "Kingston"
    And the shift on "2026-03-09" has been cancelled
    When the roster is queried for the week "2026-03-09" to "2026-03-15"
    Then the query returns 2 shift rows
    And the row for "2026-03-09" has cancelled "true"
    And the row for "2026-03-11" has cancelled "false"

  # ─────────────────────────────────────────────────────────────
  # Rule SCH5-7: Per-person view shows upcoming shifts on the Staff page
  # ─────────────────────────────────────────────────────────────

  Scenario: Per-person view returns upcoming shifts for a specific staff member
    Given "Maria Brown" has a planned shift on "2026-03-09" at "Kingston"
    And "Maria Brown" has a planned shift on "2026-03-16" at "Montego Bay"
    And today is "2026-03-05"
    When the per-person shift list is queried for "Maria Brown" from "2026-03-05" onwards
    Then both shifts appear in the results ordered by date
    And "John Clarke"'s shifts do not appear

  Scenario: Per-person view excludes past shifts
    Given "Maria Brown" had a shift on "2026-03-01" (past) at "Kingston"
    And "Maria Brown" has an upcoming shift on "2026-03-09" at "Kingston"
    And today is "2026-03-05"
    When the per-person shift list is queried for "Maria Brown" from "2026-03-05" onwards
    Then only the "2026-03-09" shift appears in the results
    And the "2026-03-01" past shift is not returned
