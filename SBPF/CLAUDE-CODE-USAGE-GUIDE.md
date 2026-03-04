# Claude Code Usage Guide

**Purpose**: Best practices for working with Claude on the Belsouri dental practice platform.

**Audience**: Tony (and any future contributors)

---

## Quick Start

### Project Context

Always work within this repository as a Claude Code project. This enables:
- File caching (reduced token usage)
- Persistent project context
- Access to `.claude/settings.json` instructions

### Key Documents to Reference

Before starting significant work, ask Claude to read:
1. `doc/internal/planning/DEVELOPMENT-PLAN.md` - Current phase and priorities
2. `HOW-WE-WORK.md` - Ceremony requirements
3. Relevant aggregate docs in `doc/internal/domain-models/aggregates/`

---

## Effective Prompting Patterns

### Starting a New Phase

```
Let's start Phase 0.1 (App Shell). Read:
- doc/internal/planning/DEVELOPMENT-PLAN.md
- doc/internal/architecture/technical-architecture.md

What's the first deliverable we should build?
```

### Implementing a Feature

```
I want to implement [feature]. This needs [no ceremony | lightweight ceremony | full ceremony].

[If needed] First, let's create the aggregate doc for [entity].
[Then] Let's implement with TDD.
```

### Domain Modeling

```
Let's design the Contact aggregate. We need to define:
- What fields it has
- What events it produces
- What invariants it must maintain
- How it interacts with modules

Create the aggregate doc in doc/internal/domain-models/aggregates/contact-aggregate.md
```

### Debugging

```
I'm seeing [problem]. Here's what I observe:
- [symptom 1]
- [symptom 2]

Read [relevant files] and help me diagnose.
```

### Code Review

```
Review [file or feature] for:
- Domain accuracy
- Event sourcing correctness
- Test coverage
- Memory efficiency
```

---

## Context Management

### When to Use /clear

- After completing a development plan phase
- After finishing a feature (before starting unrelated work)
- When context feels cluttered
- After long sessions (>50k tokens)

### When to Start Fresh Conversation

- Switching between unrelated tasks
- After major milestones
- When debugging persistent issues (fresh perspective helps)

### Checkpointing Progress

Before using /clear:
```
Summarize what we accomplished this session so I can reference it later.
```

Copy the summary to your notes, then /clear.

---

## Working Patterns

### Pair Programming Mode

Claude writes code, you review:
1. Describe the feature or fix needed
2. Claude implements with tests
3. You review for domain accuracy
4. Claude addresses feedback
5. You approve and commit

### Architecture Discussion Mode

Thinking through design:
1. Describe the problem or decision
2. Claude proposes options with trade-offs
3. You ask clarifying questions
4. Converge on approach
5. Document in ADR if significant

### Documentation Mode

Creating artifacts:
1. Specify which artifact (aggregate doc, Example Map, etc.)
2. Claude drafts using templates from HOW-WE-WORK.md
3. You review for domain accuracy
4. Claude refines
5. Commit the artifact

---

## Technology-Specific Tips

### Rust (src-tauri/)

```
When implementing [Rust feature], remember:
- Use rusqlite for SQLite access
- Events are append-only (never delete)
- Projections must be deterministic
- Test with cargo test
```

### Svelte (src/)

```
When implementing [Svelte component]:
- Use stores from src/lib/stores/ for shared state
- Follow CSS variables from app.css
- Components should be as stateless as possible
- Test with pnpm check
```

### Event Sourcing

```
When adding a new event type:
1. Define event struct in src-tauri/src/events/
2. Add serialization/deserialization
3. Update relevant projection
4. Write test that event roundtrips correctly
```

---

## Common Tasks

### Creating an Aggregate Doc

```
Create an aggregate doc for [Entity] following the template in HOW-WE-WORK.md.
Include: Purpose, Fields, Events, Commands, Invariants.
```

### Writing BDD Scenarios

```
Write BDD scenarios for [feature] based on:
- The Example Map in doc/internal/scenarios/example-maps/[feature]-examples.md
- Domain language from [aggregate doc]

Put scenarios in features/[feature].feature
```

### Implementing with TDD

```
Implement [feature] using TDD:
1. Write a failing test first
2. Show me the test before implementing
3. Implement minimum code to pass
4. Refactor if needed

Commit after each green test.
```

---

## Anti-Patterns to Avoid

### Don't: Skip the development plan
```
# Bad
"Let's implement insurance claims tracking"  # Not in current phase

# Good
"Let's implement [thing from current phase per DEVELOPMENT-PLAN.md]"
```

### Don't: Write code without tests for domain logic
```
# Bad
"Just implement the Contact aggregate, we'll add tests later"

# Good
"Implement Contact aggregate with TDD - test first"
```

### Don't: Design online-only features
```
# Bad
"Let's add real-time sync that requires constant connection"

# Good
"Let's design this to work offline, sync when available"
```

### Don't: Over-ceremony simple work
```
# Bad
"Before we update the CSS variables, let's do an Example Map"

# Good
"This is just infrastructure, no ceremony needed"
```

---

## Debugging with Claude

### Effective Bug Reports

```
Bug: [what's broken]
Expected: [what should happen]
Actual: [what happens instead]

Relevant files:
- [file1]
- [file2]

Steps to reproduce:
1. ...
2. ...
```

### Getting Unstuck

```
I've tried [approaches] but [problem persists].
What am I missing? Read [relevant files] and suggest alternatives.
```

---

## Commit Workflow

When ready to commit:
```
Let's commit our changes. Run git status and git diff,
then create a commit with a message focusing on why we made these changes.
```

Claude will:
1. Show you the changes
2. Draft a commit message
3. Stage specific files
4. Include co-author attribution

---

## Project-Specific Instructions

The `.claude/settings.json` file contains project-specific instructions that Claude follows automatically:

- **Development Plan Authority**: Always check current phase before building features
- **Ceremony Scaling**: Match ceremony effort to feature complexity
- **Offline-First Priority**: Every feature must work without network
- **Module Contribution**: Respect platform vs. module boundaries
- **TDD for Domain Logic**: No exceptions for domain code

---

**Version**: 2.0.0
**Last Updated**: 2026-02-06
**Maintained By**: Tony + Claude
