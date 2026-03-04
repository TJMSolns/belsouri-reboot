# Development Plan

## Current Status

**Phase 0: Foundation** -- COMPLETE

All foundational documents are in place:
- `HOW-WE-WORK.md` -- ceremony-based SDLC framework
- `CLAUDE.md` -- developer guidance and conventions
- `doc/` structure -- domain artifacts, governance, scenarios
- `ADR-001` -- technology stack confirmed (Rust + Tauri + Svelte + SQLite)
- `LESSONS-LEARNED.md` -- institutional memory

**Phase 1 Track B: All Domain Discovery** -- COMPLETE (all 6 contexts, all 5 ceremonies each)

**Phase 2: All Domain Implementations** -- COMPLETE (all 6 contexts, all 5 ceremonies each)

**Next: Track A -- Infrastructure Vertical Slice**

---

## MVP Scope

What a dentist needs on day one to run their practice:

| Context | What It Covers |
|---------|---------------|
| **Practice Setup** | Offices, providers, procedure types -- the static configuration of a practice |
| **Staff Scheduling** | Provider availability patterns, working hours, exceptions (holidays, time off) |
| **Patient Management** | Patient registration, demographics, search and lookup |
| **Patient Scheduling** | Appointment booking against available staff/office slots, today's schedule |

Clinical records, billing, insurance, and Jamaica EHR integration are post-MVP.

---

## Development Phases

### Phase 0: Foundation (COMPLETE)

- [x] HOW-WE-WORK.md ceremony framework
- [x] CLAUDE.md developer guidance
- [x] Document structure (`doc/domain/`, `doc/governance/`, `doc/scenarios/`)
- [x] ADR-001: Technology stack decision
- [x] LESSONS-LEARNED.md

### Phase 1: Infrastructure + Domain Discovery

**Runs as two parallel tracks.**

#### Track A: Infrastructure Vertical Slice (COMPLETE)

Prove the entire toolchain works end-to-end with one command flowing through all layers.

- [x] Tauri project scaffolding with tauri-specta
- [x] Svelte frontend skeleton
- [x] SQLite event store (WAL mode, append-only events)
- [x] SQLite projection store (WAL mode, incremental projection)
- [x] One end-to-end command: UI -> Tauri invoke -> Rust command -> event store -> projection -> query -> UI display
- [x] License enforcement: license check on startup, 30-day offline grace period, license key validation
- [x] CI: backend tests (`cargo test`), frontend checks (`pnpm check`, `pnpm lint`), clippy

#### Track B: Domain Discovery (ALL COMPLETE)

Phase 1 ceremonies for all bounded contexts (per HOW-WE-WORK.md):

**Practice Setup**

- [x] 1.1 Event Storming (`doc/domain/event-storming/practice-setup-events.md`)
- [x] 1.2 Ubiquitous Language (`doc/domain/ubiquitous-language.md`)
- [x] 1.3 Domain Modeling -- aggregate docs (`doc/domain/aggregates/`)
- [x] 1.4 Context Mapping (`doc/domain/context-maps/context-map.md`)
- [x] 1.5 Governance Review

**Licensing**

- [x] 1.1 Event Storming (`doc/domain/event-storming/licensing-events.md`)
- [x] 1.2 Ubiquitous Language (`doc/domain/ubiquitous-language.md`)
- [x] 1.3 Domain Modeling -- aggregate docs (`doc/domain/aggregates/`)
- [x] 1.4 Context Mapping (`doc/domain/context-maps/context-map.md`)
- [x] 1.5 Governance Review

**Staff Management**

- [x] 1.1 Event Storming (`doc/domain/event-storming/staff-management-events.md`)
- [x] 1.2 Ubiquitous Language (`doc/domain/ubiquitous-language.md`)
- [x] 1.3 Domain Modeling -- aggregate docs (`doc/domain/aggregates/`)
- [x] 1.4 Context Mapping (`doc/domain/context-maps/context-map.md`)
- [x] 1.5 Governance Review

**Staff Scheduling** (projection-first model confirmed)

- [x] 1.1 Event Storming (`doc/domain/event-storming/staff-scheduling-events.md`)
- [x] 1.2 Ubiquitous Language (`doc/domain/ubiquitous-language.md`)
- [x] 1.3 Domain Modeling -- aggregate docs (`doc/domain/aggregates/`)
- [x] 1.4 Context Mapping (`doc/domain/context-maps/context-map.md`)
- [x] 1.5 Governance Review

**Patient Management**

- [x] 1.1 Event Storming (`doc/domain/event-storming/patient-management-events.md`)
- [x] 1.2 Ubiquitous Language (`doc/domain/ubiquitous-language.md`)
- [x] 1.3 Domain Modeling -- aggregate docs (`doc/domain/aggregates/`)
- [x] 1.4 Context Mapping (`doc/domain/context-maps/context-map.md`)
- [x] 1.5 Governance Review

**Patient Scheduling**

- [x] 1.1 Event Storming (`doc/domain/event-storming/patient-scheduling-events.md`)
- [x] 1.2 Ubiquitous Language (`doc/domain/ubiquitous-language.md`)
- [x] 1.3 Domain Modeling -- aggregate docs (`doc/domain/aggregates/`)
- [x] 1.4 Context Mapping (`doc/domain/context-maps/context-map.md`)
- [x] 1.5 Governance Review

### Phase 2: Domain Implementation Ceremonies (ALL COMPLETE)

Phase 2 ceremonies (Three Amigos, Example Mapping, Acceptance Criteria Review, BDD Scenarios, Governance) for all bounded contexts:

**Licensing Phase 2**

- [x] 2.1 Three Amigos
- [x] 2.2 Example Mapping (`doc/scenarios/example-maps/licensing-examples.md`)
- [x] 2.3 Acceptance Criteria Review
- [x] 2.4 BDD Scenarios (`features/licensing.feature`)
- [x] 2.5 Governance Review

**Practice Setup Phase 2**

- [x] 2.1 Three Amigos
- [x] 2.2 Example Mapping (`doc/scenarios/example-maps/practice-setup-examples.md`)
- [x] 2.3 Acceptance Criteria Review
- [x] 2.4 BDD Scenarios (`features/practice-setup.feature`)
- [x] 2.5 Governance Review

**Staff Management Phase 2**

- [x] 2.1 Three Amigos
- [x] 2.2 Example Mapping (`doc/scenarios/example-maps/staff-management-examples.md`)
- [x] 2.3 Acceptance Criteria Review
- [x] 2.4 BDD Scenarios (`features/staff-management.feature`)
- [x] 2.5 Governance Review

**Staff Scheduling Phase 2**

- [x] 2.1 Three Amigos
- [x] 2.2 Example Mapping (`doc/scenarios/example-maps/staff-scheduling-examples.md`)
- [x] 2.3 Acceptance Criteria Review
- [x] 2.4 BDD Scenarios (`features/staff-scheduling.feature`)
- [x] 2.5 Governance Review

**Patient Management Phase 2**

- [x] 2.1 Three Amigos
- [x] 2.2 Example Mapping (`doc/scenarios/example-maps/patient-management-examples.md`)
- [x] 2.3 Acceptance Criteria Review
- [x] 2.4 BDD Scenarios (`features/patient-management.feature`)
- [x] 2.5 Governance Review

**Patient Scheduling Phase 2**

- [x] 2.1 Three Amigos
- [x] 2.2 Example Mapping (`doc/scenarios/example-maps/patient-scheduling-examples.md`)
- [x] 2.3 Acceptance Criteria Review
- [x] 2.4 BDD Scenarios (`features/patient-scheduling.feature`)
- [x] 2.5 Governance Review

### Phase 2: Domain Vertical Slices

**Practice Setup**

- [ ] Events, commands, projections (Rust backend)
- [ ] Tauri commands: create_office, list_offices, create_provider, list_providers, create_procedure_type, list_procedure_types
- [ ] Frontend: offices page, providers page, procedure types page
- [ ] End-to-end smoke test verified

**Patient Management**

- [ ] Events, commands, projections
- [ ] Tauri commands: register_patient, update_patient_demographics, search_patients, get_patient
- [ ] Frontend: patient registration form, patient search, patient detail
- [ ] End-to-end smoke test verified

**Staff Management**

- [ ] Events, commands, projections
- [ ] Tauri commands: create_staff_member, set_pin, change_pin, reset_pin, authenticate_staff
- [ ] Frontend: staff list, staff creation, PIN setup/change
- [ ] End-to-end smoke test verified

**Staff Scheduling**

- [ ] Events, commands, projections (projection-first model)
- [ ] Tauri commands: set_availability_pattern, add_exception, get_provider_schedule
- [ ] Frontend: provider schedule view
- [ ] End-to-end smoke test verified

**Patient Scheduling (Appointments)**

- [ ] Events, commands, projections (5 hard-stop constraints)
- [ ] Tauri commands: book_appointment, reschedule_appointment, cancel_appointment, get_schedule
- [ ] Frontend: appointment booking, daily schedule view
- [ ] End-to-end smoke test verified

### Post-MVP

Ordered by expected priority. Scope and sequencing TBD after MVP delivery.

- Clinical Records
- Billing/Insurance
- Jamaica EHR Integration
- Background sync engine
- Multi-practice support

---

## Dependency Map

```
Infrastructure ─┬─> Practice Setup ──> Staff Scheduling ─┬─> Patient Scheduling
                │                                         │
                └─> Patient Management ──────────────────┘
```

- **Practice Setup** requires infrastructure (event store, projections, Tauri commands working)
- **Staff Scheduling** requires Practice Setup (providers and offices must exist)
- **Patient Management** requires infrastructure (but not Practice Setup -- can run in parallel with Staff Scheduling)
- **Patient Scheduling** requires both Staff Scheduling (available slots) and Patient Management (patients to book)

---

## Process Improvement Backlog

Revisit weekly. These are ongoing explorations, not blocking items.

| Area | Description | Status |
|------|-------------|--------|
| Agent strategy | Custom agents for ceremony checking, governance enforcement, context sharing | Exploring |
| Token efficiency | Separating concerns across agent contexts to stay within token budgets | Exploring |
| Governance automation | Agents that validate Phase 1/2 artifacts are complete before implementation begins | Exploring |
| Lessons-learned agent | Automated capture of insights during development | Exploring |

---

## Decisions Log

Architectural and product decisions are recorded in:

- **ADRs**: `doc/governance/ADR/` -- architectural decisions (e.g., ADR-001: technology stack)
- **PDRs**: `doc/governance/PDR/` -- product decisions
- **Policies**: `doc/governance/POL/` -- standing policies

---

## Backlog / Deferred Decisions

- **ADR-003-license-server-hosting**: Tony to decide Fly.io vs AWS Lambda vs VPS. ADR-003 is drafted with Fly.io recommendation -- awaiting Tony's Accepted sign-off.
- **Staff Management SM-1 through SM-5**: Phase 2.3/2.5 staff-management-examples.md and staff-management.feature still carry ASSUMED markers for Tony's OQ confirmations. These were confirmed verbally by Tony in the 2026-03-04 session but the artifact wasn't updated before the --force disaster. To be fixed before Staff Management enters Track A implementation.
