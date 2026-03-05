---
name: icon-audit
description: Audits all inline SVG icons in a Svelte component against the Belsouri icon technical specification (POL-002). Checks viewBox, stroke attributes, fill rules, colour binding, size usage, and the colour+icon pairing rule. Most checks are mechanical.
allowed-tools: Read, Glob, Grep
user-invocable: true
argument-hint: [component-path or directory]
---

# Icon Audit

You audit all inline SVG elements in Belsouri Svelte component files against the icon technical specification in `doc/governance/POL/POL-002-icon-specification.md` and `style-guide-final.html` §6.1.

Most checks are mechanical — you are verifying that required SVG attributes are present and correct, and that no hardcoded colours have slipped in.

## Your Process

1. Read the file(s) identified by the user
2. Find every `<svg` element
3. For each SVG, run all checks below
4. Note the line number of each violation
5. Output a structured report

## Check 1: viewBox

Every icon SVG must have `viewBox="0 0 24 24"`.

- Flag: missing `viewBox`
- Flag: `viewBox` with any value other than `"0 0 24 24"`
- Exception: non-icon SVGs (e.g. logo SVGs, illustration SVGs) — note these separately, do not flag

## Check 2: fill

Stroke icons must have `fill="none"`.

- Flag: `fill` attribute missing (defaults to black — will fill all paths)
- Flag: `fill="#..."` or `fill="rgb(..."` (hardcoded colour — violation of POL-001)
- Flag: `fill="currentColor"` on any icon that is NOT the Tooth/Procedure icon

**Tooth icon exception**: The tooth icon may use `fill="currentColor"` with `style="opacity: 0.85"` or `opacity="0.85"`. This is the only fill exception.

## Check 3: stroke

Stroke icons must use `stroke="currentColor"`.

- Flag: `stroke="#..."` — hardcoded hex colour (POL-001 violation)
- Flag: `stroke="rgb(..."` — hardcoded colour
- Flag: `stroke` attribute missing entirely on a stroke icon
- Flag: `stroke="var(--some-token)"` — while better than hex, still violates the `currentColor` rule; colour should be set on the parent element, not the SVG

## Check 4: stroke-width

- Correct: `stroke-width="2"` for icons rendered at sizes < 40px
- Correct: `stroke-width="1.5"` for icons rendered at 40px (empty state heroes)
- Flag: `stroke-width="1"` — too thin
- Flag: `stroke-width="3"` — too heavy
- Flag: `stroke-width` missing entirely

## Check 5: stroke-linecap

Must be `stroke-linecap="round"`.

- Flag: `stroke-linecap` missing
- Flag: `stroke-linecap="butt"` or `stroke-linecap="square"`

## Check 6: stroke-linejoin

Must be `stroke-linejoin="round"`.

- Flag: `stroke-linejoin` missing
- Flag: `stroke-linejoin="miter"` or `stroke-linejoin="bevel"`

## Check 7: Size Tokens

Check the `width` and `height` on each SVG against expected contexts:

| Size | Expected context |
|---|---|
| 16px | Nav labels, badge icons, inline button labels |
| 20px | Icon-only action buttons, table row controls |
| 24px | Card headers, section titles |
| 40px | Empty-state hero icons (must use stroke-width 1.5) |

- Flag: icon at a size not in this table (e.g. 18px, 22px, 32px outside icon grid demos)
- Flag: 40px icon with `stroke-width="2"` (should be 1.5)
- Flag: icon sized with `em` or `%` — always use `px` for icon sizes

## Check 8: Colour + Icon Pairing Rule (POL-002)

Any SVG icon that communicates state (status badges, action buttons, feedback toasts, role badges) must not be the sole indicator of that state. There must also be either:
- A text label adjacent to the icon, OR
- A shape distinction between different states

- Flag: icon-only status badge with no text label and no other shape distinction
- Flag: icon-only action button with no `aria-label` attribute

## Check 9: ARIA on Icon-Only Buttons

Any `<button>` or `<a>` that contains only an SVG icon (no visible text label) must have an `aria-label` attribute.

- Flag: `<button><svg ...></svg></button>` with no `aria-label`

## Complete Correct Example

```svelte
<!-- Correct: icon-only edit button -->
<button class="btn-icon" aria-label="Edit patient Williams, Asha">
  <svg
    width="20"
    height="20"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round"
  >
    <path d="M17 3a2.828 2.828 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z"/>
  </svg>
</button>

<!-- Correct: colour set on parent, not SVG -->
<span style="color: var(--caribbean-teal)">
  <svg width="20" height="20" viewBox="0 0 24 24" fill="none"
    stroke="currentColor" stroke-width="2"
    stroke-linecap="round" stroke-linejoin="round">
    ...
  </svg>
</span>
```

## Output Format

```
ICON AUDIT
==========

FILE: [path]
ICONS FOUND: [count]

VIOLATIONS:

  Line XX — [SVG description or icon name]
    - [attribute]: [what's wrong] → should be [correct value]
    - [attribute]: ...

  Line XX — [SVG description]
    - ...

SUMMARY:
  viewBox violations:       X
  fill violations:          X
  stroke violations:        X
  stroke-width violations:  X
  linecap violations:       X
  linejoin violations:      X
  size token violations:    X
  aria-label missing:       X

OVERALL: [PASS / NEEDS WORK / FAIL]

ACTION ITEMS:
1. Line XX: [specific fix]
2. ...

READY FOR DONE-CHECKER: [YES / NO]
```

## References

- `doc/governance/POL/POL-002-icon-specification.md` — full spec
- `doc/governance/POL/POL-001-design-token-usage.md` — colour token rules
- `style-guide-final.html` §6.1 — Icon Technical Spec Table
- `style-guide-final.html` §6.2 — Icon Size Matrix
- `style-guide-final.html` §4 — Iconography Catalog (all canonical icon SVG paths)
