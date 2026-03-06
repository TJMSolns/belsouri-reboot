# Example Map: DM-1 — Staff/Provider Domain Model Correction

**Date**: 2026-03-06
**Phase**: 2.2 Example Mapping
**Feature**: Clinical configuration moves from Practice Setup (Provider aggregate) to Staff Management (StaffMember aggregate)

---

## Three Amigos Summary (2.1)

**PO perspective (Tony)**:
- Every person at the practice is a StaffMember. Clinical staff (dentist, hygienist, specialist) are StaffMembers with the Provider role and a ClinicalSpecialization.
- Registration happens once — on the Staff tab. Clinical configuration (type, offices, availability) is done from the same context.
- There should be no separate "Providers" section that manages the same person.
- Archiving should be simple: archiving a StaffMember with Provider role means they are gone from scheduling too. No more "archived staff but active provider" split.

**Dev perspective (Claude)**:
- `provider_id` is replaced by `staff_member_id` in all scheduling commands and projections.
- Clinical commands (SetProviderType, AssignProviderToOffice, etc.) require the Provider role to be held. The role is the gate.
- Events move from the Practice Setup event stream to the Staff Management event stream.
- Staff Scheduling now reads clinical data from Staff Management projections, not Practice Setup.
- Pre-production: DB wipe + rebuild is acceptable. No migration events needed.

**Tester perspective (Tony)**:
- What happens if PM assigns Provider role to a receptionist accidentally? She should NOT appear in scheduling until ClinicalSpecialization is set.
- What if Provider role is removed from someone who had availability configured? We don't want to lose all that config if the role is re-added. Config should be preserved but the person should be hidden from scheduling.
- Booking an appointment with an archived StaffMember should fail — one aggregate, one lifecycle.

**Open questions resolved**:
- [x] Provider role removal: **preserve clinical config** (availability stays, but provider is hidden from scheduling until role is re-added). Do not auto-clear.
- [x] Provider step in setup checklist: reads from Staff Management — active StaffMember with Provider role + ClinicalSpecialization set + ≥1 office assignment + ≥1 availability day.
- [x] Migration strategy: **DB wipe**. Pre-production, no real patient data. Rebuild cleanly.
- [x] C7 capability booking constraint (SCH-4): booking constraint reads `staff_member.clinical_specialization` instead of `provider.provider_type`. Same logic, different source column.
- [x] Archive semantics: archiving a StaffMember with Provider role = they disappear from scheduling. No separate ProviderArchived event needed. One event (StaffMemberArchived) covers both.

---

## Story

As a Practice Manager, I want to configure clinical staff from the Staff Management context so that all staff records are managed in one place and scheduling can find providers without crossing aggregate boundaries.

---

## Rules

### Rule C1: Provider role is required before clinical configuration commands are accepted

**Examples**:
- Given Maria holds only the Staff role, when PM tries to set her ClinicalSpecialization to Hygienist, then the command is rejected: "Maria Brown does not hold the Provider role. Assign it before configuring clinical details."
- Given Dr. Brown holds the Provider role, when PM sets ClinicalSpecialization to Dentist, then ProviderTypeSet is recorded.
- Given Dr. Brown holds both Staff and Provider roles, when PM assigns her to office Kingston, then ProviderAssignedToOffice is recorded.

**Questions**:
- [x] Can a StaffMember hold Provider role without a ClinicalSpecialization set? → Yes. The role unlocks the commands; the specialization must be set separately before they appear in scheduling.

---

### Rule C2: ClinicalSpecialization must be set before a provider appears in scheduling

**Examples**:
- Given Dr. Brown holds Provider role but has no ClinicalSpecialization set, when Scheduling queries providers at Kingston, then Dr. Brown is not returned.
- Given Dr. Brown holds Provider role and has ClinicalSpecialization Dentist set, when Scheduling queries providers at Kingston (after office assignment + availability), then Dr. Brown is returned as a Dentist.
- Given Dr. Brown holds Provider role with ClinicalSpecialization Hygienist, when booking a Root Canal (requires Dentist), then the booking is rejected on C7: "Dr. Brown is a Hygienist; this procedure requires a Dentist."

---

### Rule C3: Office assignment required before setting availability

Same as old Provider aggregate Rule PR2 — now on StaffMember.

**Examples**:
- Given Dr. Brown holds Provider role and is assigned to Kingston, when PM sets Monday 08:00–17:00 availability at Kingston, then ProviderAvailabilitySet is recorded.
- Given Dr. Brown holds Provider role and is NOT assigned to Kingston, when PM tries to set availability at Kingston, then rejected: "Dr. Brown is not assigned to Kingston. Assign them to this office first."

---

### Rule C4: No cross-office availability overlap on the same day

Same as old Provider aggregate Rule PR4.

**Examples**:
- Given Dr. Brown is available Monday 08:00–12:00 at Kingston, when PM sets Monday 13:00–17:00 at Montego Bay, then ProviderAvailabilitySet is recorded (non-overlapping).
- Given Dr. Brown is available Monday 08:00–14:00 at Kingston, when PM sets Monday 12:00–17:00 at Montego Bay, then rejected: "Dr. Brown has overlapping availability at Kingston on Monday (08:00–14:00)."

---

### Rule C5: Exceptions are provider-wide and override all availability

Same as old Provider aggregate Rule PR6.

**Examples**:
- Given Dr. Brown is assigned to Kingston and Montego Bay, when PM sets exception Dec 20–31 with reason "Holiday vacation", then ProviderExceptionSet is recorded and Dr. Brown is unavailable at both offices for that range.
- Given exception Dec 20–31 is set and Dr. Brown has 3 appointments in that range, when PM sets the exception, then it is recorded with a warning: "3 appointments exist in this date range — they will not be cancelled."

---

### Rule C6: Archiving a StaffMember with Provider role removes them from scheduling

**CHANGED from old model**: Previously required two separate archive actions (StaffMemberArchived + ProviderArchived). Now one action covers both.

**Examples**:
- Given Dr. Brown holds Provider role with availability configured at Kingston, when PM archives Dr. Brown, then StaffMemberArchived is recorded and Dr. Brown no longer appears in Scheduling queries.
- Given Dr. Brown is archived, when Scheduling queries providers at Kingston for Monday, then Dr. Brown is not returned.
- Given Dr. Brown is archived, when PM tries to book an appointment with Dr. Brown, then the booking is rejected: "Dr. Brown is not available (archived)."

---

### Rule C7: Provider role removal preserves clinical config but hides provider from scheduling

**NEW**: When the Provider role is removed, clinical data is NOT auto-cleared (contrast with archiving). The provider becomes invisible to scheduling until the role is re-added.

**Examples**:
- Given Dr. Brown holds Provider role with Monday availability at Kingston, when PM removes the Provider role from Dr. Brown, then RoleRemoved(Provider) is recorded AND Dr. Brown no longer appears in Scheduling queries.
- Given PM re-adds Provider role to Dr. Brown, when Scheduling queries providers at Kingston for Monday, then Dr. Brown reappears with their previously configured availability intact.
- Given Dr. Brown's Provider role is removed, when PM tries to set availability at Kingston, then rejected: "Dr. Brown does not hold the Provider role."

---

### Rule C8: Setup checklist provider step reads from Staff Management

**CHANGED**: Previously read from Practice Setup Provider aggregate. Now reads from Staff Management.

Completion criteria: at least one active StaffMember with ALL of: Provider role held + ClinicalSpecialization set + ≥1 office assignment + ≥1 availability day set.

**Examples**:
- Given Dr. Brown holds Provider role but ClinicalSpecialization is not set, when checklist evaluates, then the provider step is "incomplete".
- Given Dr. Brown holds Provider role, ClinicalSpecialization Dentist, is assigned to Kingston, but has no availability set, when checklist evaluates, then the provider step is "incomplete".
- Given Dr. Brown holds Provider role, ClinicalSpecialization Dentist, is assigned to Kingston, and has Monday availability set, when checklist evaluates, then the provider step is "complete".
- Given the above complete state, when Dr. Brown is archived, then the provider step reverts to "incomplete".
- Given the above complete state, when the Provider role is removed from Dr. Brown, then the provider step reverts to "incomplete".

---

## 2.3 Acceptance Criteria Review

- [x] All rules use ubiquitous language: StaffMember, Provider role, ClinicalSpecialization, ProviderTypeSet, ProviderAssignedToOffice, ProviderAvailabilitySet, ProviderExceptionSet, StaffMemberArchived — correct.
- [x] No legacy language: no "ProviderRegistered", no "Practice Setup Provider", no "provider_id" — correct.
- [x] Happy paths and key edge cases covered for each rule.
- [x] Migration question resolved: DB wipe (pre-production).
- [x] C7 capability booking constraint dependency noted.
- [x] Archive semantics clarified: one lifecycle, one event.
- [x] Role removal vs archive semantics clarified: preservation of config on role removal.
- [x] No unresolved questions remain.

---

## 2.5 Governance Verification

- [x] Example map created: `doc/scenarios/example-maps/dm1-staff-provider-merge-examples.md`
- [x] All 8 rules have at least one example (happy path) and at least one edge case
- [x] Scenarios use ubiquitous language correctly
- [x] No unresolved questions
- [x] BDD scenarios: `features/staff-management.feature` (clinical config section added), `features/practice-setup.feature` (Provider lifecycle section retired)

---

**Maintained By**: Tony + Claude
