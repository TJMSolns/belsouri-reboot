# ADR-001: Technology Stack -- Rust + Tauri + Svelte + SQLite

**Status**: Accepted
**Date**: 2026-03-02

## Context

Belsouri is a dental practice management application targeting Caribbean healthcare environments (Jamaica initially). The target market imposes severe constraints:

**Hardware**: Windows 10 dominant (80%+). 4GB RAM common, 8GB typical. CPUs 8+ years old. Single monitors, some at 1024x768. Some older CPUs lack AES-NI hardware encryption.

**Infrastructure**: Daily power outages. Voltage fluctuates 85V-140V. Internet 1-10 Mbps and unreliable -- some practices have none. Cellular (Digicel/Flow) or satellite may be only connectivity. USB distribution needed for remote practices.

**Regulatory**: Jamaica Ministry of Health and Wellness requires event-sourced audit trails, 7-year data retention, ICD-10-CM/CPT/CDT coding standards.

**Team**: Tony (Product Owner, C programmer background, Java experience, FP preference) reviews all code. Claude writes implementation. Code must be reviewable.

A previous attempt used Scala 3 + JavaFX but was abandoned due to concerns about JavaFX aging, JVM memory weight on target hardware, and desktop deployment complexity (Tony's Scala experience is server-side/REST). A subsequent Rust + Tauri + Svelte attempt (belsouri-old) produced working but buggy software -- failures stemmed from agent discipline problems AND front/back type boundary friction (manually synchronized Rust/TypeScript DTOs that drifted silently).

## Decision

**Rust + Tauri 2.x + Svelte + SQLite**, with the following mandatory constraints:

1. **`tauri-specta`** for all Tauri commands -- auto-generates TypeScript bindings from Rust type signatures. No manual DTO synchronization. Build fails if types mismatch.

2. **No `rename_all` on `#[tauri::command]`** -- Tauri v2 default behavior automatically maps JavaScript camelCase → Rust snake_case. `tauri-specta` generates camelCase INVOKE calls relying on this default. Adding `rename_all = "snake_case"` overrides the default and silently drops all multi-word parameters (they become `None` with no error). Use `pnpm check:ipc` to enforce this mechanically.

3. **Thin Svelte frontend** -- Svelte is a rendering layer only. All business logic, validation, and state management lives in Rust. Minimal boundary surface area.

4. **Incremental projections** -- projections track position and apply new events only. Never rebuild an entire projection from scratch on every write.

5. **SQLite in WAL mode** -- write-ahead logging for crash resilience during power outages.

## Why Tauri

| Factor | Tauri + Rust | Scala 3 + JavaFX + GraalVM | Electron |
|--------|-------------|---------------------------|----------|
| Binary size | **~5-15 MB** | ~60-80 MB | ~300 MB+ |
| RAM at runtime | **~30-80 MB** | ~200-400 MB | ~200 MB+ |
| Startup time | <1s | <1s (native image) | 2-5s |
| Crash resilience | **Rust + SQLite WAL** | JVM/native varies | Node.js varies |
| Windows 10 support | WebView2 pre-installed | Yes | Yes |
| Memory safety | **Compile-time (Rust)** | GC-managed | GC-managed |
| Type boundary | **Auto-generated (tauri-specta)** | Shared (one language) | Shared (one language) |
| UI framework health | Svelte: very active | JavaFX: aging, community-maintained | Chromium: active |

On a 4GB machine (common in target market), Windows 10 uses ~1.5-2GB, leaving ~2-2.5GB for applications. Dental practices run other software alongside (imaging, billing, browser). Tauri's 30-80MB footprint takes 1.5-4% of available memory. A JVM app at 200-400MB takes 8-20%.

For distribution: 5-15MB downloads in seconds on 10 Mbps, under 3 minutes on 1 Mbps. Updates via cellular (Digicel/Flow) cost real money per MB -- 4-16x size difference matters for monthly updates.

## Alternatives Considered

### Scala 3 + JavaFX (SBPF recommendation)

The `Cross-Platform-Desktop-Development-Strategies.md` SBPF rates Java/Scala at 5 stars for Caribbean context. However:

- **JavaFX is aging** -- community-maintained (OpenJFX), limited new investment. The SBPF itself acknowledges "less modern than web technologies."
- **JVM memory on 4GB machines** -- 200-400MB is 8-20% of available RAM alongside other practice software.
- **GraalVM + JavaFX native image** is notoriously complex to configure and maintain (reflection config, JavaFX framework linking, 2-5 min builds per platform).
- **Previously abandoned** by this team for these exact reasons.
- **Tony's Scala experience is server-side** -- not desktop. JavaFX would be new territory.

Trade-off acknowledged: Scala 3 has superior FP ergonomics and eliminates the type boundary entirely (one language). But the type boundary is now addressable with tauri-specta, and the memory/size constraints are harder to solve.

### Electron + TypeScript

- Binary 300MB+, RAM 200MB+ baseline. Non-starter for 4GB Caribbean machines.
- Would eliminate type boundary (single language). But hardware constraints disqualify it.

### .NET MAUI

- Good Windows integration, reasonable performance. But C# (not FP), Microsoft ecosystem lock-in, macOS support immature.

## Consequences

**Benefits:**
- Smallest possible binary and memory footprint for target hardware
- Compile-time memory safety for healthcare data
- Crash resilience via Rust + SQLite WAL for daily power outages
- Auto-generated type bindings prevent the DTO drift that plagued belsouri-old
- Modern web-based UI (Svelte) with active ecosystem

**Costs:**
- Tony reviews Rust code he doesn't write -- mitigated by domain-level review (BDD scenarios, aggregate docs, test behavior) rather than line-by-line Rust syntax
- Two languages (Rust + TypeScript) -- mitigated by tauri-specta and thin frontend
- Smaller healthcare library ecosystem in Rust vs. Java -- mitigated by Rust's C FFI for existing C/C++ libraries when needed
- Rust is more verbose for CRUD than Scala -- accepted trade-off for runtime efficiency on target hardware
