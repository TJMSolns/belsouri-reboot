---
name: check-done
description: Runs the Definition of Done checklist before claiming any work is complete. MUST be invoked before saying "done" or "complete".
allowed-tools: Read, Glob, Grep, Bash
user-invocable: true
argument-hint: [description of work being completed]
---

# Definition of Done Checker

You verify that work meets the Definition of Done from HOW-WE-WORK.md before it can be claimed as complete.

## Definition of Done Checklist

```
- [ ] All Phase 1 artifacts complete (for new contexts)
- [ ] All Phase 2 artifacts complete (for features)
- [ ] Tests pass (`cargo test`, `pnpm check`)
- [ ] Code reviewed and approved
- [ ] Documentation updated if needed
- [ ] No new lint warnings
```

## Extended Checklist

### Code Quality
- [ ] `cargo build` succeeds without errors
- [ ] `cargo test` passes (all tests)
- [ ] `cargo clippy` has no new warnings
- [ ] `pnpm check:ipc` PASSES (IPC contract check — catches rename_all and snake_case key bugs)
- [ ] `pnpm check` passes (TypeScript)

### IPC Contract (MANDATORY — run `pnpm check:ipc`)
- [ ] No `rename_all = "snake_case"` on any `#[tauri::command]`
- [ ] No snake_case keys in any TAURI_INVOKE call in bindings.ts
- [ ] For every new command added: open bindings.ts, find the INVOKE call, verify all multi-word Rust params appear as camelCase keys (e.g. `address_line_1` → `addressLine1`)

### Database Verification (MANDATORY for any command that writes data)
Run these after exercising the feature end-to-end:
```bash
sqlite3 ~/.local/share/com.belsouri.app/events.db \
  "SELECT payload FROM events WHERE event_type = 'YourEventType' ORDER BY id DESC LIMIT 1;"
# Verify NO fields that the user entered are null

sqlite3 ~/.local/share/com.belsouri.app/projections.db \
  "SELECT * FROM your_table ORDER BY rowid DESC LIMIT 1;"
# Verify projection matches what was written
```

### Integration Verification
- [ ] Frontend can call backend commands successfully
- [ ] Data persists to database correctly — CHECKED IN DB, not assumed
- [ ] Projections reflect current state

### User Experience (READ EVERY AFFECTED SVELTE FILE)
- [ ] Every `<input>` has an associated `<label for="...">` with matching `id`
- [ ] Every expandable/collapsible element has `aria-expanded={bool}`
- [ ] Every search dropdown has a "no results" message
- [ ] Every empty state has a concrete next-action (not just "No X yet")
- [ ] Every async action has a loading state AND success feedback AND error display
- [ ] No silent failures — if an action requires preconditions (e.g. select an office first), tell the user
- [ ] First-run / fresh-install experience makes sense with zero data

### Contract Alignment
- [ ] Rust DTOs match TypeScript interfaces
- [ ] `#[tauri::command]` has NO `rename_all` attribute (Tauri v2 default handles camelCase→snake_case)
- [ ] bindings.ts INVOKE calls use camelCase keys (verified by `pnpm check:ipc`)

## Your Process

1. Run all checks below
2. For any command that writes data: query the database to verify data persisted
3. Report results with specific failures — never assume

## Commands to Run

```bash
# IPC contract check (MUST RUN FIRST — catches the worst silent failures)
pnpm check:ipc

# Backend checks
cd src-tauri && cargo build 2>&1
cd src-tauri && cargo test 2>&1
cd src-tauri && cargo clippy 2>&1

# Frontend checks
pnpm check 2>&1

# Database verification (for any write command)
sqlite3 ~/.local/share/com.belsouri.app/events.db "SELECT payload FROM events ORDER BY id DESC LIMIT 3;"
sqlite3 ~/.local/share/com.belsouri.app/projections.db ".tables"
```

## Output Format

```
DEFINITION OF DONE CHECK
========================

WORK COMPLETED: [description]
WORK TYPE: [NO_CEREMONY / NEW_CONTEXT / NEW_FEATURE / NEW_AGGREGATE]

CEREMONY ARTIFACTS:
- [x] Phase 1 complete (or N/A)
- [x] Phase 2 complete (or N/A)

IPC CONTRACT:
- [x] pnpm check:ipc: PASS
- [x] No rename_all on commands
- [x] bindings.ts INVOKE keys are camelCase

BUILD CHECKS:
- [x] cargo build: PASS
- [x] cargo test: PASS (X tests)
- [x] cargo clippy: PASS
- [x] pnpm check: PASS

DATABASE VERIFICATION:
- [x] Events written with all fields non-null: CONFIRMED
- [x] Projection updated correctly: CONFIRMED

USER EXPERIENCE:
- [x] Loading states: Present
- [x] Success feedback: Present
- [x] Error messages: Helpful

OVERALL: [PASS / FAIL]

BLOCKERS (if FAIL):
1. Specific blocker with file:line reference

READY TO CLAIM DONE: [YES / NO]
```

## Important Rules

1. NEVER claim done without running this checklist
2. `pnpm check:ipc` MUST pass — it is not optional
3. For write commands: QUERY THE DATABASE. "It compiled" is not verification.
4. Build/test failures are BLOCKERS - fix before claiming done
5. Integration issues (frontend can't call backend) are BLOCKERS
6. If any check fails, the work is NOT DONE
