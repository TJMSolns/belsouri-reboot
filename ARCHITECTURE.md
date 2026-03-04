# Architecture Overview

## Belsouri - Offline-First Dental Practice Platform

**Status**: See Technical Architecture
**Technology Stack**: Rust + Tauri + Svelte
**Last Updated**: 2026-02-06

---

## Documentation

For comprehensive architecture documentation, see:

- **[Technical Architecture](doc/internal/architecture/technical-architecture.md)** - Full system design including:
  - Target environment and hardware constraints
  - Module architecture (Outreach, Scheduling, Payment, Charting)
  - Event sourcing and data storage
  - Sync and licensing architecture
  - UI shell design
  - Performance targets
  - Repository structure

- **[ADR-002: Technology Stack](doc/internal/governance/ADR/ADR-002-technology-stack-rust-tauri.md)** - Decision record for Rust + Tauri + Svelte

---

## Quick Reference

### Technology Stack

| Layer | Technology |
|-------|------------|
| Backend | Rust |
| Frontend | Svelte (HTML/CSS/TypeScript) |
| Shell | Tauri |
| Database | SQLite |
| Build | Cargo + pnpm |

### Module Structure

```
src-tauri/src/
├── modules/
│   ├── platform/     # Always-present substrate (Contacts, Practice setup)
│   ├── outreach/     # Phase 0 - Recall campaigns, work queues
│   ├── scheduling/   # Phase 1 - Appointments, calendar
│   ├── payment/      # Phase 2 - Billing, ledger
│   └── charting/     # Phase 3 - Clinical documentation
├── store/            # Event store (SQLite)
├── projections/      # Materialized views
├── sync/             # Cloud synchronization
└── license/          # Entitlement management

src/
├── lib/
│   ├── components/   # Reusable UI components
│   ├── stores/       # Svelte state management
│   └── utils/        # Tauri API wrappers
└── routes/           # Page components
```

### Key Design Principles

1. **Client-first**: All business logic runs locally
2. **Offline-always**: 100% functional without network
3. **Event-sourced**: Append-only log, deterministic projections
4. **Module-composable**: Features contributed by licensed modules
5. **Resource-respectful**: <500MB working set, <3s startup
