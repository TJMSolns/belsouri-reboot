# PDR-003: Journey-Aware Navigation Architecture

**Status**: Accepted
**Date**: 2026-03-04
**Deciders**: Tony (Product Owner)
**Category**: Product / UX Architecture
**Source**: `style-guide-final.html` §2.6

---

## Guiding Principle

> "Every user arrives at any screen with an intent: a goal they were already pursuing before they got here. Our job is to support that intent to completion without adding friction that forces them to abandon or restart."

---

## Context

The most common UX failure in practice management software is not missing features — it is **interrupting users who already know what they are trying to do**. A receptionist booking a 9 a.m. appointment has intent. A provider checking a patient record between procedures has intent. Software that forces them to abandon their context to complete a sub-task directly causes errors, delays, and support calls.

Staff in our target environment are working fast, under time pressure, and between patients. Every unnecessary navigation step doubles the risk of error or abandonment.

---

## Decision

**Every navigation decision must be classified as page, confirmation modal, or bottom sheet based on the user's relationship to their current intent — not based on how much content there is.**

### The Three Layout Patterns

| Pattern | When to use | Governing rule |
|---|---|---|
| **Full Page** | The user's intent IS to be here. This is a terminus or deliberate new context. | May have full CRUD, deep links, back navigation. Back button returns to the previous intent. |
| **Confirmation Modal** | A single irreversible decision within the current intent. The user does not leave. | Two actions maximum (confirm + cancel). Never collect data. Escape/Cancel restores previous state exactly. |
| **Bottom Sheet / Side Panel** | A focused sub-task that is a step in the user's current intent — not a destination. | Background context stays visible and locked. Completing the sheet returns the user to exactly where they were. |

### Intent Continuity Rules

1. **Never navigate away from a sheet to collect required information.** If a sub-task requires selecting a provider, bring the selector into the sheet — do not navigate to a provider list page.

2. **CTAs must carry intent forward, not restart it.** "Book Appointment" on a patient page must pre-populate the patient in the booking sheet. The user already identified the subject — honour that.

3. **Closing a sheet always restores context exactly.** If staff opened a sheet from row 47 in a table, Escape closes the sheet and row 47 is still visible. Scroll position is preserved.

4. **Modals are not sheets.** Never collect multi-field data in a confirmation modal. Modals are for single yes/no decisions only.

5. **Full-page views must support deep links.** Pages are addressable; sheets and modals are not. Every page must make sense when navigated to directly.

6. **Avoid multi-step wizards** unless the task is genuinely sequential and each step requires the output of the previous. A flat form with clear sections is faster for experienced users.

### Journey-Portable Components

Every detail view (Patient Detail, Staff Profile, Appointment Detail) must be designed to work in both contexts:

- **As a full page**: reached by direct navigation — full history, full CRUD, all actions available
- **As an embedded panel**: reached mid-journey (e.g., verifying a patient during booking) — summary view, context preserved behind it, closes back to the originating task

Components must not assume they know why the user is viewing them.

---

## Consequences

**Positive:**
- Staff complete tasks without losing context — fewer errors, less frustration
- Shorter task completion time — no unnecessary round-trips through page navigation
- CTAs that pre-populate context feel intelligent and responsive
- Consistent "Escape always works" behaviour builds user confidence

**Negative / Constraints:**
- Components must be designed for dual-context use (page AND panel) — this adds design complexity upfront
- Sheet state must be preserved on close and reliably restored — requires Svelte store or local state management discipline
- Cannot take shortcuts like "just navigate to the new page" for quick feature prototypes — architecture must be correct from the start

---

## Compliance

The `/ux-review` skill enforces this policy. Any component that navigates away from an in-progress task, discards user context, or opens a blank form when the subject is already known fails this policy.

---

## References

- `style-guide-final.html` §2.6 — Journey-Aware Layouts (full specification with examples)
- `PDR-002-android-first-ux.md` — parent principle (Android bottom sheet pattern)
- `POL-003-ux-standards.md` — checklist implementation of these rules
- `style-guide-final.html` §9 — Dialogs & Overlays (rendered component examples)
