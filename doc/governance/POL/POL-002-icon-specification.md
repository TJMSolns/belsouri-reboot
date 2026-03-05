# POL-002: Icon Technical Specification

**Status**: Active
**Date**: 2026-03-04
**Applies to**: All inline SVG icons in Svelte components (`src/`)
**Source**: `style-guide-final.html` §6.1

---

## Policy

**All icons in Belsouri must be inline SVG, drawn to a 24×24 viewBox with 2px round-cap strokes using `currentColor`. No icon font libraries, no external sprite sheets, no PNG/WebP icons.**

---

## Specification

| Property | Value | Notes |
|---|---|---|
| `viewBox` | `0 0 24 24` | Always. No exceptions. |
| `fill` | `none` | For all stroke icons. See Tooth exception below. |
| `stroke` | `currentColor` | Inherits from CSS context. Never hardcode a colour. |
| `stroke-width` | `2` | Use `1.5` at rendered sizes ≥ 40px |
| `stroke-linecap` | `round` | Always |
| `stroke-linejoin` | `round` | Always |
| `xmlns` | Not required | Omit in Svelte inline SVG |

### Minimum required attributes for every icon SVG element:

```svelte
<svg
  width="{size}"
  height="{size}"
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="2"
  stroke-linecap="round"
  stroke-linejoin="round"
>
```

---

## Size Tokens

| Size | Context |
|---|---|
| 16px | Inline with text — nav labels, badge icons, button labels |
| 20px | Icon-only action buttons — table row controls, action bar |
| 24px | Card headers, section titles |
| 40px | Empty-state hero icons — reduce stroke-width to 1.5px at this size |

---

## Tooth Icon Exception

The tooth/procedure icon uses fill, not stroke. At small sizes (16–20px), a stroked tooth path loses legibility. The Tooth icon must use:

```svelte
<svg ... fill="currentColor" style="opacity: 0.85">
```

No other icon may use `fill="currentColor"`. All other icons remain stroke-only.

---

## Colour Binding Rule

Icons must **never** have hardcoded `stroke` or `fill` colour values. Always use `stroke="currentColor"` and control the colour from the parent element's CSS:

```svelte
<!-- Correct -->
<span style="color: var(--caribbean-teal)">
  <svg stroke="currentColor" ...>
</span>

<!-- Violation -->
<svg stroke="#008B99" ...>
```

This ensures icons respect theme changes and dark-mode adaptations without modification.

---

## Color + Icon Pairing Rule

Icons must never be the sole indicator of meaning. Every icon that communicates state (status badges, action buttons, feedback) must be paired with either:
- A text label, OR
- A background/border colour that is also communicated by shape

This is required for WCAG 1.4.1 (Use of Colour) compliance and for users in bright or poor lighting conditions.

---

## Implementation

Icons are implemented as **inline Svelte components** at MVP. One component per icon:

```
src/lib/components/icons/
  IconPatients.svelte
  IconSchedule.svelte
  IconBooked.svelte
  IconTooth.svelte
  ...
```

Each component accepts a `size` prop (default `24`) and inherits colour from its parent context via `currentColor`.

---

## Enforcement

- `/icon-audit` skill: mechanically checks all SVG elements in a Svelte file against this spec
- `CLAUDE.md` Design System Conventions: always-on rule during development

---

## References

- `style-guide-final.html` §6.1 — Icon Technical Spec Table
- `style-guide-final.html` §6.2 — Icon Size Matrix (rendered at 16/20/24/40px)
- `style-guide-final.html` §4 — Iconography Catalog (all icon SVG paths)
- `POL-001-design-token-usage.md` — colour token rules that apply to icon stroke colours
