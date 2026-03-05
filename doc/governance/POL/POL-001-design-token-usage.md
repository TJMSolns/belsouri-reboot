# POL-001: Design Token Usage

**Status**: Active
**Date**: 2026-03-04
**Applies to**: All Svelte frontend code (`src/`)
**Source**: `style-guide-final.html` §3.1, §6.4

---

## Policy

**All colour values in Svelte components must use CSS custom properties defined in `src/app.css`. Hardcoded hex, RGB, or HSL colour values are prohibited in Svelte files.**

---

## Canonical Token Names

### Brand Palette

| Token | Hex | Primary use |
|---|---|---|
| `--caribbean-teal` | `#008B99` | Primary actions, navigation, Add/Edit/Save |
| `--healthy-coral` | `#FF7F6A` | Destructive & alert: Remove, Cancel, Reset PIN |
| `--pearl-mist` | `#F0F4F5` | App background, table headers, icon tiles |
| `--abyss-navy` | `#1A2D33` | Nav bar background, headings, body text |
| `--slate-fog` | `#6B7C82` | Neutral/muted: Search, Archive, No-Show |
| `--island-palm` | `#27AE60` | Success, Completed appointments, Verify PIN |

### Appointment Status

| Token | Hex | Status |
|---|---|---|
| `--color-booked` | `#008B99` | Booked |
| `--color-completed` | `#27AE60` | Completed |
| `--color-cancelled` | `#FF7F6A` | Cancelled |
| `--color-noshow` | `#6B7C82` | No-Show |
| `--color-rescheduled` | `#5B7FA6` | Rescheduled |

### Staff Roles

| Token | Hex | Role |
|---|---|---|
| `--color-role-pm` | `#008B99` | Practice Manager |
| `--color-role-provider` | `#27AE60` | Provider |
| `--color-role-staff` | `#6B7C82` | Staff |

---

## Rules

1. **No hardcoded hex in Svelte files.** `color: #008B99` is a violation. `color: var(--caribbean-teal)` is correct.

2. **Tokens must be used semantically.** Using `--healthy-coral` for a success message because "it looks nice" is a violation. Tokens have defined semantic roles — use the right token for the right context.

3. **New colours require a token.** If a colour is needed that has no existing token, add it to `src/app.css` with a name that describes its semantic role, not its appearance. `--error-text` not `--dark-red`.

4. **`--rescheduled` is a status-only token**, not a general-purpose palette colour. It may only appear on the Rescheduled status badge.

5. **Coral on Pearl Mist or white backgrounds for body text is prohibited** — the contrast ratio fails WCAG AA. Coral may only be used as stroke/icon colour, or as text on coral-tinted error backgrounds (`#FFEAE6`, `#FFF5F4`).

---

## Enforcement

- `CLAUDE.md` Design System Conventions section: always-on rule during development
- `/icon-audit` skill: flags hardcoded stroke colours in SVG icons
- `/ux-review` skill: flags hardcoded colours in component style blocks

---

## Adding New Tokens

To add a new design token:
1. Confirm no existing token covers the use case
2. Add to `src/app.css` in the appropriate group with a semantic name
3. Update this document's table
4. Reference `style-guide-final.html` §3.1 to verify alignment with the palette

---

## References

- `style-guide-final.html` §3.1 — Color Palette (swatches + semantic roles)
- `style-guide-final.html` §6.4 — CSS Variables Block (canonical `:root {}`)
- `style-guide-final.html` §6.3 — Semantic Color Map
- `src/app.css` — implementation
