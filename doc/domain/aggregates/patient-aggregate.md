# Patient Aggregate

**Context**: Patient Management
**Last Updated**: 2026-03-04

---

## Purpose

A person who receives dental care at the practice. The Patient aggregate is the authoritative record of patient identity and contact information, consumed by Patient Scheduling (for booking) and Clinical Records (post-MVP, for charting).

Patient Management is intentionally thin at MVP: demographics, contact info, and notes. No clinical history — that belongs to Clinical Records (post-MVP).

---

## Fields

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| id | UUID | Yes | System-generated |
| first_name | String | Yes | Non-empty |
| last_name | String | Yes | Non-empty |
| date_of_birth | Date? | No | YYYY-MM-DD. Strongly recommended — used for disambiguation when two patients share a name. |
| phone | String? | Conditional | At least phone or email required |
| email | String? | Conditional | At least phone or email required |
| preferred_contact_channel | PreferredContactChannel | No | WhatsApp (default), SMS, Phone, Email |
| preferred_office_id | UUID? | No | References Practice Setup office. Optional — for filtering, not a hard binding. |
| address_line_1 | String? | No | |
| city_town | String? | No | |
| subdivision | String? | No | Parish for Jamaica; country-aware label |
| country | String? | No | "Jamaica" default |
| notes | List<PatientNote> | No | Append-only. Indexed by note_id. |
| registered_by | staff_member_id | Yes | Who created the record — full audit trail |
| archived | bool | Yes | Default false |

### Value Objects

**PatientNote**:
| Field | Type | Notes |
|-------|------|-------|
| note_id | UUID | System-generated |
| text | String | Required, non-empty |
| recorded_by | staff_member_id | Required — accountability (Nico: "when staff edits patients we need to audit that") |
| recorded_at | Timestamp (UTC) | Required |

**PreferredContactChannel**: WhatsApp | SMS | Phone | Email (shared with Practice Setup and Staff Management)

---

## Events

| Event | Fields | When |
|-------|--------|------|
| **PatientRegistered** | patient_id, first_name, last_name, phone?, email?, preferred_contact_channel?, preferred_office_id?, date_of_birth?, registered_by (staff_member_id) | Front desk creates a new patient record |
| **PatientDemographicsUpdated** | patient_id, first_name, last_name, date_of_birth?, address fields?, updated_by (staff_member_id) | Front desk or PM updates name, DOB, or address |
| **PatientContactInfoUpdated** | patient_id, phone?, email?, preferred_contact_channel?, updated_by (staff_member_id) | Front desk updates contact details |
| **PatientNoteAdded** | patient_id, note_id, text, recorded_by (staff_member_id), recorded_at | Staff member adds a note |
| **PatientArchived** | patient_id, archived_by (staff_member_id) | PM soft-deletes the patient record |
| **PatientUnarchived** | patient_id, unarchived_by (staff_member_id) | PM restores an archived patient |

---

## Commands

| Command | Input | Produces | Preconditions |
|---------|-------|----------|---------------|
| RegisterPatient | first_name, last_name, phone?, email?, preferred_contact_channel?, preferred_office_id?, date_of_birth?, registered_by | PatientRegistered | first_name not empty, last_name not empty, at least one of phone or email provided |
| UpdatePatientDemographics | patient_id, first_name, last_name, date_of_birth?, address fields?, updated_by | PatientDemographicsUpdated | patient active, first_name not empty, last_name not empty |
| UpdatePatientContactInfo | patient_id, phone?, email?, preferred_contact_channel?, updated_by | PatientContactInfoUpdated | patient active, at least one of phone or email after update |
| AddPatientNote | patient_id, text, recorded_by | PatientNoteAdded | patient active (notes can also be added to archived patients — see HS-6), text not empty |
| ArchivePatient | patient_id, archived_by | PatientArchived | patient active |
| UnarchivePatient | patient_id, unarchived_by | PatientUnarchived | patient archived |

---

## Invariants

1. **Name required**: Both first_name and last_name must be non-empty
2. **Contact required**: At least one of phone or email must be provided (at registration and after any UpdatePatientContactInfo)
3. **Notes are append-only**: PatientNoteAdded is the only way to change notes — individual notes cannot be edited or deleted
4. **Full audit trail**: Every command that writes state must carry a staff_member_id (registered_by, updated_by, recorded_by, archived_by). No anonymous writes.
5. **Soft-delete only**: Patient records are never hard-deleted. Archive/Unarchive only.
6. **Practice-wide**: A patient record belongs to the practice, not to a specific office. preferred_office_id is a hint, not a binding.

---

## State Machine

```
stateDiagram-v2
    [*] --> Active : RegisterPatient
    Active --> Active : UpdatePatientDemographics / UpdatePatientContactInfo / AddPatientNote
    Active --> Archived : ArchivePatient
    Archived --> Active : UnarchivePatient
```

---

## Projections

### PatientList

Queryable by: name (prefix search), phone, email, preferred_office_id.

```
Table: patient_list
  patient_id           UUID
  first_name           TEXT
  last_name            TEXT
  full_name_display    TEXT    -- "Brown, Maria" (last, first) for sorted list
  phone                TEXT?
  email                TEXT?
  date_of_birth        DATE?
  preferred_office_id  UUID?
  archived             BOOL
```

### PatientDetail

Full detail view for a patient card.

```
Table: patient_detail
  patient_id                UUID
  first_name                TEXT
  last_name                 TEXT
  date_of_birth             DATE?
  phone                     TEXT?
  email                     TEXT?
  preferred_contact_channel TEXT?
  preferred_office_id       UUID?
  address_line_1            TEXT?
  city_town                 TEXT?
  subdivision               TEXT?
  country                   TEXT?
  archived                  BOOL
  registered_by             UUID    -- staff_member_id
```

### PatientNoteList

Per-patient list of notes for the patient card notes tab.

```
Table: patient_notes
  note_id      UUID
  patient_id   UUID
  text         TEXT
  recorded_by  UUID    -- staff_member_id
  recorded_at  TIMESTAMP
```

---

## Open Questions

| # | Question | Assumption | Status |
|---|----------|-----------|--------|
| PM-1 | Patient bound to one office or practice-wide? | Practice-wide with optional preferred_office_id | [OPEN QUESTION — Tony to confirm] |
| PM-2 | Single-word patient names — first/last structure or single field? | First + last (both required); convention for single-word names TBD | [OPEN QUESTION — Tony to confirm] |
| PM-3 | Deduplication at MVP — soft warning or blocked? | Soft warning on same name + phone (not blocked) | [OPEN QUESTION — Tony to confirm] |
| PM-6 | Who can archive a patient? Front desk or only PM? | Only Practice Manager | [ASSUMED — Tony to confirm] |

---

## Design Decisions

- **"Register", not "Create"**: Follows the established domain pattern (providers are registered, staff members are registered, now patients are registered).
- **Notes are append-only**: Never edit or delete notes. This is a deliberate audit trail design — Nico explicitly mentioned accountability for who edited patient data.
- **Notes travel with the patient**: PatientNotes belong to the Patient aggregate and are available to any context that reads Patient Management projections (Scheduling, Clinical Records post-MVP). Not context-specific.
- **Thin at MVP**: No clinical fields in this aggregate. Chief complaint, treatment history, allergies, medications — all belong to Clinical Records (post-MVP). The boundary is intentional.
- **Full audit on every write**: registered_by, updated_by, recorded_by on every state change. Caribbean practices need accountability — who did what, when.

---

**Maintained By**: Tony + Claude
