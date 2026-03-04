# Example Map: Licensing

**Date**: 2026-03-03
**Participants**: Tony (Product Owner / Business), Claude (Developer / Tester)
**Feature**: Licensing context — full lifecycle
**Status**: Phase 2.2 + 2.3 complete — all open questions resolved

---

## Guiding Principle

> "The data is always yours and you should never be blocked from it. But if you ain't paying, you should not be getting value out of it."

---

## Three Amigos Summary (Phase 2.1)

All open questions resolved during discussion:

| OQ | Question | Resolution |
|----|----------|-----------|
| OQ-1 | What exactly is allowed in Degraded Mode? | Read access always, unconditionally. Write access blocked per expired module. Principle above governs. |
| OQ-2 | Grace period default? | Per-module: 90 days for critical modules (scheduling, charting); 30 days for non-critical (recall, imaging, etc.). Caribbean weather events can take weeks to resolve. |
| OQ-3 | MVP module list? | `scheduling` only. Covers Practice Setup + Patient Management + Staff Scheduling + Patient Scheduling. Future: charting, recall, imaging, portability, reporting. |
| OQ-4 | Can renewal prompt be dismissed? | No blocking prompt at all. Status **banner** only — persistent, visible, never interrupts workflow or blocks navigation. |
| OQ-5 | Renewal key with earlier expiry — accept or warn? | Warn and confirm. "This key expires earlier than your current license — do you want to replace it?" |
| OQ-6 | Mid-session status change — startup-only acceptable? | Startup enforces. 48-hour background check updates the banner only. No mid-session enforcement. No surprises. |
| OQ-7 | Visible eval countdown? | Yes — eval countdown shown in the status banner. |

---

## Rule Cards

---

### Rule 1: Fresh Install Enters Eval Period

**Rule**: On first launch, PracticeIdentity is established and a 30-day eval period begins automatically. No user action required. Eval countdown shown in banner.

| # | Example | Type |
|---|---------|------|
| 1a | First launch → PracticeIdentityEstablished, EvalStarted, status = Eval, banner shows "Trial — 30 days remaining" | ✅ Happy path |
| 1b | Second launch (day 5) → Eval continues, banner shows "Trial — 25 days remaining" | ✅ Happy path |
| 1c | First launch, machineId cannot be read → Error shown, app cannot start | ❌ Edge case |

---

### Rule 2: Eval Period Expires

**Rule**: When the system clock is past `installDate + 30 days` and license status is Eval, the scheduling module transitions to Expired on startup. No grace period for eval.

| # | Example | Type |
|---|---------|------|
| 2a | Launch on day 30 (exactly) → Status remains Eval | ✅ Boundary |
| 2b | Launch on day 31 → LicenseExpired (module: scheduling), status = Expired | ✅ Happy path |
| 2c | Expired: view existing patient record → Displayed normally | ✅ Happy path |
| 2d | Expired: view existing schedule → Displayed normally | ✅ Happy path |
| 2e | Expired: create new appointment → Blocked; banner shows "Scheduling has expired — renew to restore scheduling" | ❌ Negative path |
| 2f | Expired: register new patient → Blocked | ❌ Negative path |

---

### Rule 3: License Key Activation

**Rule**: A Practice Manager enters a valid LicenseKey. The app validates signature and practiceId locally (no server call). On success, the module's status transitions to Active at next startup. (Write access restored at next startup — current session is not affected.)

| # | Example | Type |
|---|---------|------|
| 3a | Valid key, matching practiceId, scheduling not expired → LicenseIssued, Active on next startup | ✅ Happy path |
| 3b | Key with invalid signature → "This license key is not valid" | ❌ Negative path |
| 3c | Key with mismatched practiceId → "This key was issued for a different installation" | ❌ Negative path |
| 3d | Key where all modules have expires_at in the past → "This license key has already expired" | ❌ Negative path |
| 3e | Key with unrecognized schema_version → "This key format is not supported by this version of Belsouri" | ❌ Edge case |
| 3f | Activation during Eval → LicenseIssued, scheduling Active on next startup | ✅ Happy path |
| 3g | Activation during Expired → LicenseIssued, scheduling Active on next startup | ✅ Happy path |
| 3h | No network connection → Validation runs locally, LicenseIssued succeeds | ✅ Happy path |

---

### Rule 4: Renewal with Earlier Expiry — Warn and Confirm

**Rule**: If the renewal key's `expires_at` for a module is earlier than the current active `expires_at` for that module, warn the Practice Manager and ask for confirmation.

| # | Example | Type |
|---|---------|------|
| 4a | Renewal key expires later than current → LicenseRenewed, no warning | ✅ Happy path |
| 4b | Renewal key expires earlier than current → "This key expires on [date], earlier than your current license ([date]). Replace it?" | ✅ Happy path |
| 4c | Manager confirms on 4b → LicenseRenewed with new (earlier) expiry | ✅ Happy path |
| 4d | Manager cancels on 4b → No event, original license unchanged | ✅ Happy path |

---

### Rule 5: Per-Module Expiry — Independent Grace Periods

**Rule**: Each module expires and degrades independently. A practice may have scheduling in Active and recall in Degraded simultaneously. The status and write access of each module is evaluated independently.

| # | Example | Type |
|---|---------|------|
| 5a | scheduling expires, recall still active → scheduling Degraded, recall fully operational | ✅ Happy path |
| 5b | Degraded scheduling: view existing appointments → Displayed | ✅ Happy path |
| 5c | Degraded scheduling: create new appointment → Blocked; recall write access unaffected | ❌ Negative path |
| 5d | scheduling expires with grace_period_days=90 → 90 days before LicenseExpired | ✅ Happy path |
| 5e | recall expires with grace_period_days=30 → 30 days before LicenseExpired | ✅ Happy path |
| 5f | scheduling expires with grace_period_days=0 → LicenseExpired immediately (no Degraded) | ✅ Edge case |

---

### Rule 6: Grace Period Exhausted → Expired

**Rule**: When `now > expires_at + grace_period_days` for a module, that module transitions to Expired on startup. Read access unchanged. Write blocked.

| # | Example | Type |
|---|---------|------|
| 6a | Launch after grace exhausted → LicenseExpired (module: scheduling) | ✅ Happy path |
| 6b | Expired: view patient record → Displayed | ✅ Happy path |
| 6c | Expired: view schedule → Displayed | ✅ Happy path |
| 6d | Expired: create appointment → Blocked | ❌ Negative path |
| 6e | Renewal during Expired → LicenseRenewed, Active on next startup | ✅ Happy path |

---

### Rule 7: Progressive Warning System

**Rule**: Warnings appear in the status banner as expiry approaches. By the time a module expires, the practice has been watching it come. No surprise.

| # | Example | Type |
|---|---------|------|
| 7a | 31+ days to expiry → No banner for that module | ✅ Happy path |
| 7b | 30 days to expiry → Subtle banner: "Scheduling expires in 30 days" | ✅ Happy path |
| 7c | 14 days to expiry → Prominent banner: "Scheduling expires in 14 days — renew soon" | ✅ Happy path |
| 7d | 7 days to expiry → Urgent banner: "Scheduling expires in 7 days" | ✅ Happy path |
| 7e | Eval: always shows countdown → "Trial — [N] days remaining" | ✅ Happy path |
| 7f | Banner never blocks navigation or data access | ✅ Happy path |

---

### Rule 8: Startup Enforces — 48h Check Informs

**Rule**: Write access restrictions apply at startup. A 48-hour background check updates the banner but never changes write access mid-session. A session that starts in Active continues in Active until next launch.

| # | Example | Type |
|---|---------|------|
| 8a | Module expires while app is running → 48h check updates banner to Degraded, write access unchanged for session | ✅ Happy path |
| 8b | 48h check fires → LicenseDegraded event recorded if newly degraded; banner updated | ✅ Happy path |
| 8c | App relaunched after 8a → Enforcement applies, write access restricted | ✅ Happy path |
| 8d | App left running for 3 days → Multiple 48h checks fire, banner keeps updating | ✅ Happy path |
| 8e | Session starts in Active (module valid) → 48h check cannot restrict writes during this session | ✅ Happy path |

---

### Rule 9: Clock Rollback Detection

**Rule**: If system clock is >24h before the last LicenseValidationSucceeded timestamp, ClockRollbackDetected is emitted and all write access is denied for the session.

| # | Example | Type |
|---|---------|------|
| 9a | Clock rolled back >24h → ClockRollbackDetected, all module writes denied | ✅ Happy path |
| 9b | Clock within 24h threshold → No rollback detected, proceeds normally | ✅ Boundary |
| 9c | Rollback: view patient record → Displayed (data is still theirs) | ✅ Happy path |
| 9d | Rollback: create appointment → Blocked with clock error banner | ❌ Negative path |
| 9e | Clock corrected, restart → Validates normally, write access restored | ✅ Happy path |
| 9f | First launch (no prior validation event) → No rollback check, proceeds normally | ✅ Edge case |

---

### Rule 10: Hardware Migration

**Rule**: A license key carries a specific practiceId. If entered on a machine with a different practiceId, it is rejected. Support provides a new key for the new practiceId.

| # | Example | Type |
|---|---------|------|
| 10a | Key for old machine on new machine → "This key was issued for a different installation" | ❌ Negative path |
| 10b | About/Support screen shows practiceId in copyable format | ✅ Happy path |
| 10c | Support provides new key for new practiceId → Activation succeeds | ✅ Happy path |

---

**Phase 2.3 Acceptance Criteria Review**: All open questions resolved (OQ-1 through OQ-7). Scenarios validated against License aggregate invariants. Ready for Phase 2.5 governance review.
