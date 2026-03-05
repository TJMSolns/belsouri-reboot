<script lang="ts">
  import { commands } from "$lib/bindings";
  import type {
    AppointmentDto, AppointmentWithNotesDto, AppointmentNoteDto,
    CallListEntryDto, ProviderScheduleEntry,
  } from "$lib/bindings";
  import { getErrorMessage } from "$lib/utils/api";

  // Placeholder until session/auth exists
  const STAFF_ID = "staff-system";

  // ── State ─────────────────────────────────────────────────────────────────

  // Data loaded from setup (needed for dropdowns)
  let offices = $state<{ id: string; name: string; chair_count: number; archived: boolean }[]>([]);
  let providers = $state<{ id: string; name: string; provider_type: string; archived: boolean }[]>([]);
  let procedures = $state<{ id: string; name: string; category: string; default_duration_minutes: number; is_active: boolean }[]>([]);
  let patients = $state<{ patient_id: string; patient_name: string; first_name: string; last_name: string; phone: string | null }[]>([]);

  // Schedule view
  let selectedOfficeId = $state("");
  let selectedDate = $state(todayLocal());
  let schedule = $state<AppointmentDto[]>([]);
  let scheduleError = $state("");
  let scheduleLoading = $state(false);

  // Call list
  let showCallList = $state(false);
  let callList = $state<CallListEntryDto[]>([]);
  let callListDate = $state(tomorrowLocal());

  // Expanded appointment detail
  let expandedId = $state<string | null>(null);
  let expandedDetail = $state<AppointmentWithNotesDto | null>(null);
  let detailLoading = $state(false);

  // Book appointment form
  let showBookForm = $state(false);
  let bookOfficeId = $state("");
  let bookPatientSearch = $state("");
  let bookPatientId = $state("");
  let bookPatientName = $state("");
  let bookProviderId = $state("");
  let bookProcedureId = $state("");
  let bookStartDate = $state(todayLocal());
  let bookStartTime = $state("");
  let bookError = $state("");
  let bookLoading = $state(false);
  let bookSuccess = $state("");

  // Patient search results for booking
  let patientSearchResults = $state<typeof patients>([]);

  // View roster (providers scheduled for currently-viewed date/office)
  let providerRoster = $state<ProviderScheduleEntry[]>([]);
  let rosterLoading = $state(false);

  // Book roster: providers scheduled for the *booking* date/office (may differ from view)
  let bookRoster = $state<ProviderScheduleEntry[]>([]);
  let bookRosterLoading = $state(false);

  // Time slots available for the selected provider on the booking date
  let availableSlots = $derived((() => {
    const entry = bookRoster.find(e => e.provider_id === bookProviderId);
    if (!entry) return [];
    return generateTimeSlots(entry.start_time, entry.end_time);
  })());

  // Note form
  let noteAppointmentId = $state("");
  let noteText = $state("");
  let noteError = $state("");
  let noteLoading = $state(false);

  // ── Helpers ───────────────────────────────────────────────────────────────

  function todayLocal(): string {
    return new Date().toISOString().slice(0, 10);
  }

  function tomorrowLocal(): string {
    const d = new Date();
    d.setDate(d.getDate() + 1);
    return d.toISOString().slice(0, 10);
  }

  function formatTime(isoLocal: string): string {
    // "2026-03-09T10:00:00" → "10:00"
    return isoLocal.slice(11, 16);
  }

  function formatDate(isoLocal: string): string {
    return isoLocal.slice(0, 10);
  }

  function buildStartTime(date: string, time: string): string {
    return `${date}T${time}:00`;
  }

  /** Generate HH:MM time slots every 15 minutes between start and end. */
  function generateTimeSlots(start: string, end: string): string[] {
    const slots: string[] = [];
    let [h, m] = start.split(":").map(Number);
    const [eh, em] = end.split(":").map(Number);
    while (h * 60 + m < eh * 60 + em) {
      slots.push(`${h.toString().padStart(2, "0")}:${m.toString().padStart(2, "0")}`);
      m += 15;
      if (m >= 60) { h += 1; m -= 60; }
    }
    return slots;
  }

  function statusBadgeClass(status: string): string {
    const map: Record<string, string> = {
      Booked: "badge-booked",
      Completed: "badge-completed",
      Cancelled: "badge-cancelled",
      NoShow: "badge-noshow",
      Rescheduled: "badge-rescheduled",
    };
    return map[status] ?? "badge-booked";
  }

  // ── Data loading ──────────────────────────────────────────────────────────

  async function loadSetupData() {
    const [officeRes, providerRes, procRes] = await Promise.all([
      commands.listOffices(),
      commands.listProviders(),
      commands.listProcedureTypes(),
    ]);
    if (officeRes.status === "ok") {
      offices = officeRes.data.filter((o) => !o.archived);
      if (!selectedOfficeId && offices.length > 0) {
        selectedOfficeId = offices[0].id;
        bookOfficeId = offices[0].id;
      }
    }
    if (providerRes.status === "ok") {
      providers = providerRes.data.filter((p) => !p.archived);
    }
    if (procRes.status === "ok") {
      procedures = procRes.data.filter((p) => p.is_active);
    }
  }

  async function loadSchedule() {
    if (!selectedOfficeId) return;
    scheduleLoading = true;
    scheduleError = "";
    const res = await commands.getSchedule(selectedOfficeId, selectedDate);
    scheduleLoading = false;
    if (res.status === "ok") {
      schedule = res.data;
    } else {
      scheduleError = getErrorMessage(res.error);
    }
  }

  async function loadProviderRoster() {
    if (!selectedOfficeId || !selectedDate) return;
    rosterLoading = true;
    const res = await commands.getOfficeProviderSchedule(selectedOfficeId, selectedDate);
    rosterLoading = false;
    if (res.status === "ok") {
      providerRoster = res.data;
    }
  }

  async function loadBookRoster() {
    if (!bookOfficeId || !bookStartDate) return;
    // Reuse already-loaded roster if office+date match the view
    if (bookOfficeId === selectedOfficeId && bookStartDate === selectedDate) {
      bookRoster = [...providerRoster];
      return;
    }
    bookRosterLoading = true;
    const res = await commands.getOfficeProviderSchedule(bookOfficeId, bookStartDate);
    bookRosterLoading = false;
    if (res.status === "ok") bookRoster = res.data;
  }

  function onBookProviderChange(providerId: string) {
    bookProviderId = providerId;
    // Auto-select first available slot for this provider
    const entry = bookRoster.find(e => e.provider_id === providerId);
    if (entry) {
      const slots = generateTimeSlots(entry.start_time, entry.end_time);
      bookStartTime = slots[0] ?? "";
    } else {
      bookStartTime = "";
    }
  }

  async function loadCallList() {
    if (!selectedOfficeId) return;
    const res = await commands.getTomorrowsCallList(selectedOfficeId, callListDate);
    if (res.status === "ok") {
      callList = res.data;
    }
  }

  async function toggleExpand(appointmentId: string) {
    if (expandedId === appointmentId) {
      expandedId = null;
      expandedDetail = null;
      return;
    }
    expandedId = appointmentId;
    detailLoading = true;
    const res = await commands.getAppointment(appointmentId);
    detailLoading = false;
    if (res.status === "ok") {
      expandedDetail = res.data;
    }
  }

  async function searchPatients() {
    if (bookPatientSearch.trim().length < 2) {
      patientSearchResults = [];
      return;
    }
    const res = await commands.searchPatients(bookPatientSearch, null, null, false);
    if (res.status === "ok") {
      patientSearchResults = res.data.map((p) => ({
        patient_id: p.patient_id,
        patient_name: p.full_name_display,
        first_name: p.first_name,
        last_name: p.last_name,
        phone: p.phone,
      }));
    }
  }

  function selectPatient(p: typeof patientSearchResults[0]) {
    bookPatientId = p.patient_id;
    bookPatientName = p.patient_name;
    bookPatientSearch = p.patient_name;
    patientSearchResults = [];
  }

  // ── Actions ───────────────────────────────────────────────────────────────

  async function doBookAppointment() {
    if (!bookPatientId) { bookError = "Select a patient from the search results."; return; }
    if (!bookProviderId) { bookError = "Select a provider."; return; }
    if (!bookProcedureId) { bookError = "Select a procedure."; return; }
    if (!bookOfficeId) { bookError = "Select an office."; return; }
    bookLoading = true;
    bookError = "";
    bookSuccess = "";

    const startTime = buildStartTime(bookStartDate, bookStartTime);
    const res = await commands.bookAppointment(
      bookOfficeId, bookPatientId, bookProcedureId, bookProviderId,
      startTime, null, STAFF_ID,
    );
    bookLoading = false;
    if (res.status === "ok") {
      bookSuccess = `Appointment booked for ${bookPatientName}.`;
      bookPatientId = "";
      bookPatientName = "";
      bookPatientSearch = "";
      bookProviderId = "";
      bookProcedureId = "";
      showBookForm = false;
      await loadSchedule();
    } else {
      bookError = getErrorMessage(res.error);
    }
  }

  async function doCancel(appointmentId: string) {
    const reason = prompt("Cancel reason (optional):");
    const res = await commands.cancelAppointment(appointmentId, STAFF_ID, reason ?? null);
    if (res.status === "ok") {
      expandedId = null;
      expandedDetail = null;
      await loadSchedule();
    } else {
      alert(getErrorMessage(res.error));
    }
  }

  async function doComplete(appointmentId: string) {
    const res = await commands.completeAppointment(appointmentId, STAFF_ID);
    if (res.status === "ok") {
      expandedId = null;
      expandedDetail = null;
      await loadSchedule();
    } else {
      alert(getErrorMessage(res.error));
    }
  }

  async function doNoShow(appointmentId: string) {
    const res = await commands.markAppointmentNoShow(appointmentId, STAFF_ID);
    if (res.status === "ok") {
      expandedId = null;
      expandedDetail = null;
      await loadSchedule();
    } else {
      alert(getErrorMessage(res.error));
    }
  }

  async function doAddNote() {
    if (!noteText.trim()) { noteError = "Note text is required."; return; }
    noteLoading = true;
    noteError = "";
    const res = await commands.addAppointmentNote(noteAppointmentId, noteText, STAFF_ID);
    noteLoading = false;
    if (res.status === "ok") {
      noteText = "";
      noteAppointmentId = "";
      // Refresh detail
      if (expandedId) {
        const dr = await commands.getAppointment(expandedId);
        if (dr.status === "ok") expandedDetail = dr.data;
      }
    } else {
      noteError = getErrorMessage(res.error);
    }
  }

  // ── Init ──────────────────────────────────────────────────────────────────

  import { onMount } from "svelte";
  onMount(async () => {
    await loadSetupData();
    await Promise.all([loadSchedule(), loadProviderRoster()]);
  });

  // Reload schedule and roster when office or date changes
  $effect(() => {
    if (selectedOfficeId && selectedDate) {
      loadSchedule();
      loadProviderRoster();
    }
  });

  // Reload booking roster when booking office or date changes (after form is open)
  $effect(() => {
    if (showBookForm && bookOfficeId && bookStartDate) {
      loadBookRoster();
    }
  });
</script>

<div class="page-wrap">
  {#if offices.length === 0}
    <div class="no-offices-banner">
      No offices configured. Go to <a href="/setup">Setup → Offices</a> to add an office before scheduling.
    </div>
  {/if}

  <div class="page-header">
    <h1>Schedule</h1>
    <div class="header-controls">
      <label for="office-select" class="sr-only">Select office</label>
      <select id="office-select" bind:value={selectedOfficeId} class="select-sm">
        {#each offices as o}
          <option value={o.id}>{o.name}</option>
        {/each}
      </select>
      <label for="schedule-date" class="sr-only">Select date</label>
      <input id="schedule-date" type="date" bind:value={selectedDate} class="date-input" />
      <button class="btn-primary" onclick={() => {
        if (!showBookForm) {
          bookOfficeId = selectedOfficeId;
          bookStartDate = selectedDate;
          bookRoster = [...providerRoster];
          bookProviderId = "";
          bookStartTime = "";
        }
        showBookForm = !showBookForm;
        bookSuccess = "";
      }}>
        {showBookForm ? "Close" : "+ Book Appointment"}
      </button>
      <button class="btn-secondary" onclick={() => { showCallList = !showCallList; loadCallList(); }}>
        {showCallList ? "Hide" : "Tomorrow's Call List"}
      </button>
    </div>
  </div>

  <!-- Provider roster -->
  {#if selectedOfficeId}
    <div class="roster-bar">
      <span class="roster-label">Providers today:</span>
      {#if rosterLoading}
        <span class="muted">Loading…</span>
      {:else if providerRoster.length === 0}
        <span class="muted">None scheduled</span>
      {:else}
        {#each providerRoster as entry}
          <span class="roster-chip">{entry.provider_name} <span class="roster-hours">{entry.start_time}–{entry.end_time}</span></span>
        {/each}
      {/if}
    </div>
  {/if}

  <!-- Book appointment form -->
  {#if showBookForm}
    <div class="card book-form">
      <h2>Book Appointment</h2>

      <!-- Step 1: Date + Office (determines who's available) -->
      <div class="form-step">
        <div class="step-label">When &amp; Where</div>
        <div class="step-fields">
          <div class="field-group">
            <label for="book-date">Date</label>
            <input id="book-date" type="date" bind:value={bookStartDate} class="input-full" />
          </div>
          <div class="field-group">
            <label for="book-office">Office</label>
            <select id="book-office" bind:value={bookOfficeId} class="select-full">
              {#each offices as o}<option value={o.id}>{o.name}</option>{/each}
            </select>
          </div>
        </div>
      </div>

      <!-- Step 2: Provider (filtered to who's actually working that day) -->
      <div class="form-step">
        <div class="step-label">Provider</div>
        {#if bookRosterLoading}
          <p class="muted">Checking who's working on {bookStartDate}…</p>
        {:else if bookRoster.length === 0}
          <div class="no-avail-warning">
            No providers scheduled on {bookStartDate} at {offices.find(o => o.id === bookOfficeId)?.name ?? "this office"}.
            Set availability in <a href="/setup">Setup → Providers</a>.
          </div>
        {:else}
          <div class="provider-chips">
            {#each bookRoster as entry}
              <button
                class="provider-chip"
                class:selected={bookProviderId === entry.provider_id}
                onclick={() => onBookProviderChange(entry.provider_id)}
              >
                <span class="chip-name">{entry.provider_name}</span>
                <span class="chip-hours">{entry.start_time}–{entry.end_time}</span>
              </button>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Step 3: Time slot (derived from provider's hours) -->
      {#if bookProviderId && availableSlots.length > 0}
        <div class="form-step">
          <div class="step-label">Time Slot</div>
          <div class="slot-grid">
            {#each availableSlots as slot}
              <button
                class="slot-btn"
                class:selected={bookStartTime === slot}
                onclick={() => (bookStartTime = slot)}
              >{slot}</button>
            {/each}
          </div>
        </div>
      {/if}

      <!-- Step 4: Patient -->
      {#if bookStartTime}
        <div class="form-step">
          <div class="step-label">Patient</div>
          <label for="book-patient" class="sr-only">Search patient by name</label>
          <div class="patient-search-wrap">
            <input
              id="book-patient"
              type="text"
              bind:value={bookPatientSearch}
              placeholder="Type name to search…"
              oninput={searchPatients}
              class="input-full"
              autocomplete="off"
            />
            {#if patientSearchResults.length > 0}
              <ul class="patient-dropdown" role="listbox" aria-label="Patient search results">
                {#each patientSearchResults as p}
                  <li role="option" aria-selected={bookPatientId === p.patient_id}>
                    <button onclick={() => selectPatient(p)} class="dropdown-item">
                      {p.patient_name}
                      {#if p.phone}<span class="muted"> · {p.phone}</span>{/if}
                    </button>
                  </li>
                {/each}
              </ul>
            {:else if bookPatientSearch.trim().length >= 2 && !bookPatientId}
              <p class="search-no-results">No patients found. <a href="/patients">Register patient first</a>.</p>
            {/if}
            {#if bookPatientName && bookPatientId}
              <div class="selected-patient">✓ <strong>{bookPatientName}</strong></div>
            {/if}
          </div>
        </div>
      {/if}

      <!-- Step 5: Procedure -->
      {#if bookPatientId}
        <div class="form-step">
          <div class="step-label">Procedure</div>
          {#if procedures.length === 0}
            <p class="field-hint">No procedures set up. Go to <a href="/setup">Setup → Procedure Types</a>.</p>
          {:else}
            <label for="book-procedure" class="sr-only">Select procedure</label>
            <select id="book-procedure" bind:value={bookProcedureId} class="select-full">
              <option value="">— Select procedure —</option>
              {#each procedures as p}
                <option value={p.id}>{p.name} ({p.default_duration_minutes} min)</option>
              {/each}
            </select>
          {/if}
        </div>
      {/if}

      {#if bookError}
        <div class="error-msg" role="alert">{bookError}</div>
      {/if}
      {#if bookSuccess}
        <div class="success-msg" role="status">{bookSuccess}</div>
      {/if}

      <div class="form-actions">
        <button class="btn-primary" onclick={doBookAppointment}
          disabled={bookLoading || !bookPatientId || !bookProviderId || !bookProcedureId || !bookStartTime}>
          {bookLoading ? "Booking…" : "Book Appointment"}
        </button>
        <button class="btn-ghost" onclick={() => showBookForm = false}>Cancel</button>
      </div>
    </div>
  {/if}

  <!-- Tomorrow's call list -->
  {#if showCallList}
    <div class="card call-list">
      <div class="call-list-header">
        <h2>Call List</h2>
        <input type="date" bind:value={callListDate} onchange={loadCallList} class="date-input" />
      </div>
      {#if callList.length === 0}
        <p class="muted">No Booked appointments for this date.</p>
      {:else}
        <table class="call-table">
          <thead>
            <tr>
              <th>Time</th>
              <th>Patient</th>
              <th>Phone</th>
              <th>Contact Pref.</th>
              <th>Procedure</th>
              <th>Provider</th>
            </tr>
          </thead>
          <tbody>
            {#each callList as e}
              <tr>
                <td>{formatTime(e.start_time)}</td>
                <td>{e.patient_name}</td>
                <td>{e.patient_phone ?? "—"}</td>
                <td>{e.preferred_contact_channel ?? "—"}</td>
                <td>{e.procedure_name}</td>
                <td>{e.provider_name}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </div>
  {/if}

  <!-- Schedule error -->
  {#if scheduleError}
    <div class="error-msg">{scheduleError}</div>
  {/if}

  <!-- Schedule list -->
  {#if scheduleLoading}
    <p class="muted">Loading…</p>
  {:else if schedule.length === 0}
    <div class="empty-schedule">
      <p>No appointments for {selectedDate}{selectedOfficeId ? ` at ${offices.find((o) => o.id === selectedOfficeId)?.name ?? "selected office"}` : ""}.</p>
      <p>Click <strong>+ Book Appointment</strong> to schedule one.</p>
    </div>
  {:else}
    <div class="schedule-list">
      {#each schedule as appt}
        <div class="appt-card" class:expanded={expandedId === appt.appointment_id}>
          <!-- Header row -->
          <button
            class="appt-header"
            aria-expanded={expandedId === appt.appointment_id}
            onclick={() => toggleExpand(appt.appointment_id)}
          >
            <span class="appt-time">{formatTime(appt.start_time)}–{formatTime(appt.end_time)}</span>
            <span class="appt-patient">{appt.patient_name}</span>
            <span class="appt-procedure">{appt.procedure_name}</span>
            <span class="appt-provider">{appt.provider_name}</span>
            <span class="badge {statusBadgeClass(appt.status)}">{appt.status}</span>
            <span class="expand-icon">{expandedId === appt.appointment_id ? "▲" : "▼"}</span>
          </button>

          <!-- Expanded detail -->
          {#if expandedId === appt.appointment_id}
            {#if detailLoading}
              <div class="detail-body"><p class="muted">Loading…</p></div>
            {:else if expandedDetail}
              <div class="detail-body">
                <div class="detail-meta">
                  <div><span class="label">Duration:</span> {appt.duration_minutes} min</div>
                  <div><span class="label">Booked by:</span> {appt.booked_by}</div>
                  {#if appt.rescheduled_from_id}
                    <div><span class="label">Rescheduled from:</span> {appt.rescheduled_from_id.slice(0, 8)}…</div>
                  {/if}
                  {#if appt.rescheduled_to_id}
                    <div><span class="label">Rescheduled to:</span> {appt.rescheduled_to_id.slice(0, 8)}…</div>
                  {/if}
                </div>

                <!-- Notes -->
                <div class="notes-section">
                  <h4>Notes ({expandedDetail.notes.length})</h4>
                  {#if expandedDetail.notes.length === 0}
                    <p class="muted">No notes.</p>
                  {:else}
                    <ul class="notes-list">
                      {#each expandedDetail.notes as note}
                        <li>
                          <span class="note-meta">{formatDate(note.recorded_at)} {formatTime(note.recorded_at)} · {note.recorded_by}</span>
                          <p class="note-text">{note.text}</p>
                        </li>
                      {/each}
                    </ul>
                  {/if}

                  <!-- Add note inline -->
                  <div class="add-note-form">
                    <textarea
                      placeholder="Add a note…"
                      bind:value={noteText}
                      onfocus={() => { noteAppointmentId = appt.appointment_id; noteError = ""; }}
                      rows={2}
                      class="note-input"
                    ></textarea>
                    {#if noteError && noteAppointmentId === appt.appointment_id}
                      <div class="error-msg">{noteError}</div>
                    {/if}
                    <button
                      class="btn-sm"
                      onclick={doAddNote}
                      disabled={noteLoading || noteAppointmentId !== appt.appointment_id}
                    >
                      {noteLoading ? "Saving…" : "Add Note"}
                    </button>
                  </div>
                </div>

                <!-- Status actions (only Booked can transition) -->
                {#if appt.status === "Booked"}
                  <div class="appt-actions">
                    <button class="btn-success btn-sm" onclick={() => doComplete(appt.appointment_id)}>
                      Mark Complete
                    </button>
                    <button class="btn-warning btn-sm" onclick={() => doNoShow(appt.appointment_id)}>
                      No-Show
                    </button>
                    <button class="btn-danger btn-sm" onclick={() => doCancel(appt.appointment_id)}>
                      Cancel
                    </button>
                  </div>
                {/if}
              </div>
            {/if}
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .page-wrap {
    padding: 1.5rem 2rem;
    font-family: system-ui, sans-serif;
    color: #e0e0e0;
    background: #0f0f1a;
    min-height: 100vh;
  }
  .page-header {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 1.25rem;
    flex-wrap: wrap;
  }
  h1 { font-size: 1.4rem; font-weight: 600; color: #7eb8f7; margin: 0; }
  h2 { font-size: 1.1rem; font-weight: 600; color: #ccc; margin: 0 0 1rem; }
  h4 { font-size: 0.85rem; color: #aaa; margin: 0.75rem 0 0.4rem; }

  .header-controls {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
    align-items: center;
    margin-left: auto;
  }

  .card {
    background: #1a1a2e;
    border: 1px solid #333;
    border-radius: 8px;
    padding: 1.25rem;
    margin-bottom: 1rem;
  }


  /* Stepped booking form */
  .form-step {
    margin-bottom: 1.1rem;
    padding-bottom: 1.1rem;
    border-bottom: 1px solid #2a2a40;
  }
  .form-step:last-of-type { border-bottom: none; margin-bottom: 0; padding-bottom: 0; }
  .step-label {
    font-size: 0.72rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: #7eb8f7;
    margin-bottom: 0.55rem;
  }
  .step-fields { display: flex; gap: 1rem; flex-wrap: wrap; }
  .field-group { display: flex; flex-direction: column; gap: 0.3rem; flex: 1; min-width: 140px; }
  .field-group label { font-size: 0.78rem; color: #aaa; }

  /* Provider chips */
  .provider-chips { display: flex; gap: 0.5rem; flex-wrap: wrap; }
  .provider-chip {
    display: flex; flex-direction: column; gap: 0.1rem;
    padding: 0.4rem 0.75rem;
    background: #12122a; border: 1px solid #333; border-radius: 6px;
    cursor: pointer; text-align: left; color: #bbb;
    transition: border-color 0.1s, background 0.1s;
  }
  .provider-chip:hover { border-color: #4a7abc; background: #1a1a3a; }
  .provider-chip.selected { border-color: #7eb8f7; background: #1a2a4a; color: #fff; }
  .chip-name { font-size: 0.875rem; font-weight: 600; }
  .chip-hours { font-size: 0.75rem; color: #7eb8f7; font-family: monospace; }
  .provider-chip.selected .chip-hours { color: #a0d4ff; }

  /* Time slot grid */
  .slot-grid {
    display: flex; flex-wrap: wrap; gap: 0.35rem;
  }
  .slot-btn {
    padding: 0.3rem 0.55rem;
    font-size: 0.8rem; font-family: monospace;
    background: #12122a; border: 1px solid #2a2a40; border-radius: 4px;
    color: #bbb; cursor: pointer;
  }
  .slot-btn:hover { border-color: #4a7abc; color: #e0e0e0; }
  .slot-btn.selected { background: #1a3a6b; border-color: #7eb8f7; color: #7eb8f7; font-weight: 600; }

  /* No availability warning */
  .no-avail-warning {
    background: #2a1a0a; border: 1px solid #6a3a0a; border-radius: 5px;
    padding: 0.6rem 0.9rem; font-size: 0.82rem; color: #f7a87e;
  }
  .no-avail-warning a { color: #f7c87e; }

  .patient-search-wrap { position: relative; }
  .patient-dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    background: #252540;
    border: 1px solid #444;
    border-radius: 4px;
    list-style: none;
    margin: 2px 0 0;
    padding: 0;
    z-index: 10;
    max-height: 180px;
    overflow-y: auto;
  }
  .dropdown-item {
    width: 100%;
    text-align: left;
    padding: 0.4rem 0.75rem;
    background: none;
    border: none;
    color: #e0e0e0;
    cursor: pointer;
    font-size: 0.85rem;
  }
  .dropdown-item:hover { background: #333; }
  .selected-patient { font-size: 0.8rem; color: #7eb8f7; margin-top: 0.25rem; }

  /* Schedule list */
  .schedule-list { display: flex; flex-direction: column; gap: 0.5rem; }

  .appt-card {
    background: #1a1a2e;
    border: 1px solid #333;
    border-radius: 6px;
    overflow: hidden;
  }
  .appt-card.expanded { border-color: #555; }

  .appt-header {
    display: grid;
    grid-template-columns: 100px 1fr 1fr 1fr 90px 24px;
    gap: 0.75rem;
    align-items: center;
    padding: 0.6rem 1rem;
    background: none;
    border: none;
    color: #e0e0e0;
    cursor: pointer;
    text-align: left;
    width: 100%;
    font-size: 0.875rem;
  }
  .appt-header:hover { background: rgba(255,255,255,0.04); }

  .appt-time { font-weight: 600; color: #7eb8f7; font-family: monospace; }
  .appt-patient { font-weight: 500; }
  .appt-procedure { color: #bbb; }
  .appt-provider { color: #aaa; font-size: 0.82rem; }
  .expand-icon { color: #666; font-size: 0.75rem; justify-self: center; }

  .detail-body {
    padding: 0.75rem 1rem 1rem;
    border-top: 1px solid #2a2a40;
  }
  .detail-meta {
    display: flex;
    gap: 1.5rem;
    font-size: 0.82rem;
    color: #aaa;
    margin-bottom: 0.75rem;
    flex-wrap: wrap;
  }
  .detail-meta .label { color: #777; }

  .notes-section { margin-top: 0.5rem; }
  .notes-list { list-style: none; padding: 0; margin: 0 0 0.75rem; display: flex; flex-direction: column; gap: 0.5rem; }
  .notes-list li { background: #12122a; border-radius: 4px; padding: 0.5rem 0.75rem; }
  .note-meta { font-size: 0.75rem; color: #777; }
  .note-text { margin: 0.2rem 0 0; font-size: 0.85rem; }

  .add-note-form { display: flex; flex-direction: column; gap: 0.35rem; }
  .note-input {
    background: #12122a;
    border: 1px solid #333;
    border-radius: 4px;
    color: #e0e0e0;
    padding: 0.4rem 0.6rem;
    font-size: 0.85rem;
    resize: vertical;
    font-family: system-ui, sans-serif;
  }

  .appt-actions {
    display: flex;
    gap: 0.5rem;
    margin-top: 0.75rem;
    flex-wrap: wrap;
  }

  /* Call list */
  .call-list-header { display: flex; gap: 1rem; align-items: center; margin-bottom: 0.75rem; }
  .call-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.85rem;
  }
  .call-table th { color: #777; font-weight: 500; text-align: left; padding: 0.4rem 0.75rem; border-bottom: 1px solid #2a2a40; }
  .call-table td { padding: 0.4rem 0.75rem; border-bottom: 1px solid #1e1e35; }

  /* Badges */
  .badge { padding: 0.15rem 0.5rem; border-radius: 10px; font-size: 0.75rem; font-weight: 600; text-align: center; }
  .badge-booked    { background: #1a3a6b; color: #7eb8f7; }
  .badge-completed { background: #1a3a2a; color: #6bcf7f; }
  .badge-cancelled { background: #3a1a1a; color: #f77e7e; }
  .badge-noshow    { background: #3a2a1a; color: #f7a87e; }
  .badge-rescheduled { background: #2a2a3a; color: #bbb; }

  /* Inputs */
  .select-sm, .date-input {
    background: #1a1a2e;
    border: 1px solid #444;
    border-radius: 4px;
    color: #e0e0e0;
    padding: 0.35rem 0.6rem;
    font-size: 0.85rem;
  }
  .select-full, .input-full {
    background: #12122a;
    border: 1px solid #333;
    border-radius: 4px;
    color: #e0e0e0;
    padding: 0.35rem 0.6rem;
    font-size: 0.85rem;
    width: 100%;
    box-sizing: border-box;
  }

  /* Buttons */
  .btn-primary {
    background: #2a5cad;
    color: #fff;
    border: none;
    border-radius: 4px;
    padding: 0.4rem 1rem;
    cursor: pointer;
    font-size: 0.85rem;
  }
  .btn-primary:hover { background: #3a6cc0; }
  .btn-primary:disabled { opacity: 0.5; cursor: default; }

  .btn-secondary {
    background: #252540;
    color: #bbb;
    border: 1px solid #444;
    border-radius: 4px;
    padding: 0.4rem 0.75rem;
    cursor: pointer;
    font-size: 0.85rem;
  }
  .btn-secondary:hover { color: #fff; }

  .btn-ghost {
    background: none;
    color: #aaa;
    border: 1px solid #444;
    border-radius: 4px;
    padding: 0.4rem 0.75rem;
    cursor: pointer;
    font-size: 0.85rem;
  }

  .btn-sm {
    background: #252540;
    color: #bbb;
    border: 1px solid #444;
    border-radius: 4px;
    padding: 0.3rem 0.6rem;
    cursor: pointer;
    font-size: 0.8rem;
  }
  .btn-sm:disabled { opacity: 0.5; cursor: default; }

  .btn-success { background: #1a3a2a; color: #6bcf7f; border: 1px solid #2a5a3a; }
  .btn-success:hover { background: #2a4a3a; }
  .btn-warning { background: #3a2a1a; color: #f7a87e; border: 1px solid #5a3a2a; }
  .btn-warning:hover { background: #4a3a2a; }
  .btn-danger  { background: #3a1a1a; color: #f77e7e; border: 1px solid #5a2a2a; }
  .btn-danger:hover  { background: #4a2a2a; }

  .form-actions { display: flex; gap: 0.5rem; margin-top: 0.5rem; }

  .error-msg  { color: #f77e7e; font-size: 0.82rem; margin: 0.4rem 0; }
  .success-msg { color: #6bcf7f; font-size: 0.82rem; margin: 0.4rem 0; }
  .muted { color: #666; font-size: 0.85rem; }
  .empty-schedule { padding: 2rem; text-align: center; color: #555; }
  .no-offices-banner {
    background: #2a1a0a; border: 1px solid #6a3a0a; color: #f7a87e;
    border-radius: 6px; padding: 0.75rem 1rem; margin-bottom: 0.75rem; font-size: 0.875rem;
  }
  .no-offices-banner a { color: #f7c87e; }
  .search-no-results { font-size: 0.8rem; color: #888; margin: 0.25rem 0 0; }
  .search-no-results a { color: #7eb8f7; }
  .field-hint { font-size: 0.78rem; color: #888; margin: 0; }
  .field-hint a { color: #7eb8f7; }
  .sr-only { position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px; overflow: hidden; clip: rect(0,0,0,0); white-space: nowrap; border: 0; }

  /* Provider roster bar */
  .roster-bar {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
    padding: 0.45rem 0.75rem;
    background: #12122a;
    border: 1px solid #2a2a40;
    border-radius: 6px;
    margin-bottom: 0.75rem;
    font-size: 0.82rem;
  }
  .roster-label { color: #777; white-space: nowrap; }
  .roster-chip {
    background: #1a2a3a;
    border: 1px solid #2a4a6a;
    border-radius: 12px;
    padding: 0.15rem 0.6rem;
    color: #7eb8f7;
    white-space: nowrap;
  }
  .roster-hours { color: #4a8ac0; font-size: 0.78rem; }
</style>
