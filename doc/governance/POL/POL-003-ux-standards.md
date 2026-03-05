# POL-003: UX Standards

**Status**: Active
**Date**: 2026-03-04
**Applies to**: All frontend screens, components, and user-facing interactions
**Source**: `style-guide-final.html` §2.4, §2.5, §2.6

---

## Policy

**Every screen, form, and action in the app must meet the standards below before it ships. These are not aspirational guidelines — they are shipment gates.**

---

## Core Philosophy

Our users are healthcare professionals who are paper-trained and phone-native. They are not software users by vocation — they use software to serve patients. Every UX decision must honour that reality.

Three principles govern everything (from `style-guide-final.html` §2.4):

1. **Coddle** — guide users at every step, never leave them guessing
2. **Protect** — prevent errors before they happen; catch them gently when they do
3. **Guide** — every state has a clear next action

---

## Standards Checklist

These are verified by the `/ux-review` skill and by the `done-checker` UI/Design section.

### Feedback & Communication

- [ ] Every error message names the **specific object**, the **specific problem**, and where possible the **resolution path**. "An error occurred" is a violation.
- [ ] Every successful action has a confirmation toast with enough detail to verify the right thing happened (e.g. "Appointment booked. Williams, A. — Tuesday 18/03/2026 at 10:00 AM.")
- [ ] Every async action has a visible loading state — the user is never left wondering if their tap registered (button disabled + spinner)
- [ ] Warnings distinguish between "you probably should not do this" and "this action cannot be undone"

### Data Safety

- [ ] No data is permanently deleted — only deactivated. Destructive actions require explicit confirmation in a named modal ("Remove Williams, Asha?")
- [ ] No navigation step discards user input without an explicit warning and the option to go back
- [ ] Forms preserve user input on error — never clear a field on validation failure

### Navigation & Context (PDR-003)

- [ ] Sub-tasks that can complete without navigating away use a sheet — not full-page navigation
- [ ] CTAs carry existing context forward — "Book Appointment" from a patient page pre-populates the patient
- [ ] Closing a sheet returns the user to exactly where they were (scroll position preserved)
- [ ] Every full-page view supports deep-linking

### Forms

- [ ] Labels are always visible above the field — never inside as placeholder-only
- [ ] Required fields are marked with a visible indicator
- [ ] Validation fires on blur (leaving the field), not on keystroke
- [ ] Tab order follows visual reading order (top-to-bottom, left-to-right)
- [ ] Every form has a clear primary action and an equally accessible cancel path

### Empty States

- [ ] Every empty state has exactly one clear call to action — never leave the user on a blank page
- [ ] Empty state copy is specific ("No appointments today") not generic ("No records found")

### Navigation & Orientation

- [ ] The current section is always highlighted in the navigation — the user always knows where they are
- [ ] The page title or heading communicates where the user is and what they are doing
- [ ] Back navigation returns to the previous intent, not an arbitrary page

### Accessibility (functional)

- [ ] Every icon-only button has an accessible label (`aria-label`)
- [ ] Every interactive element has a visible focus ring
- [ ] Color is never the sole indicator of state (always paired with icon or text)
- [ ] Touch/click targets are minimum 44×44px

---

## Error Message Standard

### Structure

Every error must include:
1. **The object**: name the specific thing that has the problem ("Phone number", "Main Office", "Williams, Asha")
2. **The problem**: describe what is wrong specifically ("must be 10 digits", "has no chairs available at 10:00 AM")
3. **The resolution** (where possible): tell the user what to do ("Try 10:30 AM or select a different office")

### Examples

| Violation | Compliant |
|---|---|
| "An error occurred. Please try again." | "Could not save patient record — phone number must be 10 digits (e.g. 876-555-0147)." |
| "Invalid input." | "Date of birth must use DD/MM/YYYY format (e.g. 14/03/1985)." |
| "Booking failed." | "No chairs available at Main Office at 10:00 AM — all 3 chairs are booked. Try 10:30 AM or select a different office." |
| "Operation successful." | "Appointment booked. Williams, A. — Tuesday 18/03/2026 at 10:00 AM with Dr. Thompson." |

---

## Friction Audit

Before shipping any navigation decision or CTA, ask:

1. Will the user lose any work or context if they follow this path?
2. If they tap Back, will they end up somewhere sensible?
3. Is any information the user has already provided being thrown away?
4. Could this action be completed in a sheet without navigating away?
5. Is this navigation step adding time, or just adding ceremony?

---

## Enforcement

- `/ux-review` skill: LLM review of any component against this checklist
- `/copy-check` skill: validates error messages, labels, and toast copy against the error message standard
- `done-checker` UI/Design section: blocks marking UI work as done without checklist confirmation
- `CLAUDE.md` Design System Conventions: always-on rules during development

---

## References

- `style-guide-final.html` §2.4 — Coddle, Protect, Guide principles
- `style-guide-final.html` §2.5 — UX Standards Checklist (rendered version)
- `style-guide-final.html` §2.6 — Journey-Aware Layouts (navigation rules)
- `style-guide-final.html` §7.2 — Error Message Standard (examples)
- `PDR-002-android-first-ux.md` — user profile that these standards serve
- `PDR-003-journey-aware-navigation.md` — navigation architecture rules
