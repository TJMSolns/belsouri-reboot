# Example Map: SCH-4 — Provider Capability / Scope of Practice

**Date**: 2026-03-05
**Participants**: Tony (Product Owner / Business), Claude (Developer / Tester)
**Feature**: Provider Capability — enforce that only qualified provider types can be booked for procedures requiring specific clinical expertise, and prevent a patient from being scheduled in overlapping appointments anywhere in the practice.
**Status**: Phase 2.2 + 2.3 complete — all open questions CONFIRMED by Tony (2026-03-05). Ready for Phase 2.4 BDD Scenarios.

---

## Three Amigos Summary (Phase 2.1)

Decisions confirmed by Tony (2026-03-05):

| Item | Decision |
|------|----------|
| Capability check enforcement | Hard block — no override, no warn + continue [CONFIRMED — SCH4-1] |
| Capability model | Capability ladder: Specialist ≥ Dentist ≥ Hygienist. Each type can perform its own procedures AND all lower-level procedures [CONFIRMED — SCH4-2] |
| No required_provider_type set | Any provider type is eligible (open access) [CONFIRMED — SCH4-3] |
| UX — provider dropdown | When user selects a procedure, provider dropdown is hard-filtered to only eligible providers [CONFIRMED — SCH4-4] |
| UX — no eligible providers | Show guidance text: "No eligible providers scheduled on [day]. [procedure] requires a [type] or higher." [CONFIRMED — SCH4-5] |
| Patient double-booking | New constraint C6: same patient cannot be in overlapping appointments at any office simultaneously [CONFIRMED — SCH4-6] |
| C6 scope | Practice-wide — a patient cannot be in two chairs anywhere in the practice at the same time (not office-scoped) [CONFIRMED — SCH4-7] |
| Constraint numbering | C6 = patient no double-booking; C7 = capability check [CONFIRMED — SCH4-8] |

---

## Capability Ladder Reference

| Required Type | Hygienist | Dentist | Specialist |
|---------------|-----------|---------|------------|
| None          | ✓         | ✓       | ✓          |
| Hygienist     | ✓         | ✓       | ✓          |
| Dentist       | ✗         | ✓       | ✓          |
| Specialist    | ✗         | ✗       | ✓          |

A provider can perform any procedure at or below their own level. A Specialist is qualified for everything; a Hygienist is qualified only for Hygienist-required procedures (and procedures with no requirement).

---

## Procedure Capability Mapping (Guidance — Not Binding)

Suggested mappings for the default seed procedures. Practice Manager can configure these via `SetProcedureTypeCapability`.

| Procedure | Suggested Required Type | Rationale |
|-----------|------------------------|-----------|
| Consultation | None | Intake and discussion — any clinical staff |
| Cleaning | Hygienist | Standard preventive hygiene work |
| Fluoride Treatment | Hygienist | Preventive hygiene work |
| Exam | Dentist | Clinical assessment and diagnosis |
| X-Ray | Dentist | Diagnostic interpretation required |
| Filling | Dentist | Restorative procedure |
| Crown | Dentist | Restorative procedure |
| Extraction | Dentist | Invasive procedure (Specialist can also perform) |
| Root Canal | Specialist | Advanced endodontic procedure |
| Whitening | Hygienist | Cosmetic preventive procedure |

These are seeded without a required_provider_type by default. The Practice Manager configures capability via `SetProcedureTypeCapability`.

---

## Rule Cards

---

## Provider Capability (C7)

---

### SCH4-Rule-C7-1: Specialist can perform any procedure regardless of required_provider_type

**Rule**: A Specialist passes C7 for every procedure, including Specialist-required, Dentist-required, Hygienist-required, and procedures with no requirement. The Specialist is at the top of the capability ladder.

| # | Example | Type |
|---|---------|------|
| SCH4-C7-1a | Procedure "Root Canal" requires Specialist; Dr. Thompson is a Specialist; booking attempted → C7 passes | ✅ Happy path |
| SCH4-C7-1b | Procedure "Filling" requires Dentist; Dr. Thompson is a Specialist; booking attempted → C7 passes (Specialist ≥ Dentist) | ✅ Happy path |
| SCH4-C7-1c | Procedure "Cleaning" requires Hygienist; Dr. Thompson is a Specialist; booking attempted → C7 passes (Specialist ≥ Hygienist) | ✅ Happy path |
| SCH4-C7-1d | Procedure "Consultation" has no required_provider_type; Dr. Thompson is a Specialist; booking attempted → C7 passes (None = any provider eligible) | ✅ Happy path |

---

### SCH4-Rule-C7-2: Dentist can perform Dentist-required and Hygienist-required procedures, but not Specialist-required

**Rule**: A Dentist passes C7 for procedures requiring Dentist or Hygienist (or no requirement), but fails C7 for Specialist-required procedures. The Dentist cannot perform procedures that require a higher-level qualification.

| # | Example | Type |
|---|---------|------|
| SCH4-C7-2a | Procedure "Filling" requires Dentist; Dr. Spence is a Dentist; booking attempted → C7 passes | ✅ Happy path |
| SCH4-C7-2b | Procedure "Cleaning" requires Hygienist; Dr. Spence is a Dentist; booking attempted → C7 passes (Dentist ≥ Hygienist) | ✅ Happy path |
| SCH4-C7-2c | Procedure "Consultation" has no required_provider_type; Dr. Spence is a Dentist; booking attempted → C7 passes | ✅ Happy path |
| SCH4-C7-2d | Procedure "Root Canal" requires Specialist; Dr. Spence is a Dentist; booking attempted → C7 fails: "Root Canal requires a Specialist or higher. Dr. Spence is a Dentist and is not eligible for this procedure." | ❌ Negative path |

---

### SCH4-Rule-C7-3: Hygienist can perform Hygienist-required procedures and open procedures only

**Rule**: A Hygienist passes C7 only for procedures requiring Hygienist or no requirement. A Hygienist fails C7 for Dentist-required or Specialist-required procedures.

| # | Example | Type |
|---|---------|------|
| SCH4-C7-3a | Procedure "Cleaning" requires Hygienist; Sarah Williams is a Hygienist; booking attempted → C7 passes | ✅ Happy path |
| SCH4-C7-3b | Procedure "Consultation" has no required_provider_type; Sarah Williams is a Hygienist; booking attempted → C7 passes | ✅ Happy path |
| SCH4-C7-3c | Procedure "Filling" requires Dentist; Sarah Williams is a Hygienist; booking attempted → C7 fails: "Filling requires a Dentist or higher. Sarah Williams is a Hygienist and is not eligible for this procedure." | ❌ Negative path |
| SCH4-C7-3d | Procedure "Root Canal" requires Specialist; Sarah Williams is a Hygienist; booking attempted → C7 fails: "Root Canal requires a Specialist or higher. Sarah Williams is a Hygienist and is not eligible for this procedure." | ❌ Negative path |

---

### SCH4-Rule-C7-4: No required_provider_type — any provider type is eligible

**Rule**: When a procedure has no required_provider_type (None), the C7 constraint always passes regardless of the provider's type. Open access means any registered provider may book.

| # | Example | Type |
|---|---------|------|
| SCH4-C7-4a | Procedure "Consultation" has no required_provider_type; Hygienist books it → C7 passes | ✅ Happy path |
| SCH4-C7-4b | Procedure "Consultation" has no required_provider_type; Dentist books it → C7 passes | ✅ Happy path |
| SCH4-C7-4c | Procedure "Consultation" has no required_provider_type; Specialist books it → C7 passes | ✅ Happy path |
| SCH4-C7-4d | Practice Manager clears required_provider_type on "Root Canal" (sets to None); any provider can now book → C7 passes for all types | ✅ Edge case |

---

## Patient No Double-Booking (C6)

---

### SCH4-Rule-C6-1: Patient cannot be in two overlapping appointments anywhere in the practice

**Rule**: When booking an appointment for a patient, the system checks all existing Booked appointments for that patient across all offices. If the proposed time window overlaps with any existing Booked appointment, C6 fails. Overlap uses the same formula as C3: `existing.start < proposed.end AND existing.end > proposed.start`. C6 is practice-wide — unlike C3 (chair capacity), which is office-scoped.

| # | Example | Type |
|---|---------|------|
| SCH4-C6-1a | Patient "Maria Brown" has a Booked appointment at Main Office 10:00–11:00; front desk books her at Main Office 13:00–14:00 → No overlap; C6 passes → AppointmentBooked | ✅ Happy path |
| SCH4-C6-1b | Patient "Maria Brown" has a Booked appointment at Main Office 10:00–11:00; front desk attempts to book her at Main Office 10:30–11:30 → Overlap (10:00 < 11:30 AND 11:00 > 10:30); C6 fails: "Patient Maria Brown already has an appointment at 10:00 — a patient cannot be in two chairs at the same time" | ❌ Negative path |
| SCH4-C6-1c | Patient "Maria Brown" has a Booked appointment at Main Office 10:00–11:00; front desk attempts to book her at Montego Bay Office 10:00–11:00 → Same-time overlap across different offices; C6 fails: "Patient Maria Brown already has an appointment at 10:00 — a patient cannot be in two chairs at the same time" | ❌ Negative path (cross-office) |
| SCH4-C6-1d | Patient "Maria Brown" has a Booked appointment at Main Office 10:00–11:00; front desk books her at Main Office 11:00–12:00 → Adjacent (existing.end = proposed.start, not strictly less than); no overlap; C6 passes | ✅ Boundary |
| SCH4-C6-1e | Patient "Maria Brown" has a Cancelled appointment at Main Office 10:00–11:00; front desk books her at Main Office 10:30–11:30 → Cancelled appointment does not count; C6 passes | ✅ Edge case |
| SCH4-C6-1f | Patient "Maria Brown" has a Completed appointment at Main Office 10:00–11:00; front desk books her at Main Office 10:30–11:30 → Completed appointment does not count; C6 passes | ✅ Edge case |
| SCH4-C6-1g | Different patient "James Clarke" has appointment at Main Office 10:00–11:00; front desk books "Maria Brown" at Main Office 10:00–11:00 → C6 checks are per-patient; Maria Brown's own schedule is clear; C6 passes | ✅ Happy path |

---

### SCH4-Rule-C6-2: Adjacent appointments are not overlapping (boundary condition)

**Rule**: Overlap is defined strictly as `existing.start < proposed.end AND existing.end > proposed.start`. When an existing appointment ends exactly at the proposed start time (or the proposed end time equals the existing start time), the appointments are adjacent — not overlapping — and C6 passes.

| # | Example | Type |
|---|---------|------|
| SCH4-C6-2a | Patient has 10:00–11:00; new appointment proposed 11:00–12:00; existing.end (11:00) = proposed.start (11:00), condition `existing.end > proposed.start` is false → no overlap; C6 passes | ✅ Boundary |
| SCH4-C6-2b | Patient has 11:00–12:00; new appointment proposed 10:00–11:00; proposed.end (11:00) = existing.start (11:00), condition `existing.start < proposed.end` is false → no overlap; C6 passes | ✅ Boundary |
| SCH4-C6-2c | Patient has 10:00–11:00; new appointment proposed 10:59–11:59; existing.start (10:00) < proposed.end (11:59) AND existing.end (11:00) > proposed.start (10:59) → overlap of 1 minute; C6 fails | ❌ Boundary |

---

## Questions Resolved

| # | Question | Resolution |
|---|----------|------------|
| SCH4-1 | Can the user override the C7 capability check? | No. Hard block at MVP — no override, no warn + continue. Confirmed by Tony (2026-03-05). |
| SCH4-2 | What happens when no eligible provider is available for the selected procedure? | The booking form shows guidance text: "No eligible providers scheduled on [day]. [procedure] requires a [type] or higher." The provider dropdown is empty. Booking cannot proceed until an eligible provider is scheduled or the procedure is changed. Confirmed by Tony (2026-03-05). |
| SCH4-3 | Does C6 (patient double-booking) apply within a single office or across the practice? | Across the practice — a patient cannot be in two chairs anywhere at the same time. C6 is practice-wide, unlike C3 (chair capacity), which is office-scoped. Confirmed by Tony (2026-03-05). |
| SCH4-4 | Does C7 apply to reschedule as well as initial booking? | Yes. The new slot for a reschedule is checked against all 7 constraints, including C6 and C7. |
| SCH4-5 | What is the capability ladder ordering? | Specialist ≥ Dentist ≥ Hygienist. Each level can perform its level and all levels below. Confirmed by Tony (2026-03-05). |
| SCH4-6 | Do Cancelled/Completed/NoShow/Rescheduled appointments count for C6? | No. Only Booked appointments count. Terminal appointments do not occupy a chair. |

---

**Phase 2.3 Acceptance Criteria Review**: All business rules validated against `appointment-aggregate.md` and `procedure-type-aggregate.md`. C6 and C7 failure messages match the spec exactly. All open questions marked CONFIRMED. No assumptions outstanding. Ubiquitous language used throughout. Ready for Phase 2.4 BDD Scenarios.

**Maintained By**: Tony + Claude
