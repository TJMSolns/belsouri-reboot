# Feature: Licensing
#
# Phase 2.4 BDD Scenarios — Licensing context
# Date: 2026-03-03
# All open questions resolved — ready for implementation.
#
# Guiding principle: "Data is always yours — never blocked from it.
# If you ain't paying, you don't get value out of it."
#
# Enforcement gate: startup only.
# 48-hour in-session check: banner updates only, no write access changes.
# Read access: unconditional, regardless of module status.

Feature: Licensing

  Background:
    Given the application data directory is clean
    And the embedded eval token is valid for 30 days with all MVP modules
    And the embedded Ed25519 public key matches the License Server's signing key

  # ─────────────────────────────────────────────────────────────
  # Rule 1: Fresh install enters eval period
  # ─────────────────────────────────────────────────────────────

  Scenario: First launch establishes practice identity and starts eval
    Given no prior events exist in the event store
    When the application is launched for the first time
    Then a PracticeIdentityEstablished event is recorded
    And an EvalStarted event is recorded
    And the license status is "Eval"
    And the eval expires 30 days from today
    And the status banner shows "Trial — 30 days remaining"
    And the scheduling module is accessible with full write access

  Scenario: Second launch continues eval from install date
    Given a PracticeIdentity was established 5 days ago
    And an EvalStarted event was recorded 5 days ago
    When the application is launched
    Then no new PracticeIdentityEstablished event is recorded
    And the license status is "Eval"
    And the status banner shows "Trial — 25 days remaining"

  # ─────────────────────────────────────────────────────────────
  # Rule 2: Eval period expires
  # ─────────────────────────────────────────────────────────────

  Scenario: Eval expires on day 31
    Given a PracticeIdentity was established 31 days ago
    And an EvalStarted event was recorded 31 days ago
    When the application is launched
    Then a LicenseExpired event is recorded for module "scheduling"
    And the scheduling module status is "Expired"
    And the status banner shows that scheduling has expired

  Scenario: Eval is still active on day 30
    Given an EvalStarted event was recorded 30 days ago
    When the application is launched
    Then the scheduling module status is "Eval"
    And no LicenseExpired event is recorded

  Scenario: Existing data is always readable after eval expires
    Given the scheduling module status is "Expired"
    And patient records exist in the database
    When the user views an existing patient record
    Then the record is displayed

  Scenario: Creating a record while eval has expired is blocked
    Given the scheduling module status is "Expired"
    When the user attempts to create a new appointment
    Then the action is rejected
    And the banner shows "Scheduling has expired — renew to restore scheduling"

  # ─────────────────────────────────────────────────────────────
  # Rule 3: License key activation
  # ─────────────────────────────────────────────────────────────

  Scenario: Activating a valid paid license key
    Given the scheduling module status is "Eval"
    And a valid paid license key exists for this installation's practiceId
    When the Practice Manager enters the license key
    Then a LicenseIssued event is recorded
    And on the next application launch the scheduling module status is "Active"

  Scenario: Activating a license key with invalid signature
    Given a license key with an invalid Ed25519 signature is provided
    When the Practice Manager enters the license key
    Then no LicenseIssued event is recorded
    And an error is shown: "This license key is not valid"

  Scenario: Activating a license key for a different machine
    Given a license key for a different practiceId is provided
    When the Practice Manager enters the license key
    Then no LicenseIssued event is recorded
    And an error is shown: "This key was issued for a different installation"

  Scenario: Activating an already-expired license key
    Given a license key where all modules have expires_at in the past
    When the Practice Manager enters the license key
    Then no LicenseIssued event is recorded
    And an error is shown: "This license key has already expired"

  Scenario: Activating a license without network connectivity
    Given no network connection is available
    And a valid paid license key exists for this installation's practiceId
    When the Practice Manager enters the license key
    Then a LicenseIssued event is recorded
    And on the next application launch the scheduling module status is "Active"

  # ─────────────────────────────────────────────────────────────
  # Rule 4: Renewal key with earlier expiry — warn and confirm
  # ─────────────────────────────────────────────────────────────

  Scenario: Renewal key with later expiry is accepted without warning
    Given the scheduling module is "Active" with expires_at "2027-03-03"
    And a renewal key has expires_at "2028-03-03" for scheduling
    When the Practice Manager enters the renewal key
    Then a LicenseRenewed event is recorded
    And no warning is shown

  Scenario: Renewal key with earlier expiry shows a warning
    Given the scheduling module is "Active" with expires_at "2027-03-03"
    And a renewal key has expires_at "2026-12-01" for scheduling
    When the Practice Manager enters the renewal key
    Then a warning is shown: "This key expires on 2026-12-01, earlier than your current license (2027-03-03). Replace it?"
    And no LicenseRenewed event is recorded yet

  Scenario: Practice Manager confirms replacement with earlier expiry
    Given a warning is shown for an earlier expiry renewal key
    When the Practice Manager confirms the replacement
    Then a LicenseRenewed event is recorded with the new expiry date

  Scenario: Practice Manager cancels replacement with earlier expiry
    Given a warning is shown for an earlier expiry renewal key
    When the Practice Manager cancels
    Then no LicenseRenewed event is recorded
    And the original license is unchanged

  # ─────────────────────────────────────────────────────────────
  # Rule 5: Per-module expiry — independent grace periods
  # ─────────────────────────────────────────────────────────────

  Scenario: Scheduling expires while recall remains active
    Given the scheduling module has expires_at of yesterday and grace_period_days of 90
    And the recall module is "Active"
    When the application is launched
    Then a LicenseDegraded event is recorded for module "scheduling"
    And the scheduling module status is "Degraded"
    And the recall module status is "Active"

  Scenario: Degraded scheduling does not affect recall write access
    Given the scheduling module status is "Degraded"
    And the recall module status is "Active"
    When the user performs a recall outreach action
    Then the action succeeds

  Scenario: Viewing existing schedule is allowed in Degraded scheduling
    Given the scheduling module status is "Degraded"
    And appointments exist in the database
    When the user views the appointment schedule
    Then the schedule is displayed

  Scenario: Creating a new appointment is blocked in Degraded scheduling
    Given the scheduling module status is "Degraded"
    When the user attempts to create a new appointment
    Then the action is rejected
    And the banner shows "Scheduling has expired. [N] days to renew before read-only access. Renew now."

  Scenario: Scheduling grace period 90 days — still operational on day 89
    Given the scheduling module entered Degraded state 89 days ago
    When the application is launched
    Then the scheduling module status is "Degraded"
    And no LicenseExpired event is recorded for scheduling

  Scenario: Scheduling grace period exhausted on day 91
    Given the scheduling module entered Degraded state 91 days ago
    When the application is launched
    Then a LicenseExpired event is recorded for module "scheduling"
    And the scheduling module status is "Expired"

  # ─────────────────────────────────────────────────────────────
  # Rule 6: Existing data always readable
  # ─────────────────────────────────────────────────────────────

  Scenario: Patient records viewable regardless of module status
    Given the scheduling module status is "Expired"
    And patient records exist in the database
    When the user views an existing patient record
    Then the record is displayed

  Scenario: Existing appointments viewable regardless of module status
    Given the scheduling module status is "Expired"
    And appointments exist in the database
    When the user views existing appointments
    Then the appointments are displayed

  Scenario: Practice configuration viewable regardless of module status
    Given the scheduling module status is "Expired"
    When the user views the offices configuration
    Then the configuration is displayed

  # ─────────────────────────────────────────────────────────────
  # Rule 7: Progressive warning system
  # ─────────────────────────────────────────────────────────────

  Scenario: No banner when module expires in more than 30 days
    Given the scheduling module expires in 45 days
    When the application is launched
    Then no expiry warning banner is shown for scheduling

  Scenario: Subtle banner at 30 days
    Given the scheduling module expires in 30 days
    When the application is launched
    Then the banner shows "Scheduling expires in 30 days"

  Scenario: Prominent banner at 14 days
    Given the scheduling module expires in 14 days
    When the application is launched
    Then the banner shows "Scheduling expires in 14 days — renew soon"

  Scenario: Urgent banner at 7 days
    Given the scheduling module expires in 7 days
    When the application is launched
    Then the banner shows "Scheduling expires in 7 days"

  Scenario: Eval countdown always shown
    Given the license status is "Eval"
    And the eval expires in 12 days
    When the user views any screen
    Then the banner shows "Trial — 12 days remaining"

  Scenario: Banner never blocks navigation
    Given the scheduling module expires in 7 days
    When the user navigates to the patient records screen
    Then the patient records screen is displayed
    And the warning banner is visible but does not block the screen

  # ─────────────────────────────────────────────────────────────
  # Rule 8: Startup enforces — 48h check informs
  # ─────────────────────────────────────────────────────────────

  Scenario: Module expires while app is running — write access unchanged mid-session
    Given the scheduling module is "Active" at session start
    And the scheduling module expires during the session
    When the 48-hour background check fires
    Then a LicenseDegraded event is recorded for module "scheduling"
    And the banner updates to show scheduling has degraded
    And write access to scheduling remains active for the current session

  Scenario: Write access restricted after relaunch following mid-session degradation
    Given a LicenseDegraded event was recorded during a prior session
    When the application is launched
    Then the scheduling module status is "Degraded"
    And creating a new appointment is rejected

  Scenario: 48h check updates banner for approaching expiry
    Given the scheduling module expires in 6 days
    And the app has been running for 48 hours since the last check
    When the 48-hour background check fires
    Then the banner updates to show the current days-remaining count
    And no write access changes occur

  # ─────────────────────────────────────────────────────────────
  # Rule 9: Clock rollback detection
  # ─────────────────────────────────────────────────────────────

  Scenario: System clock rolled back more than 24 hours is detected
    Given a LicenseValidationSucceeded event exists with timestamp "2026-03-03T10:00:00Z"
    And the system clock reads "2026-03-01T08:00:00Z"
    When the application is launched
    Then a ClockRollbackDetected event is recorded
    And all module write access is denied for the session
    And the banner shows a clock error message

  Scenario: Existing data is readable during clock rollback
    Given the license status is "Invalid" due to clock rollback
    And patient records exist in the database
    When the user views an existing patient record
    Then the record is displayed

  Scenario: Clock within 24-hour threshold is not treated as rollback
    Given a LicenseValidationSucceeded event exists with timestamp "2026-03-03T10:00:00Z"
    And the system clock reads "2026-03-02T18:00:00Z"
    When the application is launched
    Then no ClockRollbackDetected event is recorded

  Scenario: Clock corrected on next startup clears Invalid status
    Given a ClockRollbackDetected event was recorded in a prior session
    And the system clock now reads a time after the last valid validation timestamp
    When the application is launched
    Then the license validates normally
    And write access is restored per current module statuses

  Scenario: No rollback check on very first launch
    Given no LicenseValidationSucceeded event exists in the event store
    When the application is launched for the first time
    Then no ClockRollbackDetected event is recorded
    And the application proceeds to establish PracticeIdentity

  # ─────────────────────────────────────────────────────────────
  # Rule 10: Hardware migration
  # ─────────────────────────────────────────────────────────────

  Scenario: License key for different machine is rejected
    Given a valid license key for a different practiceId
    When the Practice Manager enters the key
    Then no LicenseIssued event is recorded
    And an error is shown: "This key was issued for a different installation"

  Scenario: Support screen shows practiceId for re-issuance
    Given PracticeIdentity has been established
    When the Practice Manager views the About or Support screen
    Then the practiceId is displayed in a copyable format
