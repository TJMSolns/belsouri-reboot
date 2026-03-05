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
