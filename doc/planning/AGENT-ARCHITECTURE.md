# Agent Architecture

Design document for the Belsouri multi-agent strategy. Agents are single-concern specialists that cross-check each other, preventing the failures from belsouri-old and enforcing the ceremony-based SDLC.

**Status**: Design (not yet implemented)

---

## Motivation

The belsouri-old codebase failed despite 177 passing tests because:

- Code was never tested through the Tauri invoke layer
- 62 of 77 commands lacked `rename_all = "snake_case"` -- broke at runtime
- TypeScript interfaces drifted from Rust structs silently
- Every write rebuilt the entire projection from scratch (O(n^2))
- The agent claimed features worked without running the app

CLAUDE.md captures these rules, but rules in a single document can be overlooked under context pressure. Separate agents with single concerns provide **structural enforcement**: the ceremony-gate blocks implementation without artifacts, the verification-enforcer blocks "done" claims without end-to-end verification, and the governance-auditor catches violations against ADRs and policies.

---

## Plugin Structure

```
.claude/plugins/belsouri-agents/
+-- .claude-plugin/
|   +-- plugin.json          # Plugin manifest
+-- agents/
|   +-- ceremony-gate.md     # Phase 1 - blocks implementation without ceremonies
|   +-- governance-auditor.md # Phase 1 - checks compliance with ADRs/policies/CLAUDE.md
|   +-- verification-enforcer.md # Phase 1 - ensures end-to-end verification happened
|   +-- domain-language-guard.md # Phase 2 - flags terms not in ubiquitous language
|   +-- architect.md          # Phase 2 - big picture guidance and impact analysis
|   +-- lessons-capture.md    # Phase 2 - extracts insights for LESSONS-LEARNED.md
+-- hooks/
|   +-- rules/
|       +-- stop-without-verification.local.md  # Phase 1
|       +-- implementation-without-ceremony.local.md # Phase 1
+-- README.md
```

---

## Agent Inventory -- Phase 1

These three agents directly prevent the belsouri-old failures. Build them first.

### ceremony-gate

The most important agent. Blocks implementation when required ceremony artifacts are missing.

| Property | Value |
|----------|-------|
| **Model** | haiku |
| **Concern** | Ceremony artifact completeness |
| **Tools** | Read, Glob, Grep (read-only, no code changes) |

**Reads**:
- `HOW-WE-WORK.md` ceremony trigger table
- `doc/domain/event-storming/`
- `doc/domain/ubiquitous-language.md`
- `doc/domain/aggregates/`
- `doc/domain/context-maps/`
- `doc/scenarios/example-maps/`
- `features/`

**Produces**: Pass/fail checklist of required artifacts for the current context or feature, referencing the ceremony trigger table from HOW-WE-WORK.md:

| Trigger | Required Ceremonies |
|---------|-------------------|
| New bounded context | Full Phase 1: Event Storming -> Ubiquitous Language -> Domain Modeling -> Context Mapping -> Governance |
| New feature in existing context | Phase 2: Three Amigos -> Example Mapping -> Acceptance Criteria Review -> BDD Scenarios -> Governance |
| New aggregate in existing context | Partial Phase 1 (aggregate doc + language update) + Phase 2 |
| Infrastructure, build config, UI scaffolding, clear-scope bug fixes | No ceremony needed |

**Triggers**: Proactively before implementation; on-demand via "check ceremonies for [context]".

**Cross-checks**: Informs governance-auditor (ceremony completeness is a governance concern).

**Why haiku**: Pure file existence and checklist matching -- doesn't need deep reasoning.

---

### governance-auditor

Checks code and artifacts against ADRs, policies, and CLAUDE.md conventions.

| Property | Value |
|----------|-------|
| **Model** | sonnet |
| **Concern** | Governance compliance |
| **Tools** | Read, Glob, Grep, Bash (for `git diff`) -- read-only |

**Reads**:
- `doc/governance/ADR/`
- `doc/governance/POL/`
- `CLAUDE.md`
- Changed code files (via `git diff`)

**Produces**: Compliance report with:
- Violations with severity (error / warning / info)
- Specific ADR/policy reference for each violation
- Fix suggestion

**Triggers**: On-demand; proactively before commits/PRs.

**Cross-checks**: Validates output of all other agents and implementation work. This is the agent that audits everyone else.

**Why sonnet**: Needs to understand both governance documents and code semantics -- pattern matching alone isn't sufficient.

---

### verification-enforcer

Ensures `pnpm tauri dev` was run and the end-to-end path was verified before claiming work is done.

| Property | Value |
|----------|-------|
| **Model** | haiku |
| **Concern** | End-to-end verification |
| **Tools** | Bash, Grep (read-only) |

**Reads**:
- Session transcript (what commands were run)
- `git diff` (what code changed)

**Produces**: Pass/block with specific instructions, e.g.: "Run `pnpm tauri dev` and verify [feature] end-to-end. Check the database to confirm persistence."

**Triggers**: Proactively when implementation session is ending; as a Stop hook.

**Cross-checks**: Final gate -- no other agent can override this. If code was modified but the app wasn't run, this agent flags it.

**Why haiku**: Transcript pattern matching -- fast and cheap.

---

## Agent Inventory -- Phase 2

Build after Phase 1 agents are working and there are code and domain artifacts to analyze.

### domain-language-guard

Flags terms in code and docs that don't appear in the ubiquitous language glossary.

| Property | Value |
|----------|-------|
| **Model** | haiku |
| **Concern** | Ubiquitous language conformance |
| **Tools** | Read, Glob, Grep (read-only) |

**Reads**:
- `doc/domain/ubiquitous-language.md`
- Changed files (code, docs, tests)

**Produces**: List of non-conforming terms with suggested replacements from the glossary.

**Triggers**: On-demand; proactively before commits touching domain code (files in `src-tauri/src/` or `src/`).

---

### architect

Big picture guidance, dependency analysis, and impact assessment.

| Property | Value |
|----------|-------|
| **Model** | inherit (opus) |
| **Concern** | Architecture and cross-cutting design |
| **Tools** | All read tools + Bash for project analysis |

**Reads**:
- `doc/planning/DEVELOPMENT-PLAN.md`
- `doc/domain/context-maps/`
- `doc/governance/ADR/`
- Aggregate docs
- Dependency map

**Produces**: Architectural guidance -- impact analysis, pattern recommendations, boundary violations, cross-context dependency warnings.

**Triggers**: On-demand for design discussions and new feature planning. Not needed for routine implementation.

**Why opus**: Cross-context reasoning requires deep analysis. This agent is invoked infrequently so cost is manageable.

---

### lessons-capture

Extracts reusable insights from development sessions for `LESSONS-LEARNED.md`.

| Property | Value |
|----------|-------|
| **Model** | haiku |
| **Concern** | Institutional memory |
| **Tools** | Read, Grep (read-only) |

**Reads**:
- Session transcript
- `LESSONS-LEARNED.md`

**Produces**: Proposed additions to `LESSONS-LEARNED.md` in the established format (What happened / What we learned / What we'll do differently / Governance). Does not write directly -- Tony reviews and approves.

**Triggers**: On-demand at end of significant sessions.

---

## Future Agents (Phase 3+ -- Only If Needed)

These are deferred because CLAUDE.md conventions and the Phase 1/2 agents cover these concerns adequately at current codebase scale. Separate agents become valuable when the codebase grows and main context can't hold everything.

| Agent | Concern | Build When |
|-------|---------|------------|
| `fp-best-practices` | Functional patterns, immutability, pure functions in Rust | When codebase is large enough to have pattern drift |
| `event-sourcing-reviewer` | Event store correctness, projection incrementality, WAL mode | When multiple aggregates exist |
| `caribbean-resilience-checker` | Auto-save, power-loss recovery, memory efficiency | When UI implementation begins |

---

## Hook Inventory

Two hooks using `.local.md` format. Hooks are lightweight shell pattern matching -- no LLM calls, minimal overhead.

### stop-without-verification

**Type**: Stop hook

**Condition**: Transcript contains file writes to `src-tauri/` or `src/` but does NOT contain `pnpm tauri dev`.

**Action**: Warn (not block -- Tony can override).

**Message**: "Code was modified but `pnpm tauri dev` wasn't run. Verify end-to-end before claiming this works."

### implementation-without-ceremony

**Type**: PreToolUse hook on Edit/Write

**Condition**: Tool target is `src-tauri/src/` or `src/` AND ceremony artifacts for the relevant context don't exist.

**Action**: Warn.

**Message**: "Implementation starting without completed ceremony artifacts. Run ceremony-gate to check."

---

## Interaction Map -- Who Checks Whom

```
                    +---------------------+
                    | governance-auditor   |
                    | (checks everything)  |
                    +----------+----------+
                               | audits
            +------------------+------------------+
            v                  v                  v
    +------------+     +---------------+    +-----------+
    | ceremony   |     | domain-lang   |    | architect |
    | gate       |     | guard         |    |           |
    +-----+------+     +-------+-------+    +-----+-----+
          |                    |                    |
          | gates              | validates          | advises
          v                    v                    v
    +---------------------------------------------+
    |         Implementation Work                 |
    +----------------------+----------------------+
                           |
                           v
    +---------------------------------------------+
    |       verification-enforcer                 |
    |       (final gate before "done")            |
    +----------------------+----------------------+
                           |
                           v
    +---------------------------------------------+
    |         lessons-capture                     |
    |         (extracts insights)                 |
    +---------------------------------------------+
```

**Flow**: ceremony-gate -> (implementation) -> governance-auditor + domain-language-guard -> verification-enforcer -> lessons-capture

**Key principle**: Governance-auditor sits above all other agents and audits their output. Verification-enforcer is the final gate that no other agent can override. Ceremony-gate runs first and blocks implementation from starting without artifacts.

---

## Token Efficiency Strategy

| Strategy | How |
|----------|-----|
| **Model tiering** | haiku for pattern matching (ceremony-gate, verification-enforcer, domain-language-guard, lessons-capture); sonnet for code+doc analysis (governance-auditor); opus only for architect |
| **Tool restriction** | Audit agents get read-only tools only -- prevents accidental edits and reduces token overhead from tool descriptions |
| **Scoped reading** | Each agent reads only its specific input files, not the whole codebase |
| **On-demand over ambient** | Agents are invoked when needed, not loaded into every session's context |
| **Lightweight hooks** | Hooks use shell pattern matching, not LLM calls |
| **Haiku-first** | Default to haiku, upgrade only when the agent demonstrably needs deeper reasoning |

---

## Phased Rollout

| Phase | What | When | Why First/Later |
|-------|------|------|-----------------|
| **Now** | This design document | This session | Blueprint before building |
| **Phase 1 Track A** | ceremony-gate + verification-enforcer + 2 hooks | When infrastructure scaffold exists | Prevents the exact failures from belsouri-old |
| **Phase 2** | governance-auditor + domain-language-guard | When code and domain artifacts exist | Need something to audit |
| **Phase 3+** | architect + lessons-capture | When multiple contexts are in play | Cross-context reasoning becomes valuable at scale |
| **If needed** | fp-best-practices, event-sourcing-reviewer, etc. | When pattern drift is observed | Premature until codebase is large |

---

## What This Design Does NOT Cover

- **Plugin implementation**: This is the blueprint. Agent `.md` files, `plugin.json`, and hook scripts come later.
- **MCP servers**: Agents + hooks cover all current needs without MCP complexity.
- **Concerns already in CLAUDE.md**: FP patterns, event sourcing rules, and Caribbean resilience conventions are in the main context and don't need separate agents yet. When the codebase outgrows main context, promote them to agents.

---

## Relationship to Other Planning Artifacts

- **DEVELOPMENT-PLAN.md**: Tracks what to build and in what order. This document tracks how agents enforce quality during that build.
- **HOW-WE-WORK.md**: Defines the ceremony framework. ceremony-gate enforces it.
- **CLAUDE.md**: Defines conventions. governance-auditor enforces them.
- **LESSONS-LEARNED.md**: Captures insights. lessons-capture proposes additions.
