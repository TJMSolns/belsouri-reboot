# TODO

**Last Updated**: 2026-03-05

---

## Active

### SCH-4b — PM Booking Override
Ceremonies complete. Implementation starting.

- [ ] Implement soft-stop override path in `appointments/commands.rs` for C1/C2 violations
- [ ] Add `override_reason: Option<String>` to `BookAppointment` command
- [ ] Emit `BookingConstraintOverridden` event
- [ ] Update appointment_list projection to surface override flag
- [ ] Frontend: override confirmation dialog with reason field (sheet pattern per PDR-003)
- [ ] Frontend: display override indicator on booked appointments

---

## Next Sprint

### SCH-4b Frontend UI
- [ ] Override confirmation sheet with reason text input
- [ ] Appointment card override badge (icon + label, POL-002)
- [ ] Watchdog pass (ux-review, copy-check, icon-audit)

---

## Post-MVP Backlog

### SCH-6 — Role-Based View Switching
Full Phase 1 ceremonies required before implementation begins (new feature in existing context = Phase 2 ceremonies). Blocked on Tony scheduling Three Amigos session.

### PDR-004 — Outcome-Based RBAC
Concept captured in `doc/governance/PDR/PDR-004.md`. Not scheduled. Revisit after SCH-6.

### Recall & Outreach Module
New bounded context — requires full Phase 1 (Event Storming through Governance) before any implementation.

### Operations
- [ ] `doc/operations/BACKUP.md` — write backup and restore procedure
- [ ] Update root MD files after each sprint (add to post-sprint checklist)

---

**Maintained By**: Tony + Claude
