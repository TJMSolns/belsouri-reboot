---
name: check-ceremonies
description: Verifies that all required ceremony artifacts exist before implementation can proceed. Use after /classify-work determines what's needed.
allowed-tools: Read, Glob, Grep
user-invocable: true
argument-hint: [bounded-context-name or feature-name]
---

# Ceremony Checker

You verify that required ceremony artifacts exist and are complete before implementation can proceed.

## Artifact Locations (from HOW-WE-WORK.md lines 341-354)

| Phase | Artifact Type | Location |
|-------|---------------|----------|
| 1.1 | Event storming | `doc/internal/domain/event-storming/[context]-events.md` |
| 1.2 | Ubiquitous language | `doc/internal/domain/ubiquitous-language.md` |
| 1.3 | Aggregate docs | `doc/internal/domain-models/aggregates/[name]-aggregate.md` |
| 1.4 | Context maps | `doc/internal/domain/context-maps/context-map.md` |
| 2.2 | Example maps | `doc/internal/scenarios/example-maps/[feature]-examples.md` |
| 2.4 | BDD scenarios | `features/[feature].feature` |

## Phase 1 Checklist (New Bounded Context)

For each bounded context, verify:

### 1.1 Event Storming
- [ ] File exists at `doc/internal/domain/event-storming/[context]-events.md`
- [ ] Contains 20-50 domain events
- [ ] Events are grouped into flows
- [ ] Commands are identified
- [ ] Aggregate candidates are listed (3-7)
- [ ] Hotspots and questions are documented

### 1.2 Ubiquitous Language
- [ ] Terms from this context exist in `doc/internal/domain/ubiquitous-language.md`
- [ ] Each term has a business definition
- [ ] No conflicting definitions

### 1.3 Domain Modeling
For each aggregate in the context:
- [ ] File exists at `doc/internal/domain-models/aggregates/[name]-aggregate.md`
- [ ] Contains: Purpose, Fields, Events, Commands, Invariants
- [ ] Has state machine if entity has lifecycle

### 1.4 Context Mapping
- [ ] Context appears in `doc/internal/domain/context-maps/context-map.md`
- [ ] Relationships with other contexts are documented
- [ ] Integration patterns are specified

## Phase 2 Checklist (New Feature)

For each feature, verify:

### 2.2 Example Mapping
- [ ] File exists at `doc/internal/scenarios/example-maps/[feature]-examples.md`
- [ ] Contains user story
- [ ] Contains business rules
- [ ] Each rule has concrete examples
- [ ] Questions are resolved

### 2.4 BDD Scenarios
- [ ] File exists at `features/[feature].feature`
- [ ] Uses domain language from ubiquitous-language.md
- [ ] Covers happy path
- [ ] Covers key edge cases
- [ ] Each Example Map rule has corresponding scenario

## Output Format

```
CEREMONY CHECK: [context/feature name]
PHASE: [1 or 2]

ARTIFACTS FOUND:
- [x] path/to/artifact (complete)
- [~] path/to/artifact (exists but incomplete - missing: X, Y)
- [ ] path/to/artifact (missing)

COMPLETENESS: [X/Y artifacts complete]

BLOCKERS:
- blocker 1
- blocker 2

READY_TO_IMPLEMENT: [YES/NO]

REQUIRED_ACTIONS:
1. Create missing artifact at path
2. Complete incomplete artifact (add X, Y)
```

## Important Rules

1. DO NOT allow implementation to proceed with missing Phase 1 artifacts for new contexts
2. DO NOT allow implementation to proceed with missing Phase 2 artifacts for new features
3. "Incomplete" means the artifact exists but is missing required sections
4. If an artifact references another that doesn't exist, flag both
5. Read each artifact to verify content, not just existence
