# ProcedureType Aggregate

**Context**: Practice Setup
**Last Updated**: 2026-03-03

---

## Purpose

A named category of dental service that can be scheduled, with a default duration and color-coded category for calendar display. Procedure types are the practice's menu of services.

Examples: Cleaning (Preventive, 30 min), Root Canal (Invasive, 90 min), Consultation (Consult, 30 min).

---

## Fields

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| id | UUID | Yes | System-generated |
| name | String | Yes | Procedure name, e.g., "Cleaning", "Root Canal" |
| category | ProcedureCategory | Yes | Color-coded grouping for calendar display |
| default_duration_minutes | u32 | Yes | Standard appointment length. Range: 15-240 minutes. |
| is_active | bool | Yes | Default true. Deactivated procedures hidden from scheduling. |

### Value Objects

**ProcedureCategory**:

| Category | Color | Typical Procedures |
|----------|-------|--------------------|
| Consult | Yellow | New patient exams, consultations |
| Preventive | Blue | Cleanings, fluoride treatments, sealants |
| Restorative | Green | Fillings, crowns, bridges |
| Invasive | Red | Extractions, root canals, oral surgery |
| Cosmetic | Purple | Whitening, veneers |
| Diagnostic | Gray | X-rays, CT scans, diagnostic exams |

Nico confirmed all six categories are appropriate.

---

## Events

| Event | Fields | When |
|-------|--------|------|
| **ProcedureTypeDefined** | id, name, category, default_duration_minutes | Admin defines a new procedure type |
| **ProcedureTypeUpdated** | id, name?, category?, default_duration_minutes? | Admin modifies any field |
| **ProcedureTypeDeactivated** | id | Admin removes procedure from active list |
| **ProcedureTypeReactivated** | id | Admin restores a deactivated procedure |

---

## Commands

| Command | Input | Produces | Preconditions |
|---------|-------|----------|---------------|
| DefineProcedureType | name, category, default_duration_minutes | ProcedureTypeDefined | name not empty, valid category, duration 15-240 |
| UpdateProcedureType | id, name?, category?, default_duration_minutes? | ProcedureTypeUpdated | at least one field changed, if duration changed must be 15-240 |
| DeactivateProcedureType | id | ProcedureTypeDeactivated | currently active |
| ReactivateProcedureType | id | ProcedureTypeReactivated | currently inactive |

---

## Invariants

1. **Name required**: Procedure type must have a non-empty name
2. **Valid category**: Must be one of the six defined categories
3. **Duration range**: Must be 15-240 minutes (MIN_DURATION to MAX_DURATION)
4. **Deactivate, not delete**: Procedure types are deactivated, not archived or deleted. Historical appointment records reference them.

---

## State Machine

```
stateDiagram-v2
    [*] --> Active : DefineProcedureType
    Active --> Active : UpdateProcedureType
    Active --> Inactive : DeactivateProcedureType
    Inactive --> Active : ReactivateProcedureType
```

---

## Seed Defaults

On first-run setup (or when the procedure list is empty), the system can offer to seed common dental procedures. This is implemented as a batch of individual `ProcedureTypeDefined` events -- no special event type.

Suggested defaults:

| Name | Category | Duration |
|------|----------|----------|
| Consultation | Consult | 30 min |
| Cleaning | Preventive | 30 min |
| Fluoride Treatment | Preventive | 15 min |
| Exam | Diagnostic | 15 min |
| X-Ray | Diagnostic | 15 min |
| Filling | Restorative | 45 min |
| Crown | Restorative | 60 min |
| Extraction | Invasive | 30 min |
| Root Canal | Invasive | 90 min |
| Whitening | Cosmetic | 60 min |

---

## Booking Constraints (consumed by Scheduling context)

The ProcedureType aggregate provides:

1. **Default duration**: Used to calculate appointment end time when booking
2. **Active status**: Only active procedure types are available for scheduling
3. **Category color**: Used for calendar display (presentation concern, not a constraint)

---

## Design Decisions

- **Define, not Create**: Domain language -- procedure types are "defined", reflecting that they describe a service offering.
- **Deactivate/Reactivate, not Archive/Unarchive**: Procedure types are configuration items, not people or places. Different lifecycle semantics from Office and Provider.
- **Category colors are fixed**: The six categories and their colors are defined by the system, not configurable per practice. Nico confirmed these cover dental practice needs.
- **Duration is a default, not a constraint**: Individual appointments can override the default duration. The procedure type provides the starting value.
- **Seed defaults as individual events**: No special `DefaultProceduresSeeded` event. Each seeded procedure is a standard `ProcedureTypeDefined` event, keeping the event store uniform.

---

## Duration Guidance

Common procedure durations (from belsouri-old, validated with Nico):

| Procedure | Typical Duration |
|-----------|-----------------|
| Cleaning | 30-45 minutes |
| Exam | 15-30 minutes |
| Filling | 45-60 minutes |
| Root Canal | 90-120 minutes |
| Extraction | 30-60 minutes |
| Consultation | 30 minutes |

---

**Maintained By**: Tony + Claude
