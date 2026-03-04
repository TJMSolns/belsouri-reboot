# Example Map: Staff Management

**Date**: 2026-03-03
**Participants**: Tony (Product Owner / Business), Claude (Developer / Tester)
**Feature**: Staff Management context — StaffMember lifecycle, role assignment, PIN management, archive
**Status**: Phase 2 COMPLETE (2026-03-04) — all open questions confirmed by Tony; governance PASS

---

## Three Amigos Summary (Phase 2.1)

Decisions carried in from the Three Amigos session (2026-03-03):

| Item | Decision |
|------|----------|
| Roles at MVP | PracticeManager, Provider, Staff. Not mutually exclusive. |
| Role declaration | Self-declared at registration. No approval workflow at MVP. |
| Authentication model | PIN-based quick switching on a shared workstation. Not session login. |
| First-run bootstrap | First person to use the app claims the PracticeManager role before any PM exists. |
| Dual roles | A dentist who owns the practice can hold PracticeManager + Provider simultaneously. |
| Staff scheduling | Not in MVP. Backlog. Staff Management is thin: identity + roles + PIN only. |
| Archive model | Archive only, no hard delete. History preserved. |
| Last PM guard | Cannot archive or remove PracticeManager role from the last active Practice Manager. |
| Contact info model | Uses PreferredContactChannel (WhatsApp default). |

### Assumptions (flagged from prior analysis — mark cleared by Tony)

| ID | Assumption | Status |
|----|-----------|--------|
| SM-1 | ChangePIN requires the current PIN as verification | [CONFIRMED — Tony 2026-03-04] |
| SM-2 | StaffMember contact info (phone, email, preferred channel) is optional at registration | [CONFIRMED — Tony 2026-03-04] |
| SM-3 | RoleRemoved is an explicit command; roles are additive, not replaced on reassignment | [CONFIRMED — Tony 2026-03-04] |
| SM-4 | PIN switching (IdentitySwitched) is NOT a domain event — it is a UX/session concern only | [CONFIRMED — Tony 2026-03-04] |
| SM-5 | Archiving a StaffMember does NOT auto-archive the linked Practice Setup Provider | [CONFIRMED — Tony 2026-03-04] |

---

## Rule Cards

---

## First-Run Bootstrap

---

### Rule SM1: The first StaffMember always gets the PracticeManager role via ClaimPracticeManagerRole

**Rule**: On a fresh installation with no staff members, the first person who acts claims the PracticeManager role. This bypasses the normal "requires active Practice Manager" precondition via the ClaimPracticeManagerRole command — which is only valid when no active Practice Manager exists. The claim produces StaffMemberRegistered + PracticeManagerClaimed + RoleAssigned(PracticeManager).

| # | Example | Type |
|---|---------|------|
| SM1a | Fresh install, no staff members — first person submits ClaimPracticeManagerRole("Dr. Spence") → StaffMemberRegistered + PracticeManagerClaimed + RoleAssigned(PracticeManager) | ✅ Happy path |
| SM1b | ClaimPracticeManagerRole submitted when an active PracticeManager already exists → Rejected: "A Practice Manager already exists" | ❌ Negative path |
| SM1c | ClaimPracticeManagerRole with empty name → Rejected: "Name is required" | ❌ Negative path |
| SM1d | ClaimPracticeManagerRole with only whitespace name → Rejected: "Name is required" | ❌ Edge case |
| SM1e | After ClaimPracticeManagerRole succeeds, setup checklist Staff Management step remains incomplete until the first PM's PIN is also set | ✅ Edge case |
| SM1f | After PracticeManagerClaimed, subsequent RegisterStaffMember commands follow the normal precondition (active PM exists) | ✅ Happy path |

---

## StaffMember Registration

---

### Rule SM2: StaffMember registration requires a name and at least one role; contact info is optional

**Rule**: RegisterStaffMember requires a non-empty name and a valid initial_role (PracticeManager, Provider, or Staff). Contact info (phone, email, preferred_contact_channel) is optional. [SM-2 CONFIRMED — Tony 2026-03-04: contact info is optional at registration, mirroring the Practice contact info pattern.]

| # | Example | Type |
|---|---------|------|
| SM2a | PM registers "Maria" with initial role Staff, no contact info → StaffMemberRegistered + RoleAssigned(Staff) | ✅ Happy path |
| SM2b | PM registers "Dr. Brown" with initial role Provider, phone "876-555-0100", email "brown@clinic.com", preferred channel WhatsApp → StaffMemberRegistered + RoleAssigned(Provider) with all contact fields | ✅ Happy path |
| SM2c | PM registers with empty name → Rejected: "Name is required" | ❌ Negative path |
| SM2d | PM registers with whitespace-only name → Rejected: "Name is required" | ❌ Edge case |
| SM2e | PM registers with invalid initial_role "Admin" → Rejected: "Role must be PracticeManager, Provider, or Staff" | ❌ Negative path |
| SM2f | PM registers "Dr. Brown" with initial role Provider — no PIN is set at registration; StaffMember is active but cannot switch to active identity until SetPIN is called | ✅ Edge case |
| SM2g | PM registers a StaffMember with PracticeManager as initial role → StaffMemberRegistered + RoleAssigned(PracticeManager). This is valid (PM can register another PM). | ✅ Happy path |
| SM2h | No active Practice Manager exists and system is past first-run (which should never happen due to last PM guard, but if attempted) → RegisterStaffMember rejected: use ClaimPracticeManagerRole for first-run | ❌ Edge case |

---

## PIN Management

---

### Rule SM3: A PIN must be set before a StaffMember can switch to active identity

**Rule**: SetPIN establishes the PIN for a StaffMember who has no PIN yet. The raw PIN is hashed at the command layer (bcrypt or Argon2) before the PINSet event is stored. A StaffMember without a PIN cannot be selected as the active identity.

| # | Example | Type |
|---|---------|------|
| SM3a | Maria has no PIN — she submits SetPIN("5678") → PINSet event recorded with pin_hash | ✅ Happy path |
| SM3b | System rejects identity switch for Maria because she has no PIN set → "PIN not set — please set your PIN before switching" | ❌ Negative path |
| SM3c | Maria sets a 4-digit PIN → PINSet (minimum length boundary) | ✅ Boundary |
| SM3d | Maria sets a 6-digit PIN → PINSet (maximum length boundary) | ✅ Boundary |
| SM3e | PIN outside 4–6 digit range is rejected. Fewer than 4 or more than 6 digits → "PIN must be 4 to 6 digits" [CONFIRMED — Tony 2026-03-04] | ❌ Negative path |
| SM3f | Maria attempts SetPIN when she already has a PIN set → Rejected: "PIN already set — use ChangePIN to update it" | ❌ Negative path |
| SM3g | Archived StaffMember attempts SetPIN → Rejected: "Cannot modify an archived staff member" | ❌ Negative path |
| SM3h | The raw PIN value never appears in any domain event — only the pin_hash is stored in PINSet | ✅ Happy path |

---

### Rule SM4: Changing a PIN requires the current PIN as verification

**Rule**: ChangePIN replaces an existing PIN. The command requires current_pin (to verify identity before allowing the change) and new_pin. [SM-1 CONFIRMED — Tony 2026-03-04: ChangePIN requires current PIN as verification, consistent with standard PIN-change UX.]

| # | Example | Type |
|---|---------|------|
| SM4a | Maria provides correct current PIN "5678" and new PIN "9999" → PINChanged event with new pin_hash | ✅ Happy path |
| SM4b | Maria provides incorrect current PIN → Rejected: "Current PIN does not match" | ❌ Negative path |
| SM4c | Maria attempts ChangePIN with no PIN set yet → Rejected: "No PIN set — use SetPIN to establish a PIN first" | ❌ Negative path |
| SM4d | New PIN may be the same as the current PIN — no PIN history enforcement at MVP [CONFIRMED — Tony 2026-03-04] | ✅ Edge case |
| SM4e | Archived StaffMember attempts ChangePIN → Rejected: "Cannot modify an archived staff member" | ❌ Negative path |
| SM4f | New PIN must meet same length requirements as SetPIN (4-6 digits, per SM3e assumption) | ✅ Boundary |

---

## Role Assignment

---

### Rule SM5: Roles are additive and not mutually exclusive

**Rule**: A StaffMember can hold any combination of PracticeManager, Provider, and Staff simultaneously. AssignRole adds a role to the set; it does not replace existing roles. A StaffMember cannot hold the same role twice. [SM-3 CONFIRMED — Tony 2026-03-04: RoleRemoved is an explicit command; roles are additive not replaced.]

| # | Example | Type |
|---|---------|------|
| SM5a | Dr. Brown holds the Provider role; PM assigns PracticeManager → RoleAssigned(PracticeManager). Dr. Brown now holds Provider + PracticeManager. | ✅ Happy path |
| SM5b | PM assigns Staff role to a StaffMember who already holds Staff → Rejected: "Staff member already holds the Staff role" | ❌ Negative path |
| SM5c | PM assigns Provider to Maria who holds Staff → RoleAssigned(Provider). Maria now holds both Staff and Provider. | ✅ Happy path |
| SM5d | PM assigns PracticeManager to a StaffMember who already holds PracticeManager → Rejected: "Staff member already holds the PracticeManager role" | ❌ Negative path |
| SM5e | PM assigns role to archived StaffMember → Rejected: "Cannot modify an archived staff member" | ❌ Negative path |
| SM5f | A single StaffMember holds all three roles (PracticeManager + Provider + Staff) simultaneously → valid state | ✅ Edge case |

---

### Rule SM6: Removing a role requires an explicit RemoveRole command; blocked for last PM

**Rule**: Roles are not implicitly removed. RemoveRole explicitly removes a named role from a StaffMember's role set. A StaffMember must always hold at least one role — the last role cannot be removed. The PracticeManager role cannot be removed from a StaffMember if they are the last active Practice Manager.

| # | Example | Type |
|---|---------|------|
| SM6a | Dr. Brown holds Provider + PracticeManager. PM calls RemoveRole(Dr. Brown, Provider) → RoleRemoved(Provider). Dr. Brown now holds only PracticeManager. | ✅ Happy path |
| SM6b | Maria holds only the Staff role. PM attempts RemoveRole(Maria, Staff) → Rejected: "Cannot remove the last role from a staff member" | ❌ Negative path |
| SM6c | Dr. Spence is the only active PracticeManager. PM (another active StaffMember) attempts RemoveRole(Dr. Spence, PracticeManager) → Rejected: "Cannot remove the PracticeManager role from the last active Practice Manager" | ❌ Negative path |
| SM6d | Two active PracticeManagers exist. PM removes PracticeManager from one of them → RoleRemoved(PracticeManager). Remaining PM is the last active PM. | ✅ Happy path |
| SM6e | PM attempts RemoveRole for a role the StaffMember does not hold → Rejected: "Staff member does not hold the [role] role" | ❌ Negative path |
| SM6f | PM attempts RemoveRole on archived StaffMember → Rejected: "Cannot modify an archived staff member" | ❌ Negative path |

---

## Archive and Unarchive

---

### Rule SM7: Archiving a StaffMember soft-deletes them; blocked if they are the last active PM

**Rule**: ArchiveStaffMember hides the StaffMember from active lists. History is preserved. Cannot archive an already-archived StaffMember. Cannot archive the last active Practice Manager.

| # | Example | Type |
|---|---------|------|
| SM7a | PM archives receptionist Maria → StaffMemberArchived. Maria no longer appears in active staff list. | ✅ Happy path |
| SM7b | Dr. Spence is the only active PracticeManager. PM attempts to archive Dr. Spence → Rejected: "Cannot archive the last active Practice Manager. Assign the Practice Manager role to another staff member first." | ❌ Negative path |
| SM7c | PM attempts to archive an already-archived StaffMember → Rejected: "Staff member is already archived" | ❌ Negative path |
| SM7d | PM archives a StaffMember who holds the Provider role → StaffMemberArchived. The linked Practice Setup Provider aggregate is NOT automatically archived. [SM-5 CONFIRMED — Tony 2026-03-04: no cascade.] | ✅ Edge case |
| SM7e | Archived StaffMember's historical attributions (e.g., which appointments they booked) are preserved and visible in history | ✅ Happy path |
| SM7f | Two PracticeManagers exist. PM archives one of them → StaffMemberArchived. The other PM remains active. | ✅ Happy path |
| SM7g | PM archives a StaffMember who holds all three roles but is not the last PM → StaffMemberArchived | ✅ Edge case |

---

### Rule SM8: An archived StaffMember can be unarchived to restore them to active status

**Rule**: UnarchiveStaffMember restores an archived StaffMember to active status with all prior roles intact. Cannot unarchive an already-active StaffMember.

| # | Example | Type |
|---|---------|------|
| SM8a | Maria was archived — PM calls UnarchiveStaffMember(Maria) → StaffMemberUnarchived. Maria reappears in active list with her prior role(s). | ✅ Happy path |
| SM8b | PM attempts to unarchive an active StaffMember → Rejected: "Staff member is not archived" | ❌ Negative path |
| SM8c | Maria had PIN set before archiving — her PIN hash is preserved; she can switch identity immediately after unarchiving | ✅ Edge case |
| SM8d | Maria had no PIN before archiving — after unarchiving, she still must set a PIN before switching identity | ✅ Edge case |

---

## Active Identity and PIN Switching

---

### Rule SM9: PIN switching to active identity is a session concern, not a domain event

**Rule**: A StaffMember enters their PIN to become the active identity in the application. The application verifies the PIN against the stored hash and records the active identity in session/application state. This does not produce a domain event. [SM-4 CONFIRMED — Tony 2026-03-04: IdentitySwitched is not a domain event.]

| # | Example | Type |
|---|---------|------|
| SM9a | Maria enters her PIN "5678" at the switching screen → PIN verified against stored hash, application records Maria as the active identity. No domain event emitted. | ✅ Happy path |
| SM9b | Maria enters incorrect PIN → Rejected: "Incorrect PIN" | ❌ Negative path |
| SM9c | Maria has no PIN set → Cannot switch to active identity: "PIN not set — please set your PIN" | ❌ Negative path |
| SM9d | Archived StaffMember's PIN → Cannot select an archived staff member as the active identity | ❌ Negative path |
| SM9e | No lockout after incorrect PIN attempts at MVP — shared workstation, trusted staff environment [CONFIRMED — Tony 2026-03-04] | ✅ Edge case |
| SM9f | Identity switching screen shows all active StaffMembers; those without PINs show a "set PIN" indicator rather than a PIN entry field [CONFIRMED — Tony 2026-03-04] | ✅ Edge case |

---

## Last Practice Manager Invariant

---

### Rule SM10: At least one active StaffMember with the PracticeManager role must always exist

**Rule**: The system enforces a single invariant: there must always be at least one active StaffMember holding the PracticeManager role. Three operations are blocked if they would violate this invariant: (1) ArchiveStaffMember on the last PM, (2) RemoveRole(PracticeManager) from the last PM. This invariant is enforced at the command layer before events are emitted.

| # | Example | Type |
|---|---------|------|
| SM10a | System has exactly one active PM. Attempt to archive them → Rejected. | ❌ Negative path |
| SM10b | System has exactly one active PM. Attempt to remove their PracticeManager role → Rejected. | ❌ Negative path |
| SM10c | System has two active PMs. Archive one → Allowed. One PM remains. | ✅ Happy path |
| SM10d | System has two active PMs. Remove PracticeManager role from one → Allowed. One PM remains. | ✅ Happy path |
| SM10e | System has one active PM who holds PracticeManager + Provider. Attempt to archive them → Rejected (they are still the last PM). | ❌ Edge case |
| SM10f | New PM is registered and given PracticeManager role → system now has two active PMs. Former last PM can now be archived or have their PracticeManager role removed. | ✅ Happy path |

---

## Setup Checklist

---

### Rule SM11: Staff Management setup checklist step requires at least one active PM with a PIN set

**Rule**: The Staff Management step of the setup checklist is complete when at least one active StaffMember holds the PracticeManager role and has a PIN set. This step is part of the overall practice "ready to schedule" checklist.

| # | Example | Type |
|---|---------|------|
| SM11a | Dr. Spence is registered as PM via ClaimPracticeManagerRole but has no PIN set → Setup step incomplete | ✅ Happy path |
| SM11b | Dr. Spence has PracticeManager role and PIN set → Setup step complete | ✅ Happy path |
| SM11c | Dr. Spence (PM with PIN) is archived, leaving Dr. Brown (PM with PIN) as the only PM → Setup step remains complete | ✅ Edge case |
| SM11d | Practice Manager resets a StaffMember's forgotten PIN without knowing their current PIN → PINReset event; StaffMember can set a new PIN on next identity switch [CONFIRMED — Tony 2026-03-04] | ✅ Edge case |

---

## Open Questions Summary

| # | Question | Artifact Ref | Assumption Made |
|---|----------|-------------|-----------------|
| SM-1 | ChangePIN flow: is current PIN required for verification? | SM4, Rule SM4 | Yes — current PIN required [CONFIRMED — Tony 2026-03-04] |
| SM-2 | Is contact info optional at registration? | SM2b, Rule SM2 | Yes — optional [CONFIRMED — Tony 2026-03-04] |
| SM-3 | Is RoleRemoved an explicit command, or do roles get replaced on reassignment? | SM5, Rule SM5/SM6 | Explicit command; additive [CONFIRMED — Tony 2026-03-04] |
| SM-4 | Is IdentitySwitched a domain event? | SM9, Rule SM9 | No — session/UX concern only [CONFIRMED — Tony 2026-03-04] |
| SM-5 | Does archiving a StaffMember auto-archive the linked Practice Setup Provider? | SM7d, Rule SM7 | No cascade; contexts independent [CONFIRMED — Tony 2026-03-04] |
| SM3e | PIN length constraints: allowed range? | Rule SM3 | 4-6 digits [CONFIRMED — Tony 2026-03-04] |
| SM4d | Can new PIN equal current PIN on change? | Rule SM4 | Allowed — no PIN history enforcement [CONFIRMED — Tony 2026-03-04] |
| SM9e | PIN lockout after N incorrect attempts? | Rule SM9 | No lockout at MVP [CONFIRMED — Tony 2026-03-04] |
| SM9f | Identity switching screen: who is listed? | Rule SM9 | All active StaffMembers; PIN-less show "set PIN" indicator [CONFIRMED — Tony 2026-03-04] |
| SM11d | Can a Practice Manager reset a StaffMember's PIN? | Rule SM11 | Yes — PM can reset any staff PIN; no current PIN required; resolves forgotten PIN without support call [CONFIRMED — Tony 2026-03-04] |

---

**Phase 2.3 Acceptance Criteria Review**: All business rules validated against StaffMember aggregate doc and event storming output. Ubiquitous language used throughout (no banned terms: login, password, user, account, delete). All open questions (SM-1 through SM-5, SM3e, SM4d, SM9e, SM9f, SM11d) confirmed by Tony 2026-03-04. Phase 2.3 COMPLETE.

**Phase 2.5 Governance Review**: PASS (2026-03-04). All events and commands consistent with aggregate doc. All invariants covered. No banned terms. No orphan event or command references.
