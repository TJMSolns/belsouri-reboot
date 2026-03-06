# Project Status: Belsouri Dental Practice Management

**Last Updated**: 2026-03-06
**Current Phase**: Post-MVP Polish Sprint — DM-1 complete, next: REL-1.5 field audit

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
| Cargo tests passing | 172 |
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
| SCH-4b | Practice Manager soft-stop override for C1/C2 constraint violations |
| SCH-5 | Shift planning and weekly roster view |
| SCH-7 | Setup checklist onboarding panel |
| STAFF-1/3/4 | Staff hub sort/filter, shift grid, plan-shift link from roster |
| Demo data | Seed/archive sample records |
| Design System v2.0 | 45 CSS tokens, Lexend + Inter fonts, toast + confirm stores |
| REL-2.3 | Setup tab hex → CSS token cleanup (all 5 sub-tabs) |
| UX-2.7/2.8 | DD/MM/YYYY date format + aria labels on schedule page |
| DS-3.5 | Watchdog fix pass: schedule page (7 UX/copy/icon fixes) |
| Watchdog audit | Full ux-review + copy-check + icon-audit pass on all setup components |
| DM-1 | Provider IS A StaffMember — merged Provider aggregate into StaffMember; retired separate Practice Setup Provider; 172 tests, 0 TS errors |

---

## Current Work

**Post-MVP Polish — backlog order (DEVELOPMENT-PLAN.md)**

DM-1 complete. Next items:
1. REL-1.5 — Field audit: form validation + error specificity sweep
2. UX-2.2 — Loading spinners on all async actions
3. REL-2.4 — Error message specificity (POL-003 compliance)
4. REL-2.1 — Print call list

---

## Post-MVP Backlog

| Item | Description | Ceremony |
|------|-------------|----------|
| DM-1 | Staff/Provider domain model correction — merge Provider into StaffMember | DONE |
| REL-1.5 | Field audit — form validation + error specificity sweep | None |
| UX-2.2 | Loading spinners on all async actions | None |
| REL-2.4 | Error message specificity (POL-003 compliance) | None |
| REL-2.1 | Print call list | None |
| REL-1.2 + REL-1.3 | Windows installer + data persistence | None |
| REL-1.1 | License Server | Separate planning session |
| REL-2.2 | Backup instructions | None |
| UX-2.4 | Search debounce | None |
| DS-2.2 + DS-2.3 | Status/role badges with icons (POL-002) | None |
| UX-1.7 | Call list placement | None |
| SCH-6 | Role-based view switching | Phase 1 ceremonies required |
| PDR-004 | Outcome-based RBAC | Post-MVP concept only |
| Recall & Outreach | New bounded context | Full Phase 1 required |
| Payments | Future phase | — |
| Charting | Future phase | — |
| STATUS.md cadence | Update STATUS.md at end of every session | Process |

---

**Maintained By**: Tony + Claude
**Review Frequency**: End of each session (not each sprint — too infrequent)
