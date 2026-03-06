# Context Map

**Last Updated**: 2026-03-06 (DM-1: Provider aggregate retired, clinical data moved to Staff Management)

---

## Overview

This map shows the bounded contexts in Belsouri, their relationships, and the integration patterns between them.

All MVP bounded contexts have completed Phase 1 and Phase 2 ceremonies. DM-1 correction applied 2026-03-06: Provider IS A StaffMember. Practice Setup no longer owns provider/clinical data — it owns offices, procedure types, and practice identity only.

---

## Bounded Contexts

| Context | Status | Purpose |
|---------|--------|---------|
| **Licensing** | Phase 1 + Phase 2 complete — ready for Track A | Machine-bound license validation, eval period, module gating, degraded mode |
| **Practice Setup** | Phase 1 + Phase 2 complete — DM-1 boundary update 2026-03-06 | Offices, procedure types, practice identity. **No longer owns providers.** |
| **Staff Management** | Phase 1 + Phase 2 complete — DM-1 boundary update 2026-03-06 | Staff records, roles, PIN, and **clinical configuration (availability, office assignments, exceptions)**. All staff including clinical providers. |
| **Staff Scheduling** | Phase 1 + Phase 2 complete — ready for Track A (projection-first model confirmed) | Provider availability patterns, working hours, exceptions |
| **Patient Management** | Phase 1 + Phase 2 complete — ready for Track A | Patient registration, demographics, search |
| **Patient Scheduling** | Phase 1 + Phase 2 complete — ready for Track A | Appointment booking, today's schedule, cancellations |
| **Clinical Records** | Planned (Post-MVP) | Charting, treatment plans, clinical notes |
| **Billing/Insurance** | Planned (Post-MVP) | Invoicing, insurance claims, payments |
| **Jamaica EHR Integration** | Planned (Post-MVP) | Regulatory compliance, data export |
| **Reporting** | Deferred (Post-MVP) | Practice-wide dashboards, capacity utilization, provider load. May emerge as a bounded context or remain a cross-cutting concern. |

---

## Context Map Diagram

```mermaid
graph TD
    LS_EXT["License Server\n(External — OHS/PL)"]
    LIC["Licensing\n(upstream, ready for Track A)"]
    PS["Practice Setup\n(upstream, ready for Track A)"]
    SM["Staff Management\n(downstream, ready for Track A)"]
    SS["Staff Scheduling\n(downstream, ready for Track A)"]
    PM["Patient Management\n(downstream, ready for Track A)"]
    PSched["Patient Scheduling\n(downstream, ready for Track A)"]
    CR["Clinical Records\n(post-MVP)"]
    BI["Billing/Insurance\n(post-MVP)"]

    LS_EXT -->|"OHS/PL → ACL"| LIC
    LIC -->|"OHS/PL (feature gate)"| PS
    LIC -->|"OHS/PL (feature gate)"| SM
    LIC -->|"OHS/PL (feature gate)"| SS
    LIC -->|"OHS/PL (feature gate)"| PM
    LIC -->|"OHS/PL (feature gate)"| PSched
    PS -->|"OHS/PL"| SM
    PS -->|"OHS/PL"| SS
    PS -->|"OHS/PL"| PM
    PS -->|"OHS/PL"| PSched
    SM -->|"OHS/PL"| SS
    SS -->|"CF"| PSched
    PM -->|"CF"| PSched
    PSched -->|"OHS/PL (future)"| CR
    CR -->|"OHS/PL (future)"| BI
```

**Legend**: OHS = Open Host Service, PL = Published Language, CF = Conformist, ACL = Anti-Corruption Layer

---

## Relationships

### License Server → Licensing

| Property | Value |
|----------|-------|
| **Direction** | License Server is upstream (external); Licensing is downstream |
| **Pattern** | Open Host Service / Published Language → Anti-Corruption Layer |
| **What flows** | Signed license keys (LicensePayload + Ed25519 signature, base64url encoded) |
| **Integration** | License Server signs keys; Practice Manager obtains and enters them manually. No runtime API call from the app — fully offline integration. |
| **Contract** | License Server publishes a stable payload schema (schema_version field). Licensing context validates against the schema version it understands. |

**Why ACL on the Licensing side**: The License Server has its own model (REST API, JSON schema). Licensing translates this into domain events (LicenseIssued, LicenseRenewed) using an Anti-Corruption Layer. If the License Server's schema evolves, only the ACL changes — not domain logic.

**Why OHS/PL on the License Server side**: Tony controls both sides. The License Server publishes a stable, versioned schema. Breaking changes require a new schema_version.

---

### Licensing → Practice Setup

| Property | Value |
|----------|-------|
| **Direction** | Licensing is upstream; Practice Setup is downstream |
| **Pattern** | Open Host Service / Published Language (feature gate) |
| **What flows** | Module access decisions (`core` module required for Practice Setup access) |
| **Integration** | Practice Setup feature access is gated by Licensing's `license_status` projection. If the `core` module is not licensed or status is Expired/Invalid, Practice Setup screens are blocked. |
| **Contract** | Practice Setup reads `license_status.modules` and `license_status.status`. It does not call Licensing commands. |

**Why upstream**: Licensing is checked before any Practice Setup operation. Practice Setup cannot function without a valid license.

---

### Licensing → Staff Scheduling

| Property | Value |
|----------|-------|
| **Direction** | Licensing is upstream; Staff Scheduling is downstream |
| **Pattern** | Open Host Service / Published Language (feature gate) |
| **What flows** | Module access decisions (`scheduling` module required) |
| **Integration** | Same pattern as Licensing → Practice Setup. Staff Scheduling reads `license_status` projection. |
| **Contract** | Confirmed during Phase 2 ceremonies. Staff Scheduling reads `license_status.modules` and `license_status.status`. |

---

### Licensing → Patient Management

| Property | Value |
|----------|-------|
| **Direction** | Licensing is upstream; Patient Management is downstream |
| **Pattern** | Open Host Service / Published Language (feature gate) |
| **What flows** | Module access decisions (`core` module required) |
| **Integration** | Same pattern as Licensing → Practice Setup. |
| **Contract** | Confirmed during Phase 2 ceremonies. Patient Management reads `license_status.modules` and `license_status.status`. |

---

### Licensing → Patient Scheduling

| Property | Value |
|----------|-------|
| **Direction** | Licensing is upstream; Patient Scheduling is downstream |
| **Pattern** | Open Host Service / Published Language (feature gate) |
| **What flows** | Module access decisions (`scheduling` module required) |
| **Integration** | Same pattern as other Licensing downstream relationships. |
| **Contract** | Confirmed during Phase 2 ceremonies. Patient Scheduling reads `license_status.modules` and `license_status.status`. |

---

### Licensing → Staff Management

| Property | Value |
|----------|-------|
| **Direction** | Licensing is upstream; Staff Management is downstream |
| **Pattern** | Open Host Service / Published Language (feature gate) |
| **What flows** | Module access decisions (`core` module required) |
| **Integration** | Staff Management reads `license_status` projection. Write operations blocked when module is Degraded/Expired/Invalid. |
| **Contract** | Confirmed during Phase 2 ceremonies. Staff Management reads `license_status.modules` and `license_status.status`. |

---

### Practice Setup → Staff Management

| Property | Value |
|----------|-------|
| **Direction** | Practice Setup is upstream; Staff Management is downstream |
| **Pattern** | Open Host Service / Published Language |
| **What flows** | Office list (for staff-office assignment), Provider types |
| **Integration** | Staff Management reads Practice Setup projections to know which offices exist when assigning staff members. |
| **Contract** | Confirmed during Phase 2 ceremonies. Minimal -- office_id and name needed for assignment. |

**Why OHS/PL**: Practice Setup publishes a stable set of offices and providers. Staff Management consumes them without translation.

---

### Staff Management → Staff Scheduling

| Property | Value |
|----------|-------|
| **Direction** | Staff Management is upstream; Staff Scheduling is downstream |
| **Pattern** | Open Host Service / Published Language |
| **What flows** | Staff identity (id, name, role), ClinicalSpecialization, office assignments, availability windows, exceptions |
| **Integration** | Staff Scheduling reads Staff Management projections to build the ResolvedSchedule. All provider availability data originates here — not from Practice Setup. |
| **Contract** | Updated DM-1 (2026-03-06). Staff Scheduling reads the full clinical configuration from Staff Management's projection. |

**Why OHS/PL**: Staff Management publishes a stable staff roster including all clinical data. Staff Scheduling consumes it without translation.

**DM-1 update**: This relationship now carries ALL clinical scheduling data (previously split between Practice Setup Provider aggregate and Staff Management). Staff Scheduling no longer needs Practice Setup for provider information.

---

### Practice Setup → Staff Scheduling

| Property | Value |
|----------|-------|
| **Direction** | Practice Setup is upstream; Staff Scheduling is downstream |
| **Pattern** | Open Host Service / Published Language |
| **What flows** | Office configuration (hours, chair count) only |
| **Integration** | Staff Scheduling reads Practice Setup projections to know which offices exist and their operating hours. It does not read provider data from Practice Setup — providers are now in Staff Management. |
| **Contract** | Staff Scheduling conforms to Practice Setup's office schema. |

**DM-1 update**: Provider availability, office assignments, and exceptions no longer flow from Practice Setup. They now flow from Staff Management (see Staff Management → Staff Scheduling below).

---

### Practice Setup → Patient Management

| Property | Value |
|----------|-------|
| **Direction** | Practice Setup is upstream; Patient Management is downstream |
| **Pattern** | Open Host Service / Published Language |
| **What flows** | Office list (for patient-office association) |
| **Integration** | Patient Management reads the list of offices for record filtering (e.g., "show patients for Kingston"). |
| **Contract** | Minimal -- only office_id and name needed. |

**Why OHS/PL**: Same reasoning as above. Lightweight dependency -- Patient Management only needs the office list.

---

### Practice Setup → Patient Scheduling

| Property | Value |
|----------|-------|
| **Direction** | Practice Setup is upstream; Patient Scheduling is downstream |
| **Pattern** | Open Host Service / Published Language |
| **What flows** | Office hours and chair count, Procedure type durations |
| **Integration** | Patient Scheduling reads Practice Setup projections to validate: office open (C1), chair capacity (C3), procedure type active (C5), procedure duration. Provider availability (C2) now comes from Staff Management via Staff Scheduling. |
| **Contract** | Updated DM-1 (2026-03-06). Practice Setup provides office + procedure data only. |

**DM-1 update**: Provider availability no longer flows from Practice Setup to Patient Scheduling. Provider-related booking constraints (C2 provider available) are validated against Staff Scheduling's ResolvedSchedule, which now sources from Staff Management.

---

### Staff Scheduling → Patient Scheduling

| Property | Value |
|----------|-------|
| **Direction** | Staff Scheduling is upstream; Patient Scheduling is downstream |
| **Pattern** | Conformist |
| **What flows** | Resolved provider schedules (combining weekly patterns with exceptions) |
| **Integration** | Patient Scheduling needs to know "is this provider available at this office at this time?" Staff Scheduling provides the authoritative answer. |
| **Contract** | Confirmed during Phase 2 ceremonies. Patient Scheduling reads resolved provider schedules from Staff Scheduling's projection. |

**Why Conformist**: Patient Scheduling has no leverage to change how Staff Scheduling models availability. It conforms to whatever Staff Scheduling publishes.

---

### Patient Management → Patient Scheduling

| Property | Value |
|----------|-------|
| **Direction** | Patient Management is upstream; Patient Scheduling is downstream |
| **Pattern** | Conformist |
| **What flows** | Patient identity (id, name) for booking |
| **Integration** | Patient Scheduling needs a patient to book an appointment for. It reads Patient Management's projection. |
| **Contract** | Confirmed during Phase 2 ceremonies. Patient Scheduling reads patient identity (id, name) from Patient Management's projection. |

---

## Integration Patterns Used

| Pattern | When We Use It | Why |
|---------|---------------|-----|
| **Open Host Service / Published Language (OHS/PL)** | License Server → Licensing (server side); Licensing → all MVP contexts; Practice Setup → all downstream contexts | Publisher maintains a stable, versioned contract. Consumers read without translation (except where ACL is noted). |
| **Anti-Corruption Layer (ACL)** | Licensing context (translating License Server responses into domain events) | License Server is an external system with its own model. ACL protects the Licensing domain from external schema changes. |
| **Conformist (CF)** | Staff Scheduling → Patient Scheduling, Patient Management → Patient Scheduling | Patient Scheduling conforms to upstream models. It has no business reason to translate or reinterpret. |

---

## Dependency Map (from DEVELOPMENT-PLAN.md)

```
License Server (external) ──► Licensing ──► All MVP Contexts

Infrastructure ──┬──► Practice Setup ──┬──► Staff Management ──► Staff Scheduling ──┬──► Patient Scheduling
                 │                     │                                             │
                 │                     └─────────────────────────────────────────────┘
                 │
                 └──► Patient Management ────────────────────────────────────────────► Patient Scheduling
```

- **Licensing** gates feature access across all MVP contexts; implemented in Track A infrastructure
- **Practice Setup** requires Infrastructure (event store, projections, Tauri commands)
- **Staff Management** requires Practice Setup (offices must exist for assignment)
- **Staff Scheduling** requires Practice Setup (offices/providers) and Staff Management (staff roster)
- **Patient Management** requires Infrastructure (not Practice Setup -- can run in parallel)
- **Patient Scheduling** requires both Staff Scheduling (available slots) and Patient Management (patients to book)

---

## Boundary Watch List

Boundaries that may shift as we learn more during future ceremonies:

| Boundary | Current | May Shift To | Trigger |
|----------|---------|-------------|---------|
| Provider availability + exceptions | ~~Practice Setup~~ → **Staff Management** | Staff Scheduling (reads from Staff Management) | **RESOLVED — DM-1 (2026-03-06)**: Provider IS A StaffMember. Clinical data now lives entirely in Staff Management. The watch-list shift has been made. |
| Procedure type ↔ Billing codes | Practice Setup owns procedure types | Billing context may add fee schedules and insurance codes | Post-MVP when Billing context is discovered |
| Office address | Practice aggregate has practice address; Office has no address | Office may need its own address | Multi-location practices with distinct addresses |
| Module gating mechanism | Licensing projection read by each context directly | Dedicated cross-cutting service / middleware | If module checks become complex enough to warrant centralization |

---

**Next update**: Expand Staff Management relationship details and update contract specifics as Track A implementation proceeds.

---

**Maintained By**: Tony + Claude
