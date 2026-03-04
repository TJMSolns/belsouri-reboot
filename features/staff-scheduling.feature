# Feature: Staff Scheduling — Provider Availability Queries
#
# Phase 2.4 BDD Scenarios — Staff Scheduling context
# Date: 2026-03-04
# All open questions resolved — SS-1 (availability stays in Practice Setup) and
# SS-2 (any active StaffMember can view schedule) confirmed by Tony 2026-03-04.
#
# Covers:
#   SS-Rule-1: ResolvedSchedule applies weekly availability (6 scenarios)
#   SS-Rule-2: Exception overrides availability (6 scenarios)
#   SS-Rule-3: Provider not assigned to office is not available there (5 scenarios)
#   SS-Rule-4: Office without configured hours has no availability (5 scenarios)
#   SS-Rule-5: query_provider_availability returns reason when unavailable (7 scenarios)
#   SS-Rule-6: get_office_provider_schedule returns all providers working on a date (7 scenarios)
#   SS-Rule-7: ResolvedSchedule updated incrementally on new Practice Setup events (7 scenarios)
#
# Context: Staff Scheduling is projection-first at MVP. No new aggregates.
# Subscribes to Practice Setup events and materialises ResolvedSchedule and
# OfficeProviderView projections. Exposes three query commands to Patient Scheduling.

Feature: Staff Scheduling — Provider Availability Queries
  As a staff member of the practice
  I want the system to know when each provider is available at each office
  So that appointments can be booked against accurate availability

  Background:
    Given the practice has an office "Main Office" with chair_count 3
    And the practice has an office "Branch Office" with chair_count 2
    And the practice has a provider "Dr. Spence" registered and assigned to "Main Office"
    And "Main Office" has operating hours set for Monday from 08:00 to 17:00
    And "Main Office" has operating hours set for Tuesday from 08:00 to 17:00
    And "Main Office" has operating hours set for Wednesday from 08:00 to 17:00

  # ─────────────────────────────────────────────────────────────
  # SS-RULE-1: ResolvedSchedule correctly applies weekly availability
  # ─────────────────────────────────────────────────────────────

  Rule: ResolvedSchedule correctly applies weekly availability

    # SS1a
    Scenario: Provider available on a day matching their weekly availability window
      Given "Dr. Spence" has availability at "Main Office" on Monday from 09:00 to 17:00
      When the system queries provider availability for "Dr. Spence" at "Main Office" on the next Monday at 10:00
      Then the result is available: true

    # SS1b
    Scenario: Provider not available on a day not covered by their weekly availability window
      Given "Dr. Spence" has availability at "Main Office" on Monday from 09:00 to 17:00
      And "Dr. Spence" has no availability set for Tuesday at "Main Office"
      When the system queries provider availability for "Dr. Spence" at "Main Office" on the next Tuesday at 10:00
      Then the result is available: false
      And the reason is "no availability"

    # SS1c
    Scenario: Query at the start boundary of an availability window returns available
      Given "Dr. Spence" has availability at "Main Office" on Monday from 09:00 to 17:00
      When the system queries provider availability for "Dr. Spence" at "Main Office" on the next Monday at 09:00
      Then the result is available: true

    # SS1d
    Scenario: Query at the end boundary of an availability window returns not available
      Given "Dr. Spence" has availability at "Main Office" on Monday from 09:00 to 17:00
      When the system queries provider availability for "Dr. Spence" at "Main Office" on the next Monday at 17:00
      Then the result is available: false
      And the reason is "no availability"

    # SS1e
    Scenario: Provider has independent availability at two offices on the same day
      Given "Dr. Spence" is also assigned to "Branch Office"
      And "Branch Office" has operating hours set for Monday from 08:00 to 17:00
      And "Dr. Spence" has availability at "Main Office" on Monday from 08:00 to 12:00
      And "Dr. Spence" has availability at "Branch Office" on Monday from 13:00 to 17:00
      When the system queries provider availability for "Dr. Spence" at "Main Office" on the next Monday at 10:00
      Then the result is available: true
      When the system queries provider availability for "Dr. Spence" at "Branch Office" on the next Monday at 14:00
      Then the result is available: true

    # SS1f
    Scenario: Provider with no availability set is not available on any day
      Given "Dr. Spence" has no availability set at any office
      When the system queries provider availability for "Dr. Spence" at "Main Office" on the next Monday at 10:00
      Then the result is available: false
      And the reason is "no availability"

  # ─────────────────────────────────────────────────────────────
  # SS-RULE-2: Exception overrides weekly availability
  # ─────────────────────────────────────────────────────────────

  Rule: Exception overrides weekly availability for the covered date range

    # SS2a
    Scenario: Provider unavailable on a date covered by an active exception
      Given "Dr. Spence" has availability at "Main Office" on Monday from 09:00 to 17:00
      And an exception is set for "Dr. Spence" from 2026-12-20 to 2026-12-31
      When the system queries provider availability for "Dr. Spence" at "Main Office" on 2026-12-22 at 10:00
      Then the result is available: false
      And the reason is "exception"

    # SS2b
    Scenario: Provider available on the day before an exception begins
      Given "Dr. Spence" has availability at "Main Office" on Monday from 09:00 to 17:00
      And an exception is set for "Dr. Spence" from 2026-12-20 to 2026-12-31
      When the system queries provider availability for "Dr. Spence" at "Main Office" on 2026-12-19 at 10:00
      Then the result is available: true

    # SS2c
    Scenario: Provider available on the day after an exception ends
      Given "Dr. Spence" has availability at "Main Office" on Monday from 09:00 to 17:00
      And an exception is set for "Dr. Spence" from 2026-12-20 to 2026-12-31
      When the system queries provider availability for "Dr. Spence" at "Main Office" on 2027-01-02 at 10:00
      Then the result is available: true

    # SS2d
    Scenario: Single-day exception blocks exactly that one date
      Given "Dr. Spence" has availability at "Main Office" on Monday from 09:00 to 17:00
      And an exception is set for "Dr. Spence" from 2026-12-28 to 2026-12-28
      When the system queries provider availability for "Dr. Spence" at "Main Office" on 2026-12-28 at 10:00
      Then the result is available: false
      And the reason is "exception"

    # SS2e
    Scenario: Removing an exception restores availability per weekly window
      Given "Dr. Spence" has availability at "Main Office" on Monday from 09:00 to 17:00
      And an exception was set for "Dr. Spence" from 2026-12-20 to 2026-12-31
      And the exception has been removed via ProviderExceptionRemoved
      When the system queries provider availability for "Dr. Spence" at "Main Office" on 2026-12-22 at 10:00
      Then the result is available: true

    # SS2f
    Scenario: Removing an exception on a day with no availability window does not create availability
      Given "Dr. Spence" has availability at "Main Office" on Monday only (not Tuesday)
      And an exception was set for "Dr. Spence" covering a Tuesday
      And the exception has been removed via ProviderExceptionRemoved
      When the system queries provider availability for "Dr. Spence" at "Main Office" on that Tuesday at 10:00
      Then the result is available: false
      And the reason is "no availability"

  # ─────────────────────────────────────────────────────────────
  # SS-RULE-3: Provider not assigned to an office is not available there
  # ─────────────────────────────────────────────────────────────

  Rule: Provider not assigned to an office is not available at that office on any day

    # SS3a
    Scenario: Provider assigned to one office is not available at a different office
      Given "Dr. Spence" is assigned to "Main Office" only and not assigned to "Branch Office"
      And "Dr. Spence" has availability at "Main Office" on Monday from 09:00 to 17:00
      And "Branch Office" has operating hours set for Monday from 08:00 to 17:00
      When the system queries provider availability for "Dr. Spence" at "Branch Office" on the next Monday at 10:00
      Then the result is available: false
      And the reason is "not assigned"

    # SS3b
    Scenario: Provider removed from an office is no longer available there
      Given "Dr. Spence" was assigned to "Main Office" and then removed via ProviderRemovedFromOffice
      When the system queries provider availability for "Dr. Spence" at "Main Office" on any date at 10:00
      Then the result is available: false
      And the reason is "not assigned"

    # SS3c
    Scenario: Provider never assigned to any office is not available anywhere
      Given "Dr. Clarke" is registered but not assigned to any office
      When the system queries provider availability for "Dr. Clarke" at "Main Office" on the next Monday at 10:00
      Then the result is available: false
      And the reason is "not assigned"

    # SS3d
    Scenario: Provider assigned to an office with availability is available there
      Given "Dr. Spence" is assigned to "Main Office"
      And "Dr. Spence" has availability at "Main Office" on Monday from 09:00 to 17:00
      When the system queries provider availability for "Dr. Spence" at "Main Office" on the next Monday at 10:00
      Then the result is available: true

    # SS3e
    Scenario: Provider re-assigned to an office after removal regains availability
      Given "Dr. Spence" was removed from "Main Office" via ProviderRemovedFromOffice
      And "Dr. Spence" has been re-assigned to "Main Office" via ProviderAssignedToOffice
      And "Dr. Spence" has availability at "Main Office" on Monday from 09:00 to 17:00
      When the system queries provider availability for "Dr. Spence" at "Main Office" on the next Monday at 10:00
      Then the result is available: true

  # ─────────────────────────────────────────────────────────────
  # SS-RULE-4: Office without configured hours has no availability
  # ─────────────────────────────────────────────────────────────

  Rule: Office with no configured hours for a day has no availability on that day

    # SS4a
    Scenario: Provider has no availability at an office with no configured hours
      Given "Dr. Spence" is assigned to "Branch Office"
      And "Branch Office" has no operating hours configured for any day
      And "Dr. Spence" has availability at "Branch Office" on Monday from 09:00 to 17:00
      When the system queries provider availability for "Dr. Spence" at "Branch Office" on the next Monday at 10:00
      Then the result is available: false
      And the reason is "office closed"

    # SS4b
    Scenario: Provider unavailable at an office on a day with no hours configured
      Given "Main Office" has hours set for Monday and Tuesday but not Wednesday
      And "Dr. Spence" has availability at "Main Office" on Wednesday from 09:00 to 17:00
      When the system queries provider availability for "Dr. Spence" at "Main Office" on the next Wednesday at 10:00
      Then the result is available: false
      And the reason is "office closed"

    # SS4c
    Scenario: Provider unavailable on a day explicitly closed via OfficeDayClosed
      Given "Main Office" had hours set for Wednesday
      And "Main Office" Wednesday has been closed via OfficeDayClosed
      And "Dr. Spence" has availability at "Main Office" on Wednesday from 09:00 to 17:00
      When the system queries provider availability for "Dr. Spence" at "Main Office" on the next Wednesday at 10:00
      Then the result is available: false
      And the reason is "office closed"

    # SS4d
    Scenario: Archived office is closed for all providers on all days
      Given "Main Office" has been archived via OfficeArchived
      And "Dr. Spence" had availability at "Main Office" on Monday from 09:00 to 17:00
      When the system queries provider availability for "Dr. Spence" at "Main Office" on the next Monday at 10:00
      Then the result is available: false
      And the reason is "office closed"

    # SS4e
    Scenario: Office hours gate is checked before provider availability window
      Given "Main Office" has no hours configured for Thursday
      And "Dr. Spence" has availability at "Main Office" on Thursday from 09:00 to 17:00
      When the system queries provider availability for "Dr. Spence" at "Main Office" on the next Thursday at 10:00
      Then the result is available: false
      And the reason is "office closed"

  # ─────────────────────────────────────────────────────────────
  # SS-RULE-5: query_provider_availability returns reason when unavailable
  # ─────────────────────────────────────────────────────────────

  Rule: query_provider_availability always includes a reason when the provider is not available

    # SS5a
    Scenario: Available provider has null reason
      Given "Dr. Spence" has availability at "Main Office" on Monday from 09:00 to 17:00
      When the system queries provider availability for "Dr. Spence" at "Main Office" on the next Monday at 10:00
      Then the result is available: true
      And the reason is null

    # SS5b
    Scenario: Reason is exception when an active exception covers the date
      Given "Dr. Spence" has availability at "Main Office" on Monday from 09:00 to 17:00
      And an exception is set for "Dr. Spence" from 2026-12-20 to 2026-12-31
      When the system queries provider availability for "Dr. Spence" at "Main Office" on 2026-12-22 at 10:00
      Then the result is available: false
      And the reason is "exception"

    # SS5c
    Scenario: Reason is not assigned when provider is not registered at the office
      Given "Dr. Spence" is not assigned to "Branch Office"
      When the system queries provider availability for "Dr. Spence" at "Branch Office" on the next Monday at 10:00
      Then the result is available: false
      And the reason is "not assigned"

    # SS5d
    Scenario: Reason is no availability when provider has no window for that day
      Given "Dr. Spence" has availability at "Main Office" on Monday only
      When the system queries provider availability for "Dr. Spence" at "Main Office" on the next Tuesday at 10:00
      Then the result is available: false
      And the reason is "no availability"

    # SS5e
    Scenario: Reason is office closed when office has no hours for the day
      Given "Main Office" has no hours configured for Thursday
      And "Dr. Spence" has availability at "Main Office" on Thursday from 09:00 to 17:00
      When the system queries provider availability for "Dr. Spence" at "Main Office" on the next Thursday at 10:00
      Then the result is available: false
      And the reason is "office closed"

    # SS5f
    Scenario: Reason is provider archived when the provider has been archived
      Given "Dr. Spence" has availability at "Main Office" on Monday from 09:00 to 17:00
      And "Dr. Spence" has been archived via ProviderArchived
      When the system queries provider availability for "Dr. Spence" at "Main Office" on the next Monday at 10:00
      Then the result is available: false
      And the reason is "provider archived"

    # SS5g
    Scenario: Provider archived takes priority over other unavailability reasons
      Given "Dr. Spence" has been archived via ProviderArchived
      And an exception is also set for "Dr. Spence" covering next Monday
      When the system queries provider availability for "Dr. Spence" at "Main Office" on the next Monday at 10:00
      Then the result is available: false
      And the reason is "provider archived"

  # ─────────────────────────────────────────────────────────────
  # SS-RULE-6: get_office_provider_schedule returns all available providers
  # ─────────────────────────────────────────────────────────────

  Rule: get_office_provider_schedule returns all providers available at an office on a given date

    # SS6a
    Scenario: All three assigned providers are available on a given day
      Given "Dr. Spence", "Dr. Clarke", and "Dr. Brown" are all assigned to "Main Office"
      And all three have availability at "Main Office" on Monday from 09:00 to 17:00
      When the system fetches the office provider schedule for "Main Office" on the next Monday
      Then the result contains 3 schedule entries
      And each entry includes provider_id, provider_name, start_time, and end_time

    # SS6b
    Scenario: Provider with an exception on the queried date is excluded from the schedule
      Given "Dr. Spence", "Dr. Clarke", and "Dr. Brown" are all assigned to "Main Office"
      And all three have availability at "Main Office" on Monday from 09:00 to 17:00
      And an exception is set for "Dr. Clarke" covering next Monday
      When the system fetches the office provider schedule for "Main Office" on the next Monday
      Then the result contains 2 schedule entries
      And "Dr. Clarke" is not in the result

    # SS6c
    Scenario: Archived provider is excluded from the schedule
      Given "Dr. Spence", "Dr. Clarke", and "Dr. Brown" are all assigned to "Main Office"
      And all three have availability at "Main Office" on Monday from 09:00 to 17:00
      And "Dr. Brown" has been archived via ProviderArchived
      When the system fetches the office provider schedule for "Main Office" on the next Monday
      Then the result contains 2 schedule entries
      And "Dr. Brown" is not in the result

    # SS6d
    Scenario: No providers assigned to the office returns an empty schedule
      Given "Branch Office" has operating hours set for Monday from 08:00 to 17:00
      And no providers are assigned to "Branch Office"
      When the system fetches the office provider schedule for "Branch Office" on the next Monday
      Then the result is an empty schedule

    # SS6e
    Scenario: All assigned providers have exceptions on the queried date returns an empty schedule
      Given "Dr. Spence" and "Dr. Clarke" are assigned to "Main Office"
      And both have availability at "Main Office" on Monday from 09:00 to 17:00
      And an exception is set for "Dr. Spence" covering next Monday
      And an exception is set for "Dr. Clarke" covering next Monday
      When the system fetches the office provider schedule for "Main Office" on the next Monday
      Then the result is an empty schedule

    # SS6f
    Scenario: Provider schedule entry reflects the specific day's availability hours
      Given "Dr. Spence" has availability at "Main Office" on Monday from 09:00 to 13:00
      And "Dr. Spence" has availability at "Main Office" on Tuesday from 14:00 to 17:00
      When the system fetches the office provider schedule for "Main Office" on the next Monday
      Then the result contains 1 schedule entry
      And the entry for "Dr. Spence" shows start_time 09:00 and end_time 13:00

    # SS6g
    Scenario: Query beyond the 90-day pre-materialisation window returns an empty schedule
      When the system fetches the office provider schedule for "Main Office" on a date 91 days from today
      Then the result is an empty schedule

  # ─────────────────────────────────────────────────────────────
  # SS-RULE-7: ResolvedSchedule updated incrementally on new events
  # ─────────────────────────────────────────────────────────────

  Rule: ResolvedSchedule is updated incrementally when Practice Setup emits new events

    # SS7a
    Scenario: New ProviderAvailabilitySet event updates only the affected provider-office rows
      Given "Dr. Spence" has no availability set at "Main Office"
      And the ResolvedSchedule contains no available rows for "Dr. Spence" at "Main Office"
      When Practice Setup emits ProviderAvailabilitySet for "Dr. Spence" at "Main Office" on Monday 09:00-17:00
      Then the ResolvedSchedule rows for "Dr. Spence" at "Main Office" on Mondays within the 90-day window are updated to available
      And rows for other providers at "Main Office" are unchanged
      And rows for "Dr. Spence" at other offices are unchanged

    # SS7b
    Scenario: ProviderExceptionSet updates only the rows in the exception date range
      Given "Dr. Spence" has availability at "Main Office" on Monday from 09:00 to 17:00
      And the ResolvedSchedule shows available for "Dr. Spence" at "Main Office" for upcoming Mondays
      When Practice Setup emits ProviderExceptionSet for "Dr. Spence" from 2026-12-20 to 2026-12-31
      Then only ResolvedSchedule rows for "Dr. Spence" in the Dec 20-31 date range are updated to not available with reason "exception"
      And rows outside that date range are unchanged

    # SS7c
    Scenario: ProviderExceptionRemoved recalculates only the formerly blocked date range
      Given "Dr. Spence" has availability at "Main Office" on Monday from 09:00 to 17:00
      And an exception for "Dr. Spence" from 2026-12-20 to 2026-12-31 was previously applied
      When Practice Setup emits ProviderExceptionRemoved for "Dr. Spence" for that date range
      Then ResolvedSchedule rows for Mondays in Dec 20-31 are recalculated and updated to available
      And rows for non-Mondays in that range remain not available with reason "no availability"
      And rows outside that date range are unchanged

    # SS7d
    Scenario: ProviderRemovedFromOffice updates only that provider-office pair
      Given "Dr. Spence" has availability at "Main Office" on Monday from 09:00 to 17:00
      And "Dr. Spence" is also assigned to "Branch Office" with availability on Tuesday
      When Practice Setup emits ProviderRemovedFromOffice for "Dr. Spence" from "Main Office"
      Then all ResolvedSchedule rows for "Dr. Spence" at "Main Office" are updated to not available with reason "not assigned"
      And rows for "Dr. Spence" at "Branch Office" are unchanged

    # SS7e
    Scenario: OfficeArchived marks all rows for that office as not available
      Given "Dr. Spence" and "Dr. Clarke" both have availability at "Main Office"
      When Practice Setup emits OfficeArchived for "Main Office"
      Then all ResolvedSchedule rows for "Main Office" are updated to not available with reason "office closed"
      And rows for other offices are unchanged

    # SS7f
    Scenario: OfficeDayClosed updates only the rows for that day-of-week at that office
      Given "Main Office" has hours set for Monday, Tuesday, and Wednesday
      And "Dr. Spence" has availability at "Main Office" on all three days
      When Practice Setup emits OfficeDayClosed for "Main Office" on Thursday
      Then ResolvedSchedule rows for "Main Office" on Thursdays are updated to not available with reason "office closed"
      And rows for Monday, Tuesday, and Wednesday at "Main Office" are unchanged

    # SS7g
    Scenario: Newly assigned provider gets rows inserted for the 90-day window only
      Given "Dr. Clarke" exists but has not been assigned to "Main Office"
      And the ResolvedSchedule has no rows for "Dr. Clarke" at "Main Office"
      When Practice Setup emits ProviderAssignedToOffice for "Dr. Clarke" at "Main Office"
      And ProviderAvailabilitySet is emitted for "Dr. Clarke" at "Main Office" on Monday 09:00-17:00
      Then new ResolvedSchedule rows are inserted for "Dr. Clarke" at "Main Office" for Mondays within the 90-day window
      And no rows are inserted beyond the 90-day pre-materialisation window
      And existing rows for other providers and offices are unchanged
