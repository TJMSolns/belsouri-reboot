# PDR-001: Licensing Model

**Status**: Accepted
**Date**: 2026-03-03
**Deciders**: Tony (Product Owner)
**Category**: Product

---

## Guiding Principle

> "The data is always yours and you should never be blocked from it. But if you ain't paying, you should not be getting value out of it."

This principle governs every licensing decision in this document.

---

## Context

Belsouri is a commercial dental practice management application targeting Caribbean dental practices. It must protect business revenue while functioning fully offline in environments with unreliable connectivity, frequent power outages, and periodic weather-related disruptions.

A licensing system is required that:

- Controls access to paid features per module
- Supports a 30-day evaluation period for new installs
- Works offline indefinitely after license issuance
- Never blocks a practice from accessing their own data
- Resists casual circumvention without internet connectivity
- Accommodates Caribbean operational realities (weather events, slow recovery)

This PDR captures product decisions. Technical implementation decisions are in ADR-002.

---

## Decisions

### Decision 1: Machine-Bound Licenses

**Decision**: Licenses are bound to a specific machine via a stable machine identifier (MachineGuid on Windows). The PracticeId is derived from `SHA-256(machineId || installDate)` and embedded in every license payload.

**Rationale**:
- Dental practices are stable environments — the practice computer rarely changes
- Machine binding prevents license sharing between practices without server involvement
- MachineGuid survives NIC replacement and Windows Update (not OS reinstall)
- Simpler than user-account-based licensing (no authentication UX required)
- No server call needed to validate machine binding — all checked locally

**Known Limitation**: Major hardware changes (motherboard, new PC) change the machineId, generating a new PracticeId and requiring re-issuance. MVP resolution: manual re-issuance via email to Tony. Post-MVP: self-service support portal.

**Known Limitation**: OS reinstall on the same hardware preserves MachineGuid (Windows registry). This is desirable — same practice, same hardware, same license key still works.

---

### Decision 2: 30-Day Pre-Signed Eval Period

**Decision**: A pre-signed wildcard eval token is embedded in the binary at release time. This token has `license_type: "eval"` and `max_duration_days: 30`. On first run, the app validates this token and records the install date as the start of the eval period.

**Rationale**:
- Eval must work fully offline — no server call required to start
- Pre-signed by the License Server using its Ed25519 private key
- Same validation code path as paid licenses — no special-casing in business logic
- 30-day duration is standard for evaluation software
- Eval token binds to PracticeIdentity on first use, preventing sharing

**What eval includes**: All modules (full access for 30 days), no grace period after expiry.

**What happens at eval expiry**: Each module transitions to Expired. A status banner appears. Application enters read-only mode. No new records can be created.

---

### Decision 3: Per-Module Licensing with Independent Expiry

**Decision**: License payloads contain a list of modules, each with its own `expires_at` and `grace_period_days`. Access to each module is evaluated independently.

**Rationale**:
- A practice may renew critical modules while letting non-critical ones lapse
- The "value" principle applies per-module: a module that is paid up should not be blocked because another module has expired
- Marketing defines bundles server-side; the app just checks module membership
- Per-module expiry enables flexible pricing without app code changes

**Module definitions (MVP)**:

| Module | What It Gates | Grace Period |
|--------|--------------|-------------|
| `scheduling` | Practice Setup, Patient Management, Staff Scheduling, Patient Scheduling | 90 days |

**Future modules (not MVP)**:

| Module | What It Gates | Grace Period |
|--------|--------------|-------------|
| `charting` | Clinical Charting, Treatment Planning | 90 days |
| `recall` | Recall & Outreach, Appointment Reminders | 30 days |
| `imaging` | Imaging & Capture | 30 days |
| `portability` | Patient Portability | 30 days |
| `reporting` | Reporting & Insights | 30 days |

**Critical vs. non-critical**: Modules where loss of write access would directly harm patient care (scheduling, charting) get 90-day grace periods. Administrative/outreach modules get 30 days. This classification is enforced by the grace period set on the License Server at issuance — the app does not hardcode criticality.

---

### Decision 4: Degraded Mode (Per Module)

**Principle**: "Data is always yours — you should never be blocked from it. But if you ain't paying, you should not be getting value out of it."

**Definition**: When a module's expiry passes but it is within its grace period, that module enters Degraded Mode.

**In Degraded Mode for a module, allowed (read-only)**:
- View all existing records belonging to that module
- View all configuration
- Print or export existing records

**In Degraded Mode for a module, not allowed**:
- Create new records gated by that module
- Modify existing records gated by that module

**Modules not in Degraded Mode**: Unaffected. A practice with `scheduling` in Degraded Mode and `recall` still active has full write access to recall features.

**After grace period expires (Expired)**: Same read-only access. The data is still theirs. Enforcement is identical to Degraded — the difference is only messaging (grace period remaining vs. fully expired).

**Data access is never revoked**: Even a fully expired module never blocks reading its data. This is unconditional.

---

### Decision 5: Status Banner — Not a Blocking Prompt

**Decision**: License status is communicated via a persistent **status banner** in the UI. The banner is never modal, never blocks navigation, and never interrupts workflow.

**Banner behavior by status**:

| Module Status | Banner |
|--------------|--------|
| Active, >30 days remaining | No banner |
| Active, 30 days remaining | Subtle: "Scheduling expires in 30 days" |
| Active, 14 days remaining | Prominent: "Scheduling expires in 14 days — renew soon" |
| Active, 7 days remaining | Urgent: "Scheduling expires in 7 days" |
| Degraded (grace period) | Warning: "Scheduling has expired. [N] days to renew before read-only. Renew now." |
| Expired | Persistent: "Scheduling has expired. Renew to restore scheduling. [View data]" |
| Eval | Subtle: "Trial — [N] days remaining" |

**The banner links to the license activation screen. It never blocks access to data.**

---

### Decision 6: Progressive Warning System

**Decision**: The app warns of approaching expiry at 30 days, 14 days, and 7 days before each module's `expires_at`. Warnings are computed from the license projection — no separate warning events are stored.

**Rationale**: By the time a module expires, the practice has seen it coming for weeks. The transition is never a surprise.

---

### Decision 7: Startup Enforcement + 48-Hour In-Session Check

**Decision**:
- **Startup**: Full license validation runs on every launch. Status transitions (Active → Degraded, Degraded → Expired) are enforced here. Write access restrictions apply from this point.
- **48-hour in-session check**: While the app is running, a background check fires every 48 hours. It updates the status banner and warning state but does **not** change write access mid-session.

**Rationale**: Mid-session enforcement is jarring and creates surprises. A session that begins in Active mode continues in Active mode until the next launch. The 48h check ensures long-running sessions (overnight, across days) still surface warnings so nothing sneaks up.

**If the 48h check detects a module has crossed into Degraded**: The banner updates to reflect this. A `LicenseDegraded` event is recorded. Write access restriction applies at next startup.

---

### Decision 8: Renewal Key with Earlier Expiry

**Decision**: If the Practice Manager enters a renewal key whose `expires_at` is earlier than the current active license's `expires_at` for that module, the app warns and asks for confirmation before accepting.

**Message**: "This key expires on [date], which is earlier than your current license ([current expires_at]). Do you want to replace it?"

**Rationale**: Mistakes happen. The app should be fair, not a hard-ass. Warn and let the Practice Manager decide rather than silently accepting or outright rejecting.

---

### Decision 9: Pause / Rollover Policy

**Decision**: When a practice is closed for ≥14 consecutive days (weather event, disaster, extended outage), billing pauses and the license is extended. This is handled **entirely by the License Server** — a new token is issued with a pushed-out `expires_at`. The app validates the new token normally. No special pause logic in the app.

**Force majeure**: No cap on pause duration during confirmed disaster events (hurricane, flooding, government-mandated closure). Extensions match actual recovery timelines.

**Rationale**: Caribbean operational reality. A weather event that knocks out infrastructure for 3 weeks should not result in a lapsed license. The pause policy preserves trust and financial predictability.

---

### Decision 10: Reinstall and Hardware Migration

| Scenario | MachineId | PracticeId | License Key | Support Needed |
|----------|-----------|------------|-------------|----------------|
| OS reinstall, same hardware | Same (MachineGuid preserved) | Same | Still valid | No |
| New PC / motherboard | New | New | Invalid (PracticeId mismatch) | Yes — re-issuance |
| NIC replacement | Same (MachineGuid not NIC-based) | Same | Still valid | No |

**MVP re-issuance process**: Practice contacts Tony, provides old and new PracticeId, Tony issues a new license for the new PracticeId.

---

### Decision 11: Anti-Rollback (Clock Manipulation)

**Decision**: On every startup, the app checks that the system clock is not more than 24 hours before the last recorded validation timestamp. If rollback is detected, all module write access is denied for the session. Data remains readable (data is always yours). Clears automatically when clock is corrected.

---

## Alternatives Considered

### User-Account-Based Licensing
Rejected: Requires authentication UX and internet for login. Incompatible with offline-first requirement.

### Honor-Based (No Enforcement)
Rejected: Commercial product requires basic revenue protection.

### Hardware Dongle
Rejected: Not practical for Caribbean distribution. USB dongles can be lost or damaged.

### Blocking Renewal Prompts (Modal Dialogs)
Rejected: Interrupts workflow. Violates the "data is always yours" principle. A banner communicates status without blocking access.

---

## Consequences

**Positive**:
- Fully offline after license issuance
- Zero-friction evaluation (no sign-up, no internet required)
- Per-module expiry and grace periods accommodate Caribbean realities
- Progressive warnings eliminate surprises
- Startup-only enforcement prevents mid-session disruption
- Data is always readable regardless of license status

**Negative**:
- Hardware migration requires manual support intervention
- Clock issues on old hardware may trigger false anti-rollback alerts
- No remote revocation — once issued, a paid license cannot be recalled before expiry

---

**Reviewed By**: Tony (Product Owner)
**Related**: ADR-002 (technical cryptography decisions), Licensing aggregate docs
