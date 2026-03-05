# StaffShift Aggregate

**Context**: Staff Management
**Last Updated**: 2026-03-05

---

## Purpose

A StaffShift is a planned working period for a non-clinical staff member at a specific office on a specific date. It records who is working, when, where, and in what role. Unlike Provider availability (which defines a recurring weekly pattern in Practice Setup), staff shifts are ad-hoc — each shift is created individually as needed.

The Practice Manager plans shifts for their team. Staff members can also declare their own shifts, reducing administrative burden on the Practice Manager.

StaffShift belongs to the Staff Management context. It is a first-class aggregate: it has its own identity, its own events, and its own lifecycle. It is not a projection of another aggregate.

**Tony's need** (confirmed 2026-03-05): "I can't tell when and where my non-clinical staff are working. I would like to know when and where everyone was working (person and roles)."

---

## Fields

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| id | UUID | Yes | System-generated. Referenced as `shift_id` in projections. |
| staff_member_id | UUID | Yes | References Staff Management StaffMember. |
| office_id | UUID | Yes | References Practice Setup Office. |
| date | YYYY-MM-DD | Yes | The working date. |
| start_time | HH:MM | Yes | Shift start (24-hour local time). |
| end_time | HH:MM | Yes | Shift end (24-hour local time). Must be after start_time. |
| role | String | Yes | The role the staff member is performing this shift. Must be one of the staff member's currently assigned roles at time of planning. |
| created_by | UUID | Yes | The staff_member_id of the person who planned this shift. Either the staff member themselves or a Practice Manager. |
| cancelled | bool | Yes | Default false. True after CancelStaffShift. |
| cancel_reason | String? | No | Optional reason recorded when the shift is cancelled. |

---

## Events

| Event | Fields | When |
|-------|--------|------|
| **StaffShiftPlanned** | shift_id, staff_member_id, office_id, date, start_time, end_time, role, created_by | A new shift is planned |
| **StaffShiftCancelled** | shift_id, cancelled_by, reason? | A planned shift is cancelled |

---

## Commands

| Command | Input | Produces | Preconditions |
|---------|-------|----------|---------------|
| PlanStaffShift | staff_member_id, office_id, date, start_time, end_time, role, created_by | StaffShiftPlanned | (1) Staff member is active and not archived; (2) office exists and is not archived; (3) end_time is strictly after start_time; (4) role is one of the staff member's currently assigned roles; (5) created_by is either the staff member themselves or a StaffMember who holds the PracticeManager role |
| CancelStaffShift | shift_id, cancelled_by, reason? | StaffShiftCancelled | (1) Shift exists; (2) shift is not already cancelled; (3) cancelled_by is either the staff member who owns the shift or a StaffMember who holds the PracticeManager role |

---

## Invariants

1. **Role must be assigned**: The role field must be one of the roles currently held by the staff member at the time PlanStaffShift is issued. It is not sufficient that the role exists in the system — the specific staff member must hold it.
2. **end_time after start_time**: end_time must be strictly later than start_time on the same date. Overnight shifts spanning midnight are not supported at MVP.
3. **Creator authorization**: Only the staff member themselves or a Practice Manager (a StaffMember with the PracticeManager role) can plan or cancel a shift. A Staff role holder cannot plan shifts for other staff members.
4. **Cancelled shifts are soft-deleted**: CancelStaffShift emits an event and sets the cancelled flag. The StaffShiftPlanned event and the aggregate record remain in the event store permanently.
5. **No modification after cancellation**: A cancelled shift cannot be modified. A new PlanStaffShift must be issued instead.

---

## State Machine

```
stateDiagram-v2
    [*] --> Planned : PlanStaffShift
    Planned --> Cancelled : CancelStaffShift
    Cancelled --> [*] : (terminal — no further transitions)
```

---

## Projection

### staff_shift_roster

| Column | Type | Source | Notes |
|--------|------|--------|-------|
| shift_id | UUID | StaffShiftPlanned.shift_id | Primary key |
| staff_member_id | UUID | StaffShiftPlanned.staff_member_id | Foreign key to staff member |
| staff_name | String | Denormalized from StaffMember projection at rebuild time | For display — avoids join at query time |
| office_id | UUID | StaffShiftPlanned.office_id | Foreign key to office |
| office_name | String | Denormalized from Office projection at rebuild time | For display — avoids join at query time |
| date | String | StaffShiftPlanned.date | YYYY-MM-DD |
| start_time | String | StaffShiftPlanned.start_time | HH:MM |
| end_time | String | StaffShiftPlanned.end_time | HH:MM |
| role | String | StaffShiftPlanned.role | Role performed this shift |
| created_by | UUID | StaffShiftPlanned.created_by | For audit trail |
| cancelled | bool | Updated by StaffShiftCancelled | Default false |
| cancel_reason | String? | StaffShiftCancelled.reason | Null if not cancelled or no reason given |

**Query patterns**:
- **Schedule page Roster tab**: `WHERE date >= week_start AND date <= week_end` — returns all shifts for the selected week across all staff
- **Staff page per-person view**: `WHERE staff_member_id = ? AND date >= today ORDER BY date, start_time` — returns upcoming shifts for one staff member

**Cancelled shift display**: Cancelled shifts are retained in the projection with `cancelled = true`. The UI renders them greyed out. They are not removed.

---

## Cross-Context Usage

StaffShift references two upstream contexts:

- **Staff Management** (own context): `staff_member_id` references the StaffMember aggregate. Command layer validates the staff member is active and holds the specified role.
- **Practice Setup**: `office_id` references the Office aggregate. Command layer validates the office is not archived.

StaffShift does not produce events consumed by other contexts. It is purely an informational planning record and does not create booking constraints. Provider appointments (Patient Scheduling) are unaffected by whether a StaffShift exists or not.

---

## Design Decisions

- **Ad-hoc, not recurring** [Confirmed by Tony 2026-03-05]: Each shift is a single event. There is no template or weekly pattern. The Practice Manager can plan a full week by creating 5 separate shifts. Recurring shift patterns are a backlog item.
- **Soft cancel, not delete** [Confirmed by Tony 2026-03-05]: Cancelled shifts remain visible in history. This preserves the record that the shift was once planned and provides an audit trail. Consistent with the append-only event store design.
- **Role at shift time** [Confirmed by Tony 2026-03-05]: The role field records which role the staff member is performing on this particular shift, not just their primary role. This captures cross-role coverage (e.g., a Staff member filling in as the Practice Manager for the day) and answers Tony's "person and roles" requirement.
- **No booking constraints**: Staff shifts are informational planning records. They do not block appointment booking and are not checked against office hours or provider availability. This keeps the model simple and avoids coupling Staff Management to Patient Scheduling.
- **Self-service model** [Confirmed by Tony 2026-03-05]: Staff members can plan their own shifts. This reflects the operational reality where staff know their own working schedule and reduces administrative burden on the Practice Manager.
- **No overlap detection**: Duplicate or overlapping shifts for the same staff member are not blocked at MVP. The PM can clean up duplicates by cancelling the incorrect shift. Overlap detection is a backlog item.
- **Clinical staff exclusion**: Providers (clinical staff) have their schedule managed via the Provider availability model in Practice Setup, which feeds the ResolvedSchedule projection in Staff Scheduling. StaffShift is explicitly for non-clinical staff members who hold the Staff or PracticeManager role.

---

**Maintained By**: Tony + Claude
