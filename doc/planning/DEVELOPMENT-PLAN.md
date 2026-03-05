# Development Plan

## Current Status

**Phase 0: Foundation** -- COMPLETE

**Phase 1 Track B: All Domain Discovery** -- COMPLETE (all 6 contexts, all 5 ceremonies each)

**Phase 2: All Domain Implementations** -- COMPLETE (all 6 contexts, all 5 ceremonies each)

**Track A: Infrastructure + All 5 Domain Vertical Slices** -- COMPLETE (155 tests, 0 TS errors)

**Phase 3: Professional MVP Polish** -- IN PROGRESS

What shipped in the 2026-03-04 polish session:
- Full Belsouri Design System v2.0 in `src/app.css` (45 CSS tokens, global component classes)
- Lexend + Inter fonts, Pearl Mist background, Abyss Navy nav bar
- Toast notification system (`toast.ts` + `Toast.svelte`) replacing all inline success/error strings and OS `alert()` calls
- ConfirmDialog component (`confirm.ts` + `ConfirmDialog.svelte`) replacing all OS `confirm()` and `prompt()` calls
- Schedule page: full light-theme rewrite, booking form as right-side drawer, 12-hour time, Today indicator, Escape key
- All pages (patients, staff, setup, schedule) converted from dark/mismatched theme to Belsouri design system
- Staff page: OS confirm() calls replaced with confirm store for archive + PIN reset

**Next: Tier 1 + Tier 2 MVP polish (groomed 2026-03-04 — see REL-1, REL-2 sections)**

Agreed execution order:
1. REL-2.3 Setup sub-tabs styling
2. REL-1.5 Field audit
3. UX-2.7 DD/MM/YYYY dates
4. UX-2.2 Loading spinners
5. REL-2.4 Error message specificity
6. REL-2.1 Print call list
7. REL-1.2 + REL-1.3 Windows installer + data persistence
8. REL-1.1 License Server (separate planning session)
9. REL-2.2 Backup instructions
10. UX-2.4 Search debounce
11. UX-2.8 Aria labels
12. DS-2.2 + DS-2.3 Status/role badges with icons
13. UX-1.7 Call list placement

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

**Practice Setup** -- COMPLETE (65/65 tests passing, 0 TS errors)

- [x] Events, commands, projections (Rust backend)
- [x] 28 Tauri commands: offices, providers, procedure types, practice info
- [x] Frontend: /setup page with 4 tabs (Practice, Offices, Providers, Procedure Types)
- [x] End-to-end smoke test verified (11 projection tables live)

**Patient Management** -- COMPLETE (84/84 tests passing, 0 TS errors)

- [x] Events, commands, projections
- [x] 8 Tauri commands: register_patient (soft dupe warning), update demographics/contact, add_note, archive/unarchive, search, get
- [x] Frontend: /patients page — live search, register form, expandable cards with demographics/contact/notes/archive
- [x] End-to-end smoke test verified

**Staff Management** — COMPLETE

- [x] Events: StaffMemberRegistered, PracticeManagerClaimed, RoleAssigned, RoleRemoved, PINSet, PINChanged, PINReset, StaffMemberArchived, StaffMemberUnarchived
- [x] Commands: claim_practice_manager_role, register_staff_member, assign_role, remove_role, set_pin, change_pin, reset_pin, archive_staff_member, unarchive_staff_member, verify_staff_pin, list_staff_members, get_staff_member_dto, get_staff_setup_status
- [x] Projections: staff_members, staff_member_roles (incremental)
- [x] Frontend: /staff — bootstrap, staff cards, roles, PIN ops, archive/unarchive
- [x] 130 backend tests pass, 0 TS errors

**Staff Scheduling** — COMPLETE (155 backend tests passing, 0 TS errors)

- [x] Projection-first — no new aggregates or events; reads Practice Setup projection tables
- [x] Tauri commands: query_provider_availability (4-reason logic), get_office_provider_schedule (90-day window)
- [x] Frontend: provider roster bar on /schedule page showing who is working today + hours
- [x] 25 backend service tests (SS1–SS6 BDD coverage, all priority rules)

**Patient Scheduling (Appointments)** — COMPLETE

- [x] Events: AppointmentBooked, AppointmentRescheduled, AppointmentCancelled, AppointmentCompleted, AppointmentMarkedNoShow, AppointmentNoteAdded
- [x] Commands: book_appointment (5 hard-stop constraints), reschedule_appointment, cancel_appointment, complete_appointment, mark_appointment_no_show, add_appointment_note, get_schedule, get_appointment, get_tomorrows_call_list
- [x] Projections: appointment_list, appointment_notes (incremental, denormalized names)
- [x] Frontend: /schedule — date/office selector, daily schedule, book form, status actions, notes, call list
- [x] 130 backend tests pass, 0 TS errors

### Phase 3: Professional MVP Polish

All vertical slices are functionally complete. The following items are **should-haves** before the
product is a professional MVP. Grouped by concern; no ceremony required (infrastructure/UI scope).

---

#### DS-1 · Design System Foundation

| # | Item | Notes |
|---|------|-------|
| ~~DS-1.1~~ | ~~`src/app.css` — CSS variable block~~ | **DONE** — Full design system stylesheet: 45 tokens (brand, status, role, spacing, typography, shadow, transition, layout), button/badge/card/drawer/spinner component classes, Google Fonts import, global reset, keyframes. Source: `style-guide-final.html` §6.4. |
| ~~DS-1.2~~ | ~~Load Lexend + Inter from Google Fonts~~ | **DONE** — `@import` in `src/app.css`. `--font-heading`/`--font-body` CSS vars defined. Global `body` and `h1–h6` rules set. |
| ~~DS-1.3~~ | ~~Pearl Mist (#F0F4F5) global background~~ | **DONE** — `body { background: var(--pearl-mist) }` in `app.css`. All five route pages converted from dark/inline backgrounds to Pearl Mist. |
| ~~DS-1.4~~ | ~~App shell / nav bar~~ | **DONE** — `+layout.svelte`: Abyss Navy bg, Caribbean Teal brand, Inter nav links, 3px bottom-border active indicator. Toast + ConfirmDialog wired into layout. |
| ~~DS-1.5~~ | ~~`prefers-reduced-motion` support~~ | **DONE** — `@media (prefers-reduced-motion: reduce)` in `app.css` zeroes all transition tokens and animation-duration. |

---

#### DS-2 · Component Library

| # | Item | Notes |
|---|------|-------|
| DS-2.1 | Button hierarchy | Primary: Caribbean Teal (#008B99). Destructive: Healthy Coral (#FF7F6A). Neutral/secondary: 1.5px border #C8D5D8. 44px minimum height. Lexend 600. All pages currently use wrong colors. |
| DS-2.2 | Status badges with icons | Always: icon + color + label (three signals, never color alone). Booked (teal), Completed (green), Cancelled (coral), No-Show (slate), Rescheduled (blue). Currently text-only. |
| DS-2.3 | Role badges with icons | Practice Manager (star/teal), Provider (chart/green), Staff (user/slate). Currently text-only. |
| ~~DS-2.4~~ | ~~Toast notification system~~ | **DONE** — `src/lib/stores/toast.ts` + `Toast.svelte`. Four types (success/error/info/warning), auto-dismiss (4s success, 6s error), slide-up animation, dismiss button. All OS `alert()` and inline success/error strings replaced on schedule + staff pages. Remaining: patients page inline errors not yet wired to toast. |
| ~~DS-2.5~~ | ~~Confirmation dialog component~~ | **DONE** — `src/lib/stores/confirm.ts` + `ConfirmDialog.svelte`. Promise-based `confirm({...})` API, destructive variant (Coral), optional required-text confirmation, Escape key, scale-in animation. All OS `confirm()` / `prompt()` replaced on schedule + staff pages. |
| DS-2.6 | Empty state component | Dashed border (#C8D5D8), Pearl Mist background, 40px hero icon (1.5px stroke), Lexend heading, Inter description, single primary CTA. |
| DS-2.7 | Form field component | Label always above (never as placeholder). Required marker (coral asterisk). Hint text below. Validate on blur, not keystroke. Preserve input on error — never clear on validation failure. Inline error with icon. |
| DS-2.8 | Focus rings everywhere | `outline: 3px solid #008B99; outline-offset: 2px` on every interactive element. `:focus-visible` only. Currently suppressed on most elements. |
| DS-2.9 | Input/select visual alignment | 1.5px border #C8D5D8, 8px radius, 10px/14px padding, focus ring. All pages currently inconsistent. |

---

#### DS-3 · Design System Compliance Audit (existing pages)

Now that the design system is active (`src/app.css`, `CLAUDE.md` rules, POL-001/002/003), the 5 existing route pages need a compliance pass before each is considered Phase 3-complete. Use the new skills: `/ux-review`, `/copy-check`, `/icon-audit`.

| # | Item | Notes |
|---|------|-------|
| DS-3.1 | Audit `+layout.svelte` (nav) | `/icon-audit` + `/copy-check`. Known violations: `#1a1a2e`, `#7eb8f7`, `system-ui`, no icons. Fix tracked as DS-1.4 above. |
| DS-3.2 | Audit `/setup` page and Setup tab components | `/ux-review` + `/copy-check` + `/icon-audit`. Known issues: hardcoded hex, inline styles, UX-4.1 sheet pattern needed. |
| DS-3.3 | Audit `/patients` page | `/ux-review` + `/copy-check` + `/icon-audit`. Check: error messages name specific object, search empty state has clear CTA, no `confirm()`/`alert()`. |
| DS-3.4 | Audit `/staff` page | `/ux-review` + `/copy-check` + `/icon-audit`. Check: PIN ops use confirmation dialog not OS `confirm()`, role badges have icons, error messages specific. |
| DS-3.5 | Audit `/schedule` page | `/ux-review` (booking drawer, call list sheet, OS dialogs). This page has the most UX-1 issues — `/ux-review` will produce the richest output. |

**How to run:** Open the page file in the IDE, then: `/ux-review src/routes/[page]/+page.svelte`, `/copy-check src/routes/[page]/+page.svelte`, `/icon-audit src/routes/[page]/+page.svelte`. Each skill returns a structured pass/flag/fail report.

---

#### UX-1 · Schedule Page Friction

| # | Item | Notes |
|---|------|-------|
| ~~UX-1.1~~ | ~~Booking form as right-side drawer~~ | **DONE** — Fixed right-side drawer (420px), overlay backdrop, grid stays visible behind it. Column click pre-fills provider + time. Only one drawer open at a time. |
| ~~UX-1.2~~ | ~~Replace OS `alert()` in action handlers~~ | **DONE** — `doComplete`, `doNoShow`, `doCancel` now use `toast.error()`. |
| ~~UX-1.3~~ | ~~Replace OS `confirm()` in doCancel/doArchive~~ | **DONE** — `doComplete` + `doNoShow` use confirm store. `doCancel` uses inline cancel-confirm form in detail drawer. Staff `doArchive` uses confirm store. |
| ~~UX-1.4~~ | ~~Replace OS `prompt()` for cancel reason~~ | **DONE** — Cancel reason is now an inline textarea inside the detail drawer's cancel-confirm section. No OS prompt. |
| UX-1.5 | After-action grid highlight | After booking/completing/cancelling: close the panel/drawer and visually highlight the affected appointment block in the grid for 2 seconds (CSS outline pulse). "Did it work?" answered visually. |
| ~~UX-1.6~~ | ~~"Today" indicator in date nav~~ | **DONE** — Caribbean Teal chip shows "Today" next to the date when viewing today. "Today" jump button appears when viewing any other date. |
| UX-1.7 | Call list as persistent bottom drawer | Currently a toggle that pushes content. Move to a bottom sheet or separate view so it doesn't disrupt the grid. |
| ~~UX-1.8~~ | ~~12-hour AM/PM time display~~ | **DONE** — All schedule times now use `minsTo12h()` / `formatTime()` returning "8:30 AM" format. Grid ticks, appointment blocks, detail drawer, call list. |
| ~~UX-1.9~~ | ~~Grid: Escape key dismisses detail panel~~ | **DONE** — `svelte:window onkeydown` handler: Escape closes booking drawer first (if open), then detail drawer. |
| UX-1.10 | Grid: keyboard navigation | Arrow keys navigate between cells/appointment blocks within the grid (WCAG 2.1 required for grid role). |

---

#### UX-2 · All-Page Friction

| # | Item | Notes |
|---|------|-------|
| ~~UX-2.1~~ | ~~Consistent page layout~~ | **DONE** — All five route pages: Pearl Mist background, Abyss Navy text, Lexend headings, Inter body, Caribbean Teal buttons, consistent padding (`var(--space-6)`), page-header pattern. |
| UX-2.2 | Loading spinners | Replace all "Loading…" text with spinner icon (`aria-busy="true"` on container). User should never wonder if their tap registered. |
| UX-2.3 | Focus return after modal/drawer close | When a drawer or dialog closes, focus must return to the element that opened it (booking button, appointment block, etc.). Currently focus goes to `body`. |
| UX-2.4 | Search: debounce + min-chars hint | Patient search currently fires on every keystroke. Debounce 250ms. Show "Type at least 2 characters" hint before firing. |
| UX-2.5 | Disabled button explanations | When a primary action is disabled (e.g., Book Appointment until patient selected), show a short inline hint why ("Select a patient first") rather than a silently-disabled button. |
| UX-2.6 | Error messages: specific not generic | Replace "Select a patient from the search results." style messages with context-specific, actionable copy per style guide voice standards. |
| UX-2.7 | DD/MM/YYYY date format | All dates shown to users should use Jamaican format. Backend stores ISO, frontend formats. |
| UX-2.8 | Aria labels on all icon-only controls | Nav buttons (‹ › « »), close buttons (✕), expand chevrons (▼ ▲) all need `aria-label`. Currently some have `title` only. |

---

#### UX-3 · Staff Page

| # | Item | Notes |
|---|------|-------|
| UX-3.1 | Provider schedule: loading state per provider | Currently `providerScheduleLoading` is shared — if you switch providers while loading, the state is ambiguous. Scope loading state to the expanded provider. |
| ~~UX-3.2~~ | ~~Staff page: replace `confirm()` in doArchive/doResetPin~~ | **DONE** — Both `doArchive` and `doResetPin` use confirm store with destructive variant. Success uses `toast.success()`. |
| UX-3.3 | PIN input: show/hide toggle | PIN fields are `type="password"`. Add an eye icon to toggle visibility. Standard pattern; reduces mis-entry. |

---

#### UX-4 · Setup Page

| # | Item | Notes |
|---|------|-------|
| UX-4.1 | Inline office hours editor | Currently: a form that replaces the office card. Should edit inline or in a sheet without leaving the page. |
| UX-4.2 | Provider availability: visual weekly grid | Provider availability is currently a list of text fields. A mini weekly grid (Mon–Sun × hours) would match the mental model of "who works when". |

---

---

#### REL-1 · Release Readiness (MVP Hard Blockers)

Items that must be complete before a real practice can use this as a paid product.
These emerged from the 2026-03-04 DoD analysis. None are in scope of any existing Phase 3 category.

| # | Item | Notes |
|---|------|-------|
| REL-1.1 | License Server implementation | The local licensing layer is complete (eval token, Ed25519 verification, startup check, degraded mode). The cloud backend (key issuance, renewal, practice registration) has not been built. ADR-003 is drafted with Fly.io recommendation — awaiting Tony's Accepted sign-off before implementation. **Hard blocker for paid licenses.** |
| REL-1.2 | Windows installer build + verification | `pnpm tauri build` produces an `.msi` or NSIS installer. Must be verified on a clean Windows machine: app launches, data dir created, no missing DLLs, no startup errors. Currently untested. |
| REL-1.3 | App data survives reinstall / update | Verify that installing a new version over an existing install does not wipe `events.db` or `projections.db`. Tauri's default data dir should persist, but this needs explicit verification with a test upgrade. |
| REL-1.4 | Installer signing or clear bypass instructions | Unsigned Windows installer triggers SmartScreen. Either sign with a code-signing cert, or document the click-through steps for practice staff. Both acceptable for MVP; signing preferred. |
| REL-1.5 | Data field audit post `rename_all` fix | The `rename_all = "snake_case"` bug silently dropped multi-word fields for an unknown period. A field-level audit of every Tauri command (spot-check: book appointment, update patient demographics, set office address) should confirm all fields persist correctly. Most were caught by the 2026-03-04 fix, but the audit should be formal. |

---

#### REL-2 · Operational Basics

Not hard blockers but required before a practice feels confident handing the app to staff.

| # | Item | Notes |
|---|------|-------|
| REL-2.1 | Print-friendly call list | Receptionist needs to print or share tomorrow's call list. Minimum viable: `window.print()` with a print stylesheet that hides nav and shows a clean table. Better: PDF export. At minimum, the call list table should be copy-pasteable into WhatsApp. |
| REL-2.2 | Basic backup instructions for practice | Document: "Your data lives in `%LOCALAPPDATA%\com.belsouri.app\`. Copy `events.db` to an external drive weekly." Include in the onboarding guide / README. `events.db` alone is sufficient to rebuild all state. |
| REL-2.3 | Setup sub-tabs design system compliance | `PracticeTab.svelte`, `OfficesTab.svelte`, `ProvidersTab.svelte`, `ProcedureTypesTab.svelte` still use old styles (hardcoded hex, `system-ui` font, dark-era button classes). The Setup tab shell was updated; the inner components were not. |
| REL-2.4 | Error message specificity audit | Some error paths still surface raw Rust error strings to the user (e.g., constraint violation text). All user-visible errors should be mapped to friendly, actionable copy per UX-2.6 and the style guide voice standards. |

---

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
| Design system skills | `/ux-review`, `/copy-check`, `/icon-audit` — validate UI against style guide | **DONE** — Live in `.claude/skills/`. Use during DS-3 audit and on all future Svelte work. |
| Design governance | POL-001 (tokens), POL-002 (icons), POL-003 (UX standards), PDR-002 (Android-first), PDR-003 (journey navigation) | **DONE** — In `doc/governance/POL/` and `doc/governance/PDR/`. Referenced by skills and `CLAUDE.md`. |

---

## Decisions Log

Architectural and product decisions are recorded in:

- **ADRs**: `doc/governance/ADR/` -- architectural decisions (e.g., ADR-001: technology stack)
- **PDRs**: `doc/governance/PDR/` -- product decisions
- **Policies**: `doc/governance/POL/` -- standing policies

---

## Backlog / Deferred Decisions

- **ADR-003-license-server-hosting**: Tony to decide Fly.io vs AWS Lambda vs VPS. ADR-003 is drafted with Fly.io recommendation -- awaiting Tony's Accepted sign-off. Tracked as REL-1.1 (hard blocker for paid licenses).
- **Staff Management SM-1 through SM-5**: Phase 2.3/2.5 staff-management-examples.md and staff-management.feature still carry ASSUMED markers for Tony's OQ confirmations. These were confirmed verbally by Tony in the 2026-03-04 session but the artifact wasn't updated before the --force disaster. To be fixed before Staff Management enters Track A implementation.
