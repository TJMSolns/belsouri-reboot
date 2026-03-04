# How We Work: Ceremony-Based SDLC

---

## Philosophy

We build software using a **ceremony-based approach** that blends three disciplines:

- **DDD (Domain-Driven Design)**: Model the domain accurately -- aggregates, events, invariants, bounded contexts
- **BDD (Behavior-Driven Design)**: Specify behavior with concrete examples before coding
- **TDD (Test-Driven Development)**: Implement with test-first discipline -- Red, Green, Refactor

These three form a pipeline: **Discovery (DDD) -> Specification (BDD) -> Implementation (TDD) -> Feedback**.

**Key Principles:**

1. **Documentation is code** -- everything lives in the repository
2. **Ceremonies are mandatory for domain work** -- never skip discovery for domain features
3. **Ship early, learn fast** -- commit working code frequently, no formal sprints
4. **Offline-first thinking** -- design for no connectivity, sync when available
5. **Outputs feed inputs** -- each ceremony's artifacts feed the next ceremony naturally

---

## Team Roles

| Role | Who | Responsibilities |
|------|-----|------------------|
| Product Owner | Tony | Business requirements, acceptance criteria, domain knowledge, pilot feedback |
| Reviewer | Tony | Approve artifacts, validate domain accuracy |
| Developer | Claude | Implementation, tests, code quality |
| Architect | Tony + Claude | Domain modeling, technical decisions, ADRs |

**Working Pattern**: Tony provides direction and domain knowledge. Claude drafts artifacts and code. Tony reviews and approves. Iterate.

**Decision Authority**: Tony has final say on domain and business decisions. Claude has strong opinions on technical implementation but defers to Tony's judgment.

---

## Ceremony Framework

### Phase 1: Discovery

**Goal**: Understand the domain before coding.

| # | Ceremony | Purpose | Artifact |
|---|----------|---------|----------|
| 1.1 | Event Storming | Surface domain events, commands, aggregate candidates | `doc/domain/event-storming/[context]-events.md` |
| 1.2 | Ubiquitous Language | Establish shared vocabulary across team and code | `doc/domain/ubiquitous-language.md` (append) |
| 1.3 | Domain Modeling | Define aggregates with events, commands, invariants | `doc/domain/aggregates/[name]-aggregate.md` |
| 1.4 | Context Mapping | Map relationships between bounded contexts | `doc/domain/context-maps/context-map.md` |
| 1.5 | Governance Verification | Checklist: all Phase 1 artifacts complete, language consistent, patterns correct | -- |

**Natural flow**: 1.1 -> 1.2 -> 1.3 -> 1.4 -> 1.5 (outputs feed inputs, but can revisit earlier ceremonies as understanding deepens).

#### 1.1 Event Storming

1. Identify all domain events (aim for 20-50 per context)
2. Group events into temporal flows
3. Discover commands that cause events
4. Identify aggregate candidates (3-7 per context)
5. Surface hotspots and open questions

#### 1.2 Ubiquitous Language

1. Extract domain terms from event storming
2. Define each term from the business perspective
3. Resolve conflicts (same term, different meanings across contexts)
4. Ban problematic generic terms (DTO, Manager, Handler, etc.)

#### 1.3 Domain Modeling

For each aggregate:
1. Define purpose and business concept
2. Document fields, events, commands, and invariants
3. Create state machine if entity has lifecycle
4. Validate against event storming output

**Aggregate doc template:**

```markdown
# [Name] Aggregate

## Purpose
What business concept does this represent?

## Fields
| Field | Type | Required | Notes |
|-------|------|----------|-------|

## Events
- **[Name]Created**: When...
- **[Name]Updated**: When...

## Commands
- **Create[Name]**: ...
- **Update[Name]**: ...

## Invariants
1. Must have [required field]
2. Cannot [invalid state]

## State Machine (if applicable)
[Mermaid stateDiagram]
```

#### 1.4 Context Mapping

1. Group aggregates into bounded contexts
2. Identify upstream/downstream relationships
3. Choose integration patterns (ACL, Conformist, OHS)
4. Define integration contracts

#### 1.5 Governance Verification

Checklist before proceeding to Phase 2:
- [ ] Event storming artifact exists for this context
- [ ] All domain terms added to ubiquitous language glossary
- [ ] Aggregate docs exist for all identified aggregates
- [ ] Context map updated if multiple contexts exist
- [ ] Language is consistent across all artifacts

### Phase 2: Specification

**Goal**: Define behavior with concrete examples before coding.

| # | Ceremony | Purpose | Artifact |
|---|----------|---------|----------|
| 2.1 | Three Amigos | Collaboratively discover requirements and edge cases from PO, Dev, and Tester perspectives | Notes feed into 2.2 |
| 2.2 | Example Mapping | Extract business rules with concrete examples | `doc/scenarios/example-maps/[feature]-examples.md` |
| 2.3 | Acceptance Criteria Review | Validate scenarios against domain model and language | -- |
| 2.4 | BDD Scenarios | Formalize examples as Gherkin scenarios (output of 2.1-2.3) | `features/[feature].feature` |
| 2.5 | Governance Verification | Checklist: all rules have scenarios, language is correct | -- |

**Natural flow**: 2.1 -> 2.2 -> 2.3 -> 2.4 -> 2.5. These ceremonies are iterative -- expect to loop back as understanding grows.

**For our team**: Tony plays PO + Tester perspectives. Claude plays Developer perspective. Both collaborate on examples.

#### 2.1 Three Amigos

- Examine the feature from all three perspectives
- Surface edge cases and assumptions
- Identify what's unknown

#### 2.2 Example Mapping

1. Write the user story
2. Extract business rules
3. Provide concrete examples for each rule
4. Surface and resolve questions

**Example map template:**

```markdown
# Example Map: [Feature]

## Story
As a [role], I want [goal] so that [benefit].

## Rules

### Rule 1: [Business rule]
**Examples**:
- Given [context], when [action], then [outcome]
- Given [edge case], when [action], then [different outcome]

### Rule 2: [Another rule]
**Examples**:
- ...

## Questions
- [ ] [Unresolved question]
- [x] [Resolved question] -> Answer: ...
```

#### 2.3 Acceptance Criteria Review

- Validate scenarios align with domain model
- Verify ubiquitous language is used correctly
- Confirm all Example Map rules have corresponding scenarios

#### 2.4 BDD Scenarios

Convert Example Map rules to Gherkin scenarios:

```gherkin
Feature: [Feature name]
  As a [role]
  I want [goal]
  So that [benefit]

  Background:
    Given [common setup]

  Scenario: [Happy path]
    Given [context]
    When [action]
    Then [expected outcome]

  Scenario: [Edge case]
    Given [different context]
    When [action]
    Then [different outcome]
```

#### 2.5 Governance Verification

Checklist before proceeding to Phase 3:
- [ ] Every Example Map rule has at least one BDD scenario
- [ ] Scenarios use ubiquitous language correctly
- [ ] Happy path and key edge cases are covered
- [ ] No unresolved questions remain

### Phase 3: Implementation

**Goal**: Build working software with test-first discipline.

| # | Ceremony | Purpose | Artifact |
|---|----------|---------|----------|
| 3.1 | TDD Implementation | Red-Green-Refactor cycle driven by BDD scenarios | Production code + tests |
| 3.2 | Property-Based Testing | Generative testing for invariants (deferred -- optional for now) | -- |

#### 3.1 TDD Implementation

1. **Red**: Write a failing test for one behavior (derived from BDD scenario)
2. **Green**: Write minimum code to make it pass
3. **Refactor**: Improve design without breaking tests
4. **Commit**: Small, frequent commits of working code

**Rules:**
- Never write production code without a failing test
- Keep tests focused on behavior, not implementation details
- Use domain language in test names
- Refactor aggressively while tests pass

### Phase 4: Feedback

**Goal**: Keep code and documentation aligned. Learn from what we build.

| # | Ceremony | Purpose | Artifact |
|---|----------|---------|----------|
| 4.1 | Domain Model Retrospective | Review domain model accuracy after implementation | `doc/retrospectives/` |
| 4.2 | Living Documentation Sync | Ongoing discipline: update docs when code changes | Updated artifacts |
| 4.3 | Scenario-to-Test Decomposition | Deferred until needed | -- |
| 4.4 | Cross-Boundary Integration Testing | Deferred until 2+ bounded contexts exist | -- |

#### 4.1 Domain Model Retrospective

After implementing a feature or context:
- Did the domain model match reality?
- What did we learn that should feed back into Phase 1 artifacts?
- Are there new aggregates, events, or invariants to document?

#### 4.2 Living Documentation Sync

Not a scheduled ceremony -- an ongoing discipline:
- New aggregate discovered -> update aggregate doc
- New business rule -> update Example Map
- BDD scenario changes -> update feature file
- Architecture decision -> create/update ADR
- Lessons learned -> update LESSONS-LEARNED.md

---

## When to Use Ceremonies

| Trigger | Required Ceremonies |
|---------|-------------------|
| New bounded context | Full Phase 1 (1.1 -> 1.2 -> 1.3 -> 1.4 -> 1.5), then Phase 2 for first features |
| New feature in existing context | Phase 2 (2.1 -> 2.2 -> 2.3 -> 2.4 -> 2.5) |
| New aggregate in existing context | Partial Phase 1 (1.3 aggregate doc + 1.2 language update) + Phase 2 |
| Infrastructure, build config, UI scaffolding | No ceremony needed |
| Bug fix with clear scope | No ceremony needed (write failing test first, then fix) |

**Ordering**: The default flow is the natural sequence listed above -- each ceremony's outputs feed the next. However, ceremonies can be done out of order when the task demands it. Phase 2 ceremonies are explicitly iterative.

---

## Artifact Locations

| Artifact | Location |
|----------|----------|
| Development plan | `doc/planning/DEVELOPMENT-PLAN.md` |
| Event storming | `doc/domain/event-storming/[context]-events.md` |
| Ubiquitous language | `doc/domain/ubiquitous-language.md` |
| Aggregate docs | `doc/domain/aggregates/[name]-aggregate.md` |
| Context maps | `doc/domain/context-maps/context-map.md` |
| Example maps | `doc/scenarios/example-maps/[feature]-examples.md` |
| BDD scenarios | `features/[feature].feature` |
| ADRs | `doc/governance/ADR/ADR-NNN-title.md` |
| Policies | `doc/governance/POL/POL-NNN-title.md` |
| Product Decision Records | `doc/governance/PDR/PDR-NNN-title.md` |
| Retrospectives | `doc/retrospectives/` |
| Lessons learned | `LESSONS-LEARNED.md` (project root) |
| SBPF reference library | `SBPF/` (project root) |

**Naming convention**: Context goes in filenames, not directory paths (e.g., `event-storming/scheduling-events.md`, not `event-storming/scheduling/events.md`).

---

## Governance

### Architecture Decision Records (ADR)

**When to write**: Technology choices, architecture pattern choices, significant trade-off decisions, reversals of previous decisions.

```markdown
# ADR-NNN: [Title]

**Status**: Proposed | Accepted | Deprecated | Superseded
**Date**: YYYY-MM-DD

## Context
What situation prompted this decision?

## Decision
What did we decide?

## Consequences
What are the trade-offs?

## Alternatives Considered
What else did we evaluate?
```

### Policy Documents (POL)

**When to write**: Recurring rules that govern how we work (coding standards, security requirements, compliance rules).

```markdown
# POL-NNN: [Title]

**Status**: Active | Deprecated
**Date**: YYYY-MM-DD

## Policy
What is the rule?

## Rationale
Why does this rule exist?

## Scope
What does this apply to?
```

### Product Decision Records (PDR)

**When to write**: Product-level decisions about scope, features, prioritization, or business rules.

```markdown
# PDR-NNN: [Title]

**Status**: Proposed | Accepted | Deprecated
**Date**: YYYY-MM-DD

## Context
What product question needed a decision?

## Decision
What did we decide?

## Impact
How does this affect the product?
```

### SBPFs (Shared Best Practice Files)

Tony's personal reference library of patterns and methodologies. Lives at `SBPF/` in the project root. When an SBPF becomes relevant to the project, its content is adapted into an ADR, POL, or PDR -- not referenced directly from project code or docs.

---

## Quality Standards

### Per Phase Definition of Done

**Phase 1 (Discovery)**:
- [ ] Event storming artifact exists with events, commands, and aggregate candidates
- [ ] All domain terms in ubiquitous language glossary
- [ ] Aggregate docs complete for all identified aggregates
- [ ] Context map updated
- [ ] Governance verification checklist passed

**Phase 2 (Specification)**:
- [ ] Example maps exist with rules and concrete examples
- [ ] BDD scenarios cover happy path and key edge cases
- [ ] Scenarios use ubiquitous language correctly
- [ ] No unresolved questions
- [ ] Governance verification checklist passed

**Phase 3 (Implementation)**:
- [ ] All tests pass
- [ ] Code reviewed and approved by Tony
- [ ] Feature verified end-to-end (not just unit tests)
- [ ] No new lint warnings
- [ ] Documentation updated if needed

**Phase 4 (Feedback)**:
- [ ] Domain model reviewed against implementation reality
- [ ] Artifacts updated to reflect what was learned
- [ ] Lessons learned captured

### Code Quality

- All production code has tests
- Tests use domain language
- No TODO comments in production code (track issues separately)

---

## Commit Protocol

1. Stage specific files (avoid `git add -A`)
2. Write a commit message focused on **why**, not what
3. End with co-author attribution

```bash
git commit -m "$(cat <<'EOF'
Add Contact aggregate with CRUD operations

Implements the foundation for all modules that need to work
with patients/contacts. Supports offline-first operation with
event sourcing.

Co-Authored-By: Claude <noreply@anthropic.com>
EOF
)"
```

---

## Lessons Learned

See `LESSONS-LEARNED.md` at project root. This is a living document, continuously updated as we work. Every significant learning -- things that went wrong, things that went right, process improvements -- gets captured there.

---

**Maintained By**: Tony + Claude
