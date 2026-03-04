# Feature: Staff Management
#
# Phase 2.4 BDD Scenarios — Staff Management context
# Date: 2026-03-03
# Assumptions SM-1 through SM-5 flagged — pending Tony review before 2.5 governance sign-off.
#
# Covers:
#   First-run bootstrap (Rule SM1)
#   StaffMember registration (Rule SM2)
#   PIN setup — SetPIN (Rule SM3)
#   PIN change — ChangePIN (Rule SM4) [SM-1 ASSUMED: current PIN required]
#   Role assignment — additive model (Rule SM5)
#   Role removal — explicit command, last PM guard (Rule SM6)
#   Archive (Rule SM7) [SM-5 ASSUMED: no Provider cascade]
#   Unarchive (Rule SM8)
#   Active identity switching — session concern, no domain event (Rule SM9) [SM-4 ASSUMED]
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
  # Rule SM4: ChangePIN requires current PIN verification [SM-1 ASSUMED]
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
  # Rule SM7: Soft-delete; blocked if last active PM; no auto-cascade to Provider [SM-5 ASSUMED]
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

  Scenario: Archiving a staff member with Provider role does not archive the linked Practice Setup Provider
    Given an active staff member "Dr. Brown" holds the "Provider" role
    And a Practice Setup Provider exists referencing "Dr. Brown"
    When the Practice Manager archives "Dr. Brown"
    Then a StaffMemberArchived event is recorded for "Dr. Brown"
    And the Practice Setup Provider linked to "Dr. Brown" remains active

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
  # Rule SM9: Session concern, not a domain event [SM-4 ASSUMED]
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
