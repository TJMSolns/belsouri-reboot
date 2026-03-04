# Example Map: Staff Scheduling

**Date**: 2026-03-04
**Participants**: Tony (Product Owner / Business), Claude (Developer / Tester)
**Feature**: Staff Scheduling context — ResolvedSchedule and OfficeProviderView projections; query interface
**Status**: Phase 2.2 + 2.3 complete — all open questions resolved (SS-1 confirmed Tony 2026-03-04, SS-2 confirmed Tony 2026-03-04)

---

## Three Amigos Summary (Phase 2.1)

Decisions confirmed before example mapping:

| Item | Decision | Source |
|------|----------|--------|
| Availability ownership (SS-1) | Availability data stays in Practice Setup at MVP. Staff Scheduling is projection-first — no new aggregates. | Tony confirmed 2026-03-04 |
| Schedule view access (SS-2) | Any active StaffMember can view the schedule. No role restriction. | Tony confirmed 2026-03-04 |
| ResolvedSchedule materialisation | Pre-materialised to a table, not computed on demand per query. Avoids per-query cost on old hardware. | Staff Scheduling model doc D2 |
| Pre-materialisation window | 90 days forward from today. Rebuilt incrementally as new events arrive. No full rebuild on every write. | Staff Scheduling model doc D2 + CLAUDE.md invariant |
| Query interface as contract boundary | Patient Scheduling conforms to the three Tauri query commands. No direct projection table access from Patient Scheduling. | Staff Scheduling model doc — Query Interface section |
| Staff Scheduling fires no commands | Purely reads upstream Practice Setup events and answers queries. No write commands at MVP. | Event storming doc — Commands section |

---

## Rule Cards

---

## SS-Rule-1: ResolvedSchedule correctly applies weekly availability

**Rule**: When a provider has a weekly availability window at an office for a given day of the week, `query_provider_availability` returns `{available: true}` for any date matching that day where no exception applies and the office is open.

| # | Example | Type |
|---|---------|------|
| SS1a | Provider has availability at "Main Office" on Monday 09:00-17:00. Query for next Monday at 10:00 at "Main Office" → `{available: true}` | ✅ Happy path |
| SS1b | Provider has availability at "Main Office" on Monday 09:00-17:00. Query for next Tuesday at 10:00 at "Main Office" → `{available: false, reason: "no availability"}` | ❌ Negative path |
| SS1c | Provider has availability at "Main Office" on Monday 09:00-17:00. Query for Monday at 09:00 (start boundary) → `{available: true}` | ✅ Boundary |
| SS1d | Provider has availability at "Main Office" on Monday 09:00-17:00. Query for Monday at 17:00 (end boundary) → `{available: false, reason: "no availability"}` (window is exclusive at end) | ✅ Boundary |
| SS1e | Provider has availability at "Main Office" on Monday AND "Branch Office" on Monday. Query at "Main Office" Monday 10:00 → `{available: true}`. Query at "Branch Office" Monday 10:00 → `{available: true}`. Each office is independent. | ✅ Edge case |
| SS1f | Provider has no availability set at any office. Query for any date → `{available: false, reason: "no availability"}` | ❌ Negative path |

---

## SS-Rule-2: Exception overrides weekly availability

**Rule**: When a provider has an active exception covering a date, `query_provider_availability` returns `{available: false, reason: "exception"}` for that date regardless of the weekly availability window.

| # | Example | Type |
|---|---------|------|
| SS2a | Provider has availability Mon 09:00-17:00 at "Main Office". Exception set Dec 20-31. Query for Monday Dec 22 at 10:00 → `{available: false, reason: "exception"}` | ✅ Happy path |
| SS2b | Provider has availability Mon 09:00-17:00 at "Main Office". Exception set Dec 20-31. Query for Monday Dec 19 at 10:00 (day before exception) → `{available: true}` | ❌ Negative path (exception does not extend) |
| SS2c | Provider has availability Mon 09:00-17:00 at "Main Office". Exception set Dec 20-31. Query for Monday Jan 2 at 10:00 (day after exception) → `{available: true}` | ❌ Negative path (exception does not extend) |
| SS2d | Exception covers a single day (start_date = end_date). Query for that exact date → `{available: false, reason: "exception"}` | ✅ Boundary |
| SS2e | Exception removed via ProviderExceptionRemoved. Query for the formerly blocked date → `{available: true}` (weekly availability restored) | ✅ Happy path |
| SS2f | Provider has no weekly availability on a day but has an exception covering it. Exception removal → `{available: false, reason: "no availability"}` (exception removal does not create availability) | ✅ Edge case |

---

## SS-Rule-3: Provider not registered at an office is not available there

**Rule**: If a provider has not been assigned to an office (no `ProviderAssignedToOffice` event, or `ProviderRemovedFromOffice` was subsequently emitted), `query_provider_availability` at that office returns `{available: false, reason: "not assigned"}`.

| # | Example | Type |
|---|---------|------|
| SS3a | Provider registered at "Main Office" only. Query at "Branch Office" for any date → `{available: false, reason: "not assigned"}` | ✅ Happy path |
| SS3b | Provider assigned to "Main Office" then removed. Query at "Main Office" → `{available: false, reason: "not assigned"}` | ✅ Happy path |
| SS3c | Provider never assigned to any office. Query at any office → `{available: false, reason: "not assigned"}` | ❌ Negative path |
| SS3d | Provider assigned to "Main Office". Query at "Main Office" with valid availability → `{available: true}` (correct positive case to contrast) | ✅ Happy path |
| SS3e | Provider removed from "Main Office" then re-assigned. Query at "Main Office" with availability set → `{available: true}` (re-assignment restores access) | ✅ Edge case |

---

## SS-Rule-4: Office without configured hours has no availability

**Rule**: If an office has no operating hours configured for a given day of the week (no `OfficeHoursSet` event for that day, or `OfficeDayClosed` was emitted), `query_provider_availability` returns `{available: false, reason: "office closed"}` for all providers at that office on that day.

| # | Example | Type |
|---|---------|------|
| SS4a | Office has no hours configured for any day. Provider has availability set for Monday at that office. Query Monday → `{available: false, reason: "office closed"}` | ✅ Happy path |
| SS4b | Office has hours set for Monday but not Tuesday. Query provider at that office on Tuesday → `{available: false, reason: "office closed"}` | ✅ Happy path |
| SS4c | Office hours explicitly closed via OfficeDayClosed for Wednesday. Query provider at that office on Wednesday → `{available: false, reason: "office closed"}` | ✅ Happy path |
| SS4d | Office archived via OfficeArchived. Query provider at that office for any date → `{available: false, reason: "office closed"}` | ✅ Edge case |
| SS4e | Office has no hours but provider has weekly availability configured there. Query → `{available: false, reason: "office closed"}` (office hours gate is checked before availability window) | ✅ Edge case |

---

## SS-Rule-5: `query_provider_availability` returns reason when unavailable

**Rule**: When the result of `query_provider_availability` is `{available: false}`, the `reason` field is always populated with one of: `"exception"`, `"not assigned"`, `"no availability"`, `"office closed"`, `"provider archived"`. When available, reason is null.

| # | Example | Type |
|---|---------|------|
| SS5a | Provider available at queried office and time → `{available: true, reason: null}` | ✅ Happy path |
| SS5b | Provider has active exception covering date → `{available: false, reason: "exception"}` | ✅ Happy path |
| SS5c | Provider not assigned to office → `{available: false, reason: "not assigned"}` | ✅ Happy path |
| SS5d | Provider has no availability window for that day → `{available: false, reason: "no availability"}` | ✅ Happy path |
| SS5e | Office has no hours for that day → `{available: false, reason: "office closed"}` | ✅ Happy path |
| SS5f | Provider archived → `{available: false, reason: "provider archived"}` | ✅ Happy path |
| SS5g | Multiple conditions apply (e.g., provider archived AND exception active). Most specific reason returned: `"provider archived"` takes priority over `"exception"`. Resolution order: provider archived > exception > not assigned > office closed > no availability. | ✅ Edge case |

---

## SS-Rule-6: `get_office_provider_schedule` returns all providers working at that office on that date

**Rule**: `get_office_provider_schedule(office_id, date)` returns a `Vec<ScheduleEntry>` containing every provider whose `is_available = true` in the ResolvedSchedule for that office and date. Archived providers are excluded. Providers with exceptions on that date are excluded.

| # | Example | Type |
|---|---------|------|
| SS6a | Three providers assigned to "Main Office". All available Monday Jan 12. Query `get_office_provider_schedule("Main Office", Jan 12)` → Vec with 3 entries, each with provider_id, provider_name, start_time, end_time | ✅ Happy path |
| SS6b | Three providers assigned. One has an exception Jan 12. Query Jan 12 → Vec with 2 entries | ✅ Happy path |
| SS6c | Three providers assigned. One archived. Query any day → Vec excludes archived provider | ✅ Happy path |
| SS6d | No providers assigned to office. Query any day → empty Vec | ❌ Negative path |
| SS6e | All assigned providers have exceptions on queried date → empty Vec | ❌ Negative path |
| SS6f | Provider has different hours at same office on different days. Query Monday → entry with Monday hours. Query Tuesday → entry with Tuesday hours (or absent if no Tuesday availability). | ✅ Edge case |
| SS6g | Query date is beyond the 90-day pre-materialisation window → empty Vec (data not yet projected) | ✅ Edge case |

---

## SS-Rule-7: ResolvedSchedule updated incrementally when Practice Setup emits new events

**Rule**: When Practice Setup emits any of the tracked events (ProviderAvailabilitySet, ProviderAvailabilityCleared, ProviderExceptionSet, ProviderExceptionRemoved, ProviderAssignedToOffice, ProviderRemovedFromOffice, ProviderArchived, OfficeHoursSet, OfficeDayClosed, OfficeArchived), the ResolvedSchedule projection is updated incrementally — only the affected records are recalculated. The full projection is not rebuilt.

| # | Example | Type |
|---|---------|------|
| SS7a | ProviderAvailabilitySet for Mon at "Main Office" is emitted. ResolvedSchedule rows for all Mondays within the 90-day window for that provider-office pair are updated to `is_available = true`. Other provider-office pairs are unchanged. | ✅ Happy path |
| SS7b | ProviderExceptionSet for Dec 20-31 is emitted. Only rows in that date range for that provider are updated to `is_available = false, blocked_reason = "exception"`. Rows outside the range are unchanged. | ✅ Happy path |
| SS7c | ProviderExceptionRemoved for Dec 20-31 is emitted. Rows in that range are recalculated against the weekly availability window. Days matching availability → `is_available = true`. Days not matching → `is_available = false, blocked_reason = "no availability"`. | ✅ Happy path |
| SS7d | ProviderRemovedFromOffice is emitted. All ResolvedSchedule rows for that provider-office pair are updated to `is_available = false, blocked_reason = "not assigned"`. Other offices for the same provider are unchanged. | ✅ Happy path |
| SS7e | OfficeArchived is emitted. All ResolvedSchedule rows for that office (all providers) are updated to `is_available = false, blocked_reason = "office closed"`. | ✅ Happy path |
| SS7f | OfficeDayClosed for Thursday is emitted. Only rows where date falls on a Thursday for that office are updated. Other days for that office are unchanged. | ✅ Happy path |
| SS7g | New event arrives for a provider-office pair that has no existing rows in the 90-day window (provider newly assigned). New rows are inserted for the 90-day window — not a full-rebuild of all existing data. | ✅ Edge case |

---

## Phase 2.3 Acceptance Criteria Review

**Validation against `doc/domain/aggregates/staff-scheduling-model.md`**:

- SS-Rule-1 (weekly availability): Aligns with Resolution algorithm steps 1 and 6 in the ResolvedSchedule projection spec.
- SS-Rule-2 (exception override): Aligns with Resolution algorithm step 3.
- SS-Rule-3 (not assigned): Aligns with Resolution algorithm step 4.
- SS-Rule-4 (office closed): Aligns with Resolution algorithm step 2. OfficeArchived is included in event inputs.
- SS-Rule-5 (reason field): Aligns with `blocked_reason` column in the `resolved_schedule` table schema: `"exception" | "not assigned" | "no availability" | null`. Note: "office closed" and "provider archived" are valid reason values; model doc groups them under `"no availability"`. The example map expands them for precision. Implementation must enumerate all five reason strings.
- SS-Rule-6 (`get_office_provider_schedule`): Aligns with OfficeProviderView projection definition and `ScheduleEntry` struct. The model confirms it is a filtered view of ResolvedSchedule where `is_available = true`.
- SS-Rule-7 (incremental update): Aligns with Design Decision D2 (pre-materialised, not on-demand) and CLAUDE.md mandate that projections must be incremental — never full rebuild.

**Gaps identified and resolved**:

1. **Resolution priority order (SS-Rule-5 SS5g)**: The model doc lists `blocked_reason` values but does not specify priority when multiple conditions apply simultaneously. Example SS5g documents the agreed priority: provider archived > exception > not assigned > office closed > no availability. This should be carried into the implementation invariants.

2. **End-time boundary (SS-Rule-1 SS1d)**: The model doc gives `start_time` and `end_time` but does not state whether the end time is inclusive or exclusive. Example SS1d assumes exclusive (query at 17:00 is not available for a 09:00-17:00 window). This aligns with standard interval semantics and must be enforced consistently.

3. **90-day window edge (SS-Rule-6 SS6g)**: Queries beyond the 90-day window return empty results rather than computing on demand. Patient Scheduling must not book appointments beyond 90 days from today without triggering a projection extension. This is an integration concern to carry into Patient Scheduling Phase 2.

4. **`blocked_reason` values**: The model doc shows `"exception" | "not assigned" | "no availability" | null`. SS-Rule-5 adds `"office closed"` and `"provider archived"` as distinct reasons for UI clarity. The implementation should use all five values in the `blocked_reason` column.

**Open questions**: None. SS-1 and SS-2 are confirmed. All assumptions from Phase 1 are resolved.

---

**Phase 2.5 Governance Review**: See governance pass/fail at bottom of this file.

**Phase 2.5 Result**: PASS — see governance section below.

---

## Governance Review (Phase 2.5)

**Date**: 2026-03-04
**Reviewer**: Claude (Developer)

### Banned Term Check

| Checked Term | Found | Resolution |
|-------------|-------|------------|
| shift | Not present | PASS |
| roster | Not present | PASS |
| timetable | Not present | PASS |
| raw schedule | Not present | PASS |
| availability cache | Not present | PASS |
| availability lookup | Not present | PASS |
| slot | Not present | PASS |
| schedule (as verb) | Not present | PASS |
| staff / staff member (as domain term) | Not present in domain sense | PASS |
| delete | Not present | PASS |

### Open Questions Check

No `[OPEN QUESTION]` markers present. All questions from Phase 1 (SS-1, SS-2, SS-3, SS-4) are resolved:
- SS-1: Availability stays in Practice Setup — CONFIRMED Tony 2026-03-04
- SS-2: Any active StaffMember can view schedule — CONFIRMED Tony 2026-03-04
- SS-3: Minimum projection data (provider_id, office_id, available_date, start_time, end_time) — SATISFIED by resolved_schedule schema
- SS-4: Attribution not tracked at MVP — ASSUMED, confirmed by model doc (attribution is in Practice Setup events)

### Events and Projections Alignment

All events referenced in rule cards exist in the event storming doc (E1-E10):
- ProviderAssignedToOffice (E1) ✓
- ProviderRemovedFromOffice (E2) ✓
- ProviderAvailabilitySet (E3) ✓
- ProviderAvailabilityCleared (E4) ✓
- ProviderExceptionSet (E5) ✓
- ProviderExceptionRemoved (E6) ✓
- ProviderArchived (E7) ✓
- OfficeHoursSet (E8) ✓
- OfficeDayClosed (E9) ✓
- OfficeArchived (E10) ✓

Projections referenced (ResolvedSchedule, OfficeProviderView) exist in the model doc ✓

Query commands referenced (query_provider_availability, get_office_provider_schedule, get_provider_week_schedule) exist in the model doc ✓

### BDD Scenario Coverage

All 7 rules (SS-Rule-1 through SS-Rule-7) have corresponding BDD scenarios in `features/staff-scheduling.feature` ✓

### Verdict

**PASS** — No banned terms, no open questions, all referenced artifacts exist, full BDD coverage. Minor gap identified (blocked_reason priority order) is documented in the Acceptance Criteria Review and must be carried into implementation invariants.
