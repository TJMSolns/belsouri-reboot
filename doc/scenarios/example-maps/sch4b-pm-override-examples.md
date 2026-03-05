# Example Map: SCH-4b — PM Booking Override Authority

**Date**: 2026-03-05
**Participants**: Tony (Product Owner / Business), Claude (Developer / Tester)
**Feature**: PM Booking Override — allow a Practice Manager to override soft booking constraints (C1: office closed, C2: provider not scheduled) with an explicit reason, while preserving hard stops (C3, C4, C5, C6, C7) for everyone including PMs.
**Status**: Phase 2.2 + 2.3 complete — all open questions CONFIRMED by Tony (2026-03-05). Ready for Phase 2.4 BDD Scenarios.

---

## Three Amigos Summary (Phase 2.1)

Decisions confirmed by Tony (2026-03-05):

| Item | Decision |
|------|----------|
| How does book_appointment know the actor is a PM? | Pass `actor_role: String` as a command parameter. The UI knows who is logged in and provides this from session state. No extra DB lookup in the command. [CONFIRMED — SCH4b-1] |
| Override audit trail: separate event or field on AppointmentBooked? | Add `override_reason: Option<String>` to the `AppointmentBooked` event payload. Non-override bookings have null. Same audit value, simpler projection, fewer event types. [CONFIRMED — SCH4b-2] |
| Visual flag on overridden appointments? | Show "PM override · [reason]" note in the appointment detail panel only. Not on the grid cell — too noisy at that level. [CONFIRMED — SCH4b-3] |
| Can C1 and C2 both be overridden in a single booking? | Yes — one reason field, one override decision covers both soft stops. PM accepts full responsibility. [CONFIRMED — SCH4b-4] |

---

## Constraint Classification Reference

| # | Constraint | Class | PM Can Override? |
|---|-----------|-------|-----------------|
| C1 | Office is open at the requested time | Soft stop | Yes — with reason |
| C2 | Provider is scheduled at that office on that day | Soft stop | Yes — with reason |
| C3 | At least one chair is available at the requested time | Hard stop | No |
| C4 | Patient is not archived | Hard stop | No |
| C5 | Procedure type is active | Hard stop | No |
| C6 | Patient has no other booked appointment at the same time | Hard stop | No |
| C7 | Provider type meets or exceeds procedure's required_provider_type | Hard stop | No |

---

## Rule Cards

---

## RULE 1: Only Practice Managers may override soft booking constraints

---

### SCH4b-Rule-1-1: PM overrides C1 (office closed) with a reason — succeeds

**Rule**: When a booking attempt fails C1 (office is not open at the requested time) and the `actor_role` is "PracticeManager", and a non-empty `override_reason` is provided, the booking succeeds. The `override_reason` is stored in the `AppointmentBooked` event payload.

| # | Example | Type |
|---|---------|------|
| SCH4b-1-1a | Office "Kingston Dental" is closed on Sundays; PM actor_role provided; PM books on a Sunday with reason "Emergency coverage arranged" → C1 soft-stopped; PM override accepted; AppointmentBooked event recorded with override_reason "Emergency coverage arranged" | ✅ Happy path |
| SCH4b-1-1b | Office "Kingston Dental" has no hours set for public holidays; PM books on a public holiday with reason "Agreed extended hours — staff meeting rescheduled" → C1 soft-stopped; PM override accepted; AppointmentBooked recorded with reason | ✅ Happy path |

---

### SCH4b-Rule-1-2: PM overrides C2 (provider not scheduled) with a reason — succeeds

**Rule**: When a booking attempt fails C2 (provider has no availability set at that office on that day) and the `actor_role` is "PracticeManager", and a non-empty `override_reason` is provided, the booking succeeds. The `override_reason` is stored in the `AppointmentBooked` event payload.

| # | Example | Type |
|---|---------|------|
| SCH4b-1-2a | Office "Kingston Dental" is open on Monday; provider "Dr. Lloyd Dentist" has no availability set for Monday at "Kingston Dental"; PM actor_role provided; PM books with reason "Provider available by arrangement" → C2 soft-stopped; PM override accepted; AppointmentBooked event recorded with override_reason "Provider available by arrangement" | ✅ Happy path |
| SCH4b-1-2b | Provider added to office but availability window not yet configured; PM books same day with reason "Temporary coverage while schedule is being set up" → C2 soft-stopped; PM override accepted | ✅ Happy path |

---

### SCH4b-Rule-1-3: Staff member cannot override C1 (office closed) — rejected

**Rule**: When a booking attempt fails C1 and the `actor_role` is "Staff" (or any role other than PracticeManager), the booking is rejected with a specific error message. No "Book anyway" option is available.

| # | Example | Type |
|---|---------|------|
| SCH4b-1-3a | Office "Kingston Dental" is closed on Sundays; actor_role is "Staff"; Staff attempts to book on Sunday → rejected: "Kingston Dental is not open on Sunday. Only a Practice Manager can override this." | ❌ Negative path |
| SCH4b-1-3b | Office closed on public holiday; actor_role is "Staff"; Staff attempts to book on that day → rejected with closed-office message; no override option presented | ❌ Negative path |

---

### SCH4b-Rule-1-4: Provider cannot override C2 (provider not scheduled) — rejected

**Rule**: When a booking attempt fails C2 and the `actor_role` is "Provider" (or any role other than PracticeManager), the booking is rejected. The override path is not available to non-PM actors.

| # | Example | Type |
|---|---------|------|
| SCH4b-1-4a | Provider "Dr. Lloyd Dentist" has no availability at "Kingston Dental" on Monday; actor_role is "Provider"; Provider attempts to book → rejected: "Dr. Lloyd Dentist is not scheduled at Kingston Dental on Monday. Only a Practice Manager can override this." | ❌ Negative path |
| SCH4b-1-4b | actor_role is "Hygienist"; C2 fails → same rejection as Provider; no override path | ❌ Negative path |

---

## RULE 2: Hard stops cannot be overridden even by PM

---

### SCH4b-Rule-2-1: PM cannot override C3 (no chairs available) — always a hard stop

**Rule**: When C3 fails (all chairs at the office are booked for the requested time window), the booking is rejected regardless of `actor_role`. Even a PM providing an `override_reason` cannot proceed — there is nowhere to physically seat the patient.

| # | Example | Type |
|---|---------|------|
| SCH4b-2-1a | All 3 chairs at "Kingston Dental" are booked at 10:00 AM on Monday; actor_role is "PracticeManager"; PM attempts to book with reason "Should fit somehow" → rejected: "No chairs available at Kingston Dental at 10:00 AM — all 3 chairs are booked. Try 10:30 AM or another office." | ❌ Negative path |
| SCH4b-2-1b | 2 of 2 chairs booked at a single-chair annex; PM attempts override → rejected; override_reason ignored; hard stop always fires | ❌ Negative path |

---

### SCH4b-Rule-2-2: PM cannot override C6 (patient double-booked) — always a hard stop

**Rule**: When C6 fails (the patient already has a Booked appointment overlapping the proposed slot anywhere in the practice), the booking is rejected regardless of `actor_role`. A patient cannot physically be in two chairs simultaneously — no PM authority overrides a physical impossibility.

| # | Example | Type |
|---|---------|------|
| SCH4b-2-2a | Patient "Carlton Patient" has a Booked appointment at "Kingston Dental" 10:00–11:00; PM attempts to book "Carlton Patient" at "Kingston Dental" 10:30–11:30 → rejected: "Patient Carlton Patient already has an appointment at 10:00 — a patient cannot be in two chairs at the same time." Even with PM role and reason. | ❌ Negative path |
| SCH4b-2-2b | Patient has appointment at another office at the same time; PM attempts same-time booking at "Kingston Dental" → C6 is practice-wide; rejected with same message | ❌ Negative path (cross-office) |

---

### SCH4b-Rule-2-3: PM cannot override C7 (provider capability) — always a hard stop

**Rule**: When C7 fails (the provider type is not eligible for the procedure's required level), the booking is rejected regardless of `actor_role`. The PM's path for a C7 failure is to select a different capable provider — the PM does not have authority to waive clinical safety requirements. See ADR-004 and ADR-005.

| # | Example | Type |
|---|---------|------|
| SCH4b-2-3a | Procedure "Root Canal" requires Specialist; provider "Sasha Hygienist" is a Hygienist; actor_role is "PracticeManager"; PM attempts to book with reason "Only provider available" → rejected: "Root Canal requires a Specialist or higher. Sasha Hygienist is a Hygienist and is not eligible for this procedure." override_reason is ignored. | ❌ Negative path |
| SCH4b-2-3b | Procedure "Filling" requires Dentist; PM attempts to book a Hygienist for it with an override reason → rejected; C7 hard stop fires before any override logic | ❌ Negative path |
| SCH4b-2-3c | C7 fails; PM should instead select a capable provider (e.g., Dr. Thompson is a Specialist who can perform Root Canal) → booking with eligible provider succeeds normally (no override needed) | ✅ Resolution path |

---

## RULE 3: Override reason is required and stored on the appointment

---

### SCH4b-Rule-3-1: Override reason is required — empty reason is rejected

**Rule**: When a PM attempts to override a soft constraint (C1 or C2), the `override_reason` field must be non-empty (not null, not blank whitespace). An empty or whitespace-only reason is rejected with a specific validation error.

| # | Example | Type |
|---|---------|------|
| SCH4b-3-1a | Office closed; PM actor_role; PM attempts override with `override_reason = ""` → rejected: "Override reason is required." No AppointmentBooked event. | ❌ Negative path |
| SCH4b-3-1b | Office closed; PM actor_role; PM attempts override with `override_reason = "   "` (whitespace only) → rejected: "Override reason is required." | ❌ Negative path |
| SCH4b-3-1c | Office closed; PM actor_role; PM provides reason "Extended hours — staff meeting rescheduled" → override accepted; AppointmentBooked event recorded with override_reason "Extended hours — staff meeting rescheduled" | ✅ Happy path |

---

### SCH4b-Rule-3-2: Override reason is stored in the AppointmentBooked event payload

**Rule**: When a PM override is accepted, the `override_reason` string is persisted as a field on the `AppointmentBooked` event. The appointment list projection materializes this field so the detail panel can display it. Non-override bookings have `override_reason = null` in the event payload.

| # | Example | Type |
|---|---------|------|
| SCH4b-3-2a | PM books with reason "Emergency coverage arranged" → AppointmentBooked event payload contains `override_reason: "Emergency coverage arranged"` | ✅ Happy path |
| SCH4b-3-2b | Front desk books a normal appointment (no soft constraint failure) → AppointmentBooked event payload contains `override_reason: null` | ✅ Happy path (non-override) |
| SCH4b-3-2c | PM books on a fully open day with a fully scheduled provider → no override needed; `override_reason` is null; no "PM override" note shown in detail panel | ✅ Edge case |

---

### SCH4b-Rule-3-3: Overridden appointments display "PM override · [reason]" in the detail panel only

**Rule**: When an appointment's `override_reason` is non-null, the appointment detail panel displays a "PM override · [reason]" note. This flag is shown in the detail panel only — not on the grid cell, to avoid visual noise on the schedule view.

| # | Example | Type |
|---|---------|------|
| SCH4b-3-3a | Appointment with override_reason "Emergency coverage arranged" is opened in the detail panel → panel shows "PM override · Emergency coverage arranged" | ✅ Happy path |
| SCH4b-3-3b | Schedule grid cell for the same appointment → no "PM override" indicator on the grid cell; only in detail panel | ✅ UX constraint |
| SCH4b-3-3c | Appointment with override_reason null → no PM override note in detail panel | ✅ Happy path (non-override) |

---

## RULE 4: C1 and C2 may both be soft-stopped in a single booking

---

### SCH4b-Rule-4-1: PM can override both C1 and C2 in a single booking with one reason

**Rule**: When both C1 (office closed) and C2 (provider not scheduled) fail simultaneously, a PM can override both in a single booking action by providing one `override_reason`. There is no need to resolve each constraint separately. The single reason covers both soft stops. The PM accepts full responsibility.

| # | Example | Type |
|---|---------|------|
| SCH4b-4-1a | Office "Kingston Dental" is closed on Sunday AND provider "Dr. Lloyd Dentist" has no Sunday availability; PM actor_role; PM provides reason "Emergency call-in coverage arranged" → C1 and C2 both soft-stopped; AppointmentBooked event recorded with override_reason "Emergency call-in coverage arranged" | ✅ Happy path |
| SCH4b-4-1b | C1 fails (office closed) and C2 fails (provider not scheduled); PM provides reason "Temporary coverage — both issues known to management" → single booking succeeds; both overrides captured in one reason field | ✅ Happy path |
| SCH4b-4-1c | C1 fails alone; PM provides reason → succeeds (only C1 overridden) | ✅ Happy path (single override) |
| SCH4b-4-1d | C2 fails alone; PM provides reason → succeeds (only C2 overridden) | ✅ Happy path (single override) |

---

### SCH4b-Rule-4-2: When C1 and C2 both fail, Staff is rejected for both — not just the first failure

**Rule**: When both C1 and C2 fail, a Staff actor receives the combined rejection. The error message names the first blocking constraint but the UX does not present any override option. The Staff actor cannot proceed with this booking.

| # | Example | Type |
|---|---------|------|
| SCH4b-4-2a | Office closed AND provider not scheduled; actor_role is "Staff" → rejected at C1: "Kingston Dental is not open on Sunday. Only a Practice Manager can override this." No further action available. | ❌ Negative path |

---

## RULE 5: Reschedule inherits the same override rules

---

### SCH4b-Rule-5-1: PM reschedule to a closed-office day succeeds with a reason

**Rule**: `RescheduleAppointment` checks all 7 constraints on the new slot. The same PM override logic applies: C1 and C2 are soft stops that a PM can override on a reschedule. C3, C4, C5, C6, C7 remain hard stops on reschedule for everyone.

| # | Example | Type |
|---|---------|------|
| SCH4b-5-1a | Existing Booked appointment for "Devon Patient"; PM reschedules to a Sunday (office closed) with reason "Patient-requested Sunday coverage" → C1 soft-stopped on new slot; AppointmentRescheduled on original + AppointmentBooked on new with override_reason | ✅ Happy path |
| SCH4b-5-1b | PM reschedules to a slot where the provider is not scheduled; reason provided → C2 soft-stopped on new slot; reschedule succeeds with override_reason on the new AppointmentBooked | ✅ Happy path |

---

### SCH4b-Rule-5-2: Staff reschedule to a closed-office day is rejected

**Rule**: When a Staff actor attempts to reschedule to a slot that fails C1, the reschedule is rejected with the standard closed-office error. No "Reschedule anyway" override is available to Staff.

| # | Example | Type |
|---|---------|------|
| SCH4b-5-2a | Existing Booked appointment; actor_role is "Staff"; Staff attempts to reschedule to a Sunday (office closed) → no AppointmentRescheduled event; error: "Kingston Dental is not open on Sunday. Only a Practice Manager can override this." Original appointment remains Booked. | ❌ Negative path |
| SCH4b-5-2b | Staff attempts to reschedule to a slot where provider is not scheduled → rejected at C2; no reschedule events; original stays Booked | ❌ Negative path |

---

## Questions Resolved

| # | Question | Resolution |
|---|----------|------------|
| SCH4b-1 | How does `book_appointment` know the actor is a PM? | Pass `actor_role: String` as a command parameter. The UI provides this from session state. No extra DB lookup in the command. Confirmed by Tony (2026-03-05). |
| SCH4b-2 | Override audit: separate event or field on AppointmentBooked? | Add `override_reason: Option<String>` to the `AppointmentBooked` event payload. Non-override bookings have null. Simpler projection, fewer event types. Confirmed by Tony (2026-03-05). |
| SCH4b-3 | Should overridden appointments be visually flagged? | Detail panel only — "PM override · [reason]" note. Not on the schedule grid cell. Confirmed by Tony (2026-03-05). |
| SCH4b-4 | Can C1 and C2 both be overridden in a single booking? | Yes — one reason field covers both soft stops. PM accepts full responsibility. Confirmed by Tony (2026-03-05). |

---

**Phase 2.3 Acceptance Criteria Review**: All business rules validated against `appointment-aggregate.md` and `ADR-005-pm-booking-override-authority.md`. C3, C4, C5, C6, C7 remain hard stops for all actors including PMs. C1 and C2 are soft stops for PMs only. Override reason is required (non-empty). All four open questions from ADR-005 are now marked CONFIRMED. No assumptions outstanding. Ubiquitous language used throughout. Ready for Phase 2.4 BDD Scenarios.

**Maintained By**: Tony + Claude
