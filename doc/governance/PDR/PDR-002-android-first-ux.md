# PDR-002: Android-First UX Design Principle

**Status**: Accepted
**Date**: 2026-03-04
**Deciders**: Tony (Product Owner)
**Category**: Product / UX
**Source**: `style-guide-final.html` §2.3

---

## Guiding Principle

> "If it would confuse a first-time Android user, it will confuse our staff."

---

## Context

Belsouri targets dental practice staff in Jamaica and the Caribbean. Extensive user research (Nico interviews, `../belsouri-old/doc/internal/research/`) established that our primary users:

- Are healthcare professionals, **not** computer scientists
- Are more comfortable with their Android smartphone than with office software
- Come from paper-based workflows — they are paper-trained, not software-trained
- Have limited patience for software that requires training, documentation, or a help desk
- Work in clinical environments under time pressure, often between patients

Traditional desktop software UI conventions (dense menus, small click targets, modal-heavy workflows, wizard forms, complex navigation hierarchies) are hostile to this user profile. Android / Material Design conventions are significantly better aligned — they have been designed for users who are task-focused, phone-native, and intolerant of friction.

---

## Decision

**All Belsouri UI/UX decisions default to Android/Material Design conventions over traditional desktop software conventions.**

Specifically:

| Android/Material Pattern | Why it works for our users |
|---|---|
| Large touch targets (minimum 44×44px) | Reduces mis-taps, works for users in gloves or clinical settings |
| Card-based content organisation | Familiar from Android apps; clear visual boundaries between entities |
| Bottom sheets for sub-tasks | Preserves context; familiar from Android drawer pattern |
| Prominent, labelled CTAs | Reduces cognitive load; action is always obvious |
| Snackbar/toast notifications | Non-blocking feedback; familiar from Android UX |
| Contextual action menus (not toolbars) | Actions appear where needed, not buried in menus |
| Progressive disclosure | Start with summary; reveal detail on demand |

---

## Consequences

**Positive:**
- UI will feel intuitive to users who already use WhatsApp, banking apps, and Google apps on Android
- Reduces training burden — staff can onboard by exploration, not by reading a manual
- Larger tap targets reduce errors in clinical environments

**Negative / Constraints:**
- Some desktop-native UX patterns are inappropriate (e.g. right-click context menus, hover-only affordances, small form fields)
- UI density will be lower than traditional practice management software — some screens will need more vertical scrolling
- Feature discoverability must be explicit (visible icons and labels) — cannot rely on "power user" conventions

**What this does NOT mean:**
- We do not build a mobile app — this is a desktop application
- We do not follow Material Design's specific visual language (colours, shapes) — we use our own brand system
- We do not slavishly copy Android patterns where they do not fit — the principle is *familiarity*, not *literal implementation*

---

## Compliance

Any frontend component or page that would confuse a first-time Android user must be revised before it ships. The `/ux-review` skill enforces this principle.

---

## References

- `style-guide-final.html` §2.3 — The Android-First Desktop
- `style-guide-final.html` §2.1 — Who Uses Belsouri (persona cards)
- `style-guide-final.html` §2.2 — The Paper-to-Screen Mental Model
- `POL-003-ux-standards.md` — UX Standards checklist derived from this principle
- `PDR-003-journey-aware-navigation.md` — Navigation architecture corollary
