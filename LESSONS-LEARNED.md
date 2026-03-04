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
