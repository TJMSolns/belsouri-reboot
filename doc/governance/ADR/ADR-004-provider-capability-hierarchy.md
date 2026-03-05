# ADR-004: Provider Capability Hierarchy for Appointment Booking

**Status**: Accepted
**Date**: 2026-03-05
**Deciders**: Tony (Product Owner), Claude (Developer)
**Supersedes**: (none)
**Category**: Domain

---

## Context

The booking system needs to enforce that only clinically qualified providers can be booked for procedures requiring specific expertise. Without this constraint, the system could allow a Hygienist to be booked for a Root Canal — a serious clinical risk that could expose the practice to liability.

The three provider types in the domain are Dentist, Hygienist, and Specialist (see `provider-aggregate.md`). Dental procedures vary in clinical complexity: some can be performed by any provider (e.g., a new patient Consultation), some require at minimum a Hygienist (e.g., Cleaning), some require at minimum a Dentist (e.g., Filling, Crown), and some require a Specialist (e.g., Root Canal, complex oral surgery).

The question is: how should the system determine and enforce which provider types are eligible for a given procedure, and what should happen when an ineligible provider is selected?

---

## Decision

**Use a capability ladder (Specialist ≥ Dentist ≥ Hygienist) as a hard booking constraint (C7).**

Each ProviderType is eligible to perform procedures at or below its own level:

| Required Type | Hygienist eligible? | Dentist eligible? | Specialist eligible? |
|---------------|---------------------|-------------------|----------------------|
| None          | Yes                 | Yes               | Yes                  |
| Hygienist     | Yes                 | Yes               | Yes                  |
| Dentist       | No                  | Yes               | Yes                  |
| Specialist    | No                  | No                | Yes                  |

The `required_provider_type` field is set per-procedure on the `ProcedureType` aggregate via the `SetProcedureTypeCapability` command. When not set (None), any provider type is eligible — open access is preserved by default.

C7 is a hard stop at booking time. There is no override and no warn + continue path at MVP. Confirmed by Tony (2026-03-05).

The booking form pre-filters the provider dropdown to only show eligible providers when a procedure is selected. If no eligible providers are scheduled on the requested day, the form shows guidance text: "No eligible providers scheduled on [day]. [procedure] requires a [type] or higher." The backend enforces C7 independently as a hard stop regardless of what the UI presents.

---

## Alternatives Considered

### Alternative 1: Exact Match Only

Require the provider type to match the procedure's required_provider_type exactly. A Filling requires Dentist → only a Dentist can book; a Specialist cannot.

**Rejected because**: A Specialist is clinically qualified to perform anything a Dentist can do. Blocking Specialists from Dentist-level procedures would be incorrect from a clinical standpoint and would cause unnecessary booking failures in practices where Specialists occasionally cover general dentistry.

---

### Alternative 2: Warning Only — Override Allowed

When an ineligible provider is selected, show a warning but allow the booking to proceed if the user confirms.

**Rejected by Tony (2026-03-05)**: Tony confirmed this must be a hard block. A warning-with-override path creates a loophole that undermines the clinical safety guarantee the feature is meant to provide. Hard block only, no override at MVP.

---

### Alternative 3: Per-Procedure Provider Whitelist

Instead of a capability level, maintain an explicit whitelist of approved provider_ids for each procedure.

**Rejected because**: Per-provider whitelists require ongoing maintenance whenever a provider is added or changes role. The capability ladder achieves the same clinical safety with far less administrative overhead. A Dentist-level classification on a provider is sufficient to determine eligibility — the practice does not need to enumerate individual providers per procedure. Too complex for MVP.

---

## Consequences

**Positive**:
- Clinical safety is enforced at the system boundary — ineligible providers cannot be booked regardless of user action
- Open access by default (required_provider_type = None) means existing procedures and workflows continue uninterrupted until the Practice Manager explicitly configures capability
- Capability ladder is intuitive to clinical staff — a Specialist performing a Filling is obviously fine
- Per-procedure configuration is flexible — Extraction can be set to Dentist even though it's in the same Invasive category as Root Canal (Specialist)
- Frontend pre-filtering improves UX by preventing the user from selecting an ineligible provider in the first place
- Backend enforcement is independent of the UI — the hard stop holds even if the frontend filter is bypassed

**Negative**:
- Practice Manager must manually configure `required_provider_type` for each procedure after setup — no automatic inference from procedure name or category
- The booking form's guidance text when no eligible providers are available ("No eligible providers scheduled on [day]. [procedure] requires a [type] or higher.") requires the frontend to know the required_provider_type for the selected procedure, adding a data dependency to the booking form's load

**Domain Impact**:
- `ProcedureType` aggregate gains `required_provider_type` field, `ProcedureTypeCapabilitySet` event, and `SetProcedureTypeCapability` command
- `BookAppointment` gains constraint C7 (capability check)
- `RescheduleAppointment` gains C7 on the new slot (same as all other constraints)
- `appointment_list` projection is unaffected — capability is checked at booking time, not stored on the appointment
- Existing procedures have `required_provider_type = None` by default — no migration required

---

## Related

- `doc/domain/aggregates/procedure-type-aggregate.md` — `required_provider_type` field, `SetProcedureTypeCapability` command, `ProcedureTypeCapabilitySet` event
- `doc/domain/aggregates/appointment-aggregate.md` — C7 booking constraint, C6 patient no double-booking constraint
- `doc/domain/aggregates/provider-aggregate.md` — ProviderType value object (Dentist | Hygienist | Specialist)
- `doc/scenarios/example-maps/sch4-capability-examples.md` — full example map with all rule cards
- `features/patient-scheduling.feature` — PS-RULE-C6 and PS-RULE-C7 BDD scenarios

**Reviewed By**: Tony (Product Owner)
