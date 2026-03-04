# Mill Tasks Reference

Authoritative reference for ceremony validation and scaffolding tasks used across the ceremony-based SDLC.

---

## Overview

Mill is our build tool and SDLC orchestrator. Tasks validate ceremony outputs, scaffold projects, and help keep quality gates fast and consistent.

- Implementation: Scala 3.3.1, side-effect free, non-blocking I/O
- Parallelism: Per-check `T.task` targets enable parallel execution with `--jobs`
- Exit codes: `0` = success, `1` = failure

---

## Namespaces

### bootstrap.*
- `bootstrap.bootstrapValidate` — Pre-flight checks before bootstrapping a new project (token/org/name/Mill version/source repo)
- `bootstrap.bootstrapExecute` — Create a new project with complete framework (HOW-WE-WORK.md, 23 ceremonies, 39 SBPFs, 34 templates, 9 Mill modules, 4 framework POLs)

Usage:
```bash
mill bootstrap.bootstrapValidate --name <project-name> [--org <organization>]
mill bootstrap.bootstrapExecute --name <project-name> [--path <dir>] [--create-repo] [--org <organization>]
```

### spinoff.*
- `spinoff.spinoffValidate <service>` — 13 pre-flight checks (charter, domain model, context map, scenarios, tests, API, events, migrations, observability, security, deployment, ownership)
- `spinoff.spinoffExecute <service>` — Extract bounded context into a production repo with full framework and service scaffolds

Usage:
```bash
mill <module>.spinoffValidate <ServiceName>
mill <module>.spinoffExecute <ServiceName>
```

### validate.* (domain/specification/context)
- `validate.domainValidate` — 10 checks: aggregates, value objects (Java Records), domain events, repositories, aggregate size, actor aggregates, immutability, cyclic-dep guard, entity identity, invariants
- `validate.languageValidate` — 7 checks: glossary presence, terms-in-code, banned terms/packages in domain, naming consistency, purity of domain layer, no technical jargon
- `contextMapValidate` — Infrastructure scope guard: context map infrastructure matches compose; contexts match build; ADR references exist
- `validate.contextValidate` — 5 checks: map exists, no leakage across boundaries, ACLs, shared kernel doc, anticorruption for legacy
- `validate.domainAll` — Runs all domain validations in order (fail-fast)

Usage:
```bash
mill <service>.domain.domainValidate
mill <service>.domain.languageValidate
mill contextMapValidate
mill <service>.domain.contextValidate
mill <service>.domain.domainAll
```

---

## Fast Validation & Performance

- Use `validateAll` composition to run domain, specification, testing, quality, and observability validations together
- Run in parallel with `--jobs <N>`
- Profile with `mill --profile <target>` and track metrics via `scripts/track-build-performance.sh` (writes `out/build-metrics.csv`)

Example:
```bash
mill <service>.validateAll --jobs 8
mill --profile <service>.domain.domainValidate
scripts/track-build-performance.sh
```

---

## CI/CD Integration

GitHub Actions example:
```yaml
name: Domain Validation
on: [push, pull_request]
jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Setup Java
        uses: actions/setup-java@v3
        with:
          java-version: '21'
          distribution: 'temurin'
      - name: Validate Domain Model
        run: mill <service>.domain.domainValidate
      - name: Validate Ubiquitous Language
        run: mill <service>.domain.languageValidate
      - name: Validate Context Map
        run: mill contextMapValidate
```

---

## Locations

- Reference file: `doc/internal/reference/MILL-TASKS-REFERENCE.md`
