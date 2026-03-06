# Lessons Learned

Living document. Updated continuously as we work.

---

### 2026-03-02: SBPF Analysis and Tech Stack Confirmation

**What happened**: Reviewed 12 Caribbean/Jamaica SBPF reference files covering hardware, infrastructure, resilience, compliance, deployment, and cross-platform strategy. The `Cross-Platform-Desktop-Development-Strategies.md` SBPF rates Java/Scala+JavaFX at 5 stars and Tauri at 3 stars for Caribbean context. This prompted a serious re-evaluation of the Rust+Tauri+Svelte decision.

**What we learned**:

1. The SBPF's concerns about Tauri ("newer technology risk", "WebView dependencies") have largely aged out. Tauri 2.x is stable, WebView2 is pre-installed on Windows 10/11.

2. Tauri's advantages are actually *more* important for Caribbean than the SBPF recognized: 5-15MB binary (vs 60-80MB GraalVM) matters for USB distribution and cellular data costs. 30-80MB RAM (vs 200-400MB JVM) matters on 4GB machines running other practice software.

3. The belsouri-old codebase failures had two causes: (a) agent discipline -- not testing, not running the app, O(n²) projection rebuilds, (b) front/back type boundary -- 77 manually synced DTOs that drifted silently. Category (a) is language-independent. Category (b) is now solvable with `tauri-specta`.

4. Tony's original reasons for abandoning Scala+JavaFX (JavaFX aging, JVM weight on target hardware, no desktop Scala experience) are still valid.

5. Caribbean practices deal with daily power outages, 4GB RAM, 8+ year old CPUs, 1-10 Mbps bandwidth, tropical heat, and voltage instability. Every architectural decision must account for these realities.

**What we'll do differently**: Use `tauri-specta` from day one (no manual DTO sync). Keep Svelte thin (rendering only). Incremental projections always (never full rebuild). Auto-save everything. Design for 4GB, test on 4GB.

**Governance**: ADR-001 formalizes this decision.

### 2026-03-02: belsouri-old Post-Mortem Summary

**What happened**: The previous Rust+Tauri+Svelte codebase had 83 Rust files, 77 Tauri commands, 177 tests, and 31 Svelte components. Despite passing tests, the application was broken when a user actually tried it.

**Root causes** (from `SBPF/archive/LESSONS-LEARNED-20260218.md`):
- Tests called internal functions instead of testing through the Tauri invoke layer
- Only 15 of 77 commands had `rename_all = "snake_case"` -- the rest broke at runtime
- Every write command rebuilt the entire projection from scratch (O(n²))
- TypeScript interfaces drifted from Rust structs -- "undefined undefined" patient names
- No loading states, no success feedback, no useful error messages
- Agent claimed features worked without running the app

**What we'll do differently**: TDD through Tauri invoke (not internal functions). `tauri-specta` for all type generation. Incremental projections. Run `pnpm tauri dev` and verify end-to-end before claiming anything works. See `SBPF/archive/LESSONS-LEARNED-20260218.md` for the full detailed post-mortem.

---

### 2026-03-04: rename_all = "snake_case" Silently Drops All Multi-Word Parameters

**What happened**: Every Tauri command was decorated with `#[tauri::command(rename_all = "snake_case")]`. This was written into CLAUDE.md as a "critical convention" based on belsouri-old experience, where the frontend was hand-written in snake_case. With tauri-specta (which generates camelCase INVOKE calls), this override tells Tauri to expect `address_line_1` from JavaScript while tauri-specta sends `addressLine1`. Tauri silently drops mismatched keys as `None`. Single-word params (`name`, `phone`) looked the same in both conventions and appeared to work. Multi-word params (`address_line_1`, `city_town`, `chair_count`, `office_id`, `day_of_week`) were silently null in every event and projection. The DB confirmed: 3 saves of practice details, all with `address_line_1: null`. Backend tests passed because they call Rust functions directly, never touching the IPC serialization layer.

**What we learned**: Tauri v2 default behavior (no `rename_all`) automatically converts JavaScript camelCase → Rust snake_case. `tauri-specta` is designed for this default. Adding `rename_all = "snake_case"` overrides the default and creates a silent mismatch. Wrong guidance in CLAUDE.md was blindly followed across 65 commands. The backend test suite has zero coverage of the IPC serialization boundary — it only tests Rust functions in isolation.

**What we'll do differently**:
1. Never use `rename_all` on `#[tauri::command]` — documented in CLAUDE.md, ADR-001, done-checker.
2. `pnpm check:ipc` runs before every claim of done — mechanically catches `rename_all` re-emergence and snake_case keys in INVOKE calls.
3. For any command that writes data: query the database directly to verify fields persisted before claiming the feature works.
4. CI enforces `pnpm check:ipc` on every push.

**Governance**: ADR-001 updated. `pnpm check:ipc` added to CI and done-checker. `scripts/check-ipc.mjs` is the enforcement mechanism.

---

### 2026-03-05: Parallel Agent Workflow, Watchdog Skills, and Documentation Drift

**What happened**: Sprint delivered SCH-4 (provider capability), SCH-5 (shift planning), SCH-7 (setup checklist), CI pipelines, and SCH-4b ceremonies — five parallel workstreams. Then a watchdog pass (ux-review + copy-check + icon-audit) ran against all changed Svelte files. Root-level MD files (README, STATUS, TODO) were found to be severely out of date — README was still the Tauri scaffold template, STATUS referenced 102 tests and belsouri-old phase names, TODO still contained pre-development ceremony scaffolding from 2026-02-06.

**What we learned**:

1. **Parallel background agents in isolated git worktrees work at scale.** Five independent agents ran simultaneously (SCH-4, SCH-5, SCH-7, CI, SCH-4b ceremonies), each in its own worktree. All five merged to main with zero conflicts. Prerequisites for clean merges: each agent must own distinct files, no agent edits `lib.rs` or `bindings.ts` while another does, and worktrees are branched from the same HEAD commit.

2. **Watchdog skills catch real issues that code review misses.** In the Phase 3 polish pass, ux-review flagged touch targets below 44px on mobile breakpoints, copy-check found raw Rust error strings surfaced directly to users (violates POL-003), icon-audit found hover-only affordances that are invisible on touch screens, and missing loading guards on async actions. None of these are caught by `pnpm check` or `cargo test`. The watchdog pass is not optional ceremony — it finds genuine UX defects.

3. **Root MD files drift without an explicit maintenance step.** README, STATUS, and TODO do not self-update. They drifted across the entire Track A + MVP build period (roughly 3 weeks) because no process required updating them. Fix: add "update root MD files" as a line item in the post-sprint checklist, same weight as running `pnpm check`.

4. **`bindings.ts` must be updated manually when `pnpm tauri dev` cannot run** (e.g., display-less CI environments, or when adding commands in a background worktree). Pattern: find an existing command entry in `bindings.ts` that has a similar signature, copy its structure exactly, change the command name and parameter types, and verify it compiles with `pnpm check` before merging. Do not hand-write from scratch — match the tauri-specta output format precisely.

**What we'll do differently**:
- Post-sprint checklist includes: update STATUS.md test count, update TODO.md active items, scan README for stale content.
- Watchdog skills run in the main thread after every push (not as background agents — Skills require user-approved permissions).
- When spawning parallel agents, pre-assign file ownership to prevent merge conflicts on shared files.

**Governance**: No new ADR — these are process refinements, not architectural decisions.

---

### 2026-03-06: Watchdog Skills Validated With Concrete Evidence

**Date**: 2026-03-06
**Context**: STAFF-1/3/4 implementation session (staff hub improvements). Features were feature-complete, then ux-review, copy-check, and icon-audit ran on the changed file.

**What happened**: Watchdog skills flagged 11 violations across UX, copy, and icon specifications — touch target buttons at 32px (below 44px minimum), generic error messages, a calendar icon at 14px (not a valid size token), missing Space key on `role="button"` elements, and missing error feedback on async operations. All 11 were fixed before pushing.

**What we learned**: The watchdog skills catch real, user-facing defects that would have shipped without them. They enforce POL-001/002/003 compliance mechanically. The workflow is: run skills → review output → fix issues → `pnpm check` → push. Skipping the skills is not a time-saver; it defers real bugs.

**What we'll do differently**: Watchdog skills (ux-review, copy-check, icon-audit) run on every touched Svelte file before claiming done. All three must pass — same weight as `pnpm check`, same hard stop as failing tests.

**Governance**: Reinforces CLAUDE.md §Mandatory Post-Build Review. Already present — treat it as a hard stop, not optional polish.

---

### 2026-03-06: Error Toasts Must Name the Affected Entity

**Date**: 2026-03-06
**Context**: STAFF-4 shift grid implementation. Copy-check flagged the error toast in `loadMemberShifts()`.

**What happened**: The initial error read "Could not load shift schedule. Check your connection and try again." A user expands Maria's card to see her shifts and gets a generic error with no indication of whose shifts failed to load. Copy-check caught this as a POL-003 violation.

**What we learned**: Even in functions that operate on a single entity (function receives an ID), error messages must surface the human-readable name. This requires a one-line lookup (name from staff array by ID) inside the error handler. The extra line is always worth the clarity — a toast appears globally and loses context without the name.

**What we'll do differently**: When writing async functions that operate on a specific entity, look up and include the entity's name in all error toasts — not just the success path. Pattern: `const name = collection.find(x => x.id === id)?.name ?? "this item"` at the top of the function.

**Governance**: No governance change needed. Reinforces POL-003 §Error Message Standard (name the object, describe the problem, give a resolution path).

---

### 2026-03-06: DEVELOPMENT-PLAN.md Does Not Self-Update — Sync Is Manual

**Date**: 2026-03-06
**Context**: STAFF-1/3/4 session. Pre-implementation review of the plan listed SCH-1/2/3 as pending work.

**What happened**: Reading the code revealed SCH-1 (Reschedule UI), SCH-2 (hide unscheduled providers), and SCH-3 (two-aggregate UX hints) were already implemented in previous sessions. DEVELOPMENT-PLAN.md still listed them as pending because no step in the post-ship process updated the document. The gap was weeks old.

**What we learned**: DEVELOPMENT-PLAN.md drifts from reality without an explicit sync mechanism. Features ship, but the plan is not updated. This compounds across sessions — new work sessions start from stale information and may re-plan already-complete features.

**What we'll do differently**: After every feature push, scan DEVELOPMENT-PLAN.md for the shipped items and mark them complete with the commit reference. Add "sync DEVELOPMENT-PLAN.md" to the post-sprint checklist at the same level as updating LESSONS-LEARNED.md.

**Governance**: Process item — no ADR needed. Candidate for a lightweight `scripts/plan-sync-check.mjs` that cross-references git log against plan items and warns if the document is stale.
