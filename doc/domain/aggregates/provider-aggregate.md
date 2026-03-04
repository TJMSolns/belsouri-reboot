# Provider Aggregate

**Context**: Practice Setup
**Last Updated**: 2026-03-03

---

## Purpose

A dental professional who provides services to patients. "Provider" is the unified domain term (confirmed by Nico) covering all clinical staff: dentists, hygienists, and specialists.

Providers can be assigned to multiple offices with office-specific weekly availability, and can have date-based exceptions (vacations, time off) that override normal availability.

---

## Fields

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| id | UUID | Yes | System-generated |
| name | String | Yes | Provider's display name |
| provider_type | ProviderType | Yes | Dentist, Hygienist, or Specialist |
| office_assignments | Set<office_id> | No | Offices this provider works at. Empty initially. |
| availability | List<AvailabilityWindow> | No | Weekly schedule per office. Empty initially. |
| exceptions | List<AvailabilityException> | No | Date-based overrides. Empty initially. |
| archived | bool | Yes | Default false. Archived providers hidden from active lists. |

### Value Objects

**ProviderType**: Dentist | Hygienist | Specialist

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

| Event | Fields | When |
|-------|--------|------|
| **ProviderRegistered** | id, name, provider_type | Admin registers a new provider |
| **ProviderRenamed** | id, new_name | Admin updates the provider's name |
| **ProviderTypeChanged** | id, new_provider_type | Admin changes the provider's classification |
| **ProviderAssignedToOffice** | id, office_id | Admin links provider to an office |
| **ProviderRemovedFromOffice** | id, office_id | Admin unlinks provider from an office |
| **ProviderAvailabilitySet** | id, office_id, day_of_week, start_time, end_time | Admin sets a working window |
| **ProviderAvailabilityCleared** | id, office_id, day_of_week | Admin removes a working window |
| **ProviderExceptionSet** | id, start_date, end_date, reason? | Admin blocks a date range |
| **ProviderExceptionRemoved** | id, start_date, end_date | Admin lifts a date block |
| **ProviderArchived** | id | Admin deactivates the provider |
| **ProviderUnarchived** | id | Admin reactivates the provider |

---

## Commands

| Command | Input | Produces | Preconditions |
|---------|-------|----------|---------------|
| RegisterProvider | name, provider_type | ProviderRegistered | name not empty, valid type |
| RenameProvider | provider_id, new_name | ProviderRenamed | not archived, name not empty |
| ChangeProviderType | provider_id, new_type | ProviderTypeChanged | not archived, valid type |
| AssignProviderToOffice | provider_id, office_id | ProviderAssignedToOffice | not archived, office exists, not already assigned |
| RemoveProviderFromOffice | provider_id, office_id | ProviderRemovedFromOffice + ProviderAvailabilityCleared (x N days) | currently assigned to that office |
| SetProviderAvailability | provider_id, office_id, day, start, end | ProviderAvailabilitySet | assigned to office, end > start, no cross-office overlap on same day/time |
| ClearProviderAvailability | provider_id, office_id, day | ProviderAvailabilityCleared | has availability for that office+day |
| SetProviderException | provider_id, start_date, end_date, reason? | ProviderExceptionSet | end_date >= start_date |
| RemoveProviderException | provider_id, start_date, end_date | ProviderExceptionRemoved | exception exists for that range |
| ArchiveProvider | provider_id | ProviderArchived | not already archived |
| UnarchiveProvider | provider_id | ProviderUnarchived | currently archived |

---

## Invariants

1. **Name required**: Provider must have a non-empty name
2. **Valid provider type**: Must be Dentist, Hygienist, or Specialist
3. **Assignment before availability**: Provider must be assigned to an office before setting availability there
4. **No cross-office overlap**: Provider cannot have overlapping availability windows at different offices on the same day. E.g., Kingston 10AM-2PM and Montego Bay 12PM-4PM is invalid. Kingston 8AM-12PM and Montego Bay 1PM-5PM is valid.
5. **Valid time ranges**: end_time > start_time for availability windows; end_date >= start_date for exceptions
6. **Removal clears availability**: Removing a provider from an office automatically clears all availability windows at that office
7. **Availability outside office hours**: Warning only, not blocked. Edge case: holiday coverage when office hours are adjusted.
8. **Exceptions override availability**: A provider with an exception for a date range is unavailable regardless of weekly availability

---

## State Machine

```
stateDiagram-v2
    [*] --> Active : RegisterProvider
    Active --> Active : RenameProvider / ChangeProviderType
    Active --> Active : AssignProviderToOffice / RemoveProviderFromOffice
    Active --> Active : SetProviderAvailability / ClearProviderAvailability
    Active --> Active : SetProviderException / RemoveProviderException
    Active --> Archived : ArchiveProvider
    Archived --> Active : UnarchiveProvider
```

---

## Booking Constraints (consumed by Scheduling context)

The Provider aggregate provides three constraints that the Scheduling context enforces:

1. **Provider must be available**: Appointment time checked against availability windows for that office+day
2. **Provider not on exception**: Appointment date checked against active exceptions
3. **No double-booking**: Checked by Scheduling context against its own appointments projection (not a Practice Setup concern)

---

## Multi-Office Example

```
Dr. Brown:
  Assigned to: Kingston, Montego Bay
  Availability:
    Kingston:    Monday 8AM-5PM, Wednesday 8AM-5PM, Friday 8AM-5PM
    Montego Bay: Tuesday 9AM-4PM, Thursday 9AM-4PM
  Exceptions:
    Dec 20-31: "Holiday vacation"
```

This is the real pattern from Nico's practice -- providers rotate between offices on different days.

---

## Design Decisions

- **Register, not Create**: Domain language -- providers are "registered" at a practice.
- **Archive/Unarchive, not delete**: Append-only event store. Archived providers are hidden but historical data preserved. Storage impact negligible.
- **Office-scoped availability**: Availability is always tied to a specific office. A provider's Monday schedule at Kingston is independent of their Monday at Montego Bay.
- **Exceptions are provider-wide**: Vacation applies across all offices, not per-office. A provider on vacation is unavailable everywhere.
- **Warn on exception conflicts**: If setting an exception over dates with existing appointments, warn but allow. Appointments are not auto-cancelled.

---

**Maintained By**: Tony + Claude
