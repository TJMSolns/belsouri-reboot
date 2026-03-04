# Office Aggregate

**Context**: Practice Setup
**Last Updated**: 2026-03-03

---

## Purpose

A physical location where dental services are provided. Each office has its own name, chair capacity, and operating hours. Offices are independently configured -- Kingston's settings do not affect Montego Bay.

Chair count is the key capacity constraint: the number of concurrent appointments at an office cannot exceed its chair count.

---

## Fields

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| id | UUID | Yes | System-generated |
| name | String | Yes | Location name, e.g., "Kingston", "Montego Bay" |
| chair_count | u32 | Yes | Physical treatment stations. Minimum 1. |
| hours | Map<DayOfWeek, TimeRange> | No | Per-day operating hours. Days not in map = closed. |
| archived | bool | Yes | Default false. Archived offices hidden from active lists. |

### Value Objects

**DayOfWeek**: Monday, Tuesday, Wednesday, Thursday, Friday, Saturday, Sunday

**TimeRange**: { start: HH:MM, end: HH:MM } where end > start

---

## Events

| Event | Fields | When |
|-------|--------|------|
| **OfficeCreated** | id, name, chair_count | Admin creates a new office |
| **OfficeRenamed** | id, new_name | Admin changes the office name |
| **OfficeChairCountUpdated** | id, new_chair_count | Admin changes chair capacity |
| **OfficeHoursSet** | id, day_of_week, open_time, close_time | Admin sets hours for a specific day |
| **OfficeDayClosed** | id, day_of_week | Admin marks a day as closed (removes hours) |
| **OfficeArchived** | id | Admin decommissions the office |

---

## Commands

| Command | Input | Produces | Preconditions |
|---------|-------|----------|---------------|
| CreateOffice | name, chair_count | OfficeCreated | name not empty, chair_count >= 1 |
| RenameOffice | office_id, new_name | OfficeRenamed | office exists, not archived, name not empty |
| UpdateChairCount | office_id, new_chair_count | OfficeChairCountUpdated | chair_count >= 1. Warn (don't block) if reducing below concurrent appointment count. |
| SetOfficeHours | office_id, day, open_time, close_time | OfficeHoursSet | valid day, close > open, valid HH:MM format |
| CloseOfficeDay | office_id, day | OfficeDayClosed | valid day |
| ArchiveOffice | office_id | OfficeArchived | not already archived |

---

## Invariants

1. **Name required**: Office must have a non-empty name
2. **Chair count >= 1**: Cannot have zero chairs
3. **Valid hours**: close_time must be after open_time (HH:MM comparison)
4. **Days without hours = closed**: No default "8-5" assumption
5. **Archive is permanent**: No unarchive (if needed, create a new office). Historical data preserved.
6. **Independent configuration**: Each office's settings are self-contained

---

## State Machine

```
stateDiagram-v2
    [*] --> Active : CreateOffice
    Active --> Active : RenameOffice / UpdateChairCount / SetOfficeHours / CloseOfficeDay
    Active --> Archived : ArchiveOffice
```

---

## Booking Constraints (consumed by Scheduling context)

The Office aggregate provides two constraints that the Scheduling context enforces:

1. **Office must be open**: Appointment time checked against operating hours for that day
2. **Chair capacity**: Number of concurrent appointments at a given time cannot exceed chair_count

---

## Design Decisions

- **Granular events**: Separate events for rename, chair count, hours (not a single OfficeUpdated). Better audit trail, less storage per event.
- **Warn on chair reduction**: If reducing chair count below existing concurrent appointments, system warns but allows the change. Existing appointments are not cancelled.
- **Duplicate names allowed**: Soft warning but not blocked. Practices may legitimately have similar names.
- **No address field on Office (MVP)**: Practice address covers the business identity. Per-office addresses are a future enhancement for multi-location practices.

---

**Maintained By**: Tony + Claude
