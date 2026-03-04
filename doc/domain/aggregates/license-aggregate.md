# License Aggregate

**Context**: Licensing
**Status**: Discovered (Phase 1.3 complete) — Phase 2 decisions incorporated
**Date**: 2026-03-03

---

## Purpose

The License aggregate represents the application's authorization to operate. It tracks whether the current installation has a valid license, the status of each licensed module (Active, Degraded, Expired), and enforces the guiding principle:

> "The data is always yours — you are never blocked from it. If you ain't paying, you don't get value out of it."

Read access to all data is unconditional. Write access per module depends on that module's status.

---

## Aggregate Root

**License** — singleton per installation. One License; multiple modules within it.

---

## Module Status State Machine

Each module in the license payload has its own independent status:

```
[Not Licensed]
      │
      │ EvalStarted (all modules included)
      │ LicenseIssued / LicenseRenewed (module in payload)
      ▼
   [Active]
      │
      │ expires_at passed, grace_period_days > 0
      ▼
  [Degraded] ──── grace expires ────► [Expired]
      │
      │ LicenseRenewed (new key includes this module)
      ▼
   [Active]

From [Expired]:
      │ LicenseRenewed (new key includes this module)
      ▼
   [Active]
```

**Overall license validity** (separate from module status):

```
[Valid] ──── ClockRollbackDetected ────► [Invalid] (session-only)
```

Clock rollback overrides all module statuses for the session. Clears at next startup when clock is corrected.

---

## Module Status → Capabilities

| Module Status | Read Access | Write Access | Notes |
|--------------|-------------|--------------|-------|
| **Active** | Full | Full | Paid up |
| **Degraded** | Full | None | Within grace period |
| **Expired** | Full | None | Past grace period; data still theirs |
| **Not Licensed** | Full (historical data) | None | Module not in payload |
| **Invalid** (clock) | Full | None | All modules blocked until clock corrected |

**Read access is unconditional regardless of module status.** This is absolute.

---

## Commands

### StartEval

**Actor**: System (first run only)
**Preconditions**:
- PracticeIdentity established
- No EvalStarted or LicenseIssued event exists

**Process**:
1. Verify embedded eval token signature against embedded Ed25519 public key
2. Compute `evalExpiresAt = installDate + max_duration_days` per the eval payload
3. Emit `EvalStarted`

**Invariants**: Eval can only start once. Same validation path as paid license.

---

### ActivateLicense

**Actor**: Practice Manager
**Preconditions**: PracticeIdentity established; license key string provided

**Process**:
1. Decode base64url → payload bytes || 64-byte signature
2. Verify Ed25519 signature over payload bytes
3. Parse payload JSON, validate `schema_version`
4. Verify `practice_id` matches established PracticeId
5. Verify at least one module has `expires_at` in the future
6. **If any module's `expires_at` is earlier than the current active `expires_at` for that module** → warn and require confirmation before proceeding
7. Emit `LicenseIssued`

**Rejection reasons** (no event emitted):
- `InvalidSignature`: Ed25519 verification failed
- `PracticeIdMismatch`: payload practice_id ≠ established practiceId
- `AllModulesExpired`: every module in payload has expires_at in the past
- `UnknownSchemaVersion`: schema_version not recognized

---

### RenewLicense

**Actor**: Practice Manager
**Process**: Same as ActivateLicense. Emits `LicenseRenewed` instead of `LicenseIssued`.

**Note**: Renewal accepted from any status (Expired, Degraded, Active). Write access restored at next startup.

---

### ValidateLicenseOnStartup

**Actor**: System (every startup)

**Process**:
1. **Anti-rollback check**: Read last `LicenseValidationSucceeded` timestamp. If system clock is >24h before it → emit `ClockRollbackDetected`, stop.
2. **Per-module status evaluation** (for each module in the current license):
   - If `now > expires_at` and `grace_period_days > 0` and not already Degraded → emit `LicenseDegraded { module_name }`
   - If `now > expires_at` and `grace_period_days = 0` and not already Expired → emit `LicenseExpired { module_name }`
   - If Degraded and `now > expires_at + grace_period_days` → emit `LicenseExpired { module_name }`
3. Emit `LicenseValidationSucceeded` (records timestamp for anti-rollback; captures current module statuses)

**Enforcement**: Write access restrictions from step 2 apply immediately on startup. This is the only enforcement gate.

---

### RunPeriodicCheck (48-hour in-session)

**Actor**: System (background, every 48 hours while app is running)

**Process**:
1. Per-module status evaluation (same as startup, steps 2)
2. If any module has newly entered Degraded → emit `LicenseDegraded { module_name }`
3. Update `license_status` projection
4. **Does NOT** emit `LicenseValidationSucceeded` (no anti-rollback update mid-session)
5. **Does NOT** change write access — enforcement is startup-only

**Purpose**: Ensures long-running sessions surface status changes and warnings in the banner. Not an enforcement mechanism.

---

### CheckModuleAccess

**Actor**: System (feature gate — pure query, no event)

**Returns**: `{ can_read: bool, can_write: bool }`

- `can_read`: always `true` (data is unconditionally theirs)
- `can_write`: `true` only if module status is `Active` AND overall license is `Valid` (no clock rollback)

---

## Events

### EvalStarted
```
EvalStarted {
    practice_id: String,
    started_at: DateTime<Utc>,
    eval_expires_at: DateTime<Utc>,       // started_at + max_duration_days
    modules: Vec<String>,                 // module names included in eval
}
```

### LicenseIssued
```
LicenseIssued {
    practice_id: String,
    license_type: LicenseType,            // Paid
    issued_at: DateTime<Utc>,
    modules: Vec<LicenseModuleEntry>,
    schema_version: u32,
}

LicenseModuleEntry {
    name: String,
    expires_at: DateTime<Utc>,
    grace_period_days: u32,
}
```

### LicenseRenewed
```
LicenseRenewed {
    // same shape as LicenseIssued
}
```

### LicenseValidationSucceeded
```
LicenseValidationSucceeded {
    validated_at: DateTime<Utc>,          // anti-rollback anchor
    license_type: LicenseType,
    module_statuses: Vec<ModuleStatus>,   // snapshot of all module states at validation
}
```

### LicenseValidationFailed
```
LicenseValidationFailed {
    failed_at: DateTime<Utc>,
    reason: ValidationFailureReason,
    // InvalidSignature | PracticeIdMismatch | AllModulesExpired | ClockRollback
}
```

### LicenseDegraded
```
LicenseDegraded {
    module_name: String,
    practice_id: String,
    degraded_at: DateTime<Utc>,
    grace_expires_at: DateTime<Utc>,
}
```

### LicenseExpired
```
LicenseExpired {
    module_name: String,
    practice_id: String,
    expired_at: DateTime<Utc>,
    prior_status: ModuleStatus,           // Active | Degraded
}
```

### ClockRollbackDetected
```
ClockRollbackDetected {
    detected_at: DateTime<Utc>,
    last_known_valid_at: DateTime<Utc>,
    clock_delta_hours: f64,
}
```

---

## Projections

### `license_status` (overall)
```sql
CREATE TABLE license_status (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    overall_validity TEXT NOT NULL,       -- 'Valid' | 'Invalid' (clock rollback)
    license_type TEXT,                    -- 'eval' | 'paid' | NULL
    eval_expires_at TEXT,                 -- ISO 8601 | NULL
    last_validated_at TEXT,               -- ISO 8601 (anti-rollback anchor)
    updated_at TEXT NOT NULL
);
```

### `license_module_status` (per module)
```sql
CREATE TABLE license_module_status (
    module_name TEXT PRIMARY KEY,
    status TEXT NOT NULL,                 -- 'Active' | 'Degraded' | 'Expired'
    expires_at TEXT NOT NULL,             -- ISO 8601
    grace_period_days INTEGER NOT NULL,
    grace_expires_at TEXT,                -- ISO 8601 | NULL (only when Degraded)
    updated_at TEXT NOT NULL
);
```

---

## Domain Rules Summary

| Rule | Description |
|------|-------------|
| R1 | License is a singleton aggregate per installation. |
| R2 | Read access to all data is unconditional, regardless of module status. |
| R3 | Write access per module requires that module's status = Active and overall validity = Valid. |
| R4 | Module statuses are independent — one module expiring does not affect others. |
| R5 | Enforcement (write access restriction) happens only at startup. 48h check updates banner only. |
| R6 | Eval has no grace period. It expires directly to Expired. |
| R7 | Grace periods are per-module, embedded in the payload. App does not hardcode criticality. |
| R8 | Startup validation always runs. Clock rollback check runs before module status evaluation. |
| R9 | If a renewal key has an earlier expiry than the current module expiry, warn and confirm before accepting. |
| R10 | Warning thresholds (30d, 14d, 7d) are computed from projection at render time — not stored as events. |
| R11 | Pause/rollover is handled by the License Server (new token with extended expiry). App has no pause logic. |

---

**Related**:
- `doc/domain/aggregates/practice-identity-aggregate.md`
- `doc/governance/PDR/PDR-001-licensing-model.md`
- `doc/governance/ADR/ADR-002-licensing-cryptography.md`
- `doc/domain/event-storming/licensing-events.md`
