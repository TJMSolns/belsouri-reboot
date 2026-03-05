# Feature: Patient Scheduling — Appointment Booking and Lifecycle
#
# Phase 2.4 BDD Scenarios — Patient Scheduling context
# Date: 2026-03-04
# All open questions CONFIRMED by Tony (2026-03-04) — ready for implementation.
#
# Covers:
#   PS-Rule-1:  BookAppointment succeeds when all 5 constraints pass
#   PS-Rule-C1: Booking rejected when office is closed
#   PS-Rule-C2: Booking rejected when provider is unavailable
#   PS-Rule-C3: Booking rejected when chair capacity is exceeded
#   PS-Rule-C4: Booking rejected when patient is archived
#   PS-Rule-C5: Booking rejected when procedure type is inactive
#   PS-Rule-2:  Duration override must be within 15–240 minutes
#   PS-Rule-3:  RescheduleAppointment — original goes terminal; new appointment created
#   PS-Rule-4:  CancelAppointment — only from Booked status; Cancelled is terminal
#   PS-Rule-5:  CompleteAppointment — only from Booked status
#   PS-Rule-6:  MarkAppointmentNoShow — only from Booked status; NoShow is terminal
#   PS-Rule-7:  AddAppointmentNote — any appointment status; text and recorded_by required
#   PS-Rule-8:  TomorrowsCallList — Booked appointments for tomorrow with patient contact info
#
# Key confirmed decisions:
#   - All 5 Booking Constraints are hard stops at MVP — no override, no warn + continue
#   - Reschedule = two aggregates (original Rescheduled terminal + new AppointmentBooked)
#   - Cancelled and NoShow are terminal — cannot be reversed
#   - No cross-office conflict check — Chair Capacity is office-scoped only
#   - AppointmentNote is separate from PatientNote; owned by Appointment aggregate
#   - Manual call list only — no SMS/email at MVP

Feature: Patient Scheduling — Appointment Booking and Lifecycle

  Background:
    Given the practice has an office "Main Office" open Monday–Friday 08:00–17:00 with 3 chairs
    And provider "Dr. Spence" is available at "Main Office" Monday–Friday 09:00–16:00
    And procedure "Cleaning" has default duration 60 minutes and is active
    And patient "Maria Brown" is active
    And no other appointments exist at "Main Office" on 2026-03-09 unless stated

  # ─────────────────────────────────────────────────────────────
  # PS-RULE-1: BookAppointment succeeds when all 5 constraints pass
  # ─────────────────────────────────────────────────────────────

  Rule: BookAppointment succeeds when all 5 Booking Constraints pass

    Scenario: Booking a valid appointment using procedure default duration
      Given no appointments at "Main Office" on 2026-03-09 between 10:00 and 11:00
      When the front desk books an appointment for "Maria Brown" with "Dr. Spence" for "Cleaning" at "Main Office" on 2026-03-09 at 10:00
      Then an AppointmentBooked event is recorded
      And the appointment has patient_id for "Maria Brown"
      And the appointment has provider_id for "Dr. Spence"
      And the appointment has procedure_type_id for "Cleaning"
      And the appointment has office_id for "Main Office"
      And the appointment start_time is 10:00 on 2026-03-09
      And the appointment end_time is 11:00 on 2026-03-09
      And the appointment duration_minutes is 60
      And the appointment status is Booked

    Scenario: Booking a valid appointment with a duration override
      Given no appointments at "Main Office" on 2026-03-09 between 10:00 and 11:30
      When the front desk books an appointment for "Maria Brown" with "Dr. Spence" for "Cleaning" at "Main Office" on 2026-03-09 at 10:00 with duration 90 minutes
      Then an AppointmentBooked event is recorded
      And the appointment duration_minutes is 90
      And the appointment end_time is 11:30 on 2026-03-09

    Scenario: Booking succeeds when 2 of 3 chairs are already used (C3 boundary)
      Given 2 appointments already booked at "Main Office" on 2026-03-09 between 10:00 and 11:00
      When the front desk books an appointment for "Maria Brown" with "Dr. Spence" for "Cleaning" at "Main Office" on 2026-03-09 at 10:00
      Then an AppointmentBooked event is recorded

  # ─────────────────────────────────────────────────────────────
  # PS-RULE-C1: Booking rejected when office is closed (hard stop)
  # ─────────────────────────────────────────────────────────────

  Rule: Booking is rejected when the office is closed at the requested time (C1)

    Scenario: Office closed on Saturday
      When the front desk attempts to book an appointment for "Maria Brown" with "Dr. Spence" for "Cleaning" at "Main Office" on 2026-03-07 at 10:00
      Then no AppointmentBooked event is recorded
      And an error is shown: "Office Main Office is not open at 10:00 on Saturday"

    Scenario: Appointment requested before office opening time
      When the front desk attempts to book an appointment for "Maria Brown" with "Dr. Spence" for "Cleaning" at "Main Office" on 2026-03-09 at 07:30
      Then no AppointmentBooked event is recorded
      And an error is shown: "Office Main Office is not open at 07:30 on Monday"

    Scenario: Appointment end time extends beyond office closing time
      When the front desk attempts to book an appointment for "Maria Brown" with "Dr. Spence" for "Cleaning" at "Main Office" on 2026-03-09 at 16:45 with duration 60 minutes
      Then no AppointmentBooked event is recorded
      And an error is shown: "Office Main Office is not open at 16:45 on Monday"

    Scenario: Appointment start exactly at office opening time is valid
      Given no appointments at "Main Office" on 2026-03-09 between 08:00 and 09:00
      When the front desk books an appointment for "Maria Brown" with "Dr. Spence" for "Cleaning" at "Main Office" on 2026-03-09 at 08:00 with duration 60 minutes
      Then an AppointmentBooked event is recorded

  # ─────────────────────────────────────────────────────────────
  # PS-RULE-C2: Booking rejected when provider is unavailable (hard stop)
  # ─────────────────────────────────────────────────────────────

  Rule: Booking is rejected when the provider is unavailable at that office at that time (C2)

    Scenario: Appointment requested before provider availability window
      When the front desk attempts to book an appointment for "Maria Brown" with "Dr. Spence" for "Cleaning" at "Main Office" on 2026-03-09 at 08:00
      Then no AppointmentBooked event is recorded
      And an error is shown: "Provider Dr. Spence is not available at Main Office at 08:00"

    Scenario: Provider has an active exception covering the requested date
      Given provider "Dr. Spence" has an active exception covering 2026-03-09
      When the front desk attempts to book an appointment for "Maria Brown" with "Dr. Spence" for "Cleaning" at "Main Office" on 2026-03-09 at 10:00
      Then no AppointmentBooked event is recorded
      And an error is shown: "Provider Dr. Spence is not available at Main Office at 10:00"

    Scenario: Provider not assigned to the requested office
      Given the practice has a second office "Montego Bay Office" open Monday–Friday 08:00–17:00 with 2 chairs
      When the front desk attempts to book an appointment for "Maria Brown" with "Dr. Spence" for "Cleaning" at "Montego Bay Office" on 2026-03-09 at 10:00
      Then no AppointmentBooked event is recorded
      And an error is shown: "Provider Dr. Spence is not available at Montego Bay Office at 10:00"

    Scenario: Provider availability starts exactly at the booked time
      Given no appointments at "Main Office" on 2026-03-09 between 09:00 and 10:00
      When the front desk books an appointment for "Maria Brown" with "Dr. Spence" for "Cleaning" at "Main Office" on 2026-03-09 at 09:00 with duration 60 minutes
      Then an AppointmentBooked event is recorded

  # ─────────────────────────────────────────────────────────────
  # PS-RULE-C3: Booking rejected when chair capacity is exceeded (hard stop)
  # ─────────────────────────────────────────────────────────────

  Rule: Booking is rejected when Chair Capacity is exceeded at the requested time (C3)

    Scenario: All chairs booked — exact overlap
      Given 3 appointments already booked at "Main Office" on 2026-03-09 between 10:00 and 11:00
      When the front desk attempts to book an appointment for "Maria Brown" with "Dr. Spence" for "Cleaning" at "Main Office" on 2026-03-09 at 10:00
      Then no AppointmentBooked event is recorded
      And an error is shown: "No chairs available at Main Office at 10:00 — all 3 chairs are booked"

    Scenario: All chairs booked — partial overlap with existing appointment
      Given 3 appointments already booked at "Main Office" on 2026-03-09 between 10:00 and 11:00
      When the front desk attempts to book an appointment for "Maria Brown" with "Dr. Spence" for "Cleaning" at "Main Office" on 2026-03-09 at 10:30
      Then no AppointmentBooked event is recorded
      And an error is shown: "No chairs available at Main Office at 10:30 — all 3 chairs are booked"

    Scenario: Adjacent appointments do not count as overlapping (boundary)
      Given 3 appointments already booked at "Main Office" on 2026-03-09 between 10:00 and 11:00
      When the front desk books an appointment for "Maria Brown" with "Dr. Spence" for "Cleaning" at "Main Office" on 2026-03-09 at 11:00 with duration 60 minutes
      Then an AppointmentBooked event is recorded

    Scenario: Chair Capacity is office-scoped — another office's chairs do not count
      Given the practice has a second office "Montego Bay Office" open Monday–Friday 08:00–17:00 with 2 chairs
      And provider "Dr. Reid" is available at "Montego Bay Office" Monday–Friday 09:00–16:00
      And 3 appointments already booked at "Main Office" on 2026-03-09 between 10:00 and 11:00
      When the front desk books an appointment for "Maria Brown" with "Dr. Reid" for "Cleaning" at "Montego Bay Office" on 2026-03-09 at 10:00
      Then an AppointmentBooked event is recorded

  # ─────────────────────────────────────────────────────────────
  # PS-RULE-C4: Booking rejected when patient is archived (hard stop)
  # ─────────────────────────────────────────────────────────────

  Rule: Booking is rejected when the patient is archived (C4)

    Scenario: Booking rejected for an archived patient
      Given patient "Maria Brown" is archived
      When the front desk attempts to book an appointment for "Maria Brown" with "Dr. Spence" for "Cleaning" at "Main Office" on 2026-03-09 at 10:00
      Then no AppointmentBooked event is recorded
      And an error is shown: "Patient is archived and cannot be booked"

    Scenario: Booking succeeds for a patient who was unarchived
      Given patient "Maria Brown" was previously archived and has since been unarchived
      And no appointments at "Main Office" on 2026-03-09 between 10:00 and 11:00
      When the front desk books an appointment for "Maria Brown" with "Dr. Spence" for "Cleaning" at "Main Office" on 2026-03-09 at 10:00
      Then an AppointmentBooked event is recorded

  # ─────────────────────────────────────────────────────────────
  # PS-RULE-C5: Booking rejected when procedure type is inactive (hard stop)
  # ─────────────────────────────────────────────────────────────

  Rule: Booking is rejected when the procedure type is inactive (C5)

    Scenario: Booking rejected for a deactivated procedure type
      Given procedure "Cleaning" is deactivated
      When the front desk attempts to book an appointment for "Maria Brown" with "Dr. Spence" for "Cleaning" at "Main Office" on 2026-03-09 at 10:00
      Then no AppointmentBooked event is recorded
      And an error is shown: "Procedure type Cleaning is no longer active"

    Scenario: Booking succeeds for a reactivated procedure type
      Given procedure "Cleaning" was previously deactivated and has since been reactivated
      And no appointments at "Main Office" on 2026-03-09 between 10:00 and 11:00
      When the front desk books an appointment for "Maria Brown" with "Dr. Spence" for "Cleaning" at "Main Office" on 2026-03-09 at 10:00
      Then an AppointmentBooked event is recorded

  # ─────────────────────────────────────────────────────────────
  # PS-RULE-2: Duration override must be within 15–240 minutes
  # ─────────────────────────────────────────────────────────────

  Rule: Duration override must be within 15–240 minutes

    Scenario: No duration override — procedure default is used
      Given no appointments at "Main Office" on 2026-03-09 between 10:00 and 11:00
      When the front desk books an appointment for "Maria Brown" with "Dr. Spence" for "Cleaning" at "Main Office" on 2026-03-09 at 10:00
      Then an AppointmentBooked event is recorded
      And the appointment duration_minutes is 60

    Scenario: Duration override to minimum valid value
      Given no appointments at "Main Office" on 2026-03-09 between 10:00 and 10:15
      When the front desk books an appointment for "Maria Brown" with "Dr. Spence" for "Cleaning" at "Main Office" on 2026-03-09 at 10:00 with duration 15 minutes
      Then an AppointmentBooked event is recorded
      And the appointment duration_minutes is 15

    Scenario: Duration override to maximum valid value
      Given no appointments at "Main Office" on 2026-03-09 between 09:00 and 13:00
      When the front desk books an appointment for "Maria Brown" with "Dr. Spence" for "Cleaning" at "Main Office" on 2026-03-09 at 09:00 with duration 240 minutes
      Then an AppointmentBooked event is recorded
      And the appointment duration_minutes is 240

    Scenario: Duration override below minimum is rejected
      When the front desk attempts to book an appointment for "Maria Brown" with "Dr. Spence" for "Cleaning" at "Main Office" on 2026-03-09 at 10:00 with duration 14 minutes
      Then no AppointmentBooked event is recorded
      And an error is shown: "Duration must be between 15 and 240 minutes"

    Scenario: Duration override above maximum is rejected
      When the front desk attempts to book an appointment for "Maria Brown" with "Dr. Spence" for "Cleaning" at "Main Office" on 2026-03-09 at 10:00 with duration 241 minutes
      Then no AppointmentBooked event is recorded
      And an error is shown: "Duration must be between 15 and 240 minutes"

    Scenario: Duration override of zero is rejected
      When the front desk attempts to book an appointment for "Maria Brown" with "Dr. Spence" for "Cleaning" at "Main Office" on 2026-03-09 at 10:00 with duration 0 minutes
      Then no AppointmentBooked event is recorded
      And an error is shown: "Duration must be between 15 and 240 minutes"

  # ─────────────────────────────────────────────────────────────
  # PS-RULE-3: RescheduleAppointment — original terminal; new appointment booked
  # ─────────────────────────────────────────────────────────────

  Rule: RescheduleAppointment marks the original appointment terminal and books a new one

    Background:
      Given an existing Booked appointment for "Maria Brown" with "Dr. Spence" for "Cleaning" at "Main Office" on 2026-03-09 at 10:00

    Scenario: Rescheduling a Booked appointment to a new valid time
      Given "Main Office" is open on Wednesday 2026-03-11 08:00–17:00
      And provider "Dr. Spence" is available at "Main Office" on Wednesday 09:00–16:00
      And no appointments at "Main Office" on 2026-03-11 between 14:00 and 15:00
      When the front desk reschedules the appointment to "Main Office" on 2026-03-11 at 14:00 with "Dr. Spence"
      Then an AppointmentRescheduled event is recorded on the original appointment
      And the original appointment status is Rescheduled
      And the original appointment rescheduled_to_id is set to the new appointment id
      And an AppointmentBooked event is recorded for the new appointment
      And the new appointment rescheduled_from_id is set to the original appointment id
      And the new appointment status is Booked

    Scenario: Rescheduling to a different provider at the same office
      Given provider "Dr. Reid" is available at "Main Office" on 2026-03-11 09:00–16:00
      And no appointments at "Main Office" on 2026-03-11 between 14:00 and 15:00
      When the front desk reschedules the appointment to "Main Office" on 2026-03-11 at 14:00 with "Dr. Reid"
      Then an AppointmentRescheduled event is recorded on the original appointment
      And an AppointmentBooked event is recorded for the new appointment with provider_id for "Dr. Reid"

    Scenario: Reschedule rejected — new time fails C1 (office closed on Sunday)
      When the front desk attempts to reschedule the appointment to "Main Office" on 2026-03-08 at 10:00
      Then no AppointmentRescheduled event is recorded
      And the original appointment status remains Booked
      And an error is shown: "Office Main Office is not open at 10:00 on Sunday"

    Scenario: Reschedule rejected — new time fails C2 (provider unavailable)
      Given provider "Dr. Spence" has an active exception covering 2026-03-11
      When the front desk attempts to reschedule the appointment to "Main Office" on 2026-03-11 at 10:00 with "Dr. Spence"
      Then no AppointmentRescheduled event is recorded
      And the original appointment status remains Booked
      And an error is shown: "Provider Dr. Spence is not available at Main Office at 10:00"

    Scenario: Reschedule rejected — new time fails C3 (chairs full)
      Given 3 appointments already booked at "Main Office" on 2026-03-11 between 14:00 and 15:00
      When the front desk attempts to reschedule the appointment to "Main Office" on 2026-03-11 at 14:00 with "Dr. Spence"
      Then no AppointmentRescheduled event is recorded
      And the original appointment status remains Booked
      And an error is shown: "No chairs available at Main Office at 14:00 — all 3 chairs are booked"

    Scenario: Cannot reschedule a Completed appointment
      Given the appointment has been completed
      When the front desk attempts to reschedule the appointment
      Then no AppointmentRescheduled event is recorded
      And an error is shown: "Only Booked appointments can be rescheduled"

    Scenario: Cannot reschedule a Cancelled appointment
      Given the appointment has been cancelled
      When the front desk attempts to reschedule the appointment
      Then no AppointmentRescheduled event is recorded
      And an error is shown: "Only Booked appointments can be rescheduled"

    Scenario: Cannot reschedule a NoShow appointment
      Given the appointment has been marked no-show
      When the front desk attempts to reschedule the appointment
      Then no AppointmentRescheduled event is recorded
      And an error is shown: "Only Booked appointments can be rescheduled"

    Scenario: Cannot reschedule an appointment already in Rescheduled status
      Given the appointment has already been rescheduled (status is Rescheduled)
      When the front desk attempts to reschedule the appointment
      Then no AppointmentRescheduled event is recorded
      And an error is shown: "Only Booked appointments can be rescheduled"

    Scenario: Patient history shows both original and new appointment after reschedule
      Given "Main Office" is open on Wednesday 2026-03-11 08:00–17:00
      And no appointments at "Main Office" on 2026-03-11 between 14:00 and 15:00
      When the front desk reschedules the appointment to "Main Office" on 2026-03-11 at 14:00 with "Dr. Spence"
      Then "Maria Brown"'s appointment history contains the original appointment with status Rescheduled
      And "Maria Brown"'s appointment history contains the new appointment with status Booked
      And the two appointments are linked via rescheduled_to_id and rescheduled_from_id

  # ─────────────────────────────────────────────────────────────
  # PS-RULE-4: CancelAppointment — only from Booked; Cancelled is terminal
  # ─────────────────────────────────────────────────────────────

  Rule: CancelAppointment is only valid from Booked status and Cancelled is terminal

    Background:
      Given an existing Booked appointment for "Maria Brown" with "Dr. Spence" for "Cleaning" at "Main Office" on 2026-03-09 at 10:00

    Scenario: Cancelling a Booked appointment with a reason
      When the front desk cancels the appointment with reason "Patient request"
      Then an AppointmentCancelled event is recorded with reason "Patient request"
      And the appointment status is Cancelled

    Scenario: Cancelling a Booked appointment without a reason
      When the front desk cancels the appointment with no reason
      Then an AppointmentCancelled event is recorded with no reason
      And the appointment status is Cancelled

    Scenario: Cannot cancel a Completed appointment
      Given the appointment has been completed
      When the front desk attempts to cancel the appointment
      Then no AppointmentCancelled event is recorded
      And an error is shown: "Appointment cannot be cancelled — it has already been completed"

    Scenario: Cannot cancel a NoShow appointment
      Given the appointment has been marked no-show
      When the front desk attempts to cancel the appointment
      Then no AppointmentCancelled event is recorded
      And an error is shown: "Appointment cannot be cancelled — it has been marked no-show"

    Scenario: Cannot cancel a Rescheduled appointment
      Given the appointment has already been rescheduled
      When the front desk attempts to cancel the appointment
      Then no AppointmentCancelled event is recorded
      And an error is shown: "Appointment cannot be cancelled — it has already been rescheduled"

    Scenario: Cannot cancel an already-Cancelled appointment
      Given the appointment has been cancelled
      When the front desk attempts to cancel the appointment again
      Then no AppointmentCancelled event is recorded
      And an error is shown: "Appointment is already cancelled"

  # ─────────────────────────────────────────────────────────────
  # PS-RULE-5: CompleteAppointment — only from Booked status
  # ─────────────────────────────────────────────────────────────

  Rule: CompleteAppointment is only valid from Booked status

    Background:
      Given an existing Booked appointment for "Maria Brown" with "Dr. Spence" for "Cleaning" at "Main Office" on 2026-03-09 at 10:00

    Scenario: Completing a Booked appointment
      When the front desk completes the appointment
      Then an AppointmentCompleted event is recorded with completed_by
      And the appointment status is Completed

    Scenario: Cannot complete a Cancelled appointment
      Given the appointment has been cancelled
      When the front desk attempts to complete the appointment
      Then no AppointmentCompleted event is recorded
      And an error is shown: "Appointment cannot be completed — it has been cancelled"

    Scenario: Cannot complete a NoShow appointment
      Given the appointment has been marked no-show
      When the front desk attempts to complete the appointment
      Then no AppointmentCompleted event is recorded
      And an error is shown: "Appointment cannot be completed — it has been marked no-show"

    Scenario: Cannot complete a Rescheduled appointment
      Given the appointment has already been rescheduled
      When the front desk attempts to complete the appointment
      Then no AppointmentCompleted event is recorded
      And an error is shown: "Appointment cannot be completed — it has been rescheduled"

    Scenario: Cannot complete an already-Completed appointment
      Given the appointment has been completed
      When the front desk attempts to complete the appointment again
      Then no AppointmentCompleted event is recorded
      And an error is shown: "Appointment is already completed"

  # ─────────────────────────────────────────────────────────────
  # PS-RULE-6: MarkAppointmentNoShow — only from Booked; NoShow is terminal
  # ─────────────────────────────────────────────────────────────

  Rule: MarkAppointmentNoShow is only valid from Booked status and NoShow is terminal

    Background:
      Given an existing Booked appointment for "Maria Brown" with "Dr. Spence" for "Cleaning" at "Main Office" on 2026-03-09 at 10:00

    Scenario: Marking a Booked appointment as no-show
      When the front desk marks the appointment as no-show
      Then an AppointmentMarkedNoShow event is recorded with recorded_by
      And the appointment status is NoShow

    Scenario: Cannot mark a Completed appointment as no-show
      Given the appointment has been completed
      When the front desk attempts to mark the appointment as no-show
      Then no AppointmentMarkedNoShow event is recorded
      And an error is shown: "Appointment cannot be marked no-show — it has already been completed"

    Scenario: Cannot mark a Cancelled appointment as no-show
      Given the appointment has been cancelled
      When the front desk attempts to mark the appointment as no-show
      Then no AppointmentMarkedNoShow event is recorded
      And an error is shown: "Appointment cannot be marked no-show — it has been cancelled"

    Scenario: Cannot mark a Rescheduled appointment as no-show
      Given the appointment has already been rescheduled
      When the front desk attempts to mark the appointment as no-show
      Then no AppointmentMarkedNoShow event is recorded
      And an error is shown: "Appointment cannot be marked no-show — it has been rescheduled"

    Scenario: Cannot mark an already-NoShow appointment as no-show again
      Given the appointment has been marked no-show
      When the front desk attempts to mark the appointment as no-show again
      Then no AppointmentMarkedNoShow event is recorded
      And an error is shown: "Appointment is already marked no-show"

    Scenario: NoShow is terminal — a new appointment must be booked for the patient to return
      Given the appointment has been marked no-show
      And no appointments at "Main Office" on 2026-03-10 between 10:00 and 11:00
      When the front desk books a new appointment for "Maria Brown" with "Dr. Spence" for "Cleaning" at "Main Office" on 2026-03-10 at 10:00
      Then an AppointmentBooked event is recorded for the new appointment
      And the no-show appointment status remains NoShow

  # ─────────────────────────────────────────────────────────────
  # PS-RULE-7: AddAppointmentNote — any status; text and recorded_by required
  # ─────────────────────────────────────────────────────────────

  Rule: AddAppointmentNote is valid for any appointment status; text and recorded_by are required

    Background:
      Given an existing Booked appointment for "Maria Brown" with "Dr. Spence" for "Cleaning" at "Main Office" on 2026-03-09 at 10:00

    Scenario: Adding a note to a Booked appointment
      When a staff member adds a note "Patient arrived 10 minutes late" to the appointment
      Then an AppointmentNoteAdded event is recorded
      And the note has text "Patient arrived 10 minutes late"
      And the note has a recorded_by staff_member_id
      And the note has a recorded_at timestamp

    Scenario: Adding a note to a Completed appointment
      Given the appointment has been completed
      When a staff member adds a note "Procedure extended 15 minutes for additional work" to the appointment
      Then an AppointmentNoteAdded event is recorded

    Scenario: Adding a note to a Cancelled appointment
      Given the appointment has been cancelled
      When a staff member adds a note "Patient called to cancel" to the appointment
      Then an AppointmentNoteAdded event is recorded

    Scenario: Adding a note to a NoShow appointment
      Given the appointment has been marked no-show
      When a staff member adds a note "Patient did not call ahead" to the appointment
      Then an AppointmentNoteAdded event is recorded

    Scenario: Adding a note to a Rescheduled appointment
      Given the appointment has been rescheduled
      When a staff member adds a note "Rescheduled at patient request" to the appointment
      Then an AppointmentNoteAdded event is recorded

    Scenario: Note with empty text is rejected
      When a staff member attempts to add a note with empty text to the appointment
      Then no AppointmentNoteAdded event is recorded
      And an error is shown: "Note text is required"

    Scenario: Note with whitespace-only text is rejected
      When a staff member attempts to add a note with text "   " to the appointment
      Then no AppointmentNoteAdded event is recorded
      And an error is shown: "Note text is required"

    Scenario: Note without recorded_by is rejected
      When a staff member attempts to add a note without specifying recorded_by
      Then no AppointmentNoteAdded event is recorded
      And an error is shown: "recorded_by is required"

    Scenario: Multiple notes can be added to the same appointment
      When a staff member adds a note "Patient arrived 10 minutes late" to the appointment
      And a staff member adds a note "Extended 15 minutes" to the appointment
      Then 2 AppointmentNoteAdded events are recorded
      And the appointment has 2 notes in its note list

    Scenario: AppointmentNote is separate from PatientNote
      Given "Maria Brown" also has a PatientNote "Prefers morning appointments"
      When a staff member adds a note "Patient requested follow-up" to the appointment
      Then the AppointmentNoteAdded event is recorded on the appointment
      And the PatientNote "Prefers morning appointments" is unchanged and still belongs to the Patient aggregate

  # ─────────────────────────────────────────────────────────────
  # PS-RULE-8: TomorrowsCallList — Booked appointments with contact info
  # ─────────────────────────────────────────────────────────────

  Rule: TomorrowsCallList returns tomorrow's Booked appointments with patient contact info

    Scenario: Call list returns all Booked appointments for tomorrow at the requested office
      Given today is 2026-03-09
      And "Main Office" has 3 Booked appointments on 2026-03-10 with patients with phone numbers
      When the front desk queries the TomorrowsCallList for "Main Office"
      Then 3 entries are returned
      And each entry includes patient_name, patient_phone, procedure_name, provider_name, and start_time

    Scenario: Call list excludes Cancelled appointments
      Given today is 2026-03-09
      And "Main Office" has 2 Booked appointments and 1 Cancelled appointment on 2026-03-10
      When the front desk queries the TomorrowsCallList for "Main Office"
      Then 2 entries are returned

    Scenario: Call list excludes Completed appointments
      Given today is 2026-03-09
      And "Main Office" has 2 Booked appointments and 1 Completed appointment on 2026-03-10
      When the front desk queries the TomorrowsCallList for "Main Office"
      Then 2 entries are returned

    Scenario: Call list excludes NoShow appointments
      Given today is 2026-03-09
      And "Main Office" has 1 Booked appointment and 1 NoShow appointment on 2026-03-10
      When the front desk queries the TomorrowsCallList for "Main Office"
      Then 1 entry is returned

    Scenario: Call list returns empty when no Booked appointments exist for tomorrow
      Given today is 2026-03-09
      And "Main Office" has no Booked appointments on 2026-03-10
      When the front desk queries the TomorrowsCallList for "Main Office"
      Then an empty list is returned

    Scenario: Call list includes null phone when patient has no phone on file
      Given today is 2026-03-09
      And "Main Office" has a Booked appointment on 2026-03-10 for a patient with no phone number
      When the front desk queries the TomorrowsCallList for "Main Office"
      Then 1 entry is returned
      And the entry has a null patient_phone
      And the entry includes preferred_contact_channel and patient_email if available

    Scenario: Call list is office-scoped — only appointments at the queried office are returned
      Given today is 2026-03-09
      And "Main Office" has 2 Booked appointments on 2026-03-10
      And "Montego Bay Office" has 3 Booked appointments on 2026-03-10
      When the front desk queries the TomorrowsCallList for "Main Office"
      Then 2 entries are returned
      And no entries from "Montego Bay Office" are included

    Scenario: TomorrowsCallList is a read-only query — no events are produced
      Given today is 2026-03-09
      And "Main Office" has 2 Booked appointments on 2026-03-10
      When the front desk queries the TomorrowsCallList for "Main Office"
      Then no domain events are recorded
      And 2 entries are returned

  # ─────────────────────────────────────────────────────────────
  # PS-RULE-C6: Booking rejected when patient already has overlapping appointment
  # SCH-4 ceremony — confirmed by Tony 2026-03-05
  # C6 is practice-wide (not office-scoped): a patient cannot be in two chairs
  # anywhere in the practice at the same time.
  # ─────────────────────────────────────────────────────────────

  Rule: Booking is rejected when the patient already has an overlapping appointment anywhere in the practice (C6)

    Background:
      And procedure "Filling" has default duration 45 minutes and is active
      And provider "Dr. Thompson" is a Specialist available at "Main Office" Monday–Friday 09:00–16:00
      And provider "Sarah Williams" is a Hygienist available at "Main Office" Monday–Friday 09:00–16:00

    Scenario: Patient books a non-overlapping appointment at the same office — succeeds
      Given patient "Maria Brown" has a Booked appointment at "Main Office" on 2026-03-09 from 10:00 to 11:00
      When the front desk books an appointment for "Maria Brown" with "Dr. Spence" for "Cleaning" at "Main Office" on 2026-03-09 at 13:00
      Then an AppointmentBooked event is recorded

    Scenario: Booking rejected when patient already has an overlapping appointment at the same office
      Given patient "Maria Brown" has a Booked appointment at "Main Office" on 2026-03-09 from 10:00 to 11:00
      When the front desk attempts to book an appointment for "Maria Brown" with "Dr. Spence" for "Cleaning" at "Main Office" on 2026-03-09 at 10:30
      Then no AppointmentBooked event is recorded
      And an error is shown: "Patient Maria Brown already has an appointment at 10:00 — a patient cannot be in two chairs at the same time"

    Scenario: Patient books an adjacent appointment — succeeds (not overlapping)
      Given patient "Maria Brown" has a Booked appointment at "Main Office" on 2026-03-09 from 10:00 to 11:00
      And no appointments at "Main Office" on 2026-03-09 between 11:00 and 12:00
      When the front desk books an appointment for "Maria Brown" with "Dr. Spence" for "Cleaning" at "Main Office" on 2026-03-09 at 11:00
      Then an AppointmentBooked event is recorded

  # ─────────────────────────────────────────────────────────────
  # PS-RULE-C7: Booking rejected when provider is not eligible for procedure
  # SCH-4 ceremony — confirmed by Tony 2026-03-05
  # Capability ladder: Specialist ≥ Dentist ≥ Hygienist
  # No required_provider_type = any provider eligible (open access)
  # ─────────────────────────────────────────────────────────────

  Rule: Booking is rejected when the provider type is not eligible for the procedure's required level (C7)

    Background:
      And procedure "Root Canal" has default duration 90 minutes, requires Specialist, and is active
      And procedure "Filling" has default duration 45 minutes, requires Dentist, and is active
      And procedure "Cleaning" has default duration 60 minutes, requires Hygienist, and is active
      And procedure "Consultation" has default duration 30 minutes, has no required provider type, and is active
      And provider "Dr. Thompson" is a Specialist available at "Main Office" Monday–Friday 09:00–16:00
      And provider "Sarah Williams" is a Hygienist available at "Main Office" Monday–Friday 09:00–16:00

    Scenario: Specialist books a Specialist-required procedure — succeeds
      Given no appointments at "Main Office" on 2026-03-09 between 10:00 and 11:30
      When the front desk books an appointment for "Maria Brown" with "Dr. Thompson" for "Root Canal" at "Main Office" on 2026-03-09 at 10:00
      Then an AppointmentBooked event is recorded

    Scenario: Dentist books a Hygienist-required procedure — succeeds (capability ladder)
      Given no appointments at "Main Office" on 2026-03-09 between 10:00 and 11:00
      When the front desk books an appointment for "Maria Brown" with "Dr. Spence" for "Cleaning" at "Main Office" on 2026-03-09 at 10:00
      Then an AppointmentBooked event is recorded

    Scenario: Booking rejected when Dentist attempts to book a Specialist-required procedure
      When the front desk attempts to book an appointment for "Maria Brown" with "Dr. Spence" for "Root Canal" at "Main Office" on 2026-03-09 at 10:00
      Then no AppointmentBooked event is recorded
      And an error is shown: "Root Canal requires a Specialist or higher. Dr. Spence is a Dentist and is not eligible for this procedure."

    Scenario: Hygienist books a Hygienist-required procedure — succeeds
      Given no appointments at "Main Office" on 2026-03-09 between 10:00 and 11:00
      When the front desk books an appointment for "Maria Brown" with "Sarah Williams" for "Cleaning" at "Main Office" on 2026-03-09 at 10:00
      Then an AppointmentBooked event is recorded

    Scenario: Booking rejected when Hygienist attempts to book a Dentist-required procedure
      When the front desk attempts to book an appointment for "Maria Brown" with "Sarah Williams" for "Filling" at "Main Office" on 2026-03-09 at 10:00
      Then no AppointmentBooked event is recorded
      And an error is shown: "Filling requires a Dentist or higher. Sarah Williams is a Hygienist and is not eligible for this procedure."

    Scenario: Procedure with no required provider type — any provider type may book
      Given no appointments at "Main Office" on 2026-03-09 between 10:00 and 10:30
      When the front desk books an appointment for "Maria Brown" with "Sarah Williams" for "Consultation" at "Main Office" on 2026-03-09 at 10:00
      Then an AppointmentBooked event is recorded
