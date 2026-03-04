# Practice Aggregate

**Context**: Practice Setup
**Last Updated**: 2026-03-03

---

## Purpose

The root organizational entity representing the dental clinic business. Singleton per installation -- there is exactly one Practice. Holds the practice's identity: name, contact information, and address.

This is the first thing configured on a fresh install and provides the practice-wide identity that appears on reports, patient communications, and the application header.

---

## Fields

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| name | String | Yes | Practice business name |
| phone | String | No | Primary contact number |
| email | String | No | Practice email address |
| website | String | No | Practice website URL |
| address_line_1 | String | No | Street address |
| address_line_2 | String | No | Suite, unit, etc. |
| city_town | String | No | City or town name |
| subdivision | String | No | Country-aware region. Labeled "Parish" for Jamaica. |
| country | String | No | Defaults to "Jamaica" for MVP. Drives subdivision label. |

---

## Events

- **PracticeDetailsUpdated**: Admin sets or changes any practice identity field. Same event for first-time setup and subsequent edits. Contains all fields (unchanged fields carry current values).

---

## Commands

| Command | Input | Produces | Preconditions |
|---------|-------|----------|---------------|
| UpdatePracticeDetails | name, phone?, email?, website?, address fields? | PracticeDetailsUpdated | name not empty |

---

## Invariants

1. **Name required**: Practice must have a non-empty name
2. **Singleton**: Exactly one Practice per installation -- no create/delete, only update

---

## State Machine

Practice has no lifecycle states -- it exists from first configuration onward. There is no "created" or "archived" state; it is always present.

```
[Unconfigured] --UpdatePracticeDetails--> [Configured] --UpdatePracticeDetails--> [Configured]
```

---

## Design Decisions

- **No separate Set/Update events**: Tony decided a single `PracticeDetailsUpdated` event suffices for both first-time setup and subsequent edits.
- **Structured address with country-aware subdivision**: Jamaica uses "Parish"; other Caribbean countries will use their local equivalent. The subdivision label is driven by the country field.
- **No ID field**: Singleton aggregate -- implicitly identified. If needed for event stream keying, use a fixed ID like `"practice"`.

---

## Cross-Context Usage

Practice identity is read by:
- **Reporting**: Practice name on reports and dashboards
- **Patient Communications**: Practice contact info on appointment confirmations (future)
- **Sync Engine**: Practice identity in sync payloads (future)

---

**Maintained By**: Tony + Claude
