# ADR-002: Licensing Cryptography

**Status**: Accepted
**Date**: 2026-03-03
**Deciders**: Tony (Product Owner), Claude (Developer)
**Supersedes**: (none)
**Category**: Architecture

---

## Context

The licensing system requires cryptographic enforcement that:

1. Cannot be bypassed by inspecting or modifying a license key
2. Works fully offline after license issuance — no server round-trips on startup
3. Verifies that license payloads are authentic and unmodified
4. Binds licenses to specific machines via PracticeId
5. Supports per-module expiry and independent grace periods
6. Is implementable in Rust using well-audited crates

The product-level decisions (what to license, eval model, degraded mode, warning system) are in PDR-001. This ADR covers only the technical cryptographic and storage choices.

---

## Decisions

### Decision 1: Ed25519 Asymmetric Signing

**Decision**: Use Ed25519 asymmetric signatures for license key authentication. The License Server holds the Ed25519 private key and signs every license payload. The Belsouri binary embeds the corresponding Ed25519 public key and verifies signatures at runtime.

**Rationale**:
- The private key never leaves the License Server — the binary cannot be used to generate valid licenses
- Payloads are readable (JSON) but unforgeable without the private key
- Ed25519 verification is ~100µs on modern hardware, negligible on startup
- 32-byte public key, 64-byte signature — minimal binary bloat
- `ring` crate (or `ed25519-dalek`) is well-audited and used in production Rust systems

**Why not symmetric AES or HMAC**:
- Symmetric algorithms require embedding the key in the binary
- Anyone with the binary and a hex editor can extract the key and sign arbitrary payloads
- The verification key and the signing key are the same — there is no way to give the app the ability to verify without also giving it the ability to forge

---

### Decision 2: License Key Format (schema_version: 2)

**Decision**: `LicenseKey = base64url(payload_json_bytes || ed25519_signature_bytes)`

Where:
- `payload_json_bytes`: UTF-8 JSON of `LicensePayload`
- `ed25519_signature_bytes`: 64-byte Ed25519 signature over `payload_json_bytes`
- `||`: concatenation (payload first, signature last)
- `base64url`: URL-safe base64, no padding (RFC 4648 §5)

**Payload schema (schema_version: 2)**:

```json
{
  "schema_version": 2,
  "practice_id": "<64-char lowercase hex>",
  "license_type": "paid",
  "issued_at": "2026-03-03T00:00:00Z",
  "modules": [
    {
      "name": "scheduling",
      "expires_at": "2027-03-03T00:00:00Z",
      "grace_period_days": 90
    },
    {
      "name": "recall",
      "expires_at": "2027-03-03T00:00:00Z",
      "grace_period_days": 30
    }
  ]
}
```

**Eval token schema** (wildcard — no practice_id, no per-module expiry):

```json
{
  "schema_version": 2,
  "practice_id": null,
  "license_type": "eval",
  "issued_at": "<release date>",
  "max_duration_days": 30,
  "modules": [
    { "name": "scheduling", "grace_period_days": 0 },
    { "name": "recall",     "grace_period_days": 0 }
  ]
}
```

For eval tokens, `expires_at` per module is computed at runtime as `installDate + max_duration_days`. No grace period on eval.

**Grace period classification** (enforced by License Server at issuance, not hardcoded in app):

| Module category | grace_period_days |
|----------------|------------------|
| Critical (scheduling, charting) | 90 |
| Non-critical (recall, imaging, portability, reporting) | 30 |

**Rationale**:
- Human-readable payload enables support debugging (decode base64, read JSON)
- Signature covers exact payload bytes — any JSON modification invalidates it
- base64url is URL-safe for email delivery and web portal distribution
- Per-module expiry + grace allows independent renewal of modules
- Grace period in the payload (not hardcoded) lets Tony adjust policy without an app release

---

### Decision 3: Machine Identifier — `machine-uid` Crate

**Decision**: Use the `machine-uid` Rust crate to derive a stable machine identifier.

| Platform | Source | Stability Notes |
|----------|--------|-----------------|
| Windows | `HKLM\SOFTWARE\Microsoft\Cryptography\MachineGuid` | Survives NIC replacement, Windows Update. Changes on OS reinstall. |
| macOS | `IOPlatformUUID` (IOKit) | Hardware-based, survives most OS reinstalls. |
| Linux | `/etc/machine-id` | Survives reboots. Changes on reinstall. |

**PracticeId derivation**:
```
practiceId = lowercase_hex(SHA-256(machineId_utf8 || ":" || installDate_iso8601_utf8))
```

Where `installDate` is the date of first application launch (YYYY-MM-DD, UTC).

**Rationale**:
- `machine-uid` abstracts platform differences cleanly
- MachineGuid is the most stable Windows identifier for this use case
- SHA-256 combination with installDate prevents practiceId from directly exposing machineId

---

### Decision 4: Anti-Rollback via events.db

**Decision**: On each successful startup license validation, emit a `LicenseValidationSucceeded` event containing the validation timestamp. On subsequent startups, read the most recent such event and compare with the current system clock. If the system clock is more than 24 hours before the last validation timestamp, emit `ClockRollbackDetected` and deny write operations for the session.

**Why events.db (not a flat file)**:
- Append-only store — users cannot edit or delete entries without understanding SQLite
- WAL mode ensures crash resilience
- Aligns with event sourcing architecture — validation is a domain event
- More tamper-resistant than a timestamp file in the app data directory

**Threshold: 24 hours**: Safely above DST/timezone changes, comfortably detects deliberate multi-day backdating.

**Recovery**: Once the system clock advances past the threshold, validation succeeds normally on next startup.

**48-hour in-session check**: A separate background check runs every 48 hours while the app is running. It does NOT emit `LicenseValidationSucceeded` (that is startup-only). It reads the current license state and updates the status banner. This check does not trigger anti-rollback enforcement — that remains startup-only.

---

### Decision 5: Pre-Signed Eval Token Embedded at Build Time

**Decision**: During the release build process, the License Server signs a wildcard eval payload and the resulting token is embedded as a Rust constant in the binary.

**Build-time process**:
1. Generate eval `LicensePayload` (no `practice_id`, `license_type: "eval"`, `max_duration_days: 30`, all current modules)
2. Sign with Ed25519 private key
3. base64url-encode → string constant in `src-tauri/src/licensing/eval_token.rs`

**Runtime first-run process**:
1. Verify eval token signature (same path as paid license)
2. Compute PracticeId from `machineId + installDate`
3. Emit `PracticeIdentityEstablished(practiceId, machineId_hash, installDate)`
4. Emit `EvalStarted(installDate, evalExpiresAt = installDate + 30 days, modules)`

**Rationale**: Eval works offline with zero server calls. Same validation code path as paid.

---

### Decision 6: License File Storage

**Decision**: The license key string is stored in a plain text file at `<app_data_dir>/license.key`. Machine-binding is enforced cryptographically (PracticeId check in payload), not by file permissions.

**Path (platform)**:
| Platform | Path |
|----------|------|
| Windows | `%APPDATA%\com.belsouri.app\license.key` |
| Linux | `~/.local/share/com.belsouri.app/license.key` |
| macOS | `~/Library/Application Support/com.belsouri.app/license.key` |

**Rationale**: Simplicity. Plain file allows backup/restore by IT-savvy support staff. Even if copied to another machine, the PracticeId won't match.

---

### Decision 7: Warning Thresholds (Computed, Not Stored)

**Decision**: Expiry warnings (30 days, 14 days, 7 days) are computed at render time from the `license_status` projection — specifically from each module's `expires_at`. No warning events are stored in the event store.

**Rationale**: Warnings are derived state. Storing them as events would create noise without value. The projection already has all the data needed to compute warning thresholds at any point.

---

## Consequences

**Positive**:
- Private key never in binary — forgery requires compromising the License Server
- Fully offline-capable after license issuance
- Ed25519 is NIST-approved, battle-tested
- Machine binding cannot be bypassed by copying the license file
- Per-module expiry and grace periods are flexible without app changes

**Negative**:
- Public key embedded in binary — determined attacker could patch the binary. Mitigation: Windows Authenticode signing (post-MVP).
- No online revocation — once issued, a license cannot be recalled before expiry. Mitigation: short expiry periods (1 year).
- Eval token in binary is decompilable. Structurally useless for generating paid licenses.

---

**Related**:
- PDR-001: Licensing Model (product decisions)
- `doc/domain/aggregates/license-aggregate.md`
- `doc/domain/aggregates/practice-identity-aggregate.md`

**Reviewed By**: Tony (Product Owner)
