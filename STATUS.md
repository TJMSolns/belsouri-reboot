# Project Status: Belsouri Dental Practice Management

**Last Updated**: 2026-03-05
**Current Phase**: Post-MVP — SCH-4b in progress

---

## MVP Bounded Contexts

All 6 MVP bounded contexts are shipped and merged to main.

| Context | Status | Notes |
|---------|--------|-------|
| Licensing | COMPLETE | Machine-bound Ed25519, startup check, 90-day grace |
| Practice Setup | COMPLETE | Practice details, offices, providers, procedure types, setup checklist |
| Patient Management | COMPLETE | Register, demographics, search, archive/unarchive, notes |
| Staff Management | COMPLETE | PM role claim, register staff, roles, PIN auth, archive |
| Staff Scheduling | COMPLETE | Provider availability queries, office provider roster, shift planning |
| Appointments | COMPLETE | Book (7 constraints), reschedule, cancel, complete, no-show, schedule grid, call list |

---

## Metrics

| Metric | Value |
|--------|-------|
| Cargo tests passing | 171 |
| TypeScript errors | 0 |
| TypeScript warnings | 0 |
| Clippy warnings | 0 |

---

## CI

| Workflow | Trigger | Status |
|----------|---------|--------|
| `ci.yml` | Push / PR to main | Tests + check + clippy |
| `build.yml` | Tag push (`v*`) | Matrix release build (Windows, macOS, Linux) |

---

## Features Shipped (Post-MVP Sprint)

| Feature | Description |
|---------|-------------|
| SCH-4 | `required_provider_type` on procedure types; C7 capability check in booking |
| SCH-5 | Shift planning and weekly roster view |
| SCH-7 | Setup checklist onboarding panel |
| Demo data | Seed/archive sample records |

---

## Current Work

**SCH-4b — PM Booking Override**: Practice Manager soft-stop override for C1/C2 constraint violations. Ceremonies complete (Three Amigos, Example Mapping, BDD scenarios). Implementation starting.

---

## Post-MVP Backlog

| Item | Description |
|------|-------------|
| SCH-6 | Role-based view switching — Phase 1 ceremonies required before implementation |
| PDR-004 | Outcome-based RBAC — post-MVP concept captured, not scheduled |
| Recall & Outreach | New bounded context — full Phase 1 ceremonies required |
| Payments | Future phase |
| Charting | Future phase |

---

**Maintained By**: Tony + Claude
**Review Frequency**: End of each sprint
