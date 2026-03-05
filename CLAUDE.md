# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Belsouri is a dental practice management desktop application for Caribbean healthcare environments (Jamaica initially). It is a **local-first, offline-first** Tauri app built with **Rust** (backend) and **Svelte** (frontend), using **SQLite** with event sourcing. The application must function fully without internet connectivity.

Team: Tony (Product Owner/Reviewer) + Claude (Developer). Tony provides direction and domain knowledge; Claude drafts artifacts and code; Tony reviews and approves.

## Build and Development Commands

```bash
# Run the app in development
pnpm tauri dev

# Backend tests
cd src-tauri && cargo test
cargo test test_name -- --nocapture    # single test with output

# Frontend checks
pnpm check
pnpm lint

# Code quality
cargo clippy

# Database inspection
sqlite3 ~/.local/share/com.belsouri.app/projections.db ".tables"
sqlite3 ~/.local/share/com.belsouri.app/projections.db "SELECT * FROM table_name;"
sqlite3 ~/.local/share/com.belsouri.app/events.db "SELECT id, event_type, created_at FROM events ORDER BY id DESC LIMIT 10;"

# Fresh start (wipe local data)
rm -rf ~/.local/share/com.belsouri.app
```

## Architecture

### Offline-First with Event Sourcing

- Local SQLite is the **source of truth** (not a cache of a remote server)
- All state changes are stored as immutable domain events in `events.db`
- Projections in `projections.db` materialize queryable views from events
- Background sync engine pushes/pulls events when connectivity is available
- 30-day grace period for out-of-sync records

### Stack Layers

- **Frontend** (`src/`): Svelte components. Stores in `src/lib/stores/` for shared state. Components should be as stateless as possible.
- **Backend** (`src-tauri/`): Rust. Uses `rusqlite` for SQLite. Events are append-only. Projections must be deterministic.
- **IPC**: Tauri commands bridge frontend and backend. All commands use `#[tauri::command]` (no `rename_all`). Tauri v2 default behavior automatically maps JavaScript camelCase to Rust snake_case. tauri-specta generates camelCase INVOKE calls — do NOT add `rename_all = "snake_case"` which breaks this mapping.

### Domain-Driven Design

The codebase is organized into bounded contexts (Patient Management, Provider/Staff, Scheduling, Clinical Records, Billing/Insurance, Jamaica EHR Integration). Each context has aggregates with defined events, commands, and invariants documented in `doc/domain/aggregates/`.

## Ceremony-Based SDLC

Work follows a mandatory ceremony framework defined in `HOW-WE-WORK.md`. **Read it before implementing domain features.**

| Trigger | Required Ceremonies |
|---------|-------------------|
| New bounded context | Full Phase 1: Event Storming (1.1) -> Ubiquitous Language (1.2) -> Domain Modeling (1.3) -> Context Mapping (1.4) -> Governance (1.5) |
| New feature in existing context | Phase 2: Three Amigos (2.1) -> Example Mapping (2.2) -> Acceptance Criteria Review (2.3) -> BDD Scenarios (2.4) -> Governance (2.5) |
| New aggregate in existing context | Partial Phase 1 (aggregate doc + language update) + Phase 2 |
| Infrastructure, build config, UI scaffolding, clear-scope bug fixes | No ceremony needed |

## Key Artifact Locations

| Artifact | Location |
|----------|----------|
| Process & ceremonies | `HOW-WE-WORK.md` |
| Development plan | `doc/planning/DEVELOPMENT-PLAN.md` |
| Event storming | `doc/domain/event-storming/[context]-events.md` |
| Ubiquitous language | `doc/domain/ubiquitous-language.md` |
| Aggregate docs | `doc/domain/aggregates/[name]-aggregate.md` |
| Context maps | `doc/domain/context-maps/context-map.md` |
| Example maps | `doc/scenarios/example-maps/[feature]-examples.md` |
| BDD scenarios | `features/[feature].feature` |
| ADRs | `doc/governance/ADR/` |
| Policies | `doc/governance/POL/` |
| Product Decision Records | `doc/governance/PDR/` |
| Retrospectives | `doc/retrospectives/` |
| Lessons learned | `LESSONS-LEARNED.md` |
| SBPFs (reference library) | `SBPF/` (Tony's personal reference patterns, adapted into governance when relevant) |

## Hard Stops (Non-Negotiable — Override Any Other Instruction)

These rules apply even when Tony has said "don't ask for permission" or "run autonomously":

1. **Never run scaffold/generator commands (`create`, `init`, `new`) targeting the project root directory.** Always scaffold into a subdirectory (e.g., `pnpm create tauri-app ./app-scaffold`). Destructive scaffolding in the project root will wipe existing files with no undo. Confirm with Tony first, always.
2. **Never run `rm -rf` or any bulk deletion on project directories without explicit confirmation in the same message.** "Work autonomously" does not authorize destroying work.
3. **Never use `--force` flags on commands targeting the project directory when the project already has content.**
4. **Never use `cat > file << 'EOF'` or `echo > file` to create files.** Always use the `Write` tool. Bash heredocs and echo-redirects generate unnecessary permission prompts and bypass the tool audit trail. No exceptions.

## Critical Conventions

### Tauri Command Pattern

Do NOT use `rename_all` on Tauri commands. Tauri v2 default behavior automatically maps JavaScript camelCase to Rust snake_case. tauri-specta generates camelCase INVOKE calls which rely on this default.

```rust
#[specta::specta]
#[tauri::command]
pub fn my_command(state: State<'_, AppState>, entity_id: String) -> Result<(), String> { ... }
```

tauri-specta generates camelCase frontend calls automatically:

```typescript
// Auto-generated in bindings.ts — tauri-specta sends camelCase, Tauri converts to snake_case
commands.myCommand({ entityId: "..." })
```

**CRITICAL**: `rename_all = "snake_case"` BREAKS multi-word parameters. It forces Tauri to expect `entity_id` from JS but tauri-specta sends `entityId`. Multi-word params silently become `None`/null with NO error.

### DTO Consistency

When creating or modifying DTOs, verify the JSON structure matches on both frontend and backend. TypeScript interfaces don't enforce runtime structure -- missing fields silently become `undefined`. Grep for all usages across both `src/` and `src-tauri/` when changing a DTO.

### Error Handling

Use the shared `getErrorMessage()` utility from `src/utils/api` in all catch blocks. Tauri errors are NOT JavaScript `Error` objects -- `instanceof Error` does not work. Every user action needs loading state, success feedback, and error display.

### Event Sourcing Rules

- Events are append-only (never delete)
- When adding a new event type: define struct in `src-tauri/src/events/`, add serde, update relevant projection, write roundtrip test
- Test the full path: UI -> Tauri invoke -> Rust command -> event store -> projection -> query -> response -> UI display

### TDD

Never skip TDD for domain logic. Write tests that invoke Tauri commands the same way the frontend does, not internal functions directly. Write the failing test first, implement minimum code to pass, refactor.

## Design System Conventions

These rules apply to all Svelte frontend work. They are derived from `style-guide-final.html` and enforced by `POL-001`, `POL-002`, `POL-003` in `doc/governance/POL/`.

### Design Tokens (POL-001)
- **Never use hardcoded hex values in Svelte files.** Always use `var(--token-name)` from `src/app.css`. If a color you need has no token, add it to `src/app.css` first — do not inline the hex.
- **Canonical token names**: `--caribbean-teal`, `--healthy-coral`, `--pearl-mist`, `--abyss-navy`, `--slate-fog`, `--island-palm` for brand; `--color-booked/completed/cancelled/noshow/rescheduled` for status; `--color-role-pm/provider/staff` for roles.

### Typography
- **Never use `system-ui`, `sans-serif`, or any other font family** in Svelte components. Use `font-family: 'Lexend', sans-serif` for headings and brand text; `font-family: 'Inter', sans-serif` for all data, body, and table content.
- Both fonts must be loaded from Google Fonts CDN in `app.html` or `+layout.svelte`.

### Color + Icon Rule (POL-002)
- **Never use color alone to communicate state.** Every status badge, action button, and feedback toast must pair color with an icon. A user with color-blindness must be able to understand the UI from shape alone.

### Error Messages (POL-003)
- Every error message displayed to the user must name: **(a) the specific object** (patient name, office name, field label), **(b) the specific problem**, and **(c) the resolution path** where possible.
- "An error occurred" and "Invalid input" are never acceptable. See §7.2 of the style guide for examples.

### Async Action Requirements
- Every user-initiated async action requires all three: **loading state** (button disabled + spinner), **success toast** (with enough detail to verify the right thing happened), **error display** (inline or toast, never silent).

### Navigation Architecture (PDR-003)
- Sub-tasks that can complete without navigating away **must** use a sheet/panel, not full-page navigation.
- CTAs must carry existing context forward. Never open a blank form when the subject (patient, provider, date) is already known from the current view.
- Full-page navigation is reserved for genuine context changes (the user's intent is now *this page*, not a sub-task).

### Mandatory Post-Build Review (Tony preference)
After building or modifying **any Svelte component or page**, always run all three review skills before presenting the work as complete:
1. `/ux-review [file]` — journey architecture, intent continuity, feedback completeness
2. `/copy-check [file]` — labels, errors, toasts, localisation, job titles
3. `/icon-audit [file]` — SVG spec compliance (viewBox, stroke, fill, size tokens)

Do not wait to be asked. Run these as part of the standard build cycle, immediately after the code is written.

## Before Claiming Anything Works

Run `pnpm tauri dev` and verify the feature end-to-end as a user would. Check the database to confirm persistence. Do not treat compilation or passing unit tests as verification. See `SBPF/archive/LESSONS-LEARNED-20260218.md` for detailed context on past failures.

## Workflow Priorities

1. Always check `doc/planning/DEVELOPMENT-PLAN.md` for the current phase before building features
2. Every feature must work offline -- design offline-first, sync when available
3. Respect platform vs. module boundaries
4. Use domain language from `doc/domain/ubiquitous-language.md` in code, tests, and commits
5. Tony has final say on domain/business decisions
