# Belsouri Dental Practice Management

Belsouri is an offline-first desktop application for dental practice management in Caribbean healthcare environments (Jamaica initially). Built with Tauri v2 + Rust backend + SvelteKit/Svelte 5 frontend + SQLite event sourcing.

## Tech Stack

- **Desktop shell**: Tauri v2
- **Backend**: Rust, SQLite (rusqlite, bundled), event sourcing
- **Frontend**: SvelteKit + Svelte 5, TypeScript
- **IPC type safety**: tauri-specta (auto-generates `src/lib/bindings.ts`)
- **Architecture**: Local-first, offline-first — SQLite is the source of truth, not a cache

## Dev Commands

```bash
# Run the app (development)
pnpm tauri dev

# Backend tests
cd src-tauri && cargo test

# Single test with output
cd src-tauri && cargo test test_name -- --nocapture

# Frontend type check
pnpm check

# Lint
pnpm lint

# Clippy
cd src-tauri && cargo clippy
```

## Data Location

| Platform | Path |
|----------|------|
| Linux    | `~/.local/share/com.belsouri.app/` |
| Windows  | `%APPDATA%\com.belsouri.app\` |
| macOS    | `~/Library/Application Support/com.belsouri.app/` |

Two databases: `events.db` (append-only event log) and `projections.db` (materialized read views).

## Fresh Start (Wipe Local Data)

```bash
rm -rf ~/.local/share/com.belsouri.app
```

## Inspect the Database

```bash
sqlite3 ~/.local/share/com.belsouri.app/events.db \
  "SELECT id, event_type, created_at FROM events ORDER BY id DESC LIMIT 10;"

sqlite3 ~/.local/share/com.belsouri.app/projections.db ".tables"
```

## For AI Assistant Instructions

See `CLAUDE.md` — it contains architecture notes, hard stops, critical conventions, and the design system rules that govern all development work on this project.
