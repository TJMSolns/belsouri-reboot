# TODO: Path to Outreach MVP Release

## Ordered Steps from Current State to Phase 0 MVP

**Current Status**: Pre-Development (Architecture & Planning Complete)
**Target**: Outreach MVP deployed to 1-2 Jamaica pilot clinics
**Technology Stack**: Rust + Tauri + Svelte (see [technical-architecture.md](doc/internal/architecture/technical-architecture.md))
**Success Criteria**: At least one recovered appointment per clinic per month

**Key**: 👤 Tony (Technical Owner) | 👨‍⚕️ Nico (Business Owner/Clinical Advisor) | 🤝 Both

---

## 📋 Pre-Development Setup (Week 1-2)

### Design System Setup
- [x] 👨‍⚕️ **Nico**: Complete initial Figma Make prototype (DONE)
  - **Status**: COMPLETE - React + TypeScript + Tailwind prototype built in Figma Make
  - **Location**: https://www.figma.com/make/J2cmoGWDcU33EyfPnGWxVF/Patient-Follow-Up-App

- [ ] 👨‍⚕️ **Nico**: Export design specifications from Figma Make
  - For each view (Dashboard, Campaigns, Work Queue, Reports):
    - Export high-res screenshots (1920x1080, 1366x768)
    - Document user workflows step-by-step
    - Note interactive behaviors (button clicks, form validation, table sorting)
  - Extract design tokens from theme.css:
    - Copy color palette (primary, secondary, background, etc.)
    - List typography (font families, sizes, weights)
    - Document spacing scale (padding, margins, gaps)
  - Deliverable: Design handoff documents in `doc/internal/planning/design-handoffs/`

- [ ] 👤 **Tony**: Extract design tokens to CSS custom properties
  - Read Nico's theme.css from Figma Make
  - Create `src/app.css` with CSS custom properties
  - Reference: [figma-to-svelte-guide.md](doc/internal/reference/figma-to-svelte-guide.md)
  - Deliverable: Working CSS file with design tokens

### Development Environment Setup
- [ ] 👤 **Tony**: Set up Rust + Tauri development environment
  - Install Rust toolchain (rustup)
  - Install Node.js 18+ and pnpm
  - Install Tauri CLI: `cargo install tauri-cli`
  - Install VS Code extensions:
    - rust-analyzer (Rust language support)
    - Svelte for VS Code
    - Tauri (official extension)
  - Verify `cargo tauri dev` works with template project
  - Deliverable: Working development environment

- [ ] 👤 **Tony**: Initialize project structure
  - Run `cargo tauri init` to create project scaffold
  - Set up Svelte frontend in `src/`
  - Set up Rust backend in `src-tauri/`
  - Configure build settings in `tauri.conf.json`
  - Deliverable: Compiling project skeleton

### Pilot Clinic Identification
- [ ] 👨‍⚕️ **Nico**: Identify 1-2 pilot clinics in Jamaica
  - Target: Small practices (1-5 chairs), paper-based or legacy PMS
  - Requirements: Willing to test Outreach module, provide feedback
  - Preferably practices Nico has relationship with
  - Deliverable: Confirmed pilot clinic commitments

- [ ] 🤝 **Both**: Pilot clinic kickoff meeting
  - Explain Outreach module purpose and workflows
  - Set expectations: Phase 0 MVP (Outreach only, no scheduling/clinical)
  - Agree on success metrics (at least 1 recovered appointment/month)
  - Schedule weekly check-ins
  - Deliverable: Pilot agreement and communication plan

---

## 🔍 Phase 1: Discovery (DDD - Domain Modeling) (Week 3-4)

### Ceremony 1.1: Event Storming for Outreach Context
- [ ] 🤝 **Both**: Run Event Storming session (4-6 hours)
  - Follow [doc/internal/ceremonies/phase1/1.event-storming.md](doc/internal/ceremonies/phase1/1.event-storming.md)
  - Map temporal flow: Import patients → Configure rules → Run campaign → Track outcomes
  - Identify domain events: `PatientImported`, `RecallRuleConfigured`, `CampaignGenerated`, `ContactOutcomeRecorded`
  - Identify commands: `ImportPatients`, `ConfigureRecallRule`, `GenerateCampaign`, `MarkContactOutcome`
  - Identify aggregates: `Campaign`, `WorkQueue`, `RecallRule`, `PatientRecord`
  - Deliverable: `doc/internal/domain-models/event-storming/outreach-context-events.md`

### Ceremony 1.2: Ubiquitous Language Workshop
- [ ] 🤝 **Both**: Define shared vocabulary (2-3 hours)
  - Follow [doc/internal/ceremonies/phase1/1.ubiquitous-language.md](doc/internal/ceremonies/phase1/1.ubiquitous-language.md)
  - Extract terms from Event Storming: Campaign, Work Queue, Recall Rule, Contact Outcome, etc.
  - Nico provides clinical definitions (e.g., "Recall" vs "Reactivation" vs "Recare")
  - Tony translates to code-friendly names (Campaign, ContactOutcome enum)
  - Deliverable: `doc/internal/domain-models/ubiquitous-language.md`

### Ceremony 1.3: Domain Modeling for Outreach Aggregates
- [ ] 👤 **Tony** (lead) + 👨‍⚕️ **Nico** (validate): Model aggregates (4-6 hours)
  - Follow [doc/internal/ceremonies/phase1/1.domain-modeling.md](doc/internal/ceremonies/phase1/1.domain-modeling.md)
  - For each aggregate:
    - Define invariants (business rules that must always hold)
    - Define state machine (Mermaid diagram)
    - Define commands and events
    - Identify entities vs value objects
  - Nico validates: Do business rules match real clinic workflows?
  - Deliverable: Aggregate files in `doc/internal/domain-models/aggregates/`

### Ceremony 1.4: Context Mapping
- [ ] 👤 **Tony** (lead): Map bounded contexts (2-3 hours)
  - Follow [doc/internal/ceremonies/phase1/1.context-mapping.md](doc/internal/ceremonies/phase1/1.context-mapping.md)
  - Phase 0 map: Clinic (substrate) + Outreach (lead module)
  - Define relationship: Outreach uses Clinic (Published Language pattern)
  - Document: Event store, license validation, sync engine interfaces
  - Deliverable: `doc/internal/domain-models/context-maps/system-context-map.md`

---

## 📝 Phase 2: Specification (BDD - Behavior-Driven Development) (Week 5-6)

### Ceremony 2.1: Example Mapping for Outreach Workflows
- [ ] 👨‍⚕️ **Nico** (lead) + 👤 **Tony** (support): Map examples (1-2 hours per story)
  - Follow [doc/internal/ceremonies/phase2/2.example-mapping.md](doc/internal/ceremonies/phase2/2.example-mapping.md)
  - For each user story (Import Patients, Configure Rules, Run Campaign, Track Outcomes, Reports):
    - Extract business rules (e.g., "Patient due if 6 months since last cleaning")
    - Provide concrete examples (e.g., "John Smith, last cleaning Jan 1, due July 1")
    - Surface questions (e.g., "What if patient has no last cleaning date?")
  - Deliverable: Example maps in `doc/internal/scenarios/example-maps/`

### Ceremony 2.2: Three Amigos for BDD Scenarios
- [ ] 🤝 **Both**: Write Gherkin scenarios (2-3 hours per story)
  - Follow [doc/internal/ceremonies/phase2/2.three-amigos.md](doc/internal/ceremonies/phase2/2.three-amigos.md)
  - Convert example maps to Given/When/Then scenarios
  - Use ubiquitous language from glossary
  - Deliverable: Gherkin `.feature` files in `features/`

### Ceremony 2.3: Acceptance Criteria Review
- [ ] 👤 **Tony** (lead) + 👨‍⚕️ **Nico** (validate): Review scenarios (1-2 hours per story)
  - Follow [doc/internal/ceremonies/phase2/2.acceptance-criteria-review.md](doc/internal/ceremonies/phase2/2.acceptance-criteria-review.md)
  - Validate scenarios align with domain model
  - Validate ubiquitous language usage
  - Approve scenarios for implementation
  - Deliverable: Acceptance criteria docs in `doc/internal/scenarios/acceptance-criteria/`

---

## 💻 Phase 3: Implementation (TDD - Test-Driven Development) (Week 7-10)

### Setup Rust Project Structure
- [ ] 👤 **Tony**: Create Rust module structure
  ```
  src-tauri/
  ├── src/
  │   ├── main.rs
  │   ├── commands/          # Tauri command handlers
  │   ├── events/            # Event definitions
  │   ├── store/             # Event store, SQLite
  │   ├── projections/       # Projection builders
  │   ├── modules/
  │   │   ├── platform/      # Base platform
  │   │   └── outreach/      # Outreach module
  │   ├── sync/              # Sync engine
  │   ├── license/           # License management
  │   └── crypto/            # Encryption, signing
  │   └── lib.rs
  ├── Cargo.toml
  └── tauri.conf.json
  ```
  - Add dependencies to Cargo.toml:
    - `rusqlite` - SQLite interface
    - `serde` / `serde_json` - Serialization
    - `tokio` - Async runtime
    - `uuid` - Unique identifiers
    - `chrono` - Date/time handling
  - Deliverable: Compiling project with module structure

### Implement Domain Layer (TDD)
- [ ] 👤 **Tony**: Implement aggregates with TDD (2-3 weeks)
  - Follow [doc/internal/ceremonies/phase3/3.test-first-pairing.md](doc/internal/ceremonies/phase3/3.test-first-pairing.md)
  - For each aggregate (Campaign, WorkQueue, RecallRule, PatientRecord):
    - Write failing unit test for invariant
    - Implement minimum code to pass test
    - Refactor while maintaining passing tests
  - Use `proptest` crate for property-based testing
  - Target: ≥90% coverage for domain aggregates
  - Deliverable: Pure domain model in `src-tauri/src/modules/outreach/domain/`

### Implement Infrastructure Layer
- [ ] 👤 **Tony**: Build persistence, sync, licensing (1-2 weeks)
  - Event store with SQLite (append-only, deterministic projections)
  - License validation (30-60 day offline grace period)
  - Opportunistic sync engine (conflict-free eventual consistency)
  - Encrypted storage (AES-256 for patient data)
  - Deliverable: Working infrastructure in `src-tauri/src/`

### Implement Svelte Frontend
- [ ] 👤 **Tony**: Build desktop UI from Figma designs (2-3 weeks)
  - Reference: [figma-to-svelte-guide.md](doc/internal/reference/figma-to-svelte-guide.md)
  - Create component library (`src/lib/components/`)
  - Implement views:
    - Dashboard
    - Campaign list and detail
    - Work queue
    - Patient import
    - Reports
  - Implement offline-first patterns (sync status, local save indicators)
  - Deliverable: Working Svelte frontend in `src/`

### Tauri Integration
- [ ] 👤 **Tony**: Wire frontend to backend
  - Define Tauri commands in Rust (`#[tauri::command]`)
  - Create TypeScript types matching Rust structs
  - Implement invoke calls in Svelte components
  - Test full data flow: UI → Tauri → Rust → SQLite → Rust → UI
  - Deliverable: End-to-end working application

---

## 🔄 Phase 4: Integration & Feedback (Week 11-12)

### Ceremony 4.1: Scenario-to-Test Decomposition
- [ ] 👤 **Tony**: Map BDD scenarios to unit tests (1-2 days)
  - Follow [doc/internal/ceremonies/phase4/4.scenario-to-test-decomposition.md](doc/internal/ceremonies/phase4/4.scenario-to-test-decomposition.md)
  - For each BDD scenario, trace to unit tests
  - Identify gaps (missing tests or wrong tests)
  - Add missing unit tests
  - Deliverable: Test coverage map, complete unit test suite

### Ceremony 4.2: Domain Model Retrospective
- [ ] 🤝 **Both**: Reflect on domain model alignment (2-3 hours)
  - Follow [doc/internal/ceremonies/phase4/4.domain-model-retrospective.md](doc/internal/ceremonies/phase4/4.domain-model-retrospective.md)
  - Review implemented code vs domain model
  - Identify friction (hard-to-test code, unclear names)
  - Propose model refinements
  - Update ubiquitous language if needed
  - Deliverable: Retrospective notes

### Ceremony 4.3: Living Documentation Sync
- [ ] 👤 **Tony**: Update docs to match code (1-2 days)
  - Follow [doc/internal/ceremonies/phase4/4.living-documentation-sync.md](doc/internal/ceremonies/phase4/4.living-documentation-sync.md)
  - Review changed BDD scenarios
  - Update domain diagrams if needed
  - Update ubiquitous language glossary
  - Archive deprecated scenarios
  - Deliverable: Updated docs, CHANGELOG.md updated

---

## 📦 MVP Release Preparation (Week 13-14)

### Package Tauri Application
- [ ] 👤 **Tony**: Build release artifacts (2-3 days)
  - Configure release build in `tauri.conf.json`
  - Build Windows installer (.msi)
  - Test on Windows 10 and Windows 11
  - Verify:
    - Startup time <3 seconds
    - Memory usage <300MB
    - Offline operation (disconnect network, verify all features work)
    - Power failure recovery (kill app mid-save, verify data integrity)
  - Deliverable: Distributable installer for Windows

### Create Documentation
- [ ] 👤 **Tony**: Write technical docs (1-2 days)
  - Installation guide (Windows: run .msi installer)
  - System requirements (Windows 10+, 8GB RAM recommended, 500MB disk)
  - Offline operation guide (how sync works, what's safe offline)
  - Troubleshooting guide (common issues, log locations)
  - Deliverable: `doc/public/installation-guide.md`, `doc/public/user-guide.md`

- [ ] 👨‍⚕️ **Nico**: Write user-facing documentation (1-2 days)
  - Quick start guide for front desk staff
  - How to import patients (CSV format, manual entry)
  - How to configure recall rules
  - How to run campaigns and print call sheets
  - How to track contact outcomes
  - Deliverable: `doc/public/outreach-user-guide.md` with screenshots

### Pilot Clinic Deployment
- [ ] 🤝 **Both**: Deploy to pilot clinic(s) (1 day per clinic)
  - Install application on clinic workstation(s)
  - Import initial patient list
  - Configure first recall campaign
  - Train front desk staff (30-60 minutes)
  - Leave printed quick reference guide
  - Schedule first weekly check-in
  - Deliverable: Deployed MVP, trained staff

---

## 📊 Post-Release Monitoring (Week 15-18)

### Week 1-2: Close Monitoring
- [ ] 🤝 **Both**: Daily check-ins with pilot clinic
  - Monitor for crashes, data loss, sync issues
  - Collect usability feedback
  - Fix critical bugs immediately
  - Track: Contacts made, appointments recovered
  - Deliverable: Bug fixes, usage metrics

### Week 3-4: Stabilization
- [ ] 🤝 **Both**: Weekly check-ins
  - Analyze appointment recovery rate (target: ≥1/month/clinic)
  - Assess pricing willingness (is $75-150/month justifiable?)
  - Identify feature gaps for Phase 1 (Scheduling)
  - Plan upsell conversations
  - Deliverable: Pilot retrospective, Phase 1 planning

### Success Validation
- [ ] 🤝 **Both**: Assess MVP success criteria
  - [ ] Time-to-first-value under 7 days ✅/❌
  - [ ] At least one recovered appointment per clinic per month ✅/❌
  - [ ] Daily staff usage by front desk team ✅/❌
  - [ ] Justifiable pricing ($75-150/month) ✅/❌
  - [ ] Zero data loss under power/network failure ✅/❌
  - [ ] 100% offline operational capability ✅/❌
  - Deliverable: Go/No-Go decision for Phase 1 (Scheduling)

---

## 🎯 Definition of Done (MVP Release)

**MVP is complete when**:
- ✅ All Phase 1-4 ceremonies executed and documented
- ✅ All P0 BDD scenarios passing (green)
- ✅ Test coverage ≥80% overall, ≥90% for domain aggregates
- ✅ Windows installer built and tested
- ✅ Deployed to 1-2 pilot clinics
- ✅ Front desk staff trained and using daily
- ✅ At least one recovered appointment per clinic documented
- ✅ No critical bugs blocking daily use
- ✅ Documentation complete (installation, user guide, troubleshooting)
- ✅ Memory usage <300MB, startup <3 seconds

---

## 📅 Estimated Timeline Summary

| Week | Phase | Key Activities | Owner |
|------|-------|---------------|-------|
| 1-2 | Pre-Dev | Figma design, pilot clinic ID, dev environment | 👨‍⚕️ + 👤 |
| 3-4 | Phase 1 (DDD) | Event Storming, Ubiquitous Language, Domain Modeling | 🤝 |
| 5-6 | Phase 2 (BDD) | Example Mapping, Three Amigos, Acceptance Criteria | 🤝 |
| 7-10 | Phase 3 (TDD) | Domain implementation, UI development, testing | 👤 |
| 11-12 | Phase 4 (Integration) | Documentation sync, retrospective | 🤝 |
| 13-14 | Release Prep | Packaging, documentation, pilot deployment | 👤 + 👨‍⚕️ |
| 15-18 | Post-Release | Monitoring, bug fixes, success validation | 🤝 |

**Total**: 14-18 weeks (3.5-4.5 months)

---

## ⚠️ Critical Success Factors

1. **Nico's Figma Designs**: Must complete before Phase 1 (foundation for domain model alignment)
2. **Pilot Clinic Commitment**: Must secure before Phase 2 (informs BDD scenarios with real workflows)
3. **Offline-First Discipline**: Every feature must work 100% offline (non-negotiable)
4. **Test Coverage**: ≥80% overall, ≥90% domain aggregates (prevents rework, enables refactoring)
5. **Weekly Tony+Nico Sync**: Communication critical for domain alignment and clinical validation
6. **No Scope Creep**: Phase 0 is Outreach ONLY (no Scheduling, no Payment, no Charting yet)

---

## 🔗 References

- **Architecture**: [technical-architecture.md](doc/internal/architecture/technical-architecture.md) - System design
- **Technology Decision**: [ADR-002](doc/internal/governance/ADR/ADR-002-technology-stack-rust-tauri.md) - Why Rust + Tauri + Svelte
- **UI Translation**: [figma-to-svelte-guide.md](doc/internal/reference/figma-to-svelte-guide.md) - Figma to code
- **Ceremony Index**: [doc/internal/ceremonies/README.md](doc/internal/ceremonies/README.md) - All ceremony instruction files
- **CHARTER**: [CHARTER.md](CHARTER.md) - Vision and objectives

---

**Last Updated**: 2026-02-06
**Maintained By**: Tony Moores (Technical Owner)
**Review Frequency**: Weekly during Phase 0, monthly after MVP release
