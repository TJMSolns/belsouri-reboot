# Example Map: Practice Setup

**Date**: 2026-03-03
**Participants**: Tony (Product Owner / Business), Claude (Developer / Tester)
**Feature**: Practice Setup context — full lifecycle (Practice, Office, Provider, ProcedureType, Setup Checklist)
**Status**: Phase 2.2 + 2.3 complete — all open questions resolved or flagged for Tony

---

## Three Amigos Summary (Phase 2.1)

Decisions carried in from the Three Amigos session (pre-decided by Tony):

| Item | Decision |
|------|----------|
| Appointment constraint (MVP) | Appointment = Patient + Procedure + Provider (available at office at time) + Chair (free at that time) + Office (open at that time) |
| Provider-procedure matching | Backlog (not MVP) |
| Chair-procedure matching | Backlog (not MVP) |
| Print schedules | Near-post-MVP (high value, but not MVP) |
| Setup checklist | Persistent, non-blocking, dashboard widget |
| Setup checklist completion order | Any order; each item can complete independently |
| WhatsApp as default contact channel | Yes — dominant channel in Jamaica |

---

## Rule Cards

---

## Practice Identity

---

### Rule P1: Practice must have a name before it is considered configured

**Rule**: The Practice is considered configured once a non-empty name has been set via UpdatePracticeDetails. All other fields are optional.

| # | Example | Type |
|---|---------|------|
| P1a | Admin enters name "Spence Dental" → PracticeDetailsUpdated, practice is configured | ✅ Happy path |
| P1b | Admin submits UpdatePracticeDetails with empty name → Rejected: "Name is required" | ❌ Negative path |
| P1c | Admin submits UpdatePracticeDetails with name = " " (whitespace only) → Rejected: "Name is required" | ❌ Edge case |
| P1d | Practice is configured (name exists) → UpdatePracticeDetails with new name → PracticeDetailsUpdated, name updated | ✅ Happy path |

---

### Rule P2: Practice contact fields are all optional except name

**Rule**: Phone, email, website, and all address fields are optional. Name is the only required field.

| # | Example | Type |
|---|---------|------|
| P2a | Admin sets only name → PracticeDetailsUpdated with phone=null, email=null, etc. | ✅ Happy path |
| P2b | Admin sets name + phone + WhatsApp as preferred channel → PracticeDetailsUpdated with all provided fields | ✅ Happy path |
| P2c | Admin sets full address (line 1, city, parish, country) → PracticeDetailsUpdated with address fields | ✅ Happy path |
| P2d | Admin updates only phone (name already set) → PracticeDetailsUpdated; name preserved from current value | ✅ Happy path |

---

### Rule P3: Address subdivision label is country-aware

**Rule**: The subdivision field is labeled "Parish" for Jamaica. The country field drives the label. MVP defaults country to "Jamaica".

| # | Example | Type |
|---|---------|------|
| P3a | Country = "Jamaica" → subdivision label shown as "Parish" | ✅ Happy path |
| P3b | Admin enters subdivision "St. Andrew" with country "Jamaica" → stored as-is, label is "Parish" | ✅ Happy path |
| P3c | [OPEN QUESTION — Tony to confirm] Other Caribbean country selected → subdivision label shown as generic "Region" or country-specific equivalent? Assumption: use generic "Region" for non-Jamaica countries at MVP. | ❓ Open question |

---

### Rule P4: Practice is a singleton — no create or delete

**Rule**: Exactly one Practice per installation. There is no CreatePractice or DeletePractice command. Only UpdatePracticeDetails exists.

| # | Example | Type |
|---|---------|------|
| P4a | App installs → Practice exists in unconfigured state | ✅ Happy path |
| P4b | UpdatePracticeDetails on unconfigured practice → PracticeDetailsUpdated, practice now configured | ✅ Happy path |
| P4c | UpdatePracticeDetails on already-configured practice → PracticeDetailsUpdated, fields updated | ✅ Happy path |
| P4d | UpdatePracticeDetails with same values as current → PracticeDetailsUpdated still emitted (idempotent write is fine) | ✅ Edge case |

---

## Office

---

### Rule O1: Office requires a name and at least one chair to be created

**Rule**: CreateOffice requires a non-empty name and chair_count >= 1.

| # | Example | Type |
|---|---------|------|
| O1a | Admin creates "Kingston" with 3 chairs → OfficeCreated | ✅ Happy path |
| O1b | Admin creates office with empty name → Rejected: "Name is required" | ❌ Negative path |
| O1c | Admin creates office with chair_count = 0 → Rejected: "At least one chair is required" | ❌ Negative path |
| O1d | Admin creates office with chair_count = 1 → OfficeCreated (minimum valid) | ✅ Boundary |
| O1e | Admin creates two offices with the same name → Both created; soft warning shown ("Another office named 'Kingston' already exists") | ✅ Edge case |

---

### Rule O2: Office operating hours are set per day; unconfigured days are closed

**Rule**: Hours are set day by day via SetOfficeHours. A day with no hours configured is considered closed. Close time must be after open time.

| # | Example | Type |
|---|---------|------|
| O2a | Admin sets Monday 08:00-17:00 → OfficeHoursSet | ✅ Happy path |
| O2b | Monday not configured → Monday is closed; no appointments can be booked | ✅ Happy path |
| O2c | Admin sets hours with close = open (e.g., 09:00-09:00) → Rejected: "Close time must be after open time" | ❌ Negative path |
| O2d | Admin sets hours with close before open (e.g., 17:00-08:00) → Rejected | ❌ Negative path |
| O2e | Admin sets hours for Saturday → OfficeHoursSet; office open on Saturdays | ✅ Happy path |
| O2f | Admin sets hours for all 7 days → OfficeHoursSet x7 | ✅ Happy path |
| O2g | Admin closes a previously-configured day → OfficeDayClosed; that day reverts to closed | ✅ Happy path |
| O2h | Admin re-sets hours for a day that was previously set → OfficeHoursSet; new hours replace old | ✅ Happy path |

---

### Rule O3: Chair count can be changed, with a warning when reducing

**Rule**: UpdateChairCount allows increasing or decreasing. Reducing below the current concurrent appointment count is allowed with a warning, not a block.

| # | Example | Type |
|---|---------|------|
| O3a | Admin increases chairs from 2 to 4 → OfficeChairCountUpdated | ✅ Happy path |
| O3b | Admin reduces chairs from 4 to 2, no concurrent appointments affected → OfficeChairCountUpdated | ✅ Happy path |
| O3c | Admin reduces chairs from 4 to 1 with 2 concurrent appointments already booked → Warning: "Reducing chairs may conflict with existing appointments"; change proceeds | ✅ Edge case |
| O3d | Admin sets chair_count = 0 → Rejected: "At least one chair is required" | ❌ Negative path |

---

### Rule O4: Office can be renamed at any time while active

**Rule**: RenameOffice is valid on any active (non-archived) office. Name must be non-empty.

| # | Example | Type |
|---|---------|------|
| O4a | Admin renames "Kingston" to "Kingston Main" → OfficeRenamed | ✅ Happy path |
| O4b | Admin renames to empty string → Rejected: "Name is required" | ❌ Negative path |
| O4c | Admin renames archived office → Rejected: "Cannot modify an archived office" | ❌ Negative path |

---

### Rule O5: Archiving an office is permanent

**Rule**: ArchiveOffice permanently decommissions the office. There is no UnarchiveOffice. Historical data is preserved.

| # | Example | Type |
|---|---------|------|
| O5a | Admin archives "Montego Bay" → OfficeArchived; office hidden from active lists | ✅ Happy path |
| O5b | Archived office appears in historical appointment records → Data preserved | ✅ Happy path |
| O5c | Admin attempts to archive already-archived office → Rejected: "Office is already archived" | ❌ Negative path |
| O5d | Admin attempts to set hours on archived office → Rejected: "Cannot modify an archived office" | ❌ Negative path |
| O5e | Admin attempts to create new office with same name as archived office → Allowed (new office is independent) | ✅ Edge case |

---

### Rule O6: Setup checklist — office step requires name + ≥1 chair + ≥1 day with hours

**Rule**: The Office step of the setup checklist is complete when at least one active office exists with a name, at least one chair, and at least one day's operating hours configured.

| # | Example | Type |
|---|---------|------|
| O6a | Office created (name + 1 chair) but no hours set → Setup step incomplete | ✅ Happy path |
| O6b | Office created + 1 day hours set → Setup step complete | ✅ Happy path |
| O6c | Only archived office exists → Setup step incomplete | ✅ Edge case |
| O6d | Two offices: one incomplete, one complete → Setup step is complete (at least one satisfies) | ✅ Edge case |

---

## Provider

---

### Rule PR1: Provider requires a name and provider type to be registered

**Rule**: RegisterProvider requires a non-empty name and a valid ProviderType (Dentist, Hygienist, or Specialist).

| # | Example | Type |
|---|---------|------|
| PR1a | Admin registers "Dr. Smith" as Dentist → ProviderRegistered | ✅ Happy path |
| PR1b | Admin registers "Maria" as Hygienist → ProviderRegistered | ✅ Happy path |
| PR1c | Admin registers with empty name → Rejected: "Name is required" | ❌ Negative path |
| PR1d | Admin registers with invalid type → Rejected: "Provider type must be Dentist, Hygienist, or Specialist" | ❌ Negative path |

---

### Rule PR2: Provider must be assigned to an office before availability can be set there

**Rule**: SetProviderAvailability at a given office is only valid after AssignProviderToOffice for that office.

| # | Example | Type |
|---|---------|------|
| PR2a | Admin assigns provider to Kingston → ProviderAssignedToOffice; availability can now be set | ✅ Happy path |
| PR2b | Admin sets availability at Kingston before assigning → Rejected: "Provider must be assigned to this office first" | ❌ Negative path |
| PR2c | Admin assigns provider to two offices → ProviderAssignedToOffice x2; availability can be set at both | ✅ Happy path |
| PR2d | Admin assigns provider to office already assigned → Rejected: "Provider is already assigned to this office" | ❌ Negative path |

---

### Rule PR3: Removing provider from an office automatically clears all availability at that office

**Rule**: RemoveProviderFromOffice produces ProviderRemovedFromOffice plus ProviderAvailabilityCleared for every availability window at that office.

| # | Example | Type |
|---|---------|------|
| PR3a | Provider has availability Mon/Wed/Fri at Kingston → Admin removes from Kingston → ProviderRemovedFromOffice + ProviderAvailabilityCleared x3 | ✅ Happy path |
| PR3b | Provider has no availability at the office → Admin removes → ProviderRemovedFromOffice only | ✅ Edge case |
| PR3c | Provider not assigned to office → RemoveProviderFromOffice → Rejected: "Provider is not assigned to this office" | ❌ Negative path |

---

### Rule PR4: Provider availability must not overlap across offices on the same day

**Rule**: A provider cannot have availability windows at two different offices that overlap in time on the same day.

| # | Example | Type |
|---|---------|------|
| PR4a | Kingston Mon 08:00-12:00 + Montego Bay Mon 13:00-17:00 → Both set; no overlap | ✅ Happy path |
| PR4b | Kingston Mon 08:00-14:00 + Montego Bay Mon 12:00-17:00 → Second rejected: "Provider has overlapping availability at Kingston on Monday (08:00-14:00)" | ❌ Negative path |
| PR4c | Kingston Mon 08:00-12:00 + Montego Bay Mon 12:00-17:00 → Adjacent windows (end = start) → Allowed | ✅ Boundary |
| PR4d | Kingston Mon 08:00-17:00 + Montego Bay Tue 08:00-17:00 → Allowed (different days) | ✅ Happy path |
| PR4e | Same office, same day, different time → Allowed (same-office overlap allowed; capacity is handled by chair count) | ✅ Edge case |

---

### Rule PR5: Provider availability outside office hours generates a warning, not a block

**Rule**: If a provider's availability window extends beyond the office's configured operating hours, the system warns but allows the change. Edge case: holiday coverage, hours adjustment.

| # | Example | Type |
|---|---------|------|
| PR5a | Office hours Mon 08:00-17:00; provider set Mon 08:00-17:00 → ProviderAvailabilitySet, no warning | ✅ Happy path |
| PR5b | Office hours Mon 08:00-17:00; provider set Mon 07:00-17:00 → Warning: "Availability starts before office opens at 08:00"; change proceeds | ✅ Edge case |
| PR5c | Office has no hours for Monday; provider set Monday → Warning: "Office is closed on Monday"; change proceeds | ✅ Edge case |

---

### Rule PR6: Exceptions block the provider across all offices for the date range

**Rule**: ProviderExceptionSet applies to all offices for the specified date range. It overrides weekly availability. Existing appointments are warned about but not cancelled.

| # | Example | Type |
|---|---------|------|
| PR6a | Admin sets exception Dec 20-31 "Holiday" → ProviderExceptionSet; provider unavailable all offices Dec 20-31 | ✅ Happy path |
| PR6b | Exception set over dates with 3 existing appointments → Warning: "3 appointments exist in this date range — they will not be cancelled"; change proceeds | ✅ Edge case |
| PR6c | Exception end_date before start_date → Rejected: "End date must be on or after start date" | ❌ Negative path |
| PR6d | Exception start_date = end_date (single day) → ProviderExceptionSet; valid single-day block | ✅ Boundary |
| PR6e | Admin removes exception → ProviderExceptionRemoved; provider available again per weekly availability | ✅ Happy path |
| PR6f | Remove exception that does not exist → Rejected: "No exception found for that date range" | ❌ Negative path |

---

### Rule PR7: Provider can be archived and unarchived

**Rule**: ArchiveProvider hides the provider from active lists. UnarchiveProvider restores them. Historical appointment data is preserved throughout.

| # | Example | Type |
|---|---------|------|
| PR7a | Admin archives provider → ProviderArchived; provider hidden from active list | ✅ Happy path |
| PR7b | Admin archives already-archived provider → Rejected: "Provider is already archived" | ❌ Negative path |
| PR7c | Admin unarchives provider → ProviderUnarchived; provider visible again | ✅ Happy path |
| PR7d | Admin unarchives active provider → Rejected: "Provider is not archived" | ❌ Negative path |
| PR7e | Archived provider's past appointments → Still viewable in history | ✅ Happy path |

---

### Rule PR8: Setup checklist — provider step requires ≥1 registered, assigned, with ≥1 day availability

**Rule**: The Provider step is complete when at least one active (non-archived) provider exists, is assigned to at least one office, and has availability set for at least one day.

| # | Example | Type |
|---|---------|------|
| PR8a | Provider registered but not assigned → Setup step incomplete | ✅ Happy path |
| PR8b | Provider registered and assigned but no availability set → Setup step incomplete | ✅ Happy path |
| PR8c | Provider registered, assigned, 1 day availability set → Setup step complete | ✅ Happy path |
| PR8d | Provider archived after setup was complete → Setup step reverts to incomplete if no other active provider satisfies | ✅ Edge case |

---

## Procedure Type

---

### Rule PT1: Procedure type requires name, category, and duration within range

**Rule**: DefineProcedureType requires non-empty name, valid ProcedureCategory, and default_duration_minutes between 15 and 240 (inclusive).

| # | Example | Type |
|---|---------|------|
| PT1a | Define "Cleaning" / Preventive / 30 min → ProcedureTypeDefined | ✅ Happy path |
| PT1b | Define with empty name → Rejected: "Name is required" | ❌ Negative path |
| PT1c | Define with invalid category → Rejected: "Category must be one of: Consult, Preventive, Restorative, Invasive, Cosmetic, Diagnostic" | ❌ Negative path |
| PT1d | Define with duration = 14 minutes → Rejected: "Duration must be between 15 and 240 minutes" | ❌ Boundary |
| PT1e | Define with duration = 15 minutes → ProcedureTypeDefined (minimum valid) | ✅ Boundary |
| PT1f | Define with duration = 240 minutes → ProcedureTypeDefined (maximum valid) | ✅ Boundary |
| PT1g | Define with duration = 241 minutes → Rejected: "Duration must be between 15 and 240 minutes" | ❌ Boundary |

---

### Rule PT2: Procedure type can be updated; at least one field must change

**Rule**: UpdateProcedureType requires at least one of name, category, or default_duration_minutes to be provided and different from the current value. Duration must remain in 15-240 range.

| # | Example | Type |
|---|---------|------|
| PT2a | Update name "Cleaning" → "Deep Cleaning" → ProcedureTypeUpdated | ✅ Happy path |
| PT2b | Update duration from 30 to 45 minutes → ProcedureTypeUpdated | ✅ Happy path |
| PT2c | Update with no fields provided → Rejected: "At least one field must be provided" | ❌ Negative path |
| PT2d | Update duration to 10 minutes → Rejected: "Duration must be between 15 and 240 minutes" | ❌ Negative path |
| PT2e | Update name on deactivated procedure type → [OPEN QUESTION — Tony to confirm] Assumption: allowed; deactivation does not freeze the configuration, just hides from scheduling. | ❓ Open question |

---

### Rule PT3: Procedure types can be deactivated and reactivated

**Rule**: DeactivateProcedureType removes the procedure from the active scheduling list. ReactivateProcedureType restores it. Historical appointment records are never affected.

| # | Example | Type |
|---|---------|------|
| PT3a | Admin deactivates "Whitening" → ProcedureTypeDeactivated; hidden from scheduling | ✅ Happy path |
| PT3b | Admin deactivates already-deactivated type → Rejected: "Procedure type is already deactivated" | ❌ Negative path |
| PT3c | Admin reactivates "Whitening" → ProcedureTypeReactivated; available for scheduling again | ✅ Happy path |
| PT3d | Admin reactivates active procedure type → Rejected: "Procedure type is not deactivated" | ❌ Negative path |
| PT3e | Deactivated procedure type referenced in past appointment → Historical record unaffected; procedure name and details preserved | ✅ Happy path |

---

### Rule PT4: Seed defaults load common procedures as individual ProcedureTypeDefined events

**Rule**: On first-run or when the list is empty, the system can seed a standard set of dental procedures. Each seeded procedure emits a standard ProcedureTypeDefined event. No special event type.

| # | Example | Type |
|---|---------|------|
| PT4a | Admin accepts seed defaults → 10 × ProcedureTypeDefined events (Consultation/Consult/30, Cleaning/Preventive/30, Fluoride Treatment/Preventive/15, Exam/Diagnostic/15, X-Ray/Diagnostic/15, Filling/Restorative/45, Crown/Restorative/60, Extraction/Invasive/30, Root Canal/Invasive/90, Whitening/Cosmetic/60) | ✅ Happy path |
| PT4b | Procedure list already has entries → Seed defaults option not shown (or guarded to prevent duplicate definitions) | ✅ Edge case |
| PT4c | Admin defines custom procedures first, then seeds → [OPEN QUESTION — Tony to confirm] Does seeding merge (skip name conflicts) or always emit? Assumption: seed is only offered when list is empty; if non-empty, the Admin must add individually. | ❓ Open question |
| PT4d | Seeded procedures appear in scheduling immediately → Active by default | ✅ Happy path |

---

### Rule PT5: Setup checklist — procedure type step is satisfied by seeded defaults

**Rule**: The Procedure Type step of the setup checklist is complete when at least one active procedure type exists. Seeded defaults satisfy this automatically.

| # | Example | Type |
|---|---------|------|
| PT5a | Seed defaults accepted → Procedure Type step complete | ✅ Happy path |
| PT5b | One procedure manually defined and active → Procedure Type step complete | ✅ Happy path |
| PT5c | All procedure types deactivated → Procedure Type step incomplete | ✅ Edge case |
| PT5d | No procedures defined → Procedure Type step incomplete | ✅ Happy path |

---

## Setup Checklist

---

### Rule SC1: The setup checklist tracks completion of five independent steps

**Rule**: The checklist has five steps. Each step completes independently based on its own criteria. Completion of all five steps indicates the practice is "ready to schedule".

| Step | Completion Criteria |
|------|---------------------|
| 1. Staff Management | ≥1 active StaffMember with PracticeManager role and PIN set |
| 2. Practice | Practice name configured (non-empty name set via UpdatePracticeDetails) |
| 3. Office | ≥1 active office with name, ≥1 chair, and ≥1 day operating hours |
| 4. Provider | ≥1 active provider, assigned to ≥1 office, with ≥1 day availability set |
| 5. Procedure Types | ≥1 active procedure type defined |

| # | Example | Type |
|---|---------|------|
| SC1a | All five steps complete → "Ready to schedule" status reached | ✅ Happy path |
| SC1b | Steps 2-5 complete but Step 1 (Staff Management) incomplete → Not ready to schedule | ✅ Happy path |
| SC1c | Steps complete in any order → Checklist shows each step's status independently | ✅ Happy path |

---

### Rule SC2: The setup checklist is non-blocking

**Rule**: The checklist is informational only. A user can navigate to any part of the application at any time, regardless of checklist completion status.

| # | Example | Type |
|---|---------|------|
| SC2a | Practice name not set → Admin can still navigate to Office setup | ✅ Happy path |
| SC2b | No providers registered → Admin can still access Procedure Types | ✅ Happy path |
| SC2c | Checklist shown persistently on dashboard as a widget, not a modal gate | ✅ Happy path |

---

### Rule SC3: The setup checklist reflects current state, including reversals

**Rule**: If a completed step later becomes incomplete (e.g., the only provider is archived), the step reverts to incomplete on the checklist.

| # | Example | Type |
|---|---------|------|
| SC3a | Provider step was complete; only active provider archived → Provider step reverts to incomplete | ✅ Edge case |
| SC3b | Office step was complete; only active office archived → Office step reverts to incomplete | ✅ Edge case |
| SC3c | All procedure types deactivated → Procedure Type step reverts to incomplete | ✅ Edge case |
| SC3d | Checklist re-completes when requirements are restored (new provider registered and configured) | ✅ Edge case |

---

### Rule SC4: "Ready to schedule" threshold

**Rule**: The practice is considered ready to schedule when all five setup checklist steps are complete simultaneously. This is a derived status — not a separate event.

| # | Example | Type |
|---|---------|------|
| SC4a | All 5 steps complete → Dashboard shows "Ready to schedule" indicator | ✅ Happy path |
| SC4b | 4 of 5 steps complete → Dashboard shows which step remains | ✅ Happy path |
| SC4c | [OPEN QUESTION — Tony to confirm] Is "ready to schedule" status shown visually as a distinct banner, colored progress bar, or checklist completion indicator? Assumption: checklist widget with checkmarks per step and a summary "X of 5 complete" indicator. | ❓ Open question |

---

## Open Questions Summary

| # | Question | Artifact Ref | Assumption Made |
|---|----------|-------------|-----------------|
| P3c | Subdivision label for non-Jamaica Caribbean countries | Rule P3c | Show generic "Region" label for non-Jamaica countries at MVP |
| PT2e | Can a deactivated procedure type still be updated? | Rule PT2e | Yes; deactivation only hides from scheduling, does not freeze config |
| PT4c | Does seeding merge with existing procedures or require empty list? | Rule PT4c | Seed is only offered when the list is empty |
| SC4c | Visual treatment of "ready to schedule" indicator | Rule SC4c | Checklist widget with per-step checkmarks and "X of 5 complete" summary |

---

**Phase 2.3 Acceptance Criteria Review**: All business rules validated against Practice aggregate, Office aggregate, Provider aggregate, and ProcedureType aggregate docs. Ubiquitous language used throughout. Open questions flagged for Tony; reasonable assumptions made in each case. Ready for Phase 2.5 governance review after Tony confirms open questions.
