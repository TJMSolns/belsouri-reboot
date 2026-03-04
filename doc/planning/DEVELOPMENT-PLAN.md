# Development Plan

## Current Status

**Phase 0: Foundation** -- COMPLETE

All foundational documents are in place:
- `HOW-WE-WORK.md` -- ceremony-based SDLC framework
- `CLAUDE.md` -- developer guidance and conventions
- `doc/` structure -- domain artifacts, governance, scenarios
- `ADR-001` -- technology stack confirmed (Rust + Tauri + Svelte + SQLite)
- `LESSONS-LEARNED.md` -- institutional memory

**Next: Phase 1 (Infrastructure + First Domain Discovery)**

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

### Phase 1: Infrastructure + First Domain Discovery

**Runs as two parallel tracks.**

#### Track A: Infrastructure Vertical Slice

Prove the entire toolchain works end-to-end with one command flowing through all layers.

- [ ] Tauri project scaffolding with tauri-specta
- [ ] Svelte frontend skeleton
- [ ] SQLite event store (WAL mode, append-only events)
- [ ] SQLite projection store (WAL mode, incremental projection)
- [ ] One end-to-end command: UI -> Tauri invoke -> Rust command -> event store -> projection -> query -> UI display
- [ ] License enforcement: license check on startup, 30-day offline grace period, license key validation
- [ ] CI: backend tests (`cargo test`), frontend checks (`pnpm check`, `pnpm lint`), clippy

#### Track B: Practice Setup Domain Discovery

Phase 1 ceremonies for the Practice Setup bounded context (per HOW-WE-WORK.md):

- [ ] 1.1 Event Storming (`doc/domain/event-storming/practice-setup-events.md`)
- [ ] 1.2 Ubiquitous Language (`doc/domain/ubiquitous-language.md`)
- [ ] 1.3 Domain Modeling -- aggregate docs (`doc/domain/aggregates/`)
- [ ] 1.4 Context Mapping (`doc/domain/context-maps/context-map.md`)
- [ ] 1.5 Governance Review

### Phase 2: Practice Setup Implementation

Implement the Practice Setup bounded context with TDD through the Tauri invoke layer.

- [ ] Phase 2 ceremonies (Three Amigos, Example Mapping, BDD Scenarios)
- [ ] Offices: create, configure hours, chairs
- [ ] Providers: register, assign to offices, define specialties
- [ ] Procedure types: define procedures offered by the practice
- [ ] Full TDD: failing test -> implement -> refactor, tested via Tauri commands

### Phase 3: Staff Scheduling

- [ ] Phase 1 ceremonies (event storming, ubiquitous language, domain modeling)
- [ ] Phase 2 ceremonies (example mapping, BDD scenarios)
- [ ] Provider availability patterns (weekly schedules, recurring blocks)
- [ ] Exceptions (holidays, time off, overrides)
- [ ] Office-provider schedule view

### Phase 4: Patient Management

- [ ] Phase 1 ceremonies (event storming, ubiquitous language, domain modeling)
- [ ] Phase 2 ceremonies (example mapping, BDD scenarios)
- [ ] Patient registration and demographics
- [ ] Patient search and lookup
- [ ] Contact information management

### Phase 5: Patient Scheduling

- [ ] Phase 2 ceremonies (example mapping, BDD scenarios -- context established by Phases 3 & 4)
- [ ] Appointment booking against available slots
- [ ] Today's schedule view
- [ ] Appointment management (reschedule, cancel)

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
