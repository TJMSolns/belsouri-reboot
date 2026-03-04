# Example Map: Patient Scheduling

**Date**: 2026-03-04
**Participants**: Tony (Product Owner / Business), Claude (Developer / Tester)
**Feature**: Patient Scheduling context — full appointment lifecycle (BookAppointment, RescheduleAppointment, CancelAppointment, CompleteAppointment, MarkAppointmentNoShow, AddAppointmentNote, TomorrowsCallList)
**Status**: Phase 2.2 + 2.3 complete — all open questions CONFIRMED by Tony (2026-03-04)

---

## Three Amigos Summary (Phase 2.1)

Decisions confirmed by Tony (2026-03-04):

| Item | Decision |
|------|----------|
| All 5 booking constraints at MVP | ALL are hard stops — no override, no warn + continue [CONFIRMED — PS-1] |
| Duration override range | 15–240 minutes, same as ProcedureType.default_duration range [CONFIRMED — PS-2] |
| Reschedule model | Two aggregates: original gets terminal Rescheduled status; new Appointment booked [CONFIRMED — PS-3] |
| Cancelled and NoShow statuses | Terminal — cannot be reversed [CONFIRMED — PS-4] |
| Cross-office conflict check | None — conflicts are office-scoped only; same patient at two offices on same day is allowed [CONFIRMED — PS-5] |
| Appointment notes vs. patient notes | Separate — AppointmentNote owned by Appointment aggregate; PatientNote is general patient context [CONFIRMED — PS-6] |
| MVP reminder approach | Manual call list only — no SMS/email at MVP; TomorrowsCallList projection provides name + contact info for front desk to call |
| Override capability | Backlog item — not MVP |

---

## Rule Cards

---

## Appointment Booking

---

### PS-Rule-1: BookAppointment succeeds when all 5 constraints pass

**Rule**: An appointment is booked when all five booking constraints are satisfied simultaneously: (C1) office open, (C2) provider available, (C3) chair capacity not exceeded, (C4) patient active, (C5) procedure type active. The command produces an AppointmentBooked event with patient_id, provider_id, procedure_type_id, office_id, start_time, end_time (computed from start + duration), duration_minutes, and booked_by.

| # | Example | Type |
|---|---------|------|
| PS-1a | Front desk books "Maria Brown" with "Dr. Spence" for "Cleaning" (60 min) at "Main Office" on Mon 2026-03-09 at 10:00; office open 08:00–17:00, Dr. Spence available 09:00–16:00, 1 of 3 chairs used, patient active, procedure active → AppointmentBooked with start_time 10:00, end_time 11:00, duration_minutes 60 | ✅ Happy path |
| PS-1b | Front desk books appointment and explicitly overrides duration to 90 min (valid range); all 5 constraints still pass → AppointmentBooked with duration_minutes 90, end_time computed as start + 90 min | ✅ Happy path |
| PS-1c | Office is at 2 of 3 chairs capacity at requested time; booking proposed for the same overlapping window → 2 < 3, constraint C3 passes → AppointmentBooked | ✅ Boundary |
| PS-1d | Front desk books appointment with no duration override → default duration from ProcedureType is used; end_time computed accordingly → AppointmentBooked | ✅ Happy path |

---

### PS-Rule-C1: Office must be open at the requested time (hard stop)

**Rule**: The office must have operating hours configured for the requested day and the appointment's time window must fall within those hours. If the office is closed on that day or the time is outside configured hours, booking is rejected with the specific error message. No override.

| # | Example | Type |
|---|---------|------|
| PS-C1a | "Main Office" has no hours configured for Saturday; front desk attempts to book on Sat 2026-03-07 at 10:00 → Rejected: "Office Main Office is not open at 10:00 on Saturday" | ❌ Negative path |
| PS-C1b | "Main Office" is open Mon–Fri 08:00–17:00; front desk attempts to book at 07:30 on Monday → Rejected: "Office Main Office is not open at 07:30 on Monday" | ❌ Negative path |
| PS-C1c | "Main Office" is open Mon–Fri 08:00–17:00; appointment would end at 17:30 (e.g., 16:45 start + 45 min) → Rejected: "Office Main Office is not open at 16:45 on [day]" (appointment end exceeds closing time) | ❌ Edge case |
| PS-C1d | "Main Office" is open Mon 08:00–17:00; front desk books at 08:00 (start exactly at open) → C1 passes | ✅ Boundary |
| PS-C1e | "Main Office" is open Mon 08:00–17:00; front desk books with end_time exactly 17:00 → C1 passes | ✅ Boundary |

---

### PS-Rule-C2: Provider must be available at that office at that time (hard stop)

**Rule**: The provider must be available at the requested office on the requested day and time, as determined by the ResolvedSchedule projection (which has already applied weekly availability windows and all active exceptions). If the provider is not available, booking is rejected. No override.

| # | Example | Type |
|---|---------|------|
| PS-C2a | Dr. Spence's availability at Main Office is Mon–Fri 09:00–16:00; front desk attempts to book at 08:00 Monday → Rejected: "Provider Dr. Spence is not available at Main Office at 08:00" | ❌ Negative path |
| PS-C2b | Dr. Spence has an active exception (e.g., holiday) covering 2026-03-09; front desk attempts to book on that date → Rejected: "Provider Dr. Spence is not available at Main Office at [time]" | ❌ Negative path |
| PS-C2c | Dr. Spence is not assigned to "Montego Bay Office"; front desk attempts to book at Montego Bay with Dr. Spence → Rejected: "Provider Dr. Spence is not available at Montego Bay Office at [time]" | ❌ Negative path |
| PS-C2d | Dr. Spence is available Mon–Fri 09:00–16:00; front desk books at 09:00 (start exactly at window open) → C2 passes | ✅ Boundary |
| PS-C2e | Dr. Spence works Mon–Fri at Main Office and Tue at Montego Bay (no overlap); front desk books at Montego Bay on Tuesday → C2 passes at Montego Bay | ✅ Happy path |

---

### PS-Rule-C3: Chair capacity must not be exceeded (hard stop)

**Rule**: The count of Booked appointments at the office whose time window overlaps the proposed window must be less than the office's chair_count. Overlap is defined as: `existing.start < proposed.end AND existing.end > proposed.start`. If all chairs are filled, booking is rejected. No override.

| # | Example | Type |
|---|---------|------|
| PS-C3a | Main Office has 2 chairs; 2 appointments already booked for 10:00–11:00; front desk attempts to book a third for 10:00–11:00 → Rejected: "No chairs available at Main Office at 10:00 — all 2 chairs are booked" | ❌ Negative path |
| PS-C3b | Main Office has 3 chairs; 3 appointments for 10:00–11:00; front desk attempts to book at 10:30–11:30 (overlaps) → Rejected: "No chairs available at Main Office at 10:30 — all 3 chairs are booked" | ❌ Negative path |
| PS-C3c | Main Office has 3 chairs; existing appointment 10:00–11:00; proposed appointment 11:00–12:00 (adjacent, no overlap) → C3 passes (no overlap: existing.end = proposed.start, not strictly less than) | ✅ Boundary |
| PS-C3d | Main Office has 3 chairs; existing appointment 10:00–11:00 at Office A; proposed at Office B 10:00–11:00 → C3 passes (chair capacity is office-scoped; Office A does not affect Office B) | ✅ Edge case |
| PS-C3e | Main Office has 3 chairs; 2 appointments for 10:00–11:00; proposed for 10:30–11:30 → 2 < 3, C3 passes → AppointmentBooked | ✅ Boundary |

---

### PS-Rule-C4: Patient must be active (hard stop)

**Rule**: The patient referenced in the booking must not be archived. If the patient is archived, booking is rejected with a specific error. No override. If a patient arrives after being marked no-show, a new appointment is booked (patient must be active and unarchived).

| # | Example | Type |
|---|---------|------|
| PS-C4a | Patient "Maria Brown" has been archived; front desk attempts to book an appointment for her → Rejected: "Patient is archived and cannot be booked" | ❌ Negative path |
| PS-C4b | Patient "Maria Brown" is active; booking proceeds through C4 | ✅ Happy path |
| PS-C4c | Patient was previously archived and then unarchived (active again); front desk books → C4 passes | ✅ Edge case |

---

### PS-Rule-C5: Procedure type must be active (hard stop)

**Rule**: The procedure type referenced in the booking must not be deactivated. If the procedure type is inactive, booking is rejected with a specific error. No override.

| # | Example | Type |
|---|---------|------|
| PS-C5a | Procedure type "Whitening" has been deactivated; front desk attempts to book a Whitening appointment → Rejected: "Procedure type Whitening is no longer active" | ❌ Negative path |
| PS-C5b | Procedure type "Cleaning" is active; booking proceeds through C5 | ✅ Happy path |
| PS-C5c | Procedure type was deactivated and then reactivated (active again); front desk books → C5 passes | ✅ Edge case |

---

### PS-Rule-2: Duration override must be within the valid range

**Rule**: The optional duration_minutes override must be within 15–240 minutes (inclusive). If a duration outside this range is provided, booking is rejected. If no override is provided, the ProcedureType's default duration is used (which is guaranteed to be in range). The end_time is always computed as start_time + duration_minutes.

| # | Example | Type |
|---|---------|------|
| PS-2a | Front desk books "Cleaning" (default 60 min) with no duration override → AppointmentBooked with duration_minutes = 60 (from ProcedureType default) | ✅ Happy path |
| PS-2b | Front desk overrides duration to 90 min (valid) → AppointmentBooked with duration_minutes = 90 | ✅ Happy path |
| PS-2c | Front desk overrides duration to 15 min (minimum) → AppointmentBooked with duration_minutes = 15 | ✅ Boundary |
| PS-2d | Front desk overrides duration to 240 min (maximum) → AppointmentBooked with duration_minutes = 240 | ✅ Boundary |
| PS-2e | Front desk overrides duration to 14 min (below minimum) → Rejected: "Duration must be between 15 and 240 minutes" | ❌ Boundary |
| PS-2f | Front desk overrides duration to 241 min (above maximum) → Rejected: "Duration must be between 15 and 240 minutes" | ❌ Boundary |
| PS-2g | Front desk overrides duration to 0 → Rejected: "Duration must be between 15 and 240 minutes" | ❌ Negative path |

---

## Appointment Reschedule

---

### PS-Rule-3: RescheduleAppointment — original goes terminal; new appointment created

**Rule**: Rescheduling an appointment produces two outcomes: (1) the original Appointment receives an AppointmentRescheduled event, transitioning it to the terminal Rescheduled status, with rescheduled_to_id linking to the new appointment; (2) a new Appointment aggregate is created (AppointmentBooked event) with rescheduled_from_id linking to the original. The new time slot is checked against all 5 booking constraints. Only Booked appointments can be rescheduled.

| # | Example | Type |
|---|---------|------|
| PS-3a | Front desk reschedules a Booked appointment from Mon 10:00 to Wed 14:00; Wed slot passes all 5 constraints → AppointmentRescheduled (original, status = Rescheduled, rescheduled_to_id set) + AppointmentBooked (new, rescheduled_from_id set) | ✅ Happy path |
| PS-3b | Front desk reschedules to a different provider at the same office; new provider passes C2 for new time → AppointmentRescheduled + AppointmentBooked with new provider_id | ✅ Happy path |
| PS-3c | Front desk reschedules to a different office; new office passes C1 and C3 → AppointmentRescheduled + AppointmentBooked with new office_id | ✅ Happy path |
| PS-3d | Front desk attempts to reschedule a Completed appointment → Rejected: "Only Booked appointments can be rescheduled" | ❌ Negative path |
| PS-3e | Front desk attempts to reschedule a Cancelled appointment → Rejected: "Only Booked appointments can be rescheduled" | ❌ Negative path |
| PS-3f | Front desk attempts to reschedule a NoShow appointment → Rejected: "Only Booked appointments can be rescheduled" | ❌ Negative path |
| PS-3g | Front desk attempts to reschedule a Rescheduled (terminal) appointment → Rejected: "Only Booked appointments can be rescheduled" | ❌ Negative path |
| PS-3h | New time slot fails C1 (office closed) → Rejected: "Office [name] is not open at [time] on [day]"; original appointment remains Booked | ❌ Negative path |
| PS-3i | New time slot fails C2 (provider unavailable) → Rejected: "Provider [name] is not available at [office] at [time]"; original appointment remains Booked | ❌ Negative path |
| PS-3j | New time slot fails C3 (chairs full) → Rejected: "No chairs available at [office] at [time] — all [N] chairs are booked"; original appointment remains Booked | ❌ Negative path |
| PS-3k | Patient has been archived since original booking; reschedule attempted → Rejected: "Patient is archived and cannot be booked"; original appointment remains Booked | ❌ Edge case |
| PS-3l | Procedure type has been deactivated since original booking; reschedule attempted for same procedure → Rejected: "Procedure type [name] is no longer active"; original appointment remains Booked | ❌ Edge case |
| PS-3m | After reschedule, patient history shows both the original (Rescheduled) and new (Booked) appointment linked via rescheduled_to_id / rescheduled_from_id | ✅ Happy path |

---

## Appointment Lifecycle — Terminal Transitions

---

### PS-Rule-4: CancelAppointment — only from Booked status

**Rule**: An appointment can be cancelled only when it is in Booked status. Completed, NoShow, and Rescheduled appointments are terminal and cannot be cancelled. The optional reason is recorded in the AppointmentCancelled event. Cancelled status is terminal — cannot be reversed.

| # | Example | Type |
|---|---------|------|
| PS-4a | Front desk cancels a Booked appointment with reason "Patient request" → AppointmentCancelled event recorded with reason | ✅ Happy path |
| PS-4b | Front desk cancels a Booked appointment with no reason provided → AppointmentCancelled event recorded; reason is null | ✅ Happy path |
| PS-4c | Front desk attempts to cancel a Completed appointment → Rejected: "Appointment cannot be cancelled — it has already been completed" | ❌ Negative path |
| PS-4d | Front desk attempts to cancel a NoShow appointment → Rejected: "Appointment cannot be cancelled — it has been marked no-show" | ❌ Negative path |
| PS-4e | Front desk attempts to cancel a Rescheduled appointment → Rejected: "Appointment cannot be cancelled — it has already been rescheduled" | ❌ Negative path |
| PS-4f | Front desk attempts to cancel an already-Cancelled appointment → Rejected: "Appointment is already cancelled" | ❌ Negative path |
| PS-4g | Cancelled appointment appears in patient history with status Cancelled and reason (if provided); no reversal is possible | ✅ Happy path |

---

### PS-Rule-5: CompleteAppointment — only from Booked status

**Rule**: An appointment can be marked completed only when it is in Booked status. The AppointmentCompleted event records the completing staff member (completed_by). Completed status is terminal.

| # | Example | Type |
|---|---------|------|
| PS-5a | Front desk marks a Booked appointment as completed → AppointmentCompleted event recorded with completed_by | ✅ Happy path |
| PS-5b | Front desk attempts to complete a Cancelled appointment → Rejected: "Appointment cannot be completed — it has been cancelled" | ❌ Negative path |
| PS-5c | Front desk attempts to complete a NoShow appointment → Rejected: "Appointment cannot be completed — it has been marked no-show" | ❌ Negative path |
| PS-5d | Front desk attempts to complete a Rescheduled appointment → Rejected: "Appointment cannot be completed — it has been rescheduled" | ❌ Negative path |
| PS-5e | Front desk attempts to complete an already-Completed appointment → Rejected: "Appointment is already completed" | ❌ Negative path |

---

### PS-Rule-6: MarkAppointmentNoShow — only from Booked status

**Rule**: An appointment can be marked as a no-show only when it is in Booked status. The AppointmentMarkedNoShow event records the staff member (recorded_by). NoShow status is terminal — cannot be reversed. If the patient later contacts the practice, a new appointment is booked.

| # | Example | Type |
|---|---------|------|
| PS-6a | Front desk marks a Booked appointment as no-show → AppointmentMarkedNoShow event recorded with recorded_by | ✅ Happy path |
| PS-6b | Front desk attempts to mark a Completed appointment as no-show → Rejected: "Appointment cannot be marked no-show — it has already been completed" | ❌ Negative path |
| PS-6c | Front desk attempts to mark a Cancelled appointment as no-show → Rejected: "Appointment cannot be marked no-show — it has been cancelled" | ❌ Negative path |
| PS-6d | Front desk attempts to mark a Rescheduled appointment as no-show → Rejected: "Appointment cannot be marked no-show — it has been rescheduled" | ❌ Negative path |
| PS-6e | Front desk attempts to mark an already-NoShow appointment as no-show → Rejected: "Appointment is already marked no-show" | ❌ Negative path |
| PS-6f | Patient contacts practice after being marked no-show; front desk books a new appointment (no-show status is terminal; new booking is independent) | ✅ Edge case |

---

## Appointment Notes

---

### PS-Rule-7: AddAppointmentNote — any status; text and recorded_by required

**Rule**: A note can be added to any appointment regardless of its status (Booked, Completed, Cancelled, NoShow, or Rescheduled). The note text must be non-empty. The recorded_by staff_member_id is required for audit trail. AppointmentNotes are separate from PatientNotes — they are specific to the appointment. Notes are append-only and cannot be edited or deleted.

| # | Example | Type |
|---|---------|------|
| PS-7a | Staff member adds note "Patient arrived 10 min late" to a Booked appointment → AppointmentNoteAdded with note_id, text, recorded_by, recorded_at | ✅ Happy path |
| PS-7b | Staff member adds note "Extended 15 min for additional work" to a Completed appointment → AppointmentNoteAdded | ✅ Happy path |
| PS-7c | Staff member adds note to a Cancelled appointment ("Patient called to cancel") → AppointmentNoteAdded | ✅ Happy path |
| PS-7d | Staff member adds note to a NoShow appointment ("Patient did not call ahead") → AppointmentNoteAdded | ✅ Happy path |
| PS-7e | Staff member adds note to a Rescheduled appointment → AppointmentNoteAdded | ✅ Happy path |
| PS-7f | Staff member submits note with empty text → Rejected: "Note text is required" | ❌ Negative path |
| PS-7g | Staff member submits note with whitespace-only text → Rejected: "Note text is required" | ❌ Edge case |
| PS-7h | Staff member submits AddAppointmentNote without recorded_by → Rejected: "recorded_by is required" | ❌ Negative path |
| PS-7i | Staff member adds a second note to the same appointment → AppointmentNoteAdded; both notes retained (append-only list) | ✅ Happy path |
| PS-7j | AppointmentNote is distinct from PatientNote — appointment note is visible in appointment detail; patient note is visible in patient profile | ✅ Happy path |

---

## Schedule Queries

---

### PS-Rule-8: TomorrowsCallList — returns next-day Booked appointments with patient contact info

**Rule**: The TomorrowsCallList query returns all Booked appointments at a given office for the next calendar day, including patient name, phone, email, and preferred_contact_channel for manual calling by front desk. Only Booked appointments are included — Completed, Cancelled, NoShow, and Rescheduled appointments are excluded. No events are produced (read-only query).

| # | Example | Type |
|---|---------|------|
| PS-8a | Main Office has 3 Booked appointments for tomorrow; front desk queries TomorrowsCallList for Main Office → returns 3 entries with patient_name, patient_phone, patient_email, preferred_contact_channel, procedure_name, provider_name, start_time | ✅ Happy path |
| PS-8b | Main Office has a mix of Booked and Cancelled appointments for tomorrow → only the 2 Booked appointments returned | ✅ Happy path |
| PS-8c | Main Office has no Booked appointments for tomorrow → empty list returned | ✅ Edge case |
| PS-8d | Patient has no phone on file; appointment is still included → patient_phone is null; preferred_contact_channel and email (if present) still shown | ✅ Edge case |
| PS-8e | Query for a specific office excludes appointments at other offices on the same day | ✅ Happy path |
| PS-8f | TomorrowsCallList is a read-only query — no events are produced | ✅ Happy path |

---

## Open Questions Summary

All open questions confirmed by Tony (2026-03-04). No open questions remain.

| # | Question | Status |
|---|----------|--------|
| PS-1 | Hard stops vs. warn + override for all 5 booking constraints? | [CONFIRMED — all 5 are hard stops at MVP] |
| PS-2 | Duration override range? | [CONFIRMED — 15–240 minutes] |
| PS-3 | Reschedule — one aggregate or two? | [CONFIRMED — two: original gets terminal Rescheduled; new appointment booked] |
| PS-4 | Cancelled and NoShow statuses reversible? | [CONFIRMED — terminal, not reversible] |
| PS-5 | Cross-office conflict check for same patient on same day? | [CONFIRMED — no; office-scoped only] |
| PS-6 | Appointment notes separate from patient notes? | [CONFIRMED — yes; owned by Appointment aggregate] |

---

**Phase 2.3 Acceptance Criteria Review**: All business rules validated against `appointment-aggregate.md`. All 5 constraint failure messages match the spec exactly. All 6 open questions marked CONFIRMED. No assumptions outstanding. Ubiquitous language used throughout — no banned terms (visit, slot, booking as noun, schedule as verb, delete appointment). Ready for Phase 2.4 BDD Scenarios.

**Maintained By**: Tony + Claude
