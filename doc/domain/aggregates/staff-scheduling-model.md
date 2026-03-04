# Staff Scheduling — Domain Model

**Context**: Staff Scheduling
**Last Updated**: 2026-03-04
**Status**: Phase 1.3 COMPLETE — projection-first model; see HS-1 for boundary decision

---

## Purpose

Staff Scheduling answers the core scheduling question: **"When is provider X available at office Y?"**

At MVP, this context is **projection-first** — it owns no aggregates that produce new events. Instead, it subscribes to upstream events from Practice Setup and materializes queryable views. This keeps implementation lean while establishing the clean integration contract that Patient Scheduling needs.

---

## Why No New Aggregates at MVP

All state that drives the schedule is commanded and recorded in Practice Setup:
- Provider availability windows → `ProviderAvailabilitySet / Cleared` events
- Provider exceptions → `ProviderExceptionSet / Removed` events
- Office hours → `OfficeHoursSet / OfficeDayClosed` events
- Provider-office assignments → `ProviderAssignedToOffice / RemovedFromOffice` events

Staff Scheduling subscribes to these events and builds projections. When richer scheduling is needed (time-off request workflows, shift patterns), Staff Scheduling will grow into an aggregate-owning context. See HS-1 in the event storming doc.

---

## Projections

### ResolvedSchedule

The core projection. Answers: "Is provider X available at office Y on date D at time T?"

**Inputs** (events consumed from Practice Setup):
- `ProviderAssignedToOffice` / `ProviderRemovedFromOffice`
- `ProviderAvailabilitySet` / `ProviderAvailabilityCleared`
- `ProviderExceptionSet` / `ProviderExceptionRemoved`
- `ProviderArchived` / `ProviderUnarchived`
- `OfficeHoursSet` / `OfficeDayClosed`
- `OfficeArchived`

**Projection schema** (materialized in `projections.db`):

```
Table: resolved_schedule
  provider_id     UUID        -- FK to Practice Setup provider
  office_id       UUID        -- FK to Practice Setup office
  date            DATE        -- Specific calendar date
  is_available    BOOL        -- After applying exceptions
  start_time      TIME?       -- Null if not available
  end_time        TIME?       -- Null if not available
  blocked_reason  TEXT?       -- "exception" | "not assigned" | "no availability" | null
```

**Resolution algorithm** (applied at projection build time):

1. Start with provider's weekly availability window for that office and day_of_week
2. Apply office hours: if office has no hours for that day_of_week → unavailable
3. Apply exceptions: if provider has an active exception covering that date → unavailable
4. If provider is not assigned to that office → unavailable
5. If provider is archived → unavailable
6. Otherwise → available with the window's start/end times

**Pre-materialization window**: Project 90 days forward from today. Rebuild incrementally as new events arrive. Do not full-rebuild on every write (avoids O(n²) performance).

---

### OfficeProviderView

The "today's schedule" projection. Shows who is working at each office on a given date.

**Inputs**: Same events as ResolvedSchedule.

**Projection schema**:

```
Table: office_provider_view
  office_id       UUID
  date            DATE
  provider_id     UUID
  provider_name   TEXT        -- Denormalized from Practice Setup
  start_time      TIME
  end_time        TIME
```

This projection is a filtered/reshaped view of the ResolvedSchedule — only includes records where `is_available = true`.

---

## Query Interface (consumed by Patient Scheduling)

Staff Scheduling exposes these queries via Tauri commands:

| Command | Input | Returns | Notes |
|---------|-------|---------|-------|
| `query_provider_availability` | provider_id, office_id, date, start_time, end_time | `{available: bool, reason?: String}` | Core booking validation query |
| `get_office_provider_schedule` | office_id, date | `Vec<ScheduleEntry>` | "Today's schedule" view for a given office |
| `get_provider_week_schedule` | provider_id, week_start_date | `Vec<ScheduleDay>` | Provider's full week across all offices |

```rust
// ScheduleEntry (for office view)
pub struct ScheduleEntry {
    pub provider_id: Uuid,
    pub provider_name: String,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
}

// ScheduleDay (for provider week view)
pub struct ScheduleDay {
    pub date: NaiveDate,
    pub office_id: Uuid,
    pub office_name: String,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub is_exception_day: bool,
}
```

---

## Context Relationships

| Upstream | What Flows In | Integration |
|----------|--------------|-------------|
| Practice Setup | Availability events (ProviderAvailabilitySet, OfficeHoursSet, etc.) | Staff Scheduling subscribes to Practice Setup's event store. No API calls — both are in the same local SQLite database. Staff Scheduling reads from the events table and builds its own projections in the projections table. |
| Staff Management | Active staff_member_id (for attribution on future time-off requests) | Not consumed at MVP. Reserved for post-MVP time-off workflow. |

| Downstream | What Flows Out | Integration |
|-----------|---------------|-------------|
| Patient Scheduling | ResolvedSchedule query answers | Patient Scheduling calls `query_provider_availability` via Tauri command before booking. |
| UI / Front Desk | OfficeProviderView (today's schedule) | UI calls `get_office_provider_schedule` to display the daily schedule. |

---

## State Machine

Staff Scheduling has no aggregate lifecycle (no create/archive flow). The projections are computed state that changes continuously as upstream events arrive.

```
Practice Setup events
         |
         ▼
[ResolvedSchedule projection]
         |
         ├──► Patient Scheduling (availability queries)
         └──► UI / Front Desk (daily schedule view)
```

---

## Design Decisions

### D1: Projection-first at MVP

No new aggregates at MVP. Keeps implementation scope small. Staff Scheduling grows into an aggregate-owning context when time-off request workflows or shift patterns are needed.

### D2: Pre-materialized projections (not computed on-demand)

The ResolvedSchedule is materialized to a table and kept current, not computed ad-hoc on each query. Avoids performance issues on old hardware (Caribbean context: 4GB RAM, 8-year-old CPUs). 90-day forward window is sufficient for scheduling.

### D3: Local SQLite — no cross-database joins

Practice Setup and Staff Scheduling both use the same local SQLite databases (`events.db`, `projections.db`). Staff Scheduling reads Practice Setup events directly from the events table and writes its own projection tables. No RPC calls, no network. Fully offline.

### D4: Boundary watch — HS-1

If time-off request workflows are added, availability exceptions will naturally migrate to Staff Scheduling (StaffMember requests time off → Staff Scheduling records the exception → ResolvedSchedule updates). At that point, Staff Scheduling becomes an aggregate-owning context and Practice Setup's `ProviderExceptionSet` events are retired. This transition should be planned as a Phase 3 Three Amigos decision.

---

**Maintained By**: Tony + Claude
