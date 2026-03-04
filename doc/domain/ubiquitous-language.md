# Ubiquitous Language

Living glossary of domain terms. Updated during Event Storming and throughout development. These terms must be used consistently in code, tests, documentation, and conversation.

**Rule**: If a term isn't in this glossary, it shouldn't be in the codebase. If you need a new term, add it here first.

---

## Practice Setup Context

### Core Entities

| Term | Definition | Not This | Notes |
|------|-----------|----------|-------|
| **Practice** | The root organizational entity representing a dental clinic business. Singleton per installation. Holds identity (name, contact info, address). | "clinic", "company", "organization" | A practice may operate from multiple offices. Nico refers to "the practice" as the whole business. |
| **Office** | A physical location where dental services are provided. Each office has its own name, chair capacity, and operating hours. | "location", "branch", "site", "clinic" | Nico uses geographic names: "Kingston", "Montego Bay". Offices are independently configured. |
| **Provider** | Any dental professional who provides services to patients. Covers dentists, hygienists, and specialists. | "staff", "doctor", "employee", "clinician", "staff member" | Nico confirmed: "provider is a word we use and covers all staff roles." This is the single unified term for all clinical personnel. |
| **Procedure Type** | A named category of dental service that can be scheduled, with a default duration and color-coded category. | "service", "treatment", "appointment type" | Examples: Cleaning, Consultation, Root Canal. Each has a category and default duration. |

### Value Objects

| Term | Definition | Not This | Notes |
|------|-----------|----------|-------|
| **Chair** | A physical treatment station within an office. Chair count is the capacity constraint that limits concurrent appointments. | "room", "operatory", "station" | Nico: "how many chairs" = office capacity. Not a bookable resource itself -- just a count that constrains scheduling. |
| **Provider Type** | Classification of a provider's clinical role: Dentist, Hygienist, or Specialist. | "role", "job title", "staff type" | Nico confirmed these three types are sufficient. |
| **Procedure Category** | A color-coded grouping of procedure types for calendar display and workflow organization. | "procedure group", "service type" | Six categories: Consult (yellow), Preventive (blue), Restorative (green), Invasive (red), Cosmetic (purple), Diagnostic (gray). Nico confirmed all appropriate. |
| **Operating Hours** | The time range during which an office is open on a given day of the week. | "business hours", "opening times", "schedule" | Per-office, per-day. Days without configured hours are considered closed. |
| **Availability** | A time window during which a provider works at a specific office on a given day of the week. | "schedule", "shift", "working hours" | Office-scoped -- same provider can have different availability at different offices. Must not overlap across offices on the same day. |
| **Exception** | A date range during which a provider is unavailable, overriding normal weekly availability. | "time off", "leave", "absence", "vacation" | Used for vacations, holidays, off-season closures. Jamaica dental care is seasonal (insurance-driven). Existing appointments are warned about but not auto-cancelled. |
| **Subdivision** | The administrative region within a country used for addresses. Parish in Jamaica; varies by Caribbean country. | "state", "province", "county" | Country-aware label. MVP uses "Parish" for Jamaica. |
| **Default Duration** | The standard time in minutes allocated for a procedure type when scheduling. | "appointment length", "time slot" | Range: 15-240 minutes. Can be overridden per appointment. |

### Actions and Concepts

| Term | Definition | Not This | Notes |
|------|-----------|----------|-------|
| **Register** (a provider) | Add a new provider to the practice with name and type. | "create", "add", "onboard" | Domain language: providers are "registered", not "created". |
| **Assign** (provider to office) | Link a provider to an office, enabling availability to be set there. | "add to", "attach", "associate" | Must happen before setting availability. Removal auto-clears availability at that office. |
| **Archive** | Soft-delete an entity (office or provider), hiding it from active lists but preserving historical data. | "delete", "remove", "deactivate" | Event sourcing is append-only -- no hard deletes. Archived entities can be unarchived. |
| **Define** (a procedure type) | Create a new procedure type with name, category, and default duration. | "create", "add" | Domain language: procedure types are "defined", not "created". |
| **Deactivate** (a procedure type) | Remove a procedure type from the active list without deleting it. | "delete", "archive", "remove" | Procedure types use deactivate/reactivate, not archive/unarchive, because they are configuration items not people/places. |
| **Practice Manager** | Role with global authority over all practice configuration. | "admin", "super user", "owner" | Full Practice Setup access. Can delegate permissions to Office Managers (future enhancement). |
| **Office Manager** | Role with authority scoped to a single office. | "local admin", "branch manager" | Scope of delegated permissions controlled by Practice Manager. MVP: Practice Manager only; delegation is future. |

### Bounded Context Terms

| Term | Definition | Not This | Notes |
|------|-----------|----------|-------|
| **Practice Setup** | The bounded context responsible for configuring the static structure of a practice: offices, providers, procedure types, and practice identity. | "admin", "settings", "configuration" | Foundational context -- all other contexts depend on it. Purely upstream (emits events, does not consume from other contexts). |
| **Bounded Context** | A logical boundary around a cohesive set of domain concepts with their own language and rules. | "module", "service", "component" | Each context has its own aggregates, events, and invariants. |
| **Aggregate** | A cluster of domain objects treated as a single unit for data changes, with an aggregate root that enforces invariants. | "entity", "model", "record" | Practice Setup has 4 aggregates: Practice, Office, Provider, ProcedureType. |
| **Domain Event** | An immutable record of something that happened in the domain. Stored in the event store and used to build projections. | "log entry", "notification", "message" | Named in past tense: OfficeCreated, ProviderRegistered. Append-only -- never deleted or modified. |
| **Projection** | A queryable read model materialized from domain events. Built incrementally, never from full rebuild. | "view", "cache", "snapshot" | Must be deterministic and incremental. O(n^2) full rebuilds caused performance collapse in belsouri-old. |
| **Command** | A request to change domain state. Validated against invariants before producing events. | "action", "request", "mutation" | Named in imperative: CreateOffice, RegisterProvider. May be rejected if invariants are violated. |

---

## Banned Terms

Terms that must NOT be used in code, tests, or documentation because they are ambiguous, generic, or conflict with domain language.

| Banned Term | Why | Use Instead |
|-------------|-----|-------------|
| staff / staff member | Ambiguous -- could mean clinical or administrative | **Provider** (for clinical staff in scheduling context) |
| clinic | Ambiguous -- could mean practice or office | **Practice** (the business) or **Office** (the location) |
| doctor | Too specific -- excludes hygienists and specialists | **Provider** |
| room | Implies bookable physical space; our constraint is chair count | **Chair** (as capacity unit) |
| slot | Implies fixed time blocks; our scheduling is continuous | **Availability** (provider's working window) |
| appointment type | Conflates scheduling concept with practice config | **Procedure Type** |
| DTO | Implementation jargon -- not domain language | Describe the actual concept |
| Manager / Handler / Service | Generic OOP terms that hide domain meaning | Use domain-specific names |
| delete | Implies permanent removal; event sourcing is append-only | **Archive** (office/provider) or **Deactivate** (procedure type) |
| user | Ambiguous -- could mean patient, provider, or admin | Use the specific role: **Practice Manager**, **Provider**, **Patient** |

---

**Next update**: Add terms from Staff Scheduling, Patient Management, and Patient Scheduling contexts as those domains are discovered.
