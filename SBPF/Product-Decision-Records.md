# Product Decision Records (PDRs)

**Category:** Documentation & Knowledge Management, Product Management
**Related:** Architecture-Decision-Records.md, Policy-Documents.md, DDD
**Exhibit:** MDSlides project (8 PDRs documented for v0.1.0 and v0.2.0)

---

## Overview

Product Decision Records (PDRs) capture important **product and business decisions** along with their context, rationale, and impact. While Architecture Decision Records (ADRs) focus on technical implementation, PDRs document **what the product should do** and **why those product choices were made**.

PDRs bridge the gap between product management and engineering, ensuring product decisions are transparent, traceable, and aligned with business goals.

---

## Why Use PDRs?

### Problem: Lost Product Context

Without PDRs, teams face:
- **"Why did we choose these limits?"** questions months later
- **Repeated debates** on settled product decisions
- **Inconsistent application** of product rules across features
- **Misalignment** between product vision and implementation
- **Difficulty onboarding** new PMs or engineers to product rationale

### Solution: Document Product Decisions

PDRs provide:
- **Product context** for engineering teams
- **Business rationale** captured at decision time
- **User impact** explicitly documented
- **Constraints** and trade-offs made visible
- **Searchable history** of product evolution

---

## PDR vs ADR vs POL

### Product Decision Records (PDRs)
**Focus:** Product requirements, business rules, user experience
**Examples:**
- PDR-001: Density Validation Limits (12 lines, 150 words)
- PDR-005: Accessibility Requirements (WCAG AA)
- PDR-008: Image Policy (background vs content images)

**Owned by:** Product Manager + Engineering + Domain Experts

### Architecture Decision Records (ADRs)
**Focus:** Technical implementation, architectural choices
**Examples:**
- ADR-010: Markdown Library Selection (Flexmark vs CommonMark)
- ADR-011: Syntax Highlighting Approach (JVM vs JavaScript)

**Owned by:** Engineering + Architects

### Policy Documents (POLs)
**Focus:** Development standards, team practices
**Examples:**
- POL-001: Ubiquitous Language Enforcement
- POL-003: Pure Functional Domain

**Owned by:** Engineering Team

### Decision Matrix

| Type | Question | Example | Owner |
|------|----------|---------|-------|
| PDR | What should the product do? | "Max 12 lines per slide" | PM + Eng + Domain |
| ADR | How do we build it technically? | "Use Flexmark for parsing" | Engineering |
| POL | How do we work as a team? | "Domain must be pure functional" | Engineering Team |

**Guideline:** If it affects **user experience or business rules**, it's a PDR. If it's **implementation choice**, it's an ADR. If it's **team practice**, it's a POL.

---

## When to Write a PDR

Write a PDR when you make a decision about:

1. **Product Constraints** - Limits, boundaries, validation rules
2. **User Experience** - Workflows, interactions, accessibility
3. **Business Rules** - Pricing, permissions, calculations
4. **Feature Scope** - What's included/excluded from MVP
5. **Non-Functional Requirements** - Performance targets, accessibility standards
6. **Product Strategy** - Market positioning, target audience

### Examples from MDSlides Project

- **PDR-001:** Density Validation Limits (cognitive load research)
- **PDR-002:** Validation Error Reporting (UX: show all errors vs fail-fast)
- **PDR-003:** Slide Deck Size Limits (1-200 slides)
- **PDR-004:** Template Selection (title vs content - which templates to support)
- **PDR-005:** Accessibility Requirements (WCAG AA compliance)
- **PDR-006:** Code Block Rendering (20-line guideline, auto-scaling)
- **PDR-007:** Theme JSON Schema (customization vs simplicity trade-off)
- **PDR-008:** Image Policy (background vs content, density rules)

### When NOT to Write a PDR

Don't write PDRs for:
- **Technical implementation details** (use ADR)
- **Team processes** (use POL)
- **Temporary decisions** (unless they become permanent)
- **Obvious choices** (standard UX patterns everyone agrees on)

---

## PDR Template

### Recommended Structure

```markdown
# PDR-XXX: [Decision Title]

**Status:** [Proposed | Accepted | Deprecated | Superseded by PDR-YYY]
**Date:** YYYY-MM-DD
**Deciders:** [Product Manager, Domain Experts, Engineering]
**Related:** [Related PDRs, ADRs, User Stories]

## Context

What is the product problem or opportunity we're addressing?
- User needs
- Business goals
- Market constraints
- Research findings

## Decision

What product direction are we taking?

## Rationale

Why did we choose this approach?
- User impact
- Business value
- Research evidence (cognitive load, accessibility, usability)
- Competitive analysis

## User Impact

How does this affect users?
- Benefits to users
- New constraints or limitations
- Learning curve

## Business Impact

How does this affect the business?
- Revenue impact
- Market positioning
- Development cost
- Maintenance burden

## Alternatives Considered

What other product approaches were evaluated?
- Option 1: [Brief description and why rejected]
- Option 2: [Brief description and why rejected]

## Consequences

### Positive
- Benefits gained
- Problems solved

### Negative
- New limitations introduced
- User friction added

### Mitigation
- How we address negative consequences

## Metrics

How will we measure success?
- Key metrics to track
- Success criteria
- How to validate the decision

## References

- User research reports
- Competitive analysis
- Industry standards
- Related PDRs/ADRs

## Revision History

- YYYY-MM-DD: Initial version
- YYYY-MM-DD: Updated after [user feedback/testing]
```

---

## PDR Lifecycle and Status

### Status Values

1. **Proposed** - Under discussion with stakeholders
2. **Accepted** - Decision approved and being implemented
3. **Deprecated** - No longer recommended but still in effect
4. **Superseded** - Replaced by newer PDR (reference it)

### Evolving Product Decisions

When product direction changes:
- **Don't delete** the old PDR
- Mark it as **Superseded by PDR-XXX**
- Create a **new PDR** explaining the new direction
- Reference the **old PDR** in context section

**Example:**
```markdown
# PDR-006: Code Block Rendering (v0.2.0)

**Status:** Accepted
**Supersedes:** None (new feature in v0.2.0)

## Context

In v0.1.0, we didn't support code blocks. For v0.2.0, technical
presenters need to include code examples...
```

---

## Integration with Ceremony-Based Development

### Phase 1: Event Storming & Domain Ceremonies

PDRs often emerge from:
- **Domain expert input** - Business constraints
- **User story definition** - Acceptance criteria
- **Event storming** - Business rules discovered

**Capture PDR candidates** during ceremonies.

### Phase 2: Governance Documentation

Write PDRs during backlog refinement:
- **Product context is fresh** from ceremonies
- **Stakeholder alignment** achieved
- **User impact** clearly understood

### Phase 3: TDD Implementation

PDRs guide development:
- **Acceptance criteria** reference PDRs
- **Validation logic** implements PDR rules
- **Tests verify** PDR constraints hold

---

## Best Practices

### 1. Include Research Evidence

**Don't:**
```markdown
## Rationale
12 lines feels about right.
```

**Do:**
```markdown
## Rationale
Cognitive load research (Miller's Law) shows working memory holds 7±2 items.
For slide presentations, research suggests 10-15 lines is the practical limit
before audience comprehension drops. We chose 12 as a balance between
completeness and cognitive load.

References:
- Miller, G.A. (1956). "The Magical Number Seven, Plus or Minus Two"
- Mayer, R.E. (2009). "Multimedia Learning" (cognitive load in presentations)
```

### 2. Quantify User Impact

**Don't:**
```markdown
## User Impact
This will make slides better.
```

**Do:**
```markdown
## User Impact

**Positive:**
- 80% of users create slides under 12 lines (no impact)
- Enforcing limit prevents overcrowded slides (improved readability)
- Clear error messages guide users to split content

**Negative:**
- 20% of users may need to split slides they previously fit on one
- Learning curve: understanding why limits exist

**Mitigation:**
- Provide actionable error messages ("Split into 2 slides for better readability")
- Document best practices in training materials
```

### 3. Define Success Metrics

**Don't:**
```markdown
We'll know it works if users like it.
```

**Do:**
```markdown
## Metrics

**Success Criteria:**
- < 5% of slides fail validation due to line limits
- User feedback scores ≥ 4/5 on "helps create readable slides"
- Support tickets about validation < 2 per month

**How to Measure:**
- Telemetry: Track validation error rates
- User surveys: Quarterly feedback on validation UX
- Support ticket analysis: Categorize and count
```

### 4. Collaborate Across Disciplines

**PDRs should involve:**
- **Product Manager** - Business goals, user needs
- **Domain Experts** - Business rules, constraints
- **Engineering** - Feasibility, technical constraints
- **UX/Design** - User experience, accessibility
- **Users** (if possible) - Direct feedback

### 5. Version PDRs Like Code

- Store in **git repository**
- Use **pull requests** for review
- Reference in **commit messages**
- Link to **user stories** in backlog

---

## Example: PDR-001 from MDSlides

**Full PDR:** Example based on MDSlides

### PDR-001: Density Validation Limits

**Status:** Accepted
**Date:** 2024-12-01
**Deciders:** Product Team, Domain Experts, Engineering
**Related:** US-001 (Title Slide), US-002 (Content Slide)

#### Context

Slide presentations often suffer from information overload - too much text on a single slide overwhelms audiences and reduces comprehension. Research in cognitive load theory provides evidence-based limits for text density.

**Problem:** Without constraints, users may create cluttered, unreadable slides.

**Goal:** Enforce readability through validated constraints.

#### Decision

Enforce the following density limits:

**Title Slide:**
- Title: max 2 lines
- Subtitle: max 2 lines
- Author: max 80 characters

**Content Slide:**
- Heading: max 80 characters, single line
- Body: max 12 lines, max 150 words

#### Rationale

**Cognitive Load Research:**
- Miller's Law: 7±2 items in working memory
- Extended to ~12 lines for textual information
- 150 words ≈ 1 minute of reading (avg presentation slide time)

**Industry Best Practices:**
- Guy Kawasaki's 10/20/30 rule
- Presentation Zen principles
- TED talk slide guidelines

**User Needs:**
- Presenters want slides that audiences can actually read
- Audiences struggle with text-heavy slides
- Validation provides guidance, not just restrictions

#### User Impact

**Positive:**
- Enforces best practices automatically
- Clear error messages guide improvement
- More readable, effective presentations

**Negative:**
- Users must split long content across slides
- Learning curve for understanding limits
- May feel restrictive initially

**Mitigation:**
- Provide clear error messages with actionable guidance
- Document rationale in training materials
- Allow warnings (not hard failures) for edge cases

#### Business Impact

**Positive:**
- Differentiation: "The presentation tool that enforces readability"
- Higher quality output = better user satisfaction
- Reduces support ("Why are my slides unreadable?")

**Negative:**
- Some users may prefer unrestricted tools
- Development cost: validation logic + tests

**Trade-off Accepted:** Quality over flexibility

#### Alternatives Considered

**Alternative 1: No Limits**
- Rejected: Doesn't solve information overload problem
- Users would create unreadable slides

**Alternative 2: Soft Warnings Only**
- Rejected: Too easy to ignore
- Research shows people ignore warnings

**Alternative 3: User-Configurable Limits**
- Rejected: Adds complexity
- Defeats purpose of opinionated tool
- May reconsider in future version

#### Metrics

**Success Criteria:**
- < 10% of slides fail validation
- User feedback: "Validation helped improve my slides" ≥ 70%
- Support tickets about limits: < 3 per month

**Measurement:**
- Track validation error rates in telemetry
- Quarterly user surveys
- Support ticket categorization

#### Consequences

**Positive:**
- Creates opinionated, best-practice-focused tool
- Distinguishes MDSlides from generic markdown converters
- Builds quality reputation

**Negative:**
- May alienate users who want full control
- Requires clear documentation of limits

**Accepted Trade-off:** Target quality-focused users over power users

#### References

- Miller, G.A. (1956). "The Magical Number Seven, Plus or Minus Two"
- Mayer, R.E. (2009). "Multimedia Learning"
- Reynolds, G. (2008). "Presentation Zen"
- Kawasaki, G. (2005). "The 10/20/30 Rule of PowerPoint"

---

## Common Pitfalls

### 1. Treating PDRs as Requirements Docs

**Problem:** PDR becomes a spec with all acceptance criteria
**Solution:** PDR captures **decision and rationale**. Link to detailed specs/stories.

### 2. Writing PDRs Without Research

**Problem:** "We think users want X" without evidence
**Solution:** Include user research, competitive analysis, or industry standards

### 3. No Metrics for Validation

**Problem:** Can't tell if decision was correct
**Solution:** Define measurable success criteria upfront

### 4. Ignoring Negative Consequences

**Problem:** Only documenting benefits, ignoring trade-offs
**Solution:** Explicitly list negative impacts and mitigations

### 5. PDRs Written by Engineering Only

**Problem:** Lacks product/business perspective
**Solution:** Collaborative authorship (PM + Eng + Domain Experts)

---

## PDR vs User Story

### User Story
**Format:** As a [user], I want [feature], so that [benefit]
**Purpose:** Describe desired functionality
**Scope:** Single feature or behavior
**Audience:** Development team

**Example:**
```
US-001: As a presenter, I want to create title slides,
so that I can introduce my presentation topics.
```

### Product Decision Record
**Format:** Structured document (see template)
**Purpose:** Explain **why** product choices were made
**Scope:** Overarching product direction or constraint
**Audience:** All stakeholders (PM, Eng, Execs, future team members)

**Example:**
```
PDR-001: Density Validation Limits
Explains WHY we enforce 12-line limit (cognitive load research)
and WHAT trade-offs we accepted (user flexibility vs quality)
```

**Relationship:** PDRs inform acceptance criteria for user stories.

---

## Tools and Automation

### Manual Approach (MDSlides)

```bash
# Directory structure
doc/governance/
├── ADR/
│   ├── ADR-001-technology-stack.md
│   └── ...
├── PDR/
│   ├── PDR-001-density-limits.md
│   ├── PDR-005-accessibility.md
│   └── ...
└── POL/
    └── ...
```

### Linking to Backlog

```markdown
# User Story US-002

As a presenter, I want to create content slides,
so that I can deliver main presentation content.

**Acceptance Criteria:**
- Heading: max 80 chars (per PDR-001)
- Body: max 12 lines, 150 words (per PDR-001)
- Validation errors clearly explain violations
```

### Git Integration

```bash
git commit -m "feat: Add body line validation

Implements PDR-001 (Density Validation Limits).
Enforces max 12 lines per content slide body."
```

---

## Integration with Other Practices

### With DDD (Domain-Driven Design)

PDRs capture **business rules** that become domain logic:
```scala
// PDR-001: Max 12 lines per body
def validateBodyLines(body: String): Either[ValidationError, String] =
  val lines = body.split("\n").length
  if lines > 12 then
    Left(ContentError(s"Body exceeds max 12 lines (has $lines)"))
  else
    Right(body)
```

### With BDD (Behavior-Driven Development)

PDRs inform **acceptance criteria**:
```gherkin
# Derived from PDR-001
Scenario: Content slide with too many lines fails validation
  Given a content slide with 15 lines in the body
  When I validate the slide
  Then validation should fail
  And the error should mention "exceeds max 12 lines"
```

### With TDD (Test-Driven Development)

PDRs guide **test cases**:
```scala
test("content slide with 12 lines passes validation") {
  val body = List.fill(12)("line").mkString("\n")
  assert(validateBodyLines(body).isRight)
}

test("content slide with 13 lines fails validation") {
  val body = List.fill(13)("line").mkString("\n")
  assert(validateBodyLines(body).isLeft)
}
```

---

## Measuring PDR Effectiveness

### Qualitative Metrics

- Are product decisions **clearly understood** by engineering?
- Do PDRs **prevent re-debates** of settled questions?
- Are PDRs **referenced in code/tests**?

### Quantitative Metrics

- **PDR count** per major feature
- **References to PDRs** in code comments, tests, ADRs
- **Time to onboard** new PMs (do PDRs reduce this?)

### Red Flags

- **No PDRs** for major features (product decisions not captured)
- **PDRs never referenced** (not being used)
- **PDRs all "Proposed"** (decisions not being finalized)

---

## Case Study: MDSlides Project

**Project:** MDSlides (Markdown to HTML slide deck generator)
**Product Vision:** Opinionated presentation tool enforcing best practices
**PDRs Written:** 8 (as of v0.2.0 planning)

### PDR Highlights

**v0.1.0 MVP:**
- PDR-001: Density Validation Limits (cognitive load research)
- PDR-002: Validation Error Reporting (show all errors vs fail-fast)
- PDR-003: Slide Deck Size Limits (1-200 slides)
- PDR-004: Template Selection (title vs content templates)
- PDR-005: Accessibility Requirements (WCAG AA compliance)

**v0.2.0 (Markdown Rendering):**
- PDR-006: Code Block Rendering (20-line guideline, font auto-scaling)
- PDR-007: Theme JSON Schema (customization vs simplicity)
- PDR-008: Image Policy (background vs content, density rules)

### Impact

**Positive:**
- Engineering understands **why** 12-line limit exists (not arbitrary)
- New team members read PDRs to understand **product philosophy**
- PDRs referenced in tests to **validate business rules**
- **No re-debates** on density limits after PDR-001 written

**Lessons Learned:**
- Writing PDRs during **Phase 2 (Governance)** preserved user research context
- Linking PDRs to **User Stories** clarified acceptance criteria
- **Metrics sections** helped validate decisions post-launch

---

## Conclusion

Product Decision Records are a lightweight practice for **preserving product knowledge** and **aligning stakeholders** around product direction. By documenting **what** the product should do and **why**, PDRs complement ADRs (technical decisions) and POLs (team practices).

**Key Takeaways:**
1. Write PDRs for **product constraints, UX, business rules**
2. Include **research evidence** and **user impact**
3. Define **success metrics** upfront
4. Collaborate across **PM + Eng + Domain Experts**
5. Link PDRs to **user stories** and **domain logic**
6. Distinguish PDRs (product) from ADRs (technical) from POLs (process)

**Start Small:** Pick one product decision and write your first PDR. Use the template provided, include research, and iterate.

---

## References

- [Architecture-Decision-Records.md](Architecture-Decision-Records.md) - Technical decision counterpart
- [Policy-Documents.md](Policy-Documents.md) - Team practice counterpart
- [DDD-Principles.md](DDD-Principles.md) - Domain-driven business rules
- [BDD-Best-Practices.md](BDD-Best-Practices.md) - Linking PDRs to acceptance criteria

---

**Document Status:** Living Document (v1.0, 2024-12-21)
**Next Review:** After v0.2.0 release
**Maintainer:** Product & Engineering Teams
