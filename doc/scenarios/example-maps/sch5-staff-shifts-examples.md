# Example Map: SCH-5 Staff Shift Roster

**Date**: 2026-03-05
**Participants**: Tony (Product Owner / Business), Claude (Developer / Tester)
**Feature**: StaffShift aggregate — plan and cancel ad-hoc shifts for non-clinical staff; weekly roster view and per-person view
**Status**: Phase 2.2 + 2.3 complete — all open questions CONFIRMED by Tony (2026-03-05)

---

## Three Amigos Summary (Phase 2.1)

Decisions confirmed by Tony (2026-03-05):

| Item | Decision | Source |
|------|----------|--------|
| Schedule type (SCH-5-1) | Ad-hoc shifts — each shift created individually. No fixed weekly pattern at MVP. | Tony confirmed 2026-03-05 |
| Shift direction (SCH-5-2) | Planned (future-facing). Not a timesheet. Not retrospective hours tracking. | Tony confirmed 2026-03-05 |
| Who creates shifts (SCH-5-3) | Both the Practice Manager AND the staff member themselves can plan shifts. A Staff role holder cannot plan shifts for another staff member. | Tony confirmed 2026-03-05 |
| UI locations (SCH-5-4) | Both (a) a Roster tab on the Schedule page showing all staff for the selected week, AND (b) per-person expansion on the Staff page showing upcoming shifts. | Tony confirmed 2026-03-05 |
| Scope (SCH-5-5) | Non-clinical staff only (StaffMember aggregate — PracticeManager, Staff). Clinical staff are Providers and already have the Provider availability model. | Tony confirmed 2026-03-05 |
| Booking constraints (SCH-5-6) | None. Staff shifts are informational planning records only. They do not block appointment booking. | Tony confirmed 2026-03-05 |
| Cancelled shift display (SCH-5-7) | Cancelled shifts remain visible in the roster, rendered greyed out. Not removed from the list. | Tony confirmed 2026-03-05 |
| Overlap detection (SCH-5-8) | Not enforced at MVP. Duplicate shifts can be cleaned up by cancelling the incorrect one. | Tony confirmed 2026-03-05 |

---

## Feature Statement

Practice Managers and staff members can plan ad-hoc shifts to record who is working, when, and where. A weekly roster view shows all staff across the practice. A per-person view on the Staff page shows upcoming shifts for individual staff members.

---

## Rule Cards

---

## SCH5-Rule-1: A Practice Manager can plan a shift for any active staff member

**Rule**: A StaffMember who holds the PracticeManager role can plan a shift for any active (non-archived) staff member, including themselves. The planned shift records who, when, where, and in what role.

| # | Example | Type |
|---|---------|------|
| SCH5-1a | PM "Dr. Reid" plans a shift for "Maria Brown" (Staff) at Kingston Office on Monday 2026-03-09 09:00–17:00, role "Staff" → StaffShiftPlanned event recorded with staff_member_id=Maria, office_id=Kingston, date=2026-03-09, start_time=09:00, end_time=17:00, role="Staff", created_by=Dr. Reid | ✅ Happy path |
| SCH5-1b | PM "Dr. Reid" plans a shift for themselves at Montego Bay Office for Tuesday 2026-03-10 08:00–16:00, role "PracticeManager" → StaffShiftPlanned event recorded with created_by=Dr. Reid and staff_member_id=Dr. Reid | ✅ Happy path |
| SCH5-1c | PM attempts to plan a shift for "Sandra Lee" who has been archived → Rejected: "Staff member Sandra Lee is archived and cannot be assigned a shift" | ❌ Negative path |
| SCH5-1d | PM plans a shift for Maria at Kingston on Monday, then plans a second shift for Maria at Kingston on Tuesday → Two separate StaffShiftPlanned events, each with their own shift_id | ✅ Happy path |
| SCH5-1e | PM plans a full week (Monday through Friday) for Maria by submitting 5 separate PlanStaffShift commands → 5 StaffShiftPlanned events recorded | ✅ Happy path |

---

## SCH5-Rule-2: A staff member can plan their own shift

**Rule**: A StaffMember can plan a shift for themselves. They cannot plan a shift for a different staff member. created_by must equal staff_member_id unless the creator holds the PracticeManager role.

| # | Example | Type |
|---|---------|------|
| SCH5-2a | "Maria Brown" (Staff) plans her own shift at Montego Bay Office on Friday 2026-03-13 09:00–17:00, role "Staff" → StaffShiftPlanned event recorded with staff_member_id=Maria, created_by=Maria | ✅ Happy path |
| SCH5-2b | "Maria Brown" (Staff) attempts to plan a shift with staff_member_id=John Clarke and created_by=Maria Brown → Rejected: "Maria Brown does not have the Practice Manager role and cannot plan a shift for another staff member" | ❌ Negative path |
| SCH5-2c | "John Clarke" (Staff) plans his own shift at Kingston Office on Tuesday 2026-03-10 08:00–14:00, role "Staff" → StaffShiftPlanned event recorded | ✅ Happy path |
| SCH5-2d | A Staff role holder submits PlanStaffShift with created_by=their own ID and staff_member_id=their own ID → Precondition 5 passes (self-planning is allowed) | ✅ Boundary |

---

## SCH5-Rule-3: Role must be one the staff member is assigned to

**Rule**: The role field in PlanStaffShift must be one of the roles currently held by the staff member referenced in staff_member_id. The set of valid roles for the shift is the staff member's actual role set, not all possible roles in the system.

| # | Example | Type |
|---|---------|------|
| SCH5-3a | Maria Brown holds role "Staff" only. PM plans a shift for Maria with role "Staff" → StaffShiftPlanned recorded. Role "Staff" is in Maria's role set. | ✅ Happy path |
| SCH5-3b | Maria Brown holds role "Staff" only. PM attempts to plan a shift for Maria with role "PracticeManager" → Rejected: "Maria Brown does not have the PracticeManager role and cannot plan a shift in that role" | ❌ Negative path |
| SCH5-3c | Dr. Reid holds roles "PracticeManager" and "Staff". PM plans a shift for Dr. Reid with role "PracticeManager" → Accepted. Role is in Dr. Reid's role set. | ✅ Happy path |
| SCH5-3d | Dr. Reid holds roles "PracticeManager" and "Staff". PM plans a shift for Dr. Reid with role "Staff" → Accepted. Both roles are valid for Dr. Reid. | ✅ Happy path |
| SCH5-3e | PM plans a shift for Maria with role "Provider" (a valid system role, but not assigned to Maria) → Rejected: "Maria Brown does not have the Provider role and cannot plan a shift in that role" | ❌ Negative path |
| SCH5-3f | After RoleAssigned(Maria, "PracticeManager") is emitted, PM plans a shift for Maria with role "PracticeManager" → Accepted. Maria now holds the role. | ✅ Edge case |
| SCH5-3g | After RoleRemoved(Maria, "Staff") is emitted (Maria now holds only "PracticeManager"), PM attempts to plan a shift for Maria with role "Staff" → Rejected: "Maria Brown does not have the Staff role and cannot plan a shift in that role" | ✅ Edge case |

---

## SCH5-Rule-4: end_time must be after start_time

**Rule**: The shift end_time must be strictly later than the shift start_time. Overnight shifts spanning midnight are not supported at MVP.

| # | Example | Type |
|---|---------|------|
| SCH5-4a | start_time "09:00", end_time "17:00" → Accepted | ✅ Happy path |
| SCH5-4b | start_time "09:00", end_time "09:00" (same time) → Rejected: "Shift end time must be after start time" | ❌ Boundary |
| SCH5-4c | start_time "17:00", end_time "09:00" (end before start) → Rejected: "Shift end time must be after start time" | ❌ Negative path |
| SCH5-4d | start_time "09:00", end_time "09:01" (one minute) → Accepted (no minimum duration at MVP) | ✅ Boundary |
| SCH5-4e | start_time "00:00", end_time "23:59" (near full day) → Accepted | ✅ Edge case |

---

## SCH5-Rule-5: Shifts can be cancelled by the shift owner or a Practice Manager

**Rule**: CancelStaffShift is accepted if the canceller is either the staff member who owns the shift (staff_member_id on the StaffShiftPlanned event) or a StaffMember who holds the PracticeManager role. A different Staff role holder cannot cancel another person's shift.

| # | Example | Type |
|---|---------|------|
| SCH5-5a | Maria Brown cancels her own shift (cancelled_by = Maria's staff_member_id) → StaffShiftCancelled event recorded with reason null | ✅ Happy path |
| SCH5-5b | PM "Dr. Reid" cancels Maria's shift with reason "Office closed — public holiday" → StaffShiftCancelled event recorded with reason "Office closed — public holiday" | ✅ Happy path |
| SCH5-5c | PM cancels a shift with no reason provided → StaffShiftCancelled event recorded with reason null | ✅ Happy path |
| SCH5-5d | "John Clarke" (Staff) attempts to cancel Maria Brown's shift (John is not a PM and not the shift owner) → Rejected: "John Clarke does not have the Practice Manager role and cannot cancel another staff member's shift" | ❌ Negative path |
| SCH5-5e | PM attempts to cancel a shift that is already cancelled → Rejected: "This shift has already been cancelled" | ❌ Negative path |
| SCH5-5f | Maria cancels her own shift. The shift_id still appears in the staff_shift_roster projection with cancelled=true. The StaffShiftPlanned and StaffShiftCancelled events are both preserved in the event store. | ✅ Happy path |

---

## SCH5-Rule-6: Roster view shows all staff for a selected week

**Rule**: The Schedule page Roster tab queries the staff_shift_roster projection by week date range. It returns all planned shifts (cancelled and active) for all staff members during that week. The display groups by staff member and renders cancelled shifts greyed out. Staff with no shifts in the selected week are not shown.

| # | Example | Type |
|---|---------|------|
| SCH5-6a | Week of 2026-03-09 to 2026-03-15. Maria has shifts on Mon, Wed, Fri at Kingston. John has shifts on Tue, Thu at Montego Bay. Query returns 5 total shift rows grouped correctly. | ✅ Happy path |
| SCH5-6b | PM cancels Maria's Wednesday shift. Week roster still shows all 5 rows. Maria's Wednesday row renders greyed out (cancelled=true). | ✅ Happy path |
| SCH5-6c | No shifts exist for the selected week → empty roster view; no staff rows displayed | ✅ Edge case |
| SCH5-6d | Week spans a month boundary (e.g., March 30 – April 5). Query uses date range filter `date >= 2026-03-30 AND date <= 2026-04-05`. Both months returned correctly. | ✅ Edge case |
| SCH5-6e | Maria has a shift on Monday this week and a shift on Monday next week. Query for this week returns only the current-week shift. | ✅ Happy path |

---

## SCH5-Rule-7: Per-person view on the Staff page shows upcoming shifts

**Rule**: The Staff page per-person expansion queries the staff_shift_roster projection filtered to staff_member_id = X and date >= today, ordered by date and start_time. Shows the staff member's future planned and cancelled shifts. Cancelled shifts are shown greyed out.

| # | Example | Type |
|---|---------|------|
| SCH5-7a | Maria has 3 upcoming shifts (next Mon, Wed, Fri). Staff page shows 3 rows under Maria's expansion with date, time, office, and role. | ✅ Happy path |
| SCH5-7b | Maria's Friday shift is cancelled. Per-person view shows 3 rows — Friday row is greyed out. | ✅ Happy path |
| SCH5-7c | Maria has no upcoming shifts → per-person expansion shows empty state | ✅ Edge case |
| SCH5-7d | Past shifts (date < today) are not shown in the per-person upcoming view | ✅ Happy path |
| SCH5-7e | Maria has both a current-week shift and a shift two weeks from now. Both appear in the per-person view, ordered chronologically. | ✅ Happy path |

---

## Phase 2.3 Acceptance Criteria Review

**Validation against `doc/domain/aggregates/staff-shift-aggregate.md`**:

- SCH5-Rule-1 (PM can plan for any active staff): Aligns with Command PlanStaffShift precondition 5 (created_by holds PracticeManager role OR is the staff member themselves).
- SCH5-Rule-2 (self-service model): Aligns with Design Decision "Self-service model" and precondition 5. The authorization check is symmetric: staff_member_id == created_by OR created_by holds PracticeManager role.
- SCH5-Rule-3 (role must be assigned): Aligns with Invariant 1 and precondition 4. Role is checked against the staff member's current role set at command time.
- SCH5-Rule-4 (end_time after start_time): Aligns with Invariant 2 and precondition 3. Overnight spans not supported at MVP.
- SCH5-Rule-5 (cancellation authorization): Aligns with Command CancelStaffShift preconditions. Soft cancel preserved in event store — aligns with Invariant 4.
- SCH5-Rule-6 (roster view): Aligns with Projection staff_shift_roster query pattern "Schedule page Roster tab". Cancelled shifts displayed greyed out — aligns with SCH-5-7 confirmed decision.
- SCH5-Rule-7 (per-person view): Aligns with Projection staff_shift_roster query pattern "Staff page per-person view".

**Gaps identified and resolved**:

1. **Error message wording for role rejection**: Example SCH5-3b establishes the canonical error message format: "[Staff member name] does not have the [Role] role and cannot plan a shift in that role". This must be carried into implementation for POL-003 compliance.
2. **Error message wording for creator authorization**: Example SCH5-2b establishes: "[Creator name] does not have the Practice Manager role and cannot plan a shift for another staff member." Implementation must use this exact phrasing.
3. **Past shifts in per-person view**: SCH5-7d confirms past shifts are excluded from the per-person upcoming view (date >= today). The roster tab on the Schedule page shows past-week data by week selection — this is a separate query.

**Open questions**: None. All decisions SCH-5-1 through SCH-5-8 are confirmed by Tony (2026-03-05). No open questions remain.

---

**Phase 2.5 Governance Review**: See governance section below.

**Phase 2.5 Result**: PASS

---

## Governance Review (Phase 2.5)

**Date**: 2026-03-05
**Reviewer**: Claude (Developer)

### Banned Term Check

| Checked Term | Found | Resolution |
|-------------|-------|------------|
| user | Not present | PASS |
| login / log in | Not present | PASS |
| password | Not present | PASS |
| delete | Not present | PASS |
| slot | Not present | PASS |
| booking (as noun for a shift) | Not present | PASS |
| schedule (as verb) | Not present | PASS |
| timetable | Not present | PASS |
| roster (as aggregate name) | Not present — "roster" used only for the projection view name and the feature, not as an aggregate | PASS |
| employee | Not present | PASS |

### Open Questions Check

No `[OPEN QUESTION]` markers present. All decisions SCH-5-1 through SCH-5-8 are resolved and confirmed by Tony (2026-03-05).

### Events and Projections Alignment

Events referenced in rule cards (StaffShiftPlanned, StaffShiftCancelled) both exist in the aggregate doc ✓

Projection referenced (staff_shift_roster) exists in the aggregate doc with correct column definitions ✓

Commands referenced (PlanStaffShift, CancelStaffShift) exist in the aggregate doc ✓

### BDD Scenario Coverage

All 7 rules (SCH5-Rule-1 through SCH5-Rule-7) have corresponding BDD scenarios in `features/staff-management.feature` ✓

### Verdict

**PASS** — No banned terms, no open questions, all referenced artifacts exist, full BDD coverage. Canonical error message phrasings documented in Acceptance Criteria section and must be carried into implementation.

---

**Maintained By**: Tony + Claude
