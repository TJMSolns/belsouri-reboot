# Provider Aggregate — RETIRED

**Status**: RETIRED — 2026-03-06 (DM-1 domain model correction)

---

## Why This Aggregate Was Retired

Provider IS A StaffMember. Having a separate Provider aggregate in Practice Setup created a cross-aggregate consistency boundary violation: scheduling invariants about "is this provider working today?" required joining two independent aggregate streams, which is architecturally wrong in DDD.

**All clinical configuration has been moved to the StaffMember aggregate in the Staff Management context.**

---

## Migration

| Was | Is Now |
|-----|--------|
| Provider aggregate (Practice Setup) | StaffMember with Provider role (Staff Management) |
| `provider_id` | `staff_member_id` |
| `provider_type` (ProviderType) | `clinical_specialization` (ClinicalSpecialization) on StaffMember |
| ProviderRegistered event | RoleAssigned(Provider) + ProviderTypeSet events on StaffMember |
| ProviderRenamed | StaffMember name is already on StaffMember — no separate event needed |
| ProviderTypeChanged | ProviderTypeSet on StaffMember |
| ProviderAssignedToOffice | ProviderAssignedToOffice on StaffMember (same event name, `staff_member_id`) |
| ProviderRemovedFromOffice | ProviderRemovedFromOffice on StaffMember |
| ProviderAvailabilitySet | ProviderAvailabilitySet on StaffMember |
| ProviderAvailabilityCleared | ProviderAvailabilityCleared on StaffMember |
| ProviderExceptionSet | ProviderExceptionSet on StaffMember |
| ProviderExceptionRemoved | ProviderExceptionRemoved on StaffMember |
| ProviderArchived | StaffMemberArchived (archiving the StaffMember covers clinical too) |
| ProviderUnarchived | StaffMemberUnarchived |

---

## See Instead

- **Aggregate**: `doc/domain/aggregates/staff-member-aggregate.md` — clinical configuration is in the "Clinical Configuration Fields" and "Clinical Configuration Events/Commands" sections
- **Ubiquitous Language**: `doc/domain/ubiquitous-language.md` — updated Provider and ClinicalSpecialization definitions
- **Context Map**: `doc/domain/context-maps/context-map.md` — updated boundary showing availability data moving from Practice Setup to Staff Management

---

**Retired By**: Tony + Claude (DM-1 correction, 2026-03-06)
