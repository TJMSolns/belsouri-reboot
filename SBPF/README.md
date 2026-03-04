# Shared Best Practice Files (SBPF)

**Purpose**: Reference guides for development patterns, methodologies, and domain-specific requirements.

**Applicability**: See [POL-005](../../governance/POL/POL-005-sbpf-applicability.md) for which SBPFs apply to Belsouri.

---

## Tier 1: Core Requirements

These SBPFs contain mandatory requirements for Belsouri:

| File | Key Content |
|------|-------------|
| Jamaica-EHR-Compliance-Patterns.md | MOHW compliance, ICD-10-CM/CPT/CDT coding, patient consent |
| Offline-First-Desktop-Architecture.md | Event sourcing, 30-day grace period, sync patterns |
| Local-First-Healthcare-Data-Architecture.md | Medical event modeling, PHI handling |
| Caribbean-Desktop-Resilience-Patterns.md | Power-loss recovery, tropical climate |

---

## Tier 2: Implementation Guidance

| File | Use When |
|------|----------|
| Windows-Desktop-Healthcare-Patterns.md | Windows 10/11 deployment |
| MacOS-Desktop-Healthcare-Patterns.md | macOS deployment (future) |
| Desktop-Healthcare-Data-Security.md | SQLite encryption, credentials |
| Desktop-Application-Performance-Patterns.md | Performance optimization |
| Caribbean-Hardware-Integration-Patterns.md | Printer/receipt hardware |
| Caribbean-Desktop-Deployment-Strategies.md | Installer and updates |
| Clinical-Desktop-UX-Patterns.md | Healthcare workflow UI |

**Platform Roadmap**: Windows → macOS → Chrome

---

## Tier 3: Methodology Reference

### DDD/BDD/TDD
- Blending-DDD-BDD-TDD.md - Ceremony integration
- DDD-Best-Practices.md, DDD-Principles.md - Domain modeling
- BDD-Best-Practices.md, Gherkin-Syntax.md - Specification
- TDD-Best-Practices.md - Test-first development

### Architecture
- Event-Driven-Architecture.md - Event sourcing patterns
- Anticorruption-Layer-Patterns.md - ACL design

### Testing
- Property-Based-Testing-Implementation.md - Generative testing
- Resilience-in-Acceptance-Criteria.md - Resilience scenarios

---

## Archived

Files in `archive/` are from previous projects and do not apply:
- Scala/Mill/Pekko stack files (superseded by ADR-002)
- Search platform files (different product)

---

## Usage

1. Check [POL-005](../../governance/POL/POL-005-sbpf-applicability.md) for applicability
2. Reference Tier 1 SBPFs when implementing core features
3. Consult Tier 2/3 as needed during implementation

---

**Last Updated**: 2026-02-06
