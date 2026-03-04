# Event Storming: Licensing Context

**Date**: 2026-03-03
**Participants**: Tony (Product Owner), Claude (Developer)
**Status**: Complete — Phase 1.1

---

## Overview

The Licensing context controls whether the application is in a valid operational state. It is upstream of every other bounded context — all feature access depends on license status. It has two aggregates: **PracticeIdentity** (immutable machine binding) and **License** (lifecycle state machine).

---

## Domain Events (Orange Stickies)

Events are listed in rough chronological order for a typical installation lifecycle.

### PracticeIdentity Aggregate

| Event | When It Occurs | Key Data |
|-------|---------------|----------|
| **PracticeIdentityEstablished** | First run — machine ID and install date are recorded and the PracticeId is computed | `practice_id`, `machine_id_hash` (SHA-256 of machineId, not raw), `install_date` |

**Notes**: PracticeIdentity is a singleton aggregate. Once established, it never changes. The raw `machineId` is hashed before storage to avoid persisting a full hardware fingerprint.

---

### License Aggregate

| Event | When It Occurs | Key Data |
|-------|---------------|----------|
| **EvalStarted** | First run — pre-signed eval token validated; eval period begins | `practice_id`, `started_at`, `expires_at`, `modules` |
| **LicenseIssued** | User enters a valid paid license key for the first time (or after a hardware migration) | `practice_id`, `license_type`, `issued_at`, `expires_at`, `grace_period_days`, `modules`, `schema_version` |
| **LicenseRenewed** | User enters a valid replacement license key | `practice_id`, `license_type`, `issued_at`, `expires_at`, `grace_period_days`, `modules`, `schema_version` |
| **LicenseValidationSucceeded** | Startup — signature valid, practice_id matches, clock check passes | `validated_at`, `license_type`, `status`, `modules` |
| **LicenseValidationFailed** | Startup — signature invalid, practice_id mismatch, or license expired beyond grace | `failed_at`, `reason` (enum: InvalidSignature, PracticeIdMismatch, Expired, ClockRollback) |
| **LicenseDegraded** | Paid license expires but grace period > 0; license enters degraded state | `practice_id`, `degraded_at`, `grace_expires_at` |
| **LicenseExpired** | Eval period ends (no grace) OR grace period ends on a paid license | `practice_id`, `expired_at`, `prior_status` |
| **ClockRollbackDetected** | Startup — system clock is >24h before last LicenseValidationSucceeded timestamp | `detected_at`, `last_known_valid_at`, `clock_delta_hours` |

**Total events**: 9 (1 PracticeIdentity + 8 License)

---

## Commands (Blue Stickies)

| Command | Actor | Preconditions | Resulting Event(s) |
|---------|-------|--------------|-------------------|
| **EstablishPracticeIdentity** | System (first run) | No prior PracticeIdentityEstablished event exists | PracticeIdentityEstablished |
| **StartEval** | System (first run) | PracticeIdentity exists; no prior LicenseIssued or EvalStarted | EvalStarted |
| **ActivateLicense** | Practice Manager | PracticeIdentity established; license key passes signature + practice_id check | LicenseIssued |
| **RenewLicense** | Practice Manager | License exists (any status); new key passes signature + practice_id check | LicenseRenewed |
| **ValidateLicenseOnStartup** | System (startup) | Always runs | LicenseValidationSucceeded or LicenseValidationFailed or ClockRollbackDetected |
| **CheckModuleAccess** | System (feature gate) | Always runs — pure query, no event | (no event — returns boolean) |

---

## Actors

| Actor | Role |
|-------|------|
| **System** | Automated actions on startup and first run |
| **Practice Manager** | The person who activates or renews a license |
| **License Server** | External system — issues signed license keys |
| **Hardware** | Provides stable machine identifier (MachineGuid) |

---

## Policies (Purple Stickies)

| Trigger Event | Policy | Resulting Command |
|---------------|--------|------------------|
| Application starts for the first time | → System establishes PracticeIdentity then starts eval | EstablishPracticeIdentity, StartEval |
| Application starts (any run) | → System validates the current license | ValidateLicenseOnStartup |
| EvalStarted occurred >30 days ago | → License expired; transition to Expired | (detected in ValidateLicenseOnStartup) |
| Paid license expires_at < now AND grace_period_days > 0 | → License enters degraded mode | (LicenseDegraded emitted in ValidateLicenseOnStartup) |
| LicenseDegraded occurred >grace_period_days ago | → License fully expired | (LicenseExpired emitted in ValidateLicenseOnStartup) |
| System clock >24h before last validation timestamp | → Rollback detected; deny writes | (ClockRollbackDetected emitted in ValidateLicenseOnStartup) |

---

## Read Models / Projections (Green Stickies)

| Projection | What It Provides | Consumed By |
|------------|-----------------|-------------|
| **license_status** | Current status (Eval/Active/Degraded/Expired/Invalid), expiry date, grace expiry date, module list | Every feature gate check; startup screen; settings screen |
| **practice_identity** | practiceId, install_date, established_at | Support info screen; license activation form; event store metadata |

---

## External Systems

| System | Role | Integration |
|--------|------|-------------|
| **License Server** | Signs and issues license keys | Offline — user obtains key and enters it manually. No runtime API call from app. |
| **Hardware (MachineGuid)** | Provides stable machine identifier | Read once on first run via `machine-uid` crate |
| **System Clock** | Provides current time for expiry checks | Read on every startup; compared against last-known-valid timestamp |

---

## License State Machine

```
[No License]
     │
     │ EvalStarted
     ▼
  [Eval]
     │
     │ LicenseExpired (30 days elapsed)
     ▼         ┌─────────────────────┐
  [Expired] ◄──┤ LicenseExpired      │
     │          │ (grace exhausted)   │
     │          └─────────────────────┘
     │                     ▲
     │ LicenseIssued        │ LicenseDegraded
     │ (or LicenseRenewed)  │ (from Active, grace > 0)
     ▼                      │
  [Active] ─────────────────┘
     │
     │ LicenseRenewed
     └──► [Active] (new expiry)


From any state:
  ClockRollbackDetected → [Invalid] (session-only; resolved on next startup with correct clock)
```

**States**:

| State | Description | What's Allowed |
|-------|-------------|---------------|
| **Eval** | 30-day free trial; all modules | Full read/write access |
| **Active** | Valid paid license; within expiry | Full read/write access to licensed modules |
| **Degraded** | Paid license expired; within grace period | Read-only access to existing data |
| **Expired** | Eval or grace period exhausted | Read-only access to existing data; renewal prompt always shown |
| **Invalid** | Clock rollback detected | Read-only access; clock error prompt shown |

---

## Hot Spots (Red Stickies)

| # | Hot Spot | Resolution / Status |
|---|----------|---------------------|
| 1 | What exactly is "read-only" in degraded/expired mode — which actions are blocked? | Proposed in PDR-001: view all, no create/modify. **Tony to confirm** during Three Amigos. |
| 2 | Grace period default: 7 days assumed | **Tony to confirm** |
| 3 | What does the user see at each status transition? | To be specified in Phase 2 Example Mapping. |
| 4 | What happens to in-flight work (e.g., open appointment form) when grace period expires mid-session? | To be specified in Phase 2 Example Mapping. |
| 5 | Module list for MVP: `["core", "scheduling"]` assumed | **Tony to confirm** during Example Mapping. |
| 6 | Renewal UX: background automatic check vs. user-initiated portal vs. manual key entry | Deferred to Phase 2. Manual key entry is baseline. |

---

**Next**: Ubiquitous Language update (1.2), then Domain Modeling (1.3)
