# Event Storming: Practice Setup

**Context**: Practice Setup
**Date**: 2026-03-03
**Participants**: Tony (Product Owner), Claude (Developer)

**Sources**: Domain knowledge from Nico (Dr. Hannif Spence) interviews (Feb 11 & Feb 25, 2026), belsouri-old aggregate docs and example maps, Figma UI design guidance, feedback from belsouri-old testing (Feb 18, 2026).

---

## Purpose

Practice Setup is the foundational bounded context that configures the static structure of a dental practice: its offices, providers, and procedure types. All other contexts (Scheduling, Patient Management, Clinical Records) depend on Practice Setup configuration to function.

This is the "day one" configuration -- what must be in place before a practice can see patients.

---

## Domain Events

### Office Lifecycle

| # | Event | Trigger | Fields | Notes |
|---|-------|---------|--------|-------|
| 1 | **OfficeCreated** | Admin creates a new office location | office_id, name, chair_count | Chair count >= 1. Name required. |
| 2 | **OfficeRenamed** | Admin changes office name | office_id, new_name | Nico uses location names: "Kingston", "Montego Bay" |
| 3 | **OfficeChairCountUpdated** | Admin changes chair capacity | office_id, new_chair_count | Warn if reducing below concurrent appointment count (don't block) |
| 4 | **OfficeHoursSet** | Admin sets operating hours for a day | office_id, day_of_week, open_time, close_time | HH:MM format. close > open. |
| 5 | **OfficeDayClosed** | Admin marks a day as closed | office_id, day_of_week | Unconfigured days default to closed |
| 6 | **OfficeArchived** | Admin decommissions an office | office_id | Soft delete -- data preserved for historical queries |

### Provider Lifecycle

| # | Event | Trigger | Fields | Notes |
|---|-------|---------|--------|-------|
| 7 | **ProviderRegistered** | Admin adds a new provider to the practice | provider_id, name, provider_type | Types: Dentist, Hygienist, Specialist. "Provider" is Nico's preferred term for all clinical staff. |
| 8 | **ProviderRenamed** | Admin corrects/updates provider name | provider_id, new_name | |
| 9 | **ProviderTypeChanged** | Admin changes provider classification | provider_id, new_provider_type | Rare but possible (e.g., specialist certification) |
| 10 | **ProviderAssignedToOffice** | Admin links provider to an office | provider_id, office_id | Must happen before setting availability at that office |
| 11 | **ProviderRemovedFromOffice** | Admin unlinks provider from an office | provider_id, office_id | Auto-clears all availability at that office |
| 12 | **ProviderArchived** | Admin deactivates a provider | provider_id | Soft delete -- historical appointments preserved |
| 13 | **ProviderUnarchived** | Admin reactivates a provider | provider_id | Provider returns from extended absence |

### Provider Availability (office-scoped)

| # | Event | Trigger | Fields | Notes |
|---|-------|---------|--------|-------|
| 14 | **ProviderAvailabilitySet** | Admin sets weekly availability at an office | provider_id, office_id, day_of_week, start_time, end_time | No overlap across offices on same day. Warning (not block) if outside office hours. |
| 15 | **ProviderAvailabilityCleared** | Admin removes a day's availability at an office | provider_id, office_id, day_of_week | |
| 16 | **ProviderExceptionSet** | Admin blocks dates (vacation, time off) | provider_id, start_date, end_date, reason | Warn if existing appointments in range. Jamaica dental is seasonal (insurance-driven off-season). |
| 17 | **ProviderExceptionRemoved** | Admin lifts a date block | provider_id, start_date, end_date | Provider becomes bookable again |

### Procedure Type Lifecycle

| # | Event | Trigger | Fields | Notes |
|---|-------|---------|--------|-------|
| 18 | **ProcedureTypeDefined** | Admin defines a procedure type | procedure_type_id, name, category, default_duration_minutes | Categories: Consult, Preventive, Restorative, Invasive, Cosmetic, Diagnostic. Duration 15-240 min. |
| 19 | **ProcedureTypeUpdated** | Admin modifies a procedure type | procedure_type_id, name?, category?, default_duration_minutes? | Any field can change independently |
| 20 | **ProcedureTypeDeactivated** | Admin removes procedure from active list | procedure_type_id | Historical records preserved |
| 21 | **ProcedureTypeReactivated** | Admin restores a deactivated procedure | procedure_type_id | |

### Practice Identity

| # | Event | Trigger | Fields | Notes |
|---|-------|---------|--------|-------|
| 22 | **PracticeDetailsSet** | Admin configures practice identity | name, phone?, email?, website?, address? | First-run setup or Settings edit. Address includes parish (Jamaica-specific). |
| 23 | **PracticeDetailsUpdated** | Admin updates any practice detail | changed_fields | Partial updates -- only changed fields emitted |

**Total: 23 events** across 5 temporal groups.

---

## Commands

| Command | Actor | Input | Produces | Preconditions |
|---------|-------|-------|----------|---------------|
| CreateOffice | Practice Manager | name, chair_count | OfficeCreated | name not empty, chair_count >= 1 |
| RenameOffice | Practice Manager | office_id, new_name | OfficeRenamed | office exists, name not empty |
| UpdateChairCount | Practice/Office Manager | office_id, new_chair_count | OfficeChairCountUpdated | chair_count >= 1 |
| SetOfficeHours | Practice/Office Manager | office_id, day, open, close | OfficeHoursSet | close > open, valid day |
| CloseOfficeDay | Practice/Office Manager | office_id, day | OfficeDayClosed | valid day |
| ArchiveOffice | Practice Manager | office_id | OfficeArchived | not already archived |
| RegisterProvider | Practice Manager | name, provider_type | ProviderRegistered | name not empty, valid type |
| RenameProvider | Practice Manager | provider_id, new_name | ProviderRenamed | name not empty |
| ChangeProviderType | Practice Manager | provider_id, new_type | ProviderTypeChanged | valid type |
| AssignProviderToOffice | Practice Manager | provider_id, office_id | ProviderAssignedToOffice | office exists, not already assigned |
| RemoveProviderFromOffice | Practice Manager | provider_id, office_id | ProviderRemovedFromOffice + ProviderAvailabilityCleared (x N) | currently assigned |
| ArchiveProvider | Practice Manager | provider_id | ProviderArchived | not already archived |
| UnarchiveProvider | Practice Manager | provider_id | ProviderUnarchived | currently archived |
| SetProviderAvailability | Practice/Office Manager | provider_id, office_id, day, start, end | ProviderAvailabilitySet | assigned to office, no cross-office overlap on same day |
| ClearProviderAvailability | Practice/Office Manager | provider_id, office_id, day | ProviderAvailabilityCleared | has availability for that day |
| SetProviderException | Practice/Office Manager | provider_id, start_date, end_date, reason? | ProviderExceptionSet | end >= start |
| RemoveProviderException | Practice/Office Manager | provider_id, start_date, end_date | ProviderExceptionRemoved | exception exists |
| DefineProcedureType | Practice Manager | name, category, duration | ProcedureTypeDefined | name not empty, duration 15-240 |
| UpdateProcedureType | Practice Manager | id, name?, category?, duration? | ProcedureTypeUpdated | at least one field changed |
| DeactivateProcedureType | Practice Manager | id | ProcedureTypeDeactivated | currently active |
| ReactivateProcedureType | Practice Manager | id | ProcedureTypeReactivated | currently inactive |
| SetPracticeDetails | Practice Manager | name, phone?, email?, website?, address? | PracticeDetailsSet | name not empty |
| UpdatePracticeDetails | Practice Manager | changed_fields | PracticeDetailsUpdated | at least one field changed |

---

## Aggregate Candidates

### 1. Office

The physical location where dental services are provided. Each office has independent configuration: name, chair capacity, and operating hours per day of week.

**Key invariant**: Chair count >= 1. Operating hours: close > open. Days without hours = closed.

**Identity**: office_id (UUID)

### 2. Provider

A dental professional who provides services to patients. Providers have a type (Dentist, Hygienist, Specialist), can be assigned to multiple offices, and have office-specific weekly availability plus date-based exceptions.

**Key invariant**: Must be assigned to an office before setting availability there. Cannot have overlapping availability at different offices on the same day/time.

**Identity**: provider_id (UUID)

### 3. ProcedureType

A category of dental service that can be scheduled, with a default duration and color-coded category for calendar display.

**Key invariant**: Duration 15-240 minutes. Name not empty. Category must be valid enum.

**Identity**: procedure_type_id (UUID)

### 4. Practice

The root organizational entity. Holds practice identity (name, contact info, address). Singleton per installation -- there is exactly one practice.

**Key invariant**: Must have a name. Address uses Jamaica-specific parish field.

**Identity**: Singleton (no ID needed, or fixed "practice" ID)

---

## Temporal Flows

### Flow 1: First-Run Setup (Bootstrap)

```
App installs -> Admin account created ->
  PracticeDetailsSet ->
  OfficeCreated (first office) ->
  OfficeHoursSet (x5-6 days) ->
  ProviderRegistered (first provider) ->
  ProviderAssignedToOffice ->
  ProviderAvailabilitySet (x5-6 days) ->
  ProcedureTypeDefined (x5-10 common procedures) ->
  Practice ready for scheduling
```

Nico's feedback: "Practice setup should be a wizard on first-time install."

### Flow 2: Adding a New Office (Multi-Office Expansion)

```
OfficeCreated ("Montego Bay", 2 chairs) ->
  OfficeHoursSet (Mon-Fri) ->
  ProviderAssignedToOffice (existing provider -> new office) ->
  ProviderAvailabilitySet (Tue/Thu at Montego Bay)
```

Real pattern from Nico: Provider rotates between Kingston (Mon/Wed/Fri) and Montego Bay (Tue/Thu).

### Flow 3: Provider Onboarding

```
ProviderRegistered ("Dr. Smith", Dentist) ->
  ProviderAssignedToOffice (Kingston) ->
  ProviderAvailabilitySet (Mon 8-5, Tue 8-5, Wed 8-12) ->
  ProviderAssignedToOffice (Montego Bay) ->
  ProviderAvailabilitySet (Thu 9-4, Fri 9-4)
```

### Flow 4: Vacation / Off-Season Closure

```
ProviderExceptionSet (Dec 20-31, "Holiday vacation") ->
  [System warns: "3 existing appointments in range"] ->
  [Appointments remain but provider shown as unavailable]
```

Jamaica dental care is seasonal -- insurance-driven off-season creates natural closure patterns.

### Flow 5: Office Decommissioning

```
ProviderRemovedFromOffice (all providers from old office) ->
  [Auto-clears availability at that office] ->
OfficeArchived ->
  [Office hidden from active lists, preserved in history]
```

---

## Hotspots and Open Questions

### Resolved (from Nico interviews and belsouri-old)

- [x] **What term for clinical staff?** -> "Provider" (Nico confirmed, covers all roles)
- [x] **Procedure categories?** -> Consult, Preventive, Restorative, Invasive, Cosmetic, Diagnostic (Nico confirmed all appropriate)
- [x] **Can provider work multiple offices?** -> Yes, office-scoped availability with no overlap constraint
- [x] **Office hours required at creation?** -> No, can be set after
- [x] **Can two offices share a name?** -> Soft warning but allowed
- [x] **Vacation support?** -> Simple date-based exceptions. No recurring patterns needed.
- [x] **What if provider availability exceeds office hours?** -> Warning only, not block (edge case: holiday coverage)
- [x] **What happens to appointments when reducing chairs?** -> Warn, don't block
- [x] **Patient status field?** -> Nico says not meaningful for their practice. Deprioritize.

### Open (need Tony's input)

- [ ] **Practice address structure**: Jamaica uses parishes, not states/provinces. Do we need a structured address (line 1, line 2, city/town, parish) or just freeform?
- [ ] **PracticeDetailsSet vs PracticeDetailsUpdated**: Should first-time setup be a distinct event from subsequent edits, or is the same event sufficient?
- [ ] **Provider archival vs deletion**: belsouri-old used archive (soft delete). Is there ever a "hard delete" scenario, or is archive always sufficient? (Event sourcing suggests archive-only.)
- [ ] **Seed defaults**: belsouri-old had a "Seed Defaults" action for common procedures. Should this be an event (`DefaultProceduresSeeded`) or just a batch of individual `ProcedureTypeDefined` events?
- [ ] **Office Manager scope**: Can an Office Manager modify only their office's configuration, or should all Practice Setup be Practice Manager only for MVP?
- [ ] **Rename event naming**: belsouri-old used `OfficeRenamed` but also had update events. Should we have granular events (OfficeRenamed, OfficeChairCountUpdated) or coarse events (OfficeUpdated with changed fields)? Granular is better for event sourcing clarity, but more event types to maintain.

---

## Cross-Context Dependencies

Practice Setup events are consumed by downstream contexts:

| Downstream Context | Events Consumed | Purpose |
|--------------------|----------------|---------|
| **Scheduling** | OfficeHoursSet, OfficeDayClosed, ProviderAvailabilitySet/Cleared, ProviderExceptionSet/Removed, OfficeChairCountUpdated | Validate appointment bookings against hours, availability, and capacity |
| **Patient Management** | OfficeCreated (list of offices for patient assignment) | Office selector for patient records |
| **Reporting** | All Practice Setup events | Practice-wide dashboards (capacity utilization, provider load) |

Practice Setup does **not** consume events from other contexts. It is purely upstream.

---

## Notes from belsouri-old Failures

These directly inform how we implement Practice Setup:

1. **OfficeRenamed was broken** -- error "Failed to rename office" with no actionable message. Every command needs clear error messages.
2. **Chair count update was broken** -- same pattern. CRUD must work reliably.
3. **Duplicate providers appeared** -- 3 "Donny Dentist" entries. Data consistency is critical.
4. **No practice identity fields** -- nowhere to enter practice name, address, phone, logo. We're adding the Practice aggregate to fix this.
5. **Provider edit was impossible** -- couldn't modify after creation. All entities must support update.
6. **"undefined undefined" patient names** -- DTO drift between Rust and TypeScript. tauri-specta prevents this.

---

**Next ceremony**: 1.2 Ubiquitous Language -- extract and define all domain terms from this event storming.
