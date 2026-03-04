# Event Storming: Staff Management

**Context**: Staff Management
**Date**: 2026-03-03
**Participants**: Tony (Product Owner), Claude (Developer)

**Sources**: Three Amigos session decisions (2026-03-03), Caribbean dental practice realities, Belsouri project MVP scope definition.

---

## Purpose

Staff Management is the thin bounded context that manages the identity, roles, and PIN-based quick switching for all people who work at the practice. It is the entry point for knowing "who is acting on the application right now."

This is not an HR system. It does not handle staff scheduling, time-off requests, payroll, or attendance. MVP scope is limited to: registering staff members, assigning roles, enabling PIN-based identity switching, and archiving staff members.

Staff Management is upstream of all other contexts that need to know who is performing an action.

---

## Domain Events

### StaffMember Lifecycle

| # | Event | Trigger | Fields | Notes |
|---|-------|---------|--------|-------|
| 1 | **StaffMemberRegistered** | Practice Manager registers a new staff member | staff_member_id, name, phone?, email?, preferred_contact_channel? | First event in a staff member's lifecycle. Staff member is active but cannot authenticate until PIN is set. |
| 2 | **StaffMemberArchived** | Practice Manager archives a staff member | staff_member_id | Blocked if staff member is the last active Practice Manager. Historical records preserved. |
| 3 | **StaffMemberUnarchived** | Practice Manager restores an archived staff member | staff_member_id | Staff member returns to active status with prior roles intact. |

### Role Assignment

| # | Event | Trigger | Fields | Notes |
|---|-------|---------|--------|-------|
| 4 | **RoleAssigned** | Staff member claims or is assigned a role | staff_member_id, role | Roles: PracticeManager, Provider, Staff. A staff member can hold multiple roles. Roles are self-declared at registration. |
| 5 | **RoleRemoved** | Practice Manager removes a role from a staff member | staff_member_id, role | Blocked if removing PracticeManager from the last active Practice Manager. |
| 6 | **PracticeManagerClaimed** | A staff member declares themselves Practice Manager during first-run setup | staff_member_id | Special event for the first-run bootstrap flow where no Practice Manager exists yet. After this event, further role changes require an existing Practice Manager. |

### PIN Management

| # | Event | Trigger | Fields | Notes |
|---|-------|---------|--------|-------|
| 7 | **PINSet** | Staff member establishes their PIN for the first time | staff_member_id, pin_hash | PIN is hashed before storage (bcrypt or Argon2). Required before staff member can be set as the active identity. |
| 8 | **PINChanged** | Staff member replaces their existing PIN | staff_member_id, pin_hash | New hash replaces old. No PIN history enforced at MVP. |

**Total: 8 events** across 3 temporal groups.

---

## Commands

| Command | Actor | Input | Produces | Preconditions |
|---------|-------|-------|----------|---------------|
| RegisterStaffMember | Practice Manager (or self on first run) | name, phone?, email?, preferred_contact_channel?, initial_role | StaffMemberRegistered + RoleAssigned | name not empty, valid role |
| ClaimPracticeManagerRole | Unregistered person (first-run only) | name | StaffMemberRegistered + PracticeManagerClaimed + RoleAssigned(PracticeManager) | No active Practice Manager exists |
| AssignRole | Practice Manager | staff_member_id, role | RoleAssigned | Staff member exists and is active, role not already held |
| RemoveRole | Practice Manager | staff_member_id, role | RoleRemoved | Staff member exists and is active, holds the role, not the last active Practice Manager if removing PracticeManager role |
| SetPIN | StaffMember (self) | staff_member_id, new_pin | PINSet | Staff member exists, active, no PIN set yet |
| ChangePIN | StaffMember (self) | staff_member_id, current_pin, new_pin | PINChanged | Staff member exists, active, PIN already set, current_pin verifies |
| ArchiveStaffMember | Practice Manager | staff_member_id | StaffMemberArchived | Not already archived, not the last active Practice Manager |
| UnarchiveStaffMember | Practice Manager | staff_member_id | StaffMemberUnarchived | Currently archived |

---

## Aggregate Candidates

### 1. StaffMember

The person who works at the practice. Has an identity (name, contact info), one or more roles, and a PIN for local identity switching.

**Key invariant**: At least one active StaffMember with the PracticeManager role must always exist.

**Identity**: staff_member_id (UUID)

---

## Temporal Flows

### Flow 1: First-Run Bootstrap

```
App installs fresh ->
  No staff members exist ->
  First person at the keyboard claims Practice Manager role:
    ClaimPracticeManagerRole("Dr. Spence") ->
      StaffMemberRegistered (staff_member_id=X, name="Dr. Spence") ->
      PracticeManagerClaimed ->
      RoleAssigned (PracticeManager)
  SetPIN(X, "1234") ->
    PINSet ->
  Staff Management step of setup checklist is now complete
```

### Flow 2: Registering a New Staff Member

```
Practice Manager registers a receptionist:
  RegisterStaffMember("Maria", initial_role=Staff) ->
    StaffMemberRegistered (staff_member_id=Y, name="Maria") ->
    RoleAssigned (Staff)
  Maria sets her own PIN:
    SetPIN(Y, "5678") -> PINSet
  Maria can now switch to active identity using PIN
```

### Flow 3: Provider Role Assignment

```
Dr. Brown is registered as a clinical provider:
  RegisterStaffMember("Dr. Brown", initial_role=Provider) ->
    StaffMemberRegistered + RoleAssigned (Provider)
  Dr. Brown is also made Practice Manager:
    AssignRole(Dr. Brown, PracticeManager) -> RoleAssigned (PracticeManager)
  Dr. Brown now holds both Provider and PracticeManager roles simultaneously.
  In Practice Setup, a separate Provider aggregate is created referencing Dr. Brown's staff_member_id.
```

### Flow 4: Staff Member Archiving (Safe Case)

```
Receptionist Maria leaves the practice:
  ArchiveStaffMember(Y) -> StaffMemberArchived
  Maria hidden from active lists.
  Historical records of her actions preserved.
  Other active staff members remain unaffected.
```

### Flow 5: Archiving Blocked — Last Practice Manager

```
Practice has only one Practice Manager (Dr. Spence):
  ArchiveStaffMember(Dr. Spence) -> Rejected:
    "Cannot archive the last active Practice Manager.
     Assign the Practice Manager role to another staff member first."
```

### Flow 6: PIN Change

```
Staff member wants to update their PIN:
  ChangePIN(staff_member_id, current_pin="1234", new_pin="5678") ->
    [System verifies current_pin hash matches stored hash] ->
    PINChanged (new pin_hash stored)
```

---

## Hotspots and Open Questions

### Resolved (from Three Amigos session, 2026-03-03)

- [x] **Roles available at MVP?** -> PracticeManager, Provider, Staff. Non-clinical Staff role is separate from Provider role.
- [x] **Can one person hold multiple roles?** -> Yes. A dentist who manages the practice holds both Provider and PracticeManager.
- [x] **Is role self-declared?** -> Yes, at registration. No approval workflow at MVP.
- [x] **Authentication model?** -> PIN-based quick switching on a shared workstation. No full session login/password.
- [x] **Staff scheduling in MVP?** -> No. Backlog. Staff Management is thin: identity + roles + PIN only.
- [x] **Is there a last-PM guard?** -> Yes. Cannot archive the last active Practice Manager.

### Open (flagged for Tony)

- [ ] **[OPEN QUESTION — Tony to confirm] ChangePIN flow**: Does changing a PIN require entering the current PIN (verification step), or is it free-form if you're already the active identity? Assumption: current PIN required for change, consistent with standard PIN-change UX.
- [ ] **[OPEN QUESTION — Tony to confirm] StaffMember contact info on MVP**: Is contact info (phone, email, preferred channel) collected at registration, or is it optional/deferred? Assumption: optional at registration; same pattern as Practice contact info.
- [ ] **[OPEN QUESTION — Tony to confirm] RoleRemoved event**: Is removing a role from a staff member an explicit command, or does reassignment replace roles? Assumption: explicit RoleRemoved command exists; roles are additive and must be explicitly removed.

---

## Cross-Context Dependencies

Staff Management events are consumed by downstream contexts:

| Downstream Context | Events / Data Consumed | Purpose |
|--------------------|----------------------|---------|
| **Practice Setup** | StaffMember identity (staff_member_id) | Provider aggregate in Practice Setup references a StaffMember by ID. The Practice Setup Provider is the scheduling resource view of the same person. |
| **Patient Management** | Active identity (who is acting) | Attribution of patient record changes to the active StaffMember |
| **Patient Scheduling** | Active identity (who is acting) | Attribution of appointment changes |
| **All contexts** | Active StaffMember identity | Every command has an actor — the active identity comes from Staff Management |

Staff Management does **not** consume events from other contexts. It is upstream of all contexts (except Licensing, which gates the application entirely).

---

## Relationship to Practice Setup Provider Aggregate

A `Provider` in Practice Setup (a scheduling resource) and a `StaffMember` with the Provider role in Staff Management represent the **same person from two different perspectives**:

- **Staff Management** perspective: "Dr. Brown is a person with a name, contact info, a PIN, and roles (Provider + PracticeManager)."
- **Practice Setup** perspective: "Dr. Brown is a scheduling resource assigned to offices with availability windows."

These are separate aggregates in separate contexts. The Practice Setup Provider aggregate stores a `staff_member_id` field that references the StaffMember. This is a foreign key by convention, not enforced by the event store -- referential integrity is checked by the Practice Setup command handler at the time of `RegisterProvider`.

**Consequence**: Registering a provider in Practice Setup requires the corresponding StaffMember (with Provider role) to exist in Staff Management first. Staff Management is upstream.

---

**Phase 1 Discovery: COMPLETE** (2026-03-03). Ceremonies 1.1-1.5 completed in this session.

### Governance Verification (1.5)

- [x] Event storming artifact exists for Staff Management context (this document)
- [x] All domain terms added to ubiquitous language glossary (StaffMember, Role, PIN, PracticeManager role, Provider role, Staff role, Register, Claim, Set PIN, Change PIN, Switch, Archive)
- [x] Aggregate doc exists for StaffMember (`doc/domain/aggregates/staff-member-aggregate.md`)
- [x] Context map updated with Staff Management context and its upstream relationships
- [x] Language is consistent across all artifacts (no banned terms: login, password, user account)

**PASS** — Phase 1 governance verification complete for Staff Management.
