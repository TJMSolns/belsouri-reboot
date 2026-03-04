# Architecture Decision Records (ADRs)

**Category:** Documentation & Knowledge Management
**Related:** Governance, Living Documentation, DDD, Technical Decision Making
**Exhibit:** MDSlides project (11 ADRs documented for v0.1.0 and v0.2.0)

---

## Overview

Architecture Decision Records (ADRs) are lightweight documents that capture important architectural decisions along with their context, rationale, and consequences. They serve as a historical record of "why" decisions were made, not just "what" was decided.

ADRs are a critical practice for **preserving architectural knowledge** in agile teams where team members may change, and decisions made months ago need to be understood by new team members or revisited when contexts change.

---

## Why Use ADRs?

### Problem: Lost Architectural Knowledge

Without ADRs, teams face:
- **"Why did we choose X?"** questions months later
- **Repeated debates** on settled decisions
- **Loss of context** when team members leave
- **Inability to evaluate** whether old decisions still apply
- **Duplicated research** when making similar decisions

### Solution: Document Decisions as They're Made

ADRs provide:
- **Historical context** for future maintainers
- **Rationale capture** at decision time (when context is fresh)
- **Consequences tracking** (what we gained/lost)
- **Decision reversal capability** (superseding old ADRs)
- **Searchable knowledge base** of architectural choices

---

## When to Write an ADR

Write an ADR when you make a decision that:

1. **Affects system structure** (modules, layers, components)
2. **Introduces dependencies** (libraries, frameworks, external services)
3. **Impacts non-functional requirements** (performance, security, scalability)
4. **Has significant cost** (time, money, technical debt)
5. **Is hard to reverse** (database choice, programming language, cloud provider)
6. **Requires team alignment** (coding standards, testing strategies)

### Examples from MDSlides Project

- **ADR-001:** Technology Stack Selection (Scala 3, Mill, Cats)
- **ADR-002:** Validation Pipeline Architecture (two-phase: structure → content)
- **ADR-006:** Rendering Architecture (standalone HTML vs server-side)
- **ADR-007:** Anticorruption Layer (isolating external library dependencies)
- **ADR-010:** Markdown Library Selection (Flexmark vs CommonMark vs Laika)
- **ADR-011:** Syntax Highlighting Approach (JVM-based vs JavaScript-based)

### When NOT to Write an ADR

Don't write ADRs for:
- **Trivial decisions** (variable naming, code formatting)
- **Obvious choices** (using standard library functions)
- **Temporary workarounds** (unless they become permanent)
- **Implementation details** (unless architecturally significant)

---

## ADR Template

### Recommended Structure

```markdown
# ADR-XXX: [Decision Title]

**Status:** [Proposed | Accepted | Deprecated | Superseded by ADR-YYY]
**Date:** YYYY-MM-DD
**Deciders:** [Who made the decision]
**Related:** [Related ADRs, User Stories, or documents]

## Context

What is the issue we're seeing that is motivating this decision or change?
- Background information
- Constraints
- Forces at play

## Decision

What is the change we're actually proposing or doing?

## Rationale

Why did we choose this option?
- Key factors considered
- Trade-offs evaluated
- Why alternatives were rejected

## Consequences

What becomes easier or more difficult to do because of this change?

### Positive
- Benefits gained
- Problems solved

### Negative
- New challenges introduced
- Limitations accepted

### Mitigation
- How we address negative consequences

## Alternatives Considered

What other options were evaluated?
- Option 1: [Brief description and why rejected]
- Option 2: [Brief description and why rejected]

## References

- Links to related documentation
- External resources consulted
- Related ADRs

## Revision History

- YYYY-MM-DD: Initial version
- YYYY-MM-DD: Updated after [event]
```

---

## ADR Lifecycle and Status

### Status Values

1. **Proposed** - Decision under discussion, not yet final
2. **Accepted** - Decision made and approved
3. **Deprecated** - Decision no longer recommended but still in effect
4. **Superseded** - Replaced by a newer ADR (reference the new one)

### Evolving Decisions

When a decision changes:
- **Don't delete** the old ADR
- Mark it as **Superseded by ADR-XXX**
- Create a **new ADR** explaining the new decision
- Reference the **old ADR** in the new one's context

**Example:**
```markdown
# ADR-010: Markdown Library Selection (v0.2.0)

**Status:** Accepted
**Supersedes:** ADR-004 (if there was an earlier decision)

## Context

In v0.1.0, we preserved markdown as plain text (ADR-004).
For v0.2.0, we need full markdown rendering...
```

---

## Integration with Ceremony-Based Development

### Phase 1: Event Storming & Domain Ceremonies

ADRs are often **identified during ceremonies**:
- Domain experts express **constraints** (performance, security)
- Technical challenges surface during **domain modeling**
- Integration questions arise when defining **bounded contexts**

**Capture these as ADR candidates** during ceremonies.

### Phase 2: Governance Documentation

After ceremonies, write ADRs before TDD implementation:
- **Context is fresh** from recent discussions
- **Alternatives have been explored** during ceremonies
- **Team alignment** has been achieved

### Phase 3: TDD Implementation

ADRs **guide implementation**:
- Developers reference ADRs for **rationale**
- Tests validate **consequences** described in ADRs
- Implementation reveals **unanticipated consequences** (update ADR)

---

## ADR vs PDR vs POL

MDSlides uses three types of decision records:

### Architecture Decision Records (ADRs)
**Focus:** Technical/architectural choices
**Examples:**
- ADR-001: Technology Stack Selection
- ADR-007: Anticorruption Layer
- ADR-011: Syntax Highlighting Approach

### Product Decision Records (PDRs)
**Focus:** Product requirements and business rules
**Examples:**
- PDR-001: Density Validation Limits (12 lines, 150 words)
- PDR-005: Accessibility Requirements (WCAG AA)
- PDR-008: Image Policy (background vs content)

### Policy Documents (POLs)
**Focus:** Development standards and team practices
**Examples:**
- POL-001: Ubiquitous Language Enforcement
- POL-003: Pure Functional Domain
- POL-004: Property-Based Testing Requirements

**Guideline:** If unsure, default to **ADR** for technical decisions, **PDR** for product decisions.

---

## Best Practices

### 1. Write ADRs Early

**Don't wait** until the decision is "final":
- Write **Proposed** ADRs to facilitate discussion
- Update to **Accepted** once decided
- Capture **context before it's lost**

### 2. Be Concise

ADRs are **not specifications**:
- Focus on **why**, not **how**
- Link to detailed specs/code rather than duplicating
- Aim for **1-2 pages max**

### 3. Capture Alternatives

Always document **what you didn't choose**:
- Prevents re-debating settled questions
- Shows **due diligence** to future readers
- Helps when contexts change (maybe alternative becomes viable)

### 4. Update Consequences

ADRs are **living documents**:
- Revisit after implementation (Did consequences match predictions?)
- Update if new consequences emerge
- Add revision history entries

### 5. Make ADRs Discoverable

- **Number sequentially** (ADR-001, ADR-002...)
- **Use descriptive titles** (not "Database Decision" but "PostgreSQL for Event Sourcing")
- **Link between related ADRs**
- **Index in README** or governance directory

### 6. Version Control ADRs

- Store in **git repository** with code
- Use **pull requests** for review
- Reference **commits/PRs** in ADR metadata

---

## Example: ADR-010 from MDSlides

**Full ADR:** [ADR-010: Markdown Library Selection](../../governance/ADR/ADR-010-markdown-library-selection.md)

**Highlights:**

**Context:**
- Need GitHub Flavored Markdown support
- Must support Mermaid diagrams
- Must work offline (no CDN)

**Decision:**
- Use Flexmark (`com.vladsch.flexmark:flexmark-all:0.64.8`)

**Rationale:**
- Built-in GFM extensions (tables, task lists, strikethrough)
- Mermaid passthrough support
- Rich extension ecosystem
- JAR size increase acceptable (5MB)

**Alternatives:**
- CommonMark-java: Rejected (no GFM, no Mermaid)
- Laika: Rejected (limited GFM, no Mermaid)

**Consequences:**
- **Positive:** Full GFM compatibility, Mermaid support, rich features
- **Negative:** 5MB JAR increase, dependency on external library
- **Mitigation:** Anticorruption layer isolates domain from Flexmark API

---

## Common Pitfalls

### 1. Writing ADRs Retroactively

**Problem:** Documenting decisions months/years later
**Solution:** Write ADRs **as decisions are made**, not after

### 2. Too Much Detail

**Problem:** ADRs become multi-page specifications
**Solution:** Focus on **decision context and rationale**, link to detailed docs

### 3. No Alternatives Documented

**Problem:** Readers don't know what was considered
**Solution:** Always include **Alternatives Considered** section

### 4. Status Never Updated

**Problem:** ADRs marked "Proposed" forever
**Solution:** Review and update status as part of **Definition of Done**

### 5. ADRs Hidden or Unfindable

**Problem:** ADRs buried in wiki/email/chat
**Solution:** Store in **version-controlled repository** with clear naming

---

## Tools and Automation

### ADR Tools

- **adr-tools** (bash): Command-line ADR management
- **log4brains** (TypeScript): Web UI for ADRs with search
- **ADR Manager** (IntelliJ plugin): IDE integration

### Manual Approach (MDSlides)

```bash
# Directory structure
doc/governance/
├── ADR/
│   ├── ADR-001-technology-stack.md
│   ├── ADR-002-validation-pipeline.md
│   ├── ADR-010-markdown-library-selection.md
│   └── ...
├── PDR/
│   └── ...
└── POL/
    └── ...
```

### Git Integration

```bash
# Link ADRs to commits
git commit -m "feat: Add Flexmark markdown parser

Implements ADR-010 (Markdown Library Selection).
Uses Flexmark for GFM + Mermaid support."
```

---

## Integration with Other Practices

### With DDD (Domain-Driven Design)

ADRs capture **bounded context integration decisions**:
- How contexts communicate (REST, events, shared kernel)
- Anticorruption layer designs
- Ubiquitous language enforcement strategies

### With BDD (Behavior-Driven Development)

ADRs reference **user stories** and **acceptance criteria**:
```markdown
**Related:** US-003 (Full Markdown Rendering), US-004 (Code Blocks)
```

### With TDD (Test-Driven Development)

ADRs guide **test strategy**:
- Testing approach for external library integrations
- Property-based vs example-based test decisions

### With Agile/Scrum

ADRs written during:
- **Sprint Planning** (when technical approach is decided)
- **Backlog Refinement** (when exploring implementation options)
- **Sprint Review** (when consequences become clear)

---

## Measuring ADR Effectiveness

### Qualitative Metrics

- Are new team members able to **understand architectural choices**?
- Do ADRs **prevent re-debating** settled decisions?
- Are ADRs **referenced in PRs/code reviews**?

### Quantitative Metrics

- **ADR count** per major feature/epic
- **Time to onboard** new developers (do ADRs reduce this?)
- **Decision reversal rate** (how often are ADRs superseded?)

### Red Flags

- **No ADRs written** in 6+ months (decisions being made but not captured)
- **ADRs never referenced** (not being used as knowledge base)
- **ADRs all "Proposed"** (decisions not being finalized)

---

## Case Study: MDSlides Project

**Project:** MDSlides (Markdown to HTML slide deck generator)
**Architecture:** Domain-Driven Design with 3-layer clean architecture
**ADRs Written:** 11 (as of v0.2.0 planning)

### ADR Highlights

**v0.1.0 MVP:**
- ADR-001: Technology Stack (Scala 3, Mill, Cats)
- ADR-002: Validation Pipeline (two-phase validation)
- ADR-006: Rendering Architecture (standalone HTML)
- ADR-007: Anticorruption Layer (isolate external dependencies)
- ADR-008: Slot-Based Content Model (flexible template system)
- ADR-009: Property-Based Testing Strategy (ScalaCheck)

**v0.2.0 (Markdown Rendering):**
- ADR-010: Markdown Library Selection (Flexmark for GFM + Mermaid)
- ADR-011: Syntax Highlighting Approach (JavaScript-based, offline)

### Impact

**Positive:**
- New developers understand **why Flexmark** (not CommonMark or Laika)
- Team avoided **re-debating** syntax highlighting approach
- Anticorruption layer design **prevented domain contamination**

**Lessons Learned:**
- Writing ADRs during **Phase 2 (Governance)** preserved context from ceremonies
- Linking ADRs to **User Stories** clarified business motivation
- **Consequences sections** accurately predicted JAR size impact

---

## Conclusion

Architecture Decision Records are a **lightweight, high-value** practice for preserving architectural knowledge in agile teams. By capturing **context, rationale, and consequences** at decision time, ADRs prevent lost knowledge, reduce re-debates, and facilitate onboarding.

**Key Takeaways:**
1. Write ADRs **as decisions are made**, not retroactively
2. Focus on **why** (rationale) not **how** (implementation)
3. Always document **alternatives considered**
4. Update **consequences** after implementation
5. Store in **version control** with clear naming
6. Distinguish **ADRs (technical) from PDRs (product) from POLs (process)**

**Start Small:** Pick one upcoming architectural decision and write your first ADR. Use the template provided, keep it concise, and iterate on the practice.

---

## References

- [Michael Nygard's ADR](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions) - Original ADR proposal
- [ADR GitHub Organization](https://adr.github.io/) - Tools and resources
- [Thoughtworks Technology Radar](https://www.thoughtworks.com/radar/techniques/lightweight-architecture-decision-records) - Industry adoption
- [MDSlides Governance Directory](../../governance/) - Real-world ADR examples

---

**Document Status:** Living Document (v1.0, 2024-12-21)
**Next Review:** After v0.2.0 release
**Maintainer:** Development Team
