---
name: copy-check
description: Scans UI copy (button labels, headings, error messages, toast text, empty state text, field labels) against Belsouri Voice & Copy Standards. Flags generic SaaS language, non-compliant error messages, wrong date/time/currency formats, and incorrect job titles.
allowed-tools: Read, Glob, Grep
user-invocable: true
argument-hint: [component-path or directory]
---

# Copy Check

You review user-facing text in Belsouri Svelte components against the Voice & Copy Standards defined in `style-guide-final.html` §7 and `doc/governance/POL/POL-003-ux-standards.md`.

Belsouri serves healthcare professionals in Jamaica and the Caribbean. Every label, error, confirmation, and toast reflects care for patients and respect for clinical staff — not generic SaaS conventions.

## Your Process

1. Read the file(s) identified by the user
2. Extract all user-facing strings: button labels, headings, error messages, toasts, placeholders, empty state copy, confirmation dialog text, field labels, ARIA labels
3. Check each string against the rules below
4. Output a structured report

## Check 1: Care-Based Verbs

Replace generic SaaS verbs with care-based alternatives:

| Flag this | Replace with |
|---|---|
| Submit | Save Record |
| Delete | Deactivate |
| Remove (permanent) | Deactivate (if soft) |
| Manage Users | [specific action] |
| Admin | Practice Manager |
| Administrator | Practice Manager |
| Doctor | Provider (covers dentists, hygienists, specialists) |
| User | [name the role: Practice Manager, Provider, Staff] |
| User ID / Account | [name the person: "Williams, Asha"] |
| Record deleted | [name the record: "Patient Williams, A. deactivated"] |
| Operation successful | [name what happened: "Appointment booked. Williams, A. — ..."] |

Flag any label or confirmation that uses generic SaaS language when specific, care-based copy is available.

## Check 2: Error Message Standard (POL-003)

Every error message must include:
1. **The specific object** — name the patient, field, office, provider, or entity with the problem
2. **The specific problem** — what exactly is wrong
3. **The resolution path** — what the user should do (where possible)

**Violations to flag:**
- "An error occurred. Please try again." → no object, no problem, no resolution
- "Invalid input." → no object, no problem
- "Something went wrong." → generic
- "Please check your entries." → no specifics
- "Error: [technical code or exception name]" → never expose technical errors to the user

**Compliant examples:**
- "Phone number must be 10 digits (e.g. 876-555-0147)."
- "No chairs available at Main Office at 10:00 AM — all 3 chairs are booked. Try 10:30 AM or select a different office."
- "Date of birth must use DD/MM/YYYY format (e.g. 14/03/1985)."

## Check 3: Confirmation & Toast Copy

Confirmation dialogs must name the specific record:
- Flag: "Are you sure you want to remove this patient?"
- Correct: "Remove Williams, Asha?" (heading), with explanation below

Success toasts must confirm what happened with enough detail to verify:
- Flag: "Saved successfully."
- Correct: "Appointment booked. Williams, A. — Tuesday 18/03/2026 at 10:00 AM with Dr. Thompson."

## Check 4: Localisation Standards

**Date format**: DD/MM/YYYY — never MM/DD/YYYY
- Flag: "03/14/1985" → Correct: "14/03/1985"
- Flag: "March 14, 1985" → Correct: "14 March 1985"
- Placeholders should show the Jamaican format: `DD/MM/YYYY`

**Time format**: 12-hour AM/PM — never 24-hour
- Flag: "14:30" → Correct: "2:30 PM"

**Currency**: JMD by default — never USD unless explicitly configured
- Flag: "$450.00" (ambiguous) → Correct: "JM$450.00" or "JMD 450.00"

**WhatsApp**: Must be listed as a primary contact channel, equal to Phone and Email — never deprioritised or unlabelled as "Other"

**Job Titles**:
- "Practice Manager" — not Admin, Administrator, Manager, Office Manager
- "Provider" — not Doctor, Dentist, Clinician (covers all clinical roles)
- "Staff" — for reception and support roles

## Check 5: Placeholder Text

Placeholders must be examples, not labels:
- Flag: placeholder="First Name" (label is doing double duty — the label above should cover this)
- Correct: placeholder="e.g. Asha" or placeholder="DD/MM/YYYY"
- Placeholders must never be the only label for a field (fails accessibility)

## Check 6: Empty State Copy

Empty state headings must be specific:
- Flag: "No records found."
- Correct: "No appointments today." / "No patients registered yet." / "No providers set up."

Empty state CTAs must be direct:
- Flag: "Get started" (vague)
- Correct: "Book First Appointment" / "Add Patient" / "Set Up Practice"

## Output Format

```
COPY CHECK
==========

FILE: [path]

CARE-BASED VERBS: [PASS / FLAG / FAIL]
Violations:
  - Line XX: "[quoted text]" → should be "[suggested replacement]"

ERROR MESSAGES: [PASS / FLAG / FAIL]
Violations:
  - Line XX: "[quoted message]"
    Missing: [object / problem / resolution]
    Suggested: "[better version]"

CONFIRMATION & TOAST COPY: [PASS / FLAG / N/A]
Violations:
  - Line XX: "[quoted text]" → [what's wrong + suggestion]

LOCALISATION: [PASS / FLAG / N/A]
Violations:
  - Line XX: "[quoted text]" → [correct format]

JOB TITLES: [PASS / FLAG / N/A]
Violations:
  - Line XX: "[wrong title]" → "[correct title]"

PLACEHOLDERS: [PASS / FLAG / N/A]
Violations:
  - Line XX: [description]

EMPTY STATES: [PASS / FLAG / N/A]
Violations:
  - Line XX: "[quoted text]" → [suggestion]

OVERALL: [PASS / NEEDS WORK / FAIL]

ACTION ITEMS:
1. [Specific fix with line reference]
2. ...

READY FOR DONE-CHECKER: [YES / NO]
```

## References

- `style-guide-final.html` §7 — Voice & Copy Standards
- `style-guide-final.html` §7.1 — Care-Based Verbs table
- `style-guide-final.html` §7.2 — Error Message Standard
- `style-guide-final.html` §7.3 — Localisation Notes
- `doc/governance/POL/POL-003-ux-standards.md` — §Error Message Standard
