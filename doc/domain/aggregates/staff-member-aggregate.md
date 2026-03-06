# StaffMember Aggregate

**Context**: Staff Management
**Last Updated**: 2026-03-06

---

## Purpose

A person who works at the practice in any capacity. StaffMember is the single aggregate for all practice personnel — non-clinical (receptionists, administrators) and clinical (dentists, hygienists, specialists) alike.

Staff members hold one or more roles (PracticeManager, Provider, Staff) and authenticate via a PIN for quick identity switching on a shared workstation.

Clinical staff (those with the Provider role) additionally have a ClinicalSpecialization, office assignments, weekly availability windows, and date exceptions stored on this aggregate. There is no separate Provider aggregate — clinical configuration lives here.

**DM-1 correction (2026-03-06)**: The previously separate Provider aggregate (Practice Setup context) has been retired and merged into StaffMember. All clinical fields, events, and commands that were on Provider are now on StaffMember.

---

## Fields

### Core Identity Fields

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| id | UUID | Yes | System-generated. Referenced as `staff_member_id` in all other aggregates. |
| name | String | Yes | Staff member's display name |
| phone | String? | No | Contact phone number |
| email | String? | No | Contact email address |
| preferred_contact_channel | PreferredContactChannel | No | Default: WhatsApp. Values: WhatsApp, SMS, Phone, Email. |
| roles | Set\<Role\> | Yes | At least one role must be held. Values: PracticeManager, Provider, Staff. |
| pin_hash | String? | No | Bcrypt or Argon2 hash of PIN. Null until SetPIN is called. Required before staff member can switch to active identity. |
| archived | bool | Yes | Default false. Archived staff members are hidden from active lists. |

### Clinical Configuration Fields (Provider role only)

These fields are null/empty for non-clinical staff. Setting them requires the Provider role to be held.

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| clinical_specialization | ClinicalSpecialization? | No | Null for non-clinical staff. Dentist, Hygienist, or Specialist. Must be set before the staff member appears in scheduling. |
| office_assignments | Set\<office_id\> | No | Offices this provider works at. Empty initially. |
| availability | List\<AvailabilityWindow\> | No | Weekly schedule per office. Empty initially. |
| exceptions | List\<AvailabilityException\> | No | Date-based overrides. Empty initially. |

### Value Objects

**Role**: PracticeManager | Provider | Staff

**PreferredContactChannel**: WhatsApp | SMS | Phone | Email (WhatsApp is default)

**ClinicalSpecialization**: Dentist | Hygienist | Specialist

**AvailabilityWindow**:
| Field | Type | Notes |
|-------|------|-------|
| office_id | UUID | Which office this window applies to |
| day_of_week | DayOfWeek | Monday through Sunday |
| start_time | HH:MM | Start of working window |
| end_time | HH:MM | End of working window. Must be after start. |

**AvailabilityException**:
| Field | Type | Notes |
|-------|------|-------|
| start_date | YYYY-MM-DD | First day of exception |
| end_date | YYYY-MM-DD | Last day of exception. Must be >= start. |
| reason | String? | Optional. E.g., "Vacation", "Conference" |

---

## Events

### Identity + Role Events

| Event | Fields | When |
|-------|--------|------|
| **StaffMemberRegistered** | staff_member_id, name, phone?, email?, preferred_contact_channel? | A new staff member is added to the practice |
| **PracticeManagerClaimed** | staff_member_id | First-run bootstrap: first person claims the Practice Manager role before any PM exists |
| **RoleAssigned** | staff_member_id, role | A role is added to a staff member's role set |
| **RoleRemoved** | staff_member_id, role | A role is removed from a staff member's role set |
| **PINSet** | staff_member_id, pin_hash | Staff member establishes their PIN for the first time |
| **PINChanged** | staff_member_id, pin_hash | Staff member replaces their existing PIN |
| **PINReset** | staff_member_id, reset_by_staff_member_id | Practice Manager clears a staff member's PIN (forgotten PIN recovery) |
| **StaffMemberArchived** | staff_member_id | Staff member is decommissioned; hidden from active lists |
| **StaffMemberUnarchived** | staff_member_id | Staff member is restored to active status |

### Clinical Configuration Events (Provider role only)

| Event | Fields | When |
|-------|--------|------|
| **ProviderTypeSet** | staff_member_id, clinical_specialization | Practice Manager sets or changes the clinical type |
| **ProviderAssignedToOffice** | staff_member_id, office_id | Practice Manager links provider to an office |
| **ProviderRemovedFromOffice** | staff_member_id, office_id | Practice Manager unlinks provider from an office |
| **ProviderAvailabilitySet** | staff_member_id, office_id, day_of_week, start_time, end_time | Practice Manager sets a working window |
| **ProviderAvailabilityCleared** | staff_member_id, office_id, day_of_week | Practice Manager removes a working window |
| **ProviderExceptionSet** | staff_member_id, start_date, end_date, reason? | Practice Manager blocks a date range (vacation, time off) |
| **ProviderExceptionRemoved** | staff_member_id, start_date, end_date | Practice Manager lifts a date block |

---

## Commands

### Identity + Role Commands

| Command | Input | Produces | Preconditions |
|---------|-------|----------|---------------|
| RegisterStaffMember | name, phone?, email?, preferred_contact_channel?, initial_role | StaffMemberRegistered + RoleAssigned | name not empty, valid role, requires active Practice Manager (except first run) |
| ClaimPracticeManagerRole | name | StaffMemberRegistered + PracticeManagerClaimed + RoleAssigned(PracticeManager) | No active Practice Manager exists |
| AssignRole | staff_member_id, role | RoleAssigned | Staff member exists and is active, does not already hold the role |
| RemoveRole | staff_member_id, role | RoleRemoved | Staff member exists and is active, holds the role, not removing PracticeManager from the last active PM |
| SetPIN | staff_member_id, new_pin | PINSet | Staff member exists, is active, has no PIN set yet |
| ChangePIN | staff_member_id, current_pin, new_pin | PINChanged | Staff member exists, is active, has a PIN, current_pin matches stored hash |
| ResetPIN | staff_member_id, reset_by_staff_member_id | PINReset | Target exists and is active; issuer holds PracticeManager role; target is not the issuer |
| ArchiveStaffMember | staff_member_id | StaffMemberArchived | Not already archived, not the last active Practice Manager |
| UnarchiveStaffMember | staff_member_id | StaffMemberUnarchived | Currently archived |

### Clinical Configuration Commands

| Command | Input | Produces | Preconditions |
|---------|-------|----------|---------------|
| SetProviderType | staff_member_id, clinical_specialization | ProviderTypeSet | Staff member exists, is active, holds Provider role |
| AssignProviderToOffice | staff_member_id, office_id | ProviderAssignedToOffice | Active, holds Provider role, office exists, not already assigned |
| RemoveProviderFromOffice | staff_member_id, office_id | ProviderRemovedFromOffice + ProviderAvailabilityCleared (× N days cleared) | Currently assigned to that office |
| SetProviderAvailability | staff_member_id, office_id, day, start, end | ProviderAvailabilitySet | Active, holds Provider role, assigned to office, end > start, no cross-office overlap on same day |
| ClearProviderAvailability | staff_member_id, office_id, day | ProviderAvailabilityCleared | Has availability for that office+day |
| SetProviderException | staff_member_id, start_date, end_date, reason? | ProviderExceptionSet | Active, holds Provider role, end_date >= start_date |
| RemoveProviderException | staff_member_id, start_date, end_date | ProviderExceptionRemoved | Exception exists for that range |

---

## Invariants

### Identity Invariants

1. **Name required**: StaffMember must have a non-empty name
2. **At least one role**: A StaffMember must hold at least one role at all times
3. **Last Practice Manager guard**: The system must always have at least one active StaffMember with the PracticeManager role. Blocked if violated:
   - ArchiveStaffMember on the last active Practice Manager
   - RemoveRole(PracticeManager) on the last active Practice Manager
4. **PIN required for identity switching**: A StaffMember can exist without a PIN, but cannot be set as the active identity until SetPIN is called
5. **Roles are not mutually exclusive**: Any combination of PracticeManager, Provider, and Staff is valid. Clinical staff hold Staff + Provider simultaneously.
6. **Archived staff members cannot be modified**: All commands except UnarchiveStaffMember are rejected if archived

### Clinical Configuration Invariants

7. **Provider role required for clinical config**: Clinical commands (SetProviderType, AssignProviderToOffice, etc.) are rejected if the staff member does not hold the Provider role
8. **Assignment before availability**: Provider must be assigned to an office before setting availability there
9. **No cross-office overlap**: Provider cannot have overlapping availability windows at different offices on the same day (e.g., Kingston 10AM–2PM and Montego Bay 12PM–4PM is invalid)
10. **Valid time ranges**: end_time > start_time for availability windows; end_date >= start_date for exceptions
11. **Removal clears availability**: Removing a provider from an office automatically clears all availability windows at that office
12. **Exceptions override availability**: A provider with an exception for a date range is unavailable regardless of weekly availability

---

## State Machine

```
stateDiagram-v2
    [*] --> Active : RegisterStaffMember / ClaimPracticeManagerRole
    Active --> Active : AssignRole / RemoveRole
    Active --> Active : SetPIN / ChangePIN / ResetPIN
    Active --> Active : SetProviderType / AssignProviderToOffice / RemoveProviderFromOffice
    Active --> Active : SetProviderAvailability / ClearProviderAvailability
    Active --> Active : SetProviderException / RemoveProviderException
    Active --> Archived : ArchiveStaffMember
    Archived --> Active : UnarchiveStaffMember
```

---

## PIN Authentication Model

PIN is used for **local quick switching** on a shared workstation — not for remote authentication or session management.

- When a staff member is registered, they have no PIN (cannot switch to active identity yet)
- SetPIN is called once to establish the PIN; ChangePIN replaces it; ResetPIN clears it (PM action for forgotten PIN recovery)
- The PIN is hashed at the command layer before the event is stored; the raw PIN never enters the event store
- "Switching" to active identity: the application asks for a PIN, verifies it against the stored hash, and records the current active identity in application state (not in the event store)
- Identity switching does not produce a domain event — it is a session concern, not a domain concern

[CONFIRMED — Tony 2026-03-04] IdentitySwitched is NOT a domain event.

---

## First-Run Bootstrap

On a fresh installation, the first person must claim the Practice Manager role before any configuration can proceed. `ClaimPracticeManagerRole`:

1. Bypasses the "requires active Practice Manager" precondition (none exists yet)
2. Creates the StaffMember
3. Emits PracticeManagerClaimed (distinct from RoleAssigned — clear audit trail)
4. Assigns the PracticeManager role

---

## Multi-Office Provider Example

```
Dr. Brown (StaffMember, roles: [Staff, Provider], specialization: Dentist):
  Assigned to: Kingston, Montego Bay
  Availability:
    Kingston:    Monday 8AM–5PM, Wednesday 8AM–5PM, Friday 8AM–5PM
    Montego Bay: Tuesday 9AM–4PM, Thursday 9AM–4PM
  Exceptions:
    Dec 20–31: "Holiday vacation"
```

This is the real pattern from Nico's practice — providers rotate between offices on different days.

---

## Cross-Context Usage

StaffMember identity is referenced by:
- **Staff Scheduling**: reads `staff_member_id`, availability, and exceptions to build ResolvedSchedule
- **Patient Scheduling**: references `staff_member_id` (formerly `provider_id`) in Appointment aggregate
- **All contexts**: Active StaffMember identity provides attribution for all write commands

---

## Design Decisions

- **Provider IS A StaffMember**: Clinical staff are not a separate aggregate. They are StaffMembers with the Provider role and clinical configuration. This removes cross-aggregate consistency boundary violations in scheduling.
- **Provider role as the gate**: Clinical commands require the Provider role. This makes the role meaningful — it is not just a display label; it unlocks clinical configuration.
- **Register, not Create**: Staff members are "registered" at a practice.
- **Claim, not Assign (for self-declaration)**: First-run bootstrap only — self-service role claim.
- **PIN not password**: For quick workstation switching. Not remote authentication.
- **Thin base, rich clinical extension**: Non-clinical staff need only the core fields. Clinical staff are the same aggregate with an optional clinical extension.
- **Archive/Unarchive, not Delete**: Append-only event store.
- **Last PM guard**: Enforced at command layer to prevent role vacancy.

---

**Maintained By**: Tony + Claude
