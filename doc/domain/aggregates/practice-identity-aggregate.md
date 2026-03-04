# PracticeIdentity Aggregate

**Context**: Licensing
**Status**: Discovered (Phase 1.3 complete)
**Date**: 2026-03-03

---

## Purpose

The PracticeIdentity aggregate represents the immutable binding between a specific hardware installation and the Belsouri application. It is established exactly once on first run and never changes. It is the foundation for the License aggregate — the `practiceId` it produces is embedded in every license key and verified on activation.

PracticeIdentity answers the question: "Who is this installation?" The License answers: "Is this installation authorized to operate?"

---

## Aggregate Root

**PracticeIdentity** — singleton per installation. Established once, immutable thereafter.

---

## State

| Field | Type | Description |
|-------|------|-------------|
| `practice_id` | String (64-char hex) | Derived identifier: `lowercase_hex(SHA-256(machineId_utf8 || ":" || installDate_utf8))` |
| `machine_id_hash` | String (64-char hex) | SHA-256 of the raw machineId — not the raw machineId itself |
| `install_date` | Date (YYYY-MM-DD) | Date of first application launch, UTC |
| `established_at` | DateTime | Timestamp of PracticeIdentityEstablished event |

**State transitions**: None. PracticeIdentity is immutable after creation.

---

## Commands

### EstablishPracticeIdentity

**Actor**: System (first run only)
**Preconditions**:
- No `PracticeIdentityEstablished` event exists in the event store

**Process**:
1. Read machineId from hardware via `machine-uid` crate
2. Record `installDate = today()` (UTC date)
3. Compute `machineIdHash = SHA-256(machineId_utf8)`
4. Compute `practiceId = lowercase_hex(SHA-256(machineId_utf8 || ":" || installDate_iso8601_utf8))`
5. Emit `PracticeIdentityEstablished`

**Invariants**:
- Can only be called once — if PracticeIdentityEstablished already exists, this command is a no-op (or error)
- machineId must be non-empty
- installDate must be the current date

**Why hash machineId before storing**: The raw machineId (e.g., MachineGuid) is a hardware fingerprint. Storing only its hash avoids persisting a value that could be used to correlate installations across systems.

---

## Events

### PracticeIdentityEstablished

```
PracticeIdentityEstablished {
    practice_id: String,        // SHA-256(machineId || ":" || installDate), hex
    machine_id_hash: String,    // SHA-256(machineId), hex — raw machineId is NOT stored
    install_date: String,       // YYYY-MM-DD (UTC)
    established_at: DateTime<Utc>,
}
```

---

## Invariants

1. **Singleton**: Only one PracticeIdentity per installation. EstablishPracticeIdentity fails if already established.
2. **Immutability**: Once established, the practiceId never changes. Hardware changes require a support-assisted re-issuance at the License level (new license key for new practiceId).
3. **No raw machineId persistence**: The raw machineId string is used to compute hashes but is never written to the event store or any projection.

---

## Projections

### `practice_identity` Projection

Built from: PracticeIdentityEstablished (single event, never updated)

```sql
CREATE TABLE practice_identity (
    id INTEGER PRIMARY KEY CHECK (id = 1),  -- singleton row
    practice_id TEXT NOT NULL,
    machine_id_hash TEXT NOT NULL,
    install_date TEXT NOT NULL,             -- YYYY-MM-DD
    established_at TEXT NOT NULL            -- ISO 8601
);
```

**Usage**:
- Displayed on the support/about screen so the Practice Manager can provide it to Tony for re-issuance
- Read by the license activation command to verify practice_id against the license payload
- Read by StartEval to populate the EvalStarted event

---

## Domain Rules Summary

| Rule | Description |
|------|-------------|
| R1 | PracticeIdentity is established exactly once on first run. |
| R2 | The raw machineId is never persisted — only its SHA-256 hash. |
| R3 | installDate is the UTC date of first launch and is fixed permanently. |
| R4 | practiceId = SHA-256(machineId || ":" || installDate) encoded as lowercase hex. |
| R5 | Any license key with a practiceId that does not match this practiceId is rejected. |

---

## Relationship to License Aggregate

```
PracticeIdentity (1) ──────────────── (1) License
                  practiceId flows into LicenseIssued/LicenseRenewed validation
```

- PracticeIdentity is established first (EstablishPracticeIdentity command)
- License aggregate's ActivateLicense command reads practiceId from the PracticeIdentity projection
- License aggregate cannot activate a key unless PracticeIdentity is established

---

**Related**:
- `doc/domain/aggregates/license-aggregate.md`
- `doc/governance/ADR/ADR-002-licensing-cryptography.md`
- `doc/domain/event-storming/licensing-events.md`
