# Policy Documents (POLs)

**Category:** Documentation & Knowledge Management, Process & Standards
**Related:** Architecture-Decision-Records.md, Product-Decision-Records.md, DDD, TDD
**Exhibit:** MDSlides project (5 POLs documented for v0.1.0 and v0.2.0)

---

## Overview

Policy Documents (POLs) capture **development standards, team practices, and engineering conventions** that guide how a team works together. Unlike ADRs (technical decisions) or PDRs (product decisions), POLs focus on **process, quality standards, and engineering discipline**.

POLs are critical for:
- **Team alignment** on coding standards and practices
- **Quality gates** that define "done"
- **Onboarding** new developers with clear expectations
- **Consistency** across codebase and team behaviors
- **Technical excellence** through enforced best practices

---

## Why Use POLs?

### Problem: Inconsistent Engineering Practices

Without POLs, teams face:
- **"How should we...?"** questions answered differently by each developer
- **Inconsistent code quality** (some modules tested, others not)
- **Tribal knowledge** of "the way we do things here"
- **Onboarding friction** (implicit rules not documented)
- **Technical debt** from lack of quality standards
- **Debates on settled practices** (testing strategy, naming conventions, etc.)

### Solution: Document Standards as Policies

POLs provide:
- **Explicit standards** for code quality, testing, naming, architecture
- **Quality gates** that must be met before merging
- **Automated enforcement** (linters, CI checks, pre-commit hooks)
- **Team contract** on how we work
- **Reference documentation** for code reviews and onboarding

---

## When to Write a POL

Write a POL when you establish a team standard for:

1. **Code Quality** (linting rules, complexity limits, formatting)
2. **Testing Strategy** (coverage requirements, test types, property-based testing)
3. **Architectural Constraints** (layering rules, dependency restrictions)
4. **Domain Modeling** (ubiquitous language enforcement, pure functional domain)
5. **Documentation** (when to write ADRs/PDRs, inline comments, API docs)
6. **Version Control** (branching strategy, commit message format, PR process)
7. **Security** (secrets management, dependency scanning, vulnerability response)
8. **Performance** (benchmarking requirements, optimization standards)

### Examples from MDSlides Project

- **POL-001:** Ubiquitous Language Enforcement (domain terms match business language)
- **POL-002:** Immutability Requirements (all data structures immutable)
- **POL-003:** Pure Functional Domain (no I/O, no exceptions, no null)
- **POL-004:** Property-Based Testing Requirements (domain logic must have PBT)
- **POL-005:** Git Commit Standards (conventional commits, sign-off required)

### When NOT to Write a POL

Don't write POLs for:
- **One-time decisions** (use ADR or PDR instead)
- **Project-specific choices** (use ADR for technical, PDR for product)
- **Obvious practices** (e.g., "write code that compiles")
- **Personal preferences** without team consensus

---

## POL Template

### Recommended Structure

```markdown
# POL-XXX: [Policy Title]

**Status:** [Draft | Active | Deprecated | Superseded by POL-YYY]
**Enforcement:** [Manual | Automated | Hybrid]
**Effective Date:** YYYY-MM-DD
**Reviewers:** [Who approved this policy]
**Related:** [Related POLs, ADRs, or PDRs]

## Purpose

Why does this policy exist? What problem does it solve?

## Scope

What does this policy apply to?
- Which modules/layers/components
- Which team members/roles
- Which scenarios

## Policy Statement

The actual rule or standard being established.

### Requirements (MUST)

Hard requirements that cannot be violated:
- Requirement 1
- Requirement 2

### Guidelines (SHOULD)

Best practices that are strongly recommended:
- Guideline 1
- Guideline 2

### Recommendations (MAY)

Optional practices that are beneficial:
- Recommendation 1
- Recommendation 2

## Rationale

Why did we establish this policy?
- Benefits to code quality
- Benefits to team collaboration
- Benefits to maintainability

## Enforcement

How is this policy enforced?

### Automated Checks
- Linters (e.g., Scalafmt, Scalafix)
- CI pipeline checks
- Pre-commit hooks
- Static analysis tools

### Manual Checks
- Code review checklist
- PR template requirements
- Periodic audits

### Violation Handling
- What happens when policy is violated?
- Process for exceptions/waivers

## Examples

### ✅ Good Examples
Code/practice that follows the policy

### ❌ Bad Examples
Code/practice that violates the policy

## Exceptions

Documented scenarios where policy does not apply or may be waived.

## Metrics

How do we measure compliance?
- Percentage of code covered by policy
- Violation rate
- Time to fix violations

## References

- Links to tools, documentation, or standards
- Related POLs, ADRs, or PDRs

## Revision History

- YYYY-MM-DD: Initial version
- YYYY-MM-DD: Updated after [event]
```

---

## POL Lifecycle and Status

### Status Values

1. **Draft** - Policy under discussion, not yet enforced
2. **Active** - Policy approved and enforced
3. **Deprecated** - Policy no longer recommended but still in effect
4. **Superseded** - Replaced by a newer POL (reference the new one)

### Evolving Policies

When a policy changes:
- **Don't delete** the old POL
- Mark it as **Superseded by POL-XXX**
- Create a **new POL** explaining the new standard
- Reference the **old POL** in the new one's context
- **Update enforcement mechanisms** (linters, CI, etc.)

**Example:**
```markdown
# POL-006: Git Commit Standards v2

**Status:** Active
**Supersedes:** POL-005 (Git Commit Standards v1)
**Effective Date:** 2024-12-01

## Purpose

Updates commit standards to require issue tracking integration.

## Changes from POL-005
- Now requires issue reference in commit message
- Adds scope field to conventional commits
- Updates commit message examples
```

---

## Integration with Ceremony-Based Development

### Phase 1: Event Storming & Domain Ceremonies

POLs are often **identified during ceremonies**:
- Domain experts express **ubiquitous language terms** → POL-001 (Language Enforcement)
- Technical constraints surface → POL-003 (Pure Functional Domain)
- Testing strategy discussed → POL-004 (Property-Based Testing)

**Capture these as POL candidates** during ceremonies.

### Phase 2: Governance Documentation

After ceremonies, write POLs before TDD implementation:
- **Team consensus** achieved during ceremonies
- **Enforcement mechanisms** designed
- **Examples prepared** for code reviews

### Phase 3: TDD Implementation

POLs **guide implementation**:
- Developers reference POLs for **coding standards**
- Tests validate **policy compliance**
- CI enforces **automated checks**
- Code reviews check **manual compliance**

---

## POL vs ADR vs PDR

MDSlides uses three types of decision records:

### Policy Documents (POLs)
**Focus:** Development standards and team practices
**Examples:**
- POL-001: Ubiquitous Language Enforcement
- POL-003: Pure Functional Domain
- POL-004: Property-Based Testing Requirements

**Question:** *How do we work as a team?*

### Architecture Decision Records (ADRs)
**Focus:** Technical/architectural choices
**Examples:**
- ADR-001: Technology Stack Selection
- ADR-007: Anticorruption Layer
- ADR-011: Syntax Highlighting Approach

**Question:** *How do we build it technically?*

### Product Decision Records (PDRs)
**Focus:** Product requirements and business rules
**Examples:**
- PDR-001: Density Validation Limits
- PDR-005: Accessibility Requirements
- PDR-008: Image Policy

**Question:** *What should the product do?*

### Decision Matrix

| Scenario | Document Type | Example |
|----------|---------------|---------|
| "All domain models must be immutable" | **POL** | POL-002: Immutability Requirements |
| "We chose Flexmark over CommonMark" | **ADR** | ADR-010: Markdown Library Selection |
| "Slides limited to 12 lines" | **PDR** | PDR-001: Density Validation Limits |
| "Domain must be pure functional" | **POL** | POL-003: Pure Functional Domain |
| "Use Cats for functional abstractions" | **ADR** | ADR-001: Technology Stack Selection |
| "Max 150 words per slide" | **PDR** | PDR-001: Density Validation Limits |
| "All domain logic needs property-based tests" | **POL** | POL-004: Property-Based Testing |

**Guideline:** If unsure, ask:
- Is it about **how we work**? → POL
- Is it about **how we build**? → ADR
- Is it about **what we build**? → PDR

---

## Best Practices

### 1. Get Team Buy-In

POLs are **team contracts**:
- Draft POLs collaboratively (pair/mob programming, team discussions)
- Get explicit approval from all team members
- Document who reviewed and approved
- Revisit periodically (quarterly or after major milestones)

### 2. Automate Enforcement

**Manual enforcement fails** over time:
- Use **linters** (Scalafmt, Scalafix, ESLint, etc.)
- Add **CI pipeline checks** (fail build on violations)
- Configure **pre-commit hooks** (catch violations before push)
- Use **static analysis** (complexity, coverage, dependency checks)

**Example (MDSlides):**
```bash
# Pre-commit hook for POL-001 (Ubiquitous Language)
#!/bin/bash
# Check domain module for forbidden terms
forbidden_terms=("DTO" "DAO" "Manager" "Helper" "Util")
for term in "${forbidden_terms[@]}"; do
  if git diff --cached --name-only | xargs grep -l "$term" | grep "domain/"; then
    echo "ERROR: POL-001 violation - forbidden term '$term' in domain module"
    exit 1
  fi
done
```

### 3. Provide Clear Examples

POLs without examples are hard to follow:
- Show **good examples** (what to do)
- Show **bad examples** (what to avoid)
- Use **real code** from your codebase
- Update examples as codebase evolves

### 4. Distinguish MUST vs SHOULD vs MAY

Use **RFC 2119 keywords**:
- **MUST** / **REQUIRED** → Hard requirement (violations block merge)
- **SHOULD** / **RECOMMENDED** → Best practice (violations trigger warnings)
- **MAY** / **OPTIONAL** → Suggestion (up to developer judgment)

**Example:**
```markdown
## Requirements

### Code Formatting (MUST)
All code **MUST** be formatted with Scalafmt before commit.
**Enforcement:** Pre-commit hook fails on unformatted code.

### Documentation (SHOULD)
Public APIs **SHOULD** have ScalaDoc comments.
**Enforcement:** Code review checklist.

### Inline Comments (MAY)
Complex logic **MAY** include inline comments explaining intent.
**Enforcement:** Developer judgment.
```

### 5. Define Exception Process

Not all policies apply 100% of the time:
- Document **valid exceptions** upfront
- Require **justification** for exceptions (in code comment or PR)
- Track **exception rate** (if >10%, policy may be too strict)

**Example:**
```markdown
## Exceptions

### When POL-003 Does Not Apply
1. **Test modules**: Test utilities may use side effects
2. **Infrastructure layer**: I/O operations are expected
3. **Performance-critical code**: After benchmarking proves necessity

### Exception Process
1. Add comment explaining exception: `// POL-003 exception: [reason]`
2. Document in PR description
3. Require approval from 2+ reviewers
```

### 6. Keep POLs Living Documents

POLs **evolve with the team**:
- Review POLs quarterly or after major releases
- Update based on lessons learned
- Deprecate policies that no longer serve the team
- Add revision history entries

---

## Common Pitfalls

### 1. Writing POLs Without Consensus

**Problem:** Policy imposed top-down without team input
**Solution:** Draft collaboratively, get explicit approval, revisit periodically

### 2. No Enforcement Mechanism

**Problem:** POL exists but nobody follows it
**Solution:** Automate with linters/CI, add to code review checklist

### 3. Too Many Policies

**Problem:** 50+ policies overwhelm developers
**Solution:** Focus on **high-impact policies** (5-10 core policies), combine related policies

### 4. Policies Never Updated

**Problem:** POLs marked "Draft" forever or outdated rules still enforced
**Solution:** Review POLs quarterly, update status, deprecate obsolete policies

### 5. No Examples or Unclear Wording

**Problem:** Developers can't tell if code complies
**Solution:** Include clear examples (good and bad), use specific language

---

## Example: POL-003 from MDSlides

**Full POL:** [POL-003: Pure Functional Domain](../../governance/POL/POL-003-pure-functional-domain.md)

**Highlights:**

### Purpose
Ensure domain module remains pure functional (no side effects, no I/O, no exceptions, no null).

### Requirements (MUST)
- Domain functions **MUST** return `Either[DomainError, A]` instead of throwing exceptions
- Domain models **MUST** be immutable (`val` fields, `case class`)
- Domain logic **MUST NOT** perform I/O (file, network, database)
- Domain logic **MUST NOT** use `null`, `var`, or mutable collections

### Guidelines (SHOULD)
- Domain types **SHOULD** use smart constructors for validation
- Domain errors **SHOULD** be modeled as ADTs (sealed traits)

### Enforcement
**Automated:**
- Scalafix rule: Ban `throw`, `null`, `var` in `domain/` module
- CI check: Domain module has no dependency on `cats-effect` or `os-lib`

**Manual:**
- Code review checklist: Domain functions pure?

### Examples

**✅ Good:**
```scala
// domain/src/.../Slide.scala
case class Slide private (id: SlideId, slots: Map[String, String])

object Slide:
  def validated(id: SlideId, slots: Map[String, String]): Either[ValidationError, Slide] =
    for
      _ <- validateTitle(slots.get("title"))
      _ <- validateBody(slots.get("body"))
    yield Slide(id, slots)
```

**❌ Bad:**
```scala
// domain/src/.../Slide.scala - VIOLATES POL-003
case class Slide(id: SlideId, slots: Map[String, String]):
  def save(): Unit =  // I/O in domain!
    os.write(os.pwd / s"${id.value}.json", toJson(this))

  def validateOrThrow(): Unit =  // Exceptions in domain!
    if (slots.isEmpty) throw new IllegalArgumentException("Empty slots")
```

---

## Tools and Automation

### Linters and Formatters

**Scala:**
- **Scalafmt** - Code formatting (`POL-002: Immutability`, `POL-005: Commit Standards`)
- **Scalafix** - Linting and refactoring rules (`POL-003: Pure Functional Domain`)
- **WartRemover** - Additional safety checks (ban `null`, `var`, `return`)

**Configuration Example:**
```scala
// .scalafix.conf
rules = [
  OrganizeImports,
  NoValInAbstract,
  ProcedureSyntax
]

// Custom rule for POL-003
DisableSyntax.noNulls = true
DisableSyntax.noVars = true
DisableSyntax.noThrows = true
```

### CI Pipeline Integration

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  quality-gates:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v3

      # POL-002: Scalafmt check
      - name: Check formatting
        run: mill _.checkFormat

      # POL-003: Scalafix check
      - name: Check linting
        run: mill _.fix --check

      # POL-004: Property-based test coverage
      - name: Run tests
        run: mill _.test

      # POL-001: Ubiquitous language check
      - name: Check forbidden terms
        run: ./scripts/check-ubiquitous-language.sh
```

### Pre-Commit Hooks

```bash
# .git/hooks/pre-commit
#!/bin/bash

echo "Running POL checks..."

# POL-002: Scalafmt
if ! mill __.checkFormat; then
  echo "ERROR: Code not formatted (POL-002)"
  exit 1
fi

# POL-003: Scalafix
if ! mill __.fix --check; then
  echo "ERROR: Linting violations (POL-003)"
  exit 1
fi

# POL-005: Commit message format
commit_msg=$(cat .git/COMMIT_EDITMSG)
if ! echo "$commit_msg" | grep -qE "^(feat|fix|docs|test|refactor|chore):"; then
  echo "ERROR: Invalid commit message format (POL-005)"
  exit 1
fi

echo "All POL checks passed!"
```

---

## Integration with Other Practices

### With DDD (Domain-Driven Design)

POLs enforce **domain modeling standards**:
- **POL-001:** Ubiquitous Language Enforcement (domain terms = business terms)
- **POL-003:** Pure Functional Domain (domain layer purity)
- **POL-007:** Bounded Context Isolation (dependencies only inward)

### With BDD (Behavior-Driven Development)

POLs guide **acceptance criteria**:
```gherkin
# Acceptance criterion referencing POL-004
Scenario: Validation logic has property-based tests
  Given a domain validation function
  When the function is tested
  Then property-based tests MUST cover all edge cases (POL-004)
```

### With TDD (Test-Driven Development)

POLs define **test strategy**:
- **POL-004:** Property-Based Testing Requirements (when to use PBT vs example-based)
- **POL-006:** Test Coverage Minimums (80% line coverage, 100% domain coverage)
- **POL-008:** Test Organization (unit, integration, property, acceptance)

### With Agile/Scrum

POLs written during:
- **Sprint 0** (initial team setup, establish core policies)
- **Retrospectives** (team identifies pain points → new policies)
- **Definition of Done** (POLs become quality gates)

---

## Measuring POL Effectiveness

### Qualitative Metrics

- Are new team members able to **understand quality standards**?
- Do POLs **reduce code review debates**?
- Are POLs **referenced in PRs/code reviews**?
- Do developers **feel ownership** of POLs?

### Quantitative Metrics

- **Violation rate** (CI failures due to POL checks)
- **Time to fix violations** (average time to resolve POL issues)
- **Exception rate** (% of code requiring POL exceptions)
- **Policy coverage** (% of codebase compliant with POLs)

### Red Flags

- **High violation rate** (>20%) → Policy too strict or poorly communicated
- **No violations ever** → Policy may not be meaningful
- **Frequent exceptions** (>10%) → Policy may need revision
- **POLs never referenced** → Not being used as standards

---

## Case Study: MDSlides Project

**Project:** MDSlides (Markdown to HTML slide deck generator)
**Architecture:** Domain-Driven Design with 3-layer clean architecture
**POLs Written:** 5 (as of v0.2.0 planning)

### POL Highlights

**Core Policies:**
- POL-001: Ubiquitous Language Enforcement
- POL-002: Immutability Requirements
- POL-003: Pure Functional Domain
- POL-004: Property-Based Testing Requirements
- POL-005: Git Commit Standards

### Impact

**Positive:**
- **Zero `null` references** in domain module (enforced by POL-003)
- **29 property-based tests** written (guided by POL-004)
- **Consistent commit messages** (enforced by POL-005)
- **Domain terminology consistency** (enforced by POL-001)

**Lessons Learned:**
- **Automated enforcement critical**: Pre-commit hooks catch 90% of violations
- **Examples essential**: Developers reference POL examples daily
- **Living documents**: POL-003 updated twice based on lessons learned

### Enforcement Statistics (v0.1.0)

```
POL-001 (Ubiquitous Language):
  - Violations: 3 (caught in code review)
  - Exceptions: 0

POL-002 (Immutability):
  - Violations: 0 (Scalafix catches at commit)
  - Exceptions: 0

POL-003 (Pure Functional Domain):
  - Violations: 2 (caught in code review)
  - Exceptions: 1 (test utilities)

POL-004 (Property-Based Testing):
  - Compliance: 100% (all domain logic has PBT)
  - Exceptions: 0

POL-005 (Git Commits):
  - Violations: 5 (pre-commit hook prevents push)
  - Exceptions: 0
```

---

## Conclusion

Policy Documents are a **critical practice** for establishing team standards, quality gates, and engineering discipline. By documenting **development practices, coding standards, and quality requirements**, POLs ensure consistency, reduce debates, and facilitate onboarding.

**Key Takeaways:**
1. Write POLs for **team standards and practices**, not one-time decisions
2. Focus on **enforcement mechanisms** (automate with linters/CI)
3. Get **team consensus** before making POL active
4. Provide **clear examples** (good and bad)
5. Distinguish **MUST (required) vs SHOULD (recommended) vs MAY (optional)**
6. Define **exception process** for edge cases
7. Review and update **quarterly** or after major releases
8. Distinguish **POLs (process) from ADRs (technical) from PDRs (product)**

**Start Small:** Pick one team practice that causes frequent debates (testing strategy, code formatting, naming conventions) and write your first POL. Use the template provided, automate enforcement, and iterate on the practice.

---

## References

- [RFC 2119: Key words for RFCs](https://www.rfc-editor.org/rfc/rfc2119) - MUST, SHOULD, MAY definitions
- [Google Engineering Practices](https://google.github.io/eng-practices/) - Code review and standards
- [Thoughtworks Technology Radar](https://www.thoughtworks.com/radar/techniques) - Industry practices
- [MDSlides Governance Directory](../../governance/) - Real-world POL examples

---

**Document Status:** Living Document (v1.0, 2024-12-21)
**Next Review:** After v0.2.0 release
**Maintainer:** Development Team
