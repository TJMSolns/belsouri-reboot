# StaffMember Aggregate

**Context**: Staff Management
**Last Updated**: 2026-03-03

---

## Purpose

A person who works at the practice in any capacity. Staff members hold one or more roles (PracticeManager, Provider, Staff) and authenticate via a PIN for quick identity switching on a shared workstation.

Staff Management is thin by design. It is not an HR system. It does not track hours, schedules, leave, or payroll. Its sole responsibilities are: registering who works here, assigning roles, enabling PIN-based switching, and archiving staff who leave.

Staff members with the Provider role also have a corresponding Provider aggregate in Practice Setup. The two aggregates represent the same person from different perspectives: Staff Management owns identity and roles; Practice Setup owns scheduling resources.

---

## Fields

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| id | UUID | Yes | System-generated. Referenced as `staff_member_id` in other aggregates. |
| name | String | Yes | Staff member's display name |
| phone | String | No | Contact phone number |
| email | String | No | Contact email address |
| preferred_contact_channel | PreferredContactChannel | No | Default: WhatsApp. Values: WhatsApp, SMS, Phone, Email. |
| roles | Set\<Role\> | Yes | At least one role must be claimed. Values: PracticeManager, Provider, Staff. |
| pin_hash | String? | No | Bcrypt or Argon2 hash of PIN. Null until SetPIN is called. Required before staff member can switch to active identity. |
| archived | bool | Yes | Default false. Archived staff members are hidden from active lists. |

### Value Objects

**Role**: PracticeManager | Provider | Staff

**PreferredContactChannel**: WhatsApp | SMS | Phone | Email (WhatsApp is default)

---

## Events

| Event | Fields | When |
|-------|--------|------|
| **StaffMemberRegistered** | staff_member_id, name, phone?, email?, preferred_contact_channel? | A new staff member is added to the practice |
| **PracticeManagerClaimed** | staff_member_id | First-run bootstrap: first person claims the Practice Manager role before any PM exists |
| **RoleAssigned** | staff_member_id, role | A role is added to a staff member's role set |
| **RoleRemoved** | staff_member_id, role | A role is removed from a staff member's role set |
| **PINSet** | staff_member_id, pin_hash | Staff member establishes their PIN for the first time |
| **PINChanged** | staff_member_id, pin_hash | Staff member replaces their existing PIN |
| **PINReset** | staff_member_id, reset_by_staff_member_id | Practice Manager clears a staff member's PIN (forgotten PIN recovery — no current PIN required) |
| **StaffMemberArchived** | staff_member_id | Staff member is decommissioned; hidden from active lists |
| **StaffMemberUnarchived** | staff_member_id | Staff member is restored to active status |

---

## Commands

| Command | Input | Produces | Preconditions |
|---------|-------|----------|---------------|
| RegisterStaffMember | name, phone?, email?, preferred_contact_channel?, initial_role | StaffMemberRegistered + RoleAssigned | name not empty, valid role, requires active Practice Manager (except on first run via ClaimPracticeManagerRole) |
| ClaimPracticeManagerRole | name | StaffMemberRegistered + PracticeManagerClaimed + RoleAssigned(PracticeManager) | No active Practice Manager exists in the system |
| AssignRole | staff_member_id, role | RoleAssigned | Staff member exists and is active, does not already hold the role |
| RemoveRole | staff_member_id, role | RoleRemoved | Staff member exists and is active, holds the role, not removing PracticeManager from the last active Practice Manager |
| SetPIN | staff_member_id, new_pin | PINSet | Staff member exists, is active, has no PIN set yet |
| ChangePIN | staff_member_id, current_pin, new_pin | PINChanged | Staff member exists, is active, has a PIN, current_pin hash matches stored hash |
| ResetPIN | staff_member_id, reset_by_staff_member_id | PINReset | Target staff member exists and is active; issuer holds PracticeManager role and is active; target staff member is not the issuer (PMs use ChangePIN for their own PIN) |
| ArchiveStaffMember | staff_member_id | StaffMemberArchived | Not already archived, not the last active Practice Manager |
| UnarchiveStaffMember | staff_member_id | StaffMemberUnarchived | Currently archived |

---

## Invariants

1. **Name required**: StaffMember must have a non-empty name
2. **At least one role**: A StaffMember must hold at least one role at all times (cannot remove the last role)
3. **Last Practice Manager guard**: The system must always have at least one active StaffMember with the PracticeManager role. The following are blocked if they would violate this:
   - ArchiveStaffMember on the last active Practice Manager
   - RemoveRole(PracticeManager) on the last active Practice Manager
4. **PIN required for identity switching**: A StaffMember can exist without a PIN, but cannot be set as the active identity until a PIN is established via SetPIN
5. **Roles are not mutually exclusive**: A StaffMember may hold any combination of PracticeManager, Provider, and Staff roles simultaneously
6. **Archived staff members cannot be modified**: All commands except UnarchiveStaffMember are rejected if the staff member is archived

---

## State Machine

```
stateDiagram-v2
    [*] --> Active : RegisterStaffMember / ClaimPracticeManagerRole
    Active --> Active : AssignRole / RemoveRole
    Active --> Active : SetPIN / ChangePIN / ResetPIN
    Active --> Archived : ArchiveStaffMember
    Archived --> Active : UnarchiveStaffMember
```

---

## PIN Authentication Model

PIN is used for **local quick switching** on a shared workstation — not for remote authentication or session management.

- When a staff member is registered, they have no PIN (cannot switch to active identity yet)
- SetPIN is called once to establish the PIN; PINChanged replaces it thereafter; ResetPIN clears it (Practice Manager action for forgotten PIN recovery)
- The PIN is hashed at the command layer before the event is stored; the raw PIN never enters the event store
- "Switching" to active identity means: the application asks for a PIN, verifies it against the stored hash, and records the current active identity in application state (not in the event store)
- Identity switching itself does not produce a domain event (it is a session concern, not a domain concern)

[CONFIRMED — Tony 2026-03-04] IdentitySwitched is NOT a domain event. PIN-based switching is a UX/session concern. The event store records what changed in the domain, not who was active.

---

## First-Run Bootstrap

On a fresh installation with no staff members, the first person to use the application must establish themselves as the Practice Manager before any other configuration can proceed. This is handled by the `ClaimPracticeManagerRole` command, which:

1. Bypasses the "requires active Practice Manager" precondition (since none exists yet)
2. Creates the StaffMember
3. Emits PracticeManagerClaimed (a distinct event from RoleAssigned, for clear audit trail)
4. Assigns the PracticeManager role

The setup checklist's Staff Management step is not satisfied until this bootstrap is complete and the first Practice Manager's PIN is set.

---

## Relationship to Practice Setup Provider

A StaffMember with the Provider role is also represented as a Provider aggregate in Practice Setup. The relationship is:

- Staff Management owns: identity (name, contact info, PIN), roles, archive status
- Practice Setup owns: provider type (Dentist/Hygienist/Specialist), office assignments, availability windows, exceptions

The Practice Setup Provider aggregate stores the `staff_member_id` as a reference. When registering a provider in Practice Setup, the command validates that the referenced `staff_member_id` belongs to an active StaffMember with the Provider role.

Archiving a StaffMember does **not** automatically archive the Practice Setup Provider. These are independent lifecycle events. [CONFIRMED — Tony 2026-03-04] No automatic cascade at MVP. Archiving a StaffMember does not trigger archiving the related Provider. The Practice Manager handles each context independently.

---

## Cross-Context Usage

StaffMember identity is referenced by:
- **Practice Setup**: Provider aggregate stores `staff_member_id`
- **All contexts**: The active StaffMember identity provides attribution for commands (who performed this action)

---

## Design Decisions

- **Register, not Create**: Domain language mirrors Provider — staff members are "registered", not "created".
- **Claim, not Assign (for self-declaration)**: Roles are self-declared; "Claim" reflects the self-service nature.
- **PIN not password**: PIN is explicitly not a password. It is for quick workstation switching. Using the term "password" would imply remote auth, session management, and security infrastructure that does not exist in this MVP.
- **Thin context**: Staff Management does not own scheduling, time off, approvals, or HR data. These belong to future contexts or backlog items.
- **Archive/Unarchive, not Delete**: Append-only event store. Archived staff members have their historical attributions preserved.
- **Last PM guard**: Without this invariant, the Practice Manager role could become vacant, locking out all administrative actions. The guard is enforced at the command layer.

---

**Maintained By**: Tony + Claude
