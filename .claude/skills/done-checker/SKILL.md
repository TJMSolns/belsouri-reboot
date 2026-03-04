---
name: check-done
description: Runs the Definition of Done checklist before claiming any work is complete. MUST be invoked before saying "done" or "complete".
allowed-tools: Read, Glob, Grep, Bash
user-invocable: true
argument-hint: [description of work being completed]
---

# Definition of Done Checker

You verify that work meets the Definition of Done from HOW-WE-WORK.md (lines 406-413) before it can be claimed as complete.

## Definition of Done Checklist

From HOW-WE-WORK.md:

```
- [ ] All Phase 1 artifacts complete (for new contexts)
- [ ] All Phase 2 artifacts complete (for features)
- [ ] Tests pass (`cargo test`, `pnpm check`)
- [ ] Code reviewed and approved
- [ ] Documentation updated if needed
- [ ] No new lint warnings
```

## Extended Checklist (from LESSONS-LEARNED)

Based on documented failures, also verify:

### Code Quality
- [ ] `cargo build` succeeds without errors
- [ ] `cargo test` passes (all tests)
- [ ] `cargo clippy` has no new warnings
- [ ] `pnpm check` passes (TypeScript)
- [ ] `pnpm build` succeeds

### Integration Verification
- [ ] Frontend can call backend commands successfully
- [ ] Data persists to database correctly
- [ ] Projections reflect current state

### User Experience
- [ ] Loading states exist for async operations
- [ ] Success feedback is shown to user
- [ ] Error messages are helpful (not generic)

### Contract Alignment
- [ ] Rust DTOs match TypeScript interfaces
- [ ] All `#[tauri::command]` have `rename_all = "snake_case"`
- [ ] Frontend sends snake_case parameters

## Your Process

1. Determine what type of work was completed (from work-classifier)
2. Run the appropriate checks
3. Report results with specific failures

## Commands to Run

```bash
# Backend checks
cd src-tauri && cargo build 2>&1
cd src-tauri && cargo test 2>&1
cd src-tauri && cargo clippy 2>&1

# Frontend checks
pnpm check 2>&1
pnpm build 2>&1
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

BUILD CHECKS:
- [x] cargo build: PASS
- [x] cargo test: PASS (X tests)
- [ ] cargo clippy: FAIL - 3 new warnings
- [x] pnpm check: PASS
- [x] pnpm build: PASS

CODE QUALITY:
- [x] No copy-paste error handling
- [x] Using getErrorMessage() utility
- [x] DTOs match across boundaries
- [x] Commands have rename_all = "snake_case"

USER EXPERIENCE:
- [ ] Loading states: MISSING in ProviderList.svelte
- [x] Success feedback: Present
- [x] Error messages: Helpful

OVERALL: [PASS / FAIL]

BLOCKERS (if FAIL):
1. Specific blocker with file:line reference
2. Another blocker

READY TO CLAIM DONE: [YES / NO]
```

## Important Rules

1. NEVER claim done without running this checklist
2. Build/test failures are BLOCKERS - fix before claiming done
3. "Works when I read the code" is NOT verification - run the checks
4. Integration issues (frontend can't call backend) are BLOCKERS
5. If any check fails, the work is NOT DONE

## What Happens After

If PASS:
- Work can be committed
- User can be informed of completion

If FAIL:
- List specific failures
- Suggest fixes
- Do NOT claim completion until failures are resolved
