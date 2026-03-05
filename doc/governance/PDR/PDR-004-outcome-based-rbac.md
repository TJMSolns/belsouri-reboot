# PDR-004: Outcome-Based Role-Based Access Control (Post-MVP)

**Status**: Proposed
**Date**: 2026-03-05
**Deciders**: Tony (Product Owner), Claude (Developer)
**Category**: Product Architecture
**Sprint**: Post-MVP

---

## The Idea (Tony's words)

> "After MVP we should think about identifying outcomes — add a patient, schedule an appointment, etc. — to create a complete inventory, then map each of our OOTB roles to if they are authed to do that outcome, then allow the PM to create new roles (like Office Manager) to create custom duties by role so they can assign roles to staff."

---

## What This Means

### Outcome Inventory

An **Outcome** is a named, atomic user action in the system. Examples:

| Domain | Outcome |
|---|---|
| Patient Management | Register Patient, Update Patient Demographics, Archive Patient, Add Patient Note, View Patient Record |
| Appointments | Book Appointment, Reschedule Appointment, Cancel Appointment, Complete Appointment, Mark No-Show |
| Practice Setup | Create Office, Register Provider, Define Procedure Type, Update Practice Details |
| Staff Management | Register Staff Member, Assign Role, Reset PIN, Archive Staff Member |
| Scheduling | View Schedule, View Roster, Plan Staff Shift, Cancel Staff Shift |
| Reports | View Call List, View Provider Schedule |

This is the **permission catalogue** — a complete, versioned inventory of everything the system can do.

### OOTB Role → Outcome Mapping

The three built-in roles ship with predefined outcome sets:

| Outcome | Practice Manager | Provider | Staff |
|---|---|---|---|
| Register Patient | ✓ | | ✓ |
| Book Appointment | ✓ | ✓ | ✓ |
| Complete Appointment | ✓ | ✓ | |
| Cancel Appointment | ✓ | | ✓ |
| Mark No-Show | ✓ | ✓ | |
| Archive Patient | ✓ | | |
| Register Staff Member | ✓ | | |
| Create Office | ✓ | | |
| Register Provider | ✓ | | |
| Plan Staff Shift | ✓ | | ✓ |
| View Schedule | ✓ | ✓ | ✓ |
| … | | | |

(Full mapping to be defined in Three Amigos before implementation.)

### Custom Roles (PM-Created)

A Practice Manager can define a new role — e.g. "Office Manager", "Head Receptionist", "Locum" — and assign it a subset of outcomes from the catalogue. Custom roles are practice-specific (not global).

Example:
- "Office Manager" = all Staff outcomes + Cancel Appointment + Archive Patient
- "Locum" = Book Appointment + Complete Appointment + View Schedule (no setup access)

### Role Assignment

Staff members can be assigned one or more roles (OOTB or custom). The union of all their roles' outcomes determines what they can do. This extends the existing `assign_role` / `remove_role` commands to support custom role names.

---

## Relationship to Existing Work

### SCH-6: Role-Based View Switching
SCH-6 (post-MVP, classified as Phase 1+2) is the UI layer of this same concept — toggling "view as Dentist" vs "view as Practice Manager." SCH-6 and PDR-004 should be designed together or PDR-004 should subsume SCH-6.

### Staff Management Bounded Context
Role assignment already exists (`RoleAssigned`, `RoleRemoved` events on StaffMember aggregate). Custom roles would extend this: `CustomRoleDefined`, `CustomRoleOutcomeGranted`, `CustomRoleOutcomeRevoked` events — likely as a new **Authorization** sub-context within Staff Management, or a new bounded context.

### Current Authorization Model (MVP)
At MVP, the system uses soft authorization: the UI shows/hides options by role, but the backend does not enforce role checks on most commands (exception: PM override in SCH-4b requires `actor_role` param). PDR-004 formalizes and hardens this.

---

## Why Ceremonies Are Required

This is a **new bounded context** or a significant extension of Staff Management. It requires:

1. **Phase 1**: Event Storming (outcome catalogue), Ubiquitous Language (Outcome, Role, Permission, Duty), Domain Modeling (Role aggregate, Outcome value object), Context Mapping (how Authorization interacts with every other context)
2. **Phase 2 per feature**: Three Amigos for OOTB mapping, custom role creation, UI enforcement points

Estimated ceremony effort: **2–3 sprint days** before implementation begins.

---

## Open Questions (for Three Amigos)

1. Is an Outcome enforced at the **backend** (command-level check) or **frontend** (UI visibility) or both? Backend enforcement is safer but requires passing actor identity to every command.
2. What happens when a PM removes an outcome from a role that a staff member currently relies on? Immediate effect or grace period?
3. Can a PM grant outcomes that exceed their own permissions? (Privilege escalation guard needed.)
4. Are OOTB role outcome mappings editable by PM, or fixed? (Fixed is safer for MVP of this feature; editable is more flexible.)
5. How does this interact with the PIN verification model? PINs identify the acting user — does the system need a "logged in" session concept for per-request authorization?
6. Should custom roles be visible across the whole practice or office-specific?

---

## Recommended Implementation Order (Post-MVP)

1. Run `/classify-work` — expected: new bounded context (Phase 1 required)
2. Phase 1 ceremonies: Event Storming → Ubiquitous Language → Domain Modeling → Context Mapping
3. Define outcome catalogue (exhaustive list, versioned)
4. Define OOTB role → outcome mapping (Tony signs off)
5. Implement backend outcome enforcement (auth middleware pattern or per-command actor check)
6. Implement custom role CRUD (PM-only)
7. Implement custom role assignment to staff
8. UI: hide/show features by role (extends SCH-6)

---

## Related

- `doc/governance/PDR/PDR-003-journey-aware-navigation.md` — navigation is role-aware
- `doc/domain/aggregates/staff-member-aggregate.md` — existing role assignment model
- SCH-6 in plan — role-based view switching (likely subsumed by this feature)
- `HOW-WE-WORK.md` — ceremony requirements for new bounded context

**Reviewed By**: Tony (Product Owner)
