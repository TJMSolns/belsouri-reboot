# ADR-005: PM Booking Override Authority

**Status**: Accepted
**Date**: 2026-03-05
**Deciders**: Tony (Product Owner), Claude (Developer)
**Supersedes**: (none — clarifies ADR-004 §C7 note)
**Category**: Domain

---

## Context

The booking system enforces seven constraints at appointment creation time (C1–C7). ADR-004 addressed C7 (provider capability) and noted it was a hard block with no override.

Tony confirmed (2026-03-05):

> "To schedule, you must have: a patient, a procedure, a provider who can do the procedure, a chair where the procedure can take place, and an open office to accommodate the chair. PM may override as long as they can provide all missing points (either by scheduling themselves or others)."

This decision classifies each constraint as either a **hard stop** (no override, not even by PM) or a **soft stop** (PM may override with explicit acknowledgment).

---

## Decision

### Constraint Classification

| # | Constraint | Class | Rationale |
|---|-----------|-------|-----------|
| C1 | Office is open at the requested time | **Soft — PM override** | A PM may extend hours or provide holiday coverage that isn't reflected in the formal schedule. The physical office exists and can accommodate the appointment. |
| C2 | Provider is scheduled at the office on that day | **Soft — PM override** | A PM may know a provider is available by arrangement even if their availability window hasn't been set up. The PM accepts responsibility for coverage. |
| C3 | At least one chair is available at the requested time | **Hard stop** | Physical constraint — the chair count is an absolute ceiling set by the practice. There is nowhere to seat the patient. No PM authority overrides physics. |
| C4 | Patient is not archived | **Hard stop** | An archived patient is not active in the practice. Booking for an inactive patient is a data integrity error, not a scheduling gap. |
| C5 | Procedure type is active | **Hard stop** | A deactivated procedure has been deliberately removed from use. A PM who wants to re-enable it should use `ReactivateProcedureType`, not book around it. |
| C6 | Patient has no other booked appointment at the same time | **Hard stop** | A patient cannot physically occupy two chairs simultaneously at any office. No PM can override a physical impossibility. |
| C7 | Provider type meets or exceeds procedure's required_provider_type | **Hard stop** | Clinical safety constraint confirmed in ADR-004. "PM may override as long as they provide a provider who **can** do the procedure" — this means the PM must select an eligible provider, not bypass the check. Override path = selecting a different capable provider. |

**Note on C7 and PM override**: The PM override does not grant authority to book an ineligible provider for a capability-gated procedure. The PM's "override" for a failed C7 is to select a different provider who meets the capability requirement, or to reschedule when an eligible provider is available. The hard block remains. This is consistent with ADR-004 which confirmed "no override" specifically for the capability check.

---

## Override Flow (UX Sketch)

The booking form presents two tracks based on the acting user's role:

### Standard Track (Staff, Providers)

Constraints C1–C7 are enforced without override. If any fail, the form shows the error message and stops. No "Book anyway" option is presented.

### PM Track (Practice Manager role)

If C1 or C2 fails, the PM sees a **warning banner** rather than a hard block, plus an override expansion:

```
┌─────────────────────────────────────────────────────────────┐
│ ⚠  Main Office is marked closed on Tuesday 18 March.       │
│    Normal bookings are blocked.                             │
│                                                             │
│    [Book anyway — I'm providing coverage]  ↓               │
└─────────────────────────────────────────────────────────────┘

↓ Expanded override section:

  Override reason (required):
  [ e.g. Extended hours — staff meeting rescheduled      ]

  Provider:
  [ All providers at this office ▾ ]   ← full list, not filtered by schedule
  Note: provider capability (C7) still applies.

  [Confirm Booking with Override]
```

C3, C4, C5, C6 still produce hard blocks even in the PM track:

```
┌─────────────────────────────────────────────────────────────┐
│ ✕  No chairs available at Main Office at 10:00 AM.         │
│    All 3 chairs are booked. Try 10:30 AM or another        │
│    office.                                                  │
└─────────────────────────────────────────────────────────────┘
```

No override option is shown for hard stops.

---

## Decisions (Phase 2 Ceremony — Confirmed 2026-03-05)

All four open questions from the initial ADR draft have been resolved:

| Question | Decision |
|---|---|
| How does book_appointment identify PM actor? | Pass `actor_role: String` as a command parameter. UI provides this from session state. |
| Override audit: separate event or payload field? | Add `override_reason: Option<String>` to `AppointmentBooked` payload. Non-override bookings have null. |
| Visual flag on overridden appointments? | Detail panel only — "PM override · [reason]" note. Not shown on grid cell. |
| Can C1 + C2 both be overridden in one booking? | Yes — single reason field covers both soft stops. |

---

## Implementation Notes

This decision modifies the `book_appointment` command (an existing booking invariant). Ceremony is required before implementation:

### Phase 2 Ceremonies Required

**Three Amigos questions to resolve:**
1. How does the system know the acting user is a PM? The `booked_by` field currently takes a free-form string. Does the command need an `actor_role` parameter, or does it look up the actor's role from the staff projection?
2. Is the override reason stored on the `AppointmentBooked` event payload, or as a separate `BookingConstraintOverridden` event on the same stream?
3. Should overridden appointments be visually flagged in the schedule grid / detail panel?
4. Can a PM override C1 (office closed) AND C2 (no provider scheduled) in a single booking, or must each be resolved separately?

**Backend changes (post-ceremony):**
- `book_appointment` command: add `pm_override: Option<PmOverrideInput>` parameter
  - `PmOverrideInput = { actor_role: String, override_reason: String }`
- Command logic: if `pm_override` is Some, verify actor is PM, then skip C1/C2 checks only
- `AppointmentBooked` event payload: add `override_reason: Option<String>` field
- Projection: add `override_reason` column to `appointment_list`

**Frontend changes (post-ceremony):**
- `schedule/+page.svelte` booking form: detect if current user is PM (from staff setup / PIN context), conditionally render override expansion when C1/C2 fail

**BDD scenarios to add** (in `features/patient-scheduling.feature`):
- PM overrides C1 (closed office) → appointment books with override note
- PM overrides C2 (provider not scheduled) → appointment books with override note
- Staff attempts C1/C2 override → rejected (no override authority)
- PM attempts to override C3/C4/C5/C6 → rejected even with PM role
- PM attempts C7 override without eligible provider → still rejected

---

## Alternatives Considered

### Alternative: PM Override for All Constraints Including C7

**Rejected**: Tony's phrasing "a provider who can do the procedure" makes clear that capability is non-negotiable. Allowing PM override of C7 would create a two-tier clinical safety model (one for PMs, one for everyone else) that undermines the guarantee ADR-004 was written to provide. The PM's authority is operational, not clinical.

### Alternative: Override by Exception Request (async approval flow)

A booking attempt that fails a soft constraint could enter a "pending PM approval" state that the PM approves or rejects separately.

**Rejected for MVP**: Too complex. The PM IS the approver — there is no upstream authority. Direct PM override at booking time achieves the same result with far less ceremony. An async approval flow may be relevant post-MVP if multi-location practices need a remote PM to approve overrides.

---

## Consequences

**Positive:**
- PMs can handle real-world scheduling gaps (last-minute coverage, informal arrangements) without requiring a formal availability update before booking
- The distinction between soft and hard constraints maps cleanly to what is physically possible vs. what is administratively configured
- Override reason is stored on the event — audit trail is automatic

**Negative:**
- The booking form must know the current user's role to render the correct track. This requires either a session concept or passing role information through the form state
- Soft-stop override adds a new code path to `book_appointment` — more complex command logic and more BDD scenarios required

**Scope:**
- `RescheduleAppointment` inherits the same constraint set and override rules — a PM reschedule can also override C1/C2 on the new slot

---

## Related

- `doc/governance/ADR/ADR-004-provider-capability-hierarchy.md` — C7 hard block confirmed; this ADR does not change that decision
- `doc/domain/aggregates/appointment-aggregate.md` — C1–C7 constraint table
- `doc/scenarios/example-maps/sch4-capability-examples.md` — C7 example mapping
- `features/patient-scheduling.feature` — PS-RULE-C6, PS-RULE-C7 (BDD scenarios for override to be added post-ceremony)

**Reviewed By**: Tony (Product Owner)
