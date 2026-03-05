<script lang="ts">
  import { commands } from "$lib/bindings";
  import type {
    AppointmentDto, AppointmentWithNotesDto, CallListEntryDto,
    ProviderScheduleEntry, OfficeDto, ProviderDto,
  } from "$lib/bindings";
  import { getErrorMessage } from "$lib/utils/api";
  import { toast } from "$lib/stores/toast";
  import { confirm } from "$lib/stores/confirm";
  import { onMount } from "svelte";

  const STAFF_ID = "staff-system";
  const SLOT_HEIGHT = 30; // px per 15-min slot

  // ── Data ──────────────────────────────────────────────────────────────────

  let offices = $state<OfficeDto[]>([]);
  let allProviders = $state<ProviderDto[]>([]);
  let procedures = $state<{ id: string; name: string; default_duration_minutes: number; is_active: boolean }[]>([]);

  // ── Grid view ─────────────────────────────────────────────────────────────

  let selectedOfficeId = $state("");
  let selectedDate = $state(todayLocal());
  let schedule = $state<AppointmentDto[]>([]);
  let providerRoster = $state<ProviderScheduleEntry[]>([]);
  let scheduleLoading = $state(false);

  // ── Detail drawer ─────────────────────────────────────────────────────────

  let detailApptId = $state<string | null>(null);
  let detailData = $state<AppointmentWithNotesDto | null>(null);
  let detailLoading = $state(false);
  let showCancelConfirm = $state(false);
  let cancelReason = $state("");

  // ── Book drawer ───────────────────────────────────────────────────────────

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
  let patientSearchResults = $state<{ patient_id: string; patient_name: string; first_name: string; last_name: string; phone: string | null }[]>([]);
  let bookRoster = $state<ProviderScheduleEntry[]>([]);
  let bookRosterLoading = $state(false);

  // ── Notes (in detail drawer) ──────────────────────────────────────────────

  let noteText = $state("");
  let noteError = $state("");
  let noteLoading = $state(false);

  // ── Call list ─────────────────────────────────────────────────────────────

  let showCallList = $state(false);
  let callList = $state<CallListEntryDto[]>([]);
  let callListDate = $state(tomorrowLocal());

  // ── Derived grid values ───────────────────────────────────────────────────

  let currentOffice = $derived(offices.find((o) => o.id === selectedOfficeId) ?? null);
  let dayName = $derived(getDayName(selectedDate));
  let officeHoursEntry = $derived(
    currentOffice?.hours.find((h) => h.day_of_week === dayName) ?? null,
  );
  let openMins = $derived(officeHoursEntry ? parseHHMM(officeHoursEntry.open_time) : 480);
  let closeMins = $derived(officeHoursEntry ? parseHHMM(officeHoursEntry.close_time) : 1020);
  let gridHeight = $derived(Math.max(0, ((closeMins - openMins) / 15) * SLOT_HEIGHT));

  let officeProviders = $derived(
    allProviders
      .filter((p) => !p.archived && p.office_ids.includes(selectedOfficeId))
      .sort((a, b) => a.name.localeCompare(b.name)),
  );

  let timeTicks = $derived(
    (() => {
      if (!officeHoursEntry) return [];
      const ticks: { label: string; top: number }[] = [];
      for (let m = openMins; m <= closeMins; m += 60) {
        ticks.push({ label: minsTo12h(m), top: ((m - openMins) / 15) * SLOT_HEIGHT });
      }
      return ticks;
    })(),
  );

  let availableSlots = $derived(
    (() => {
      const entry = bookRoster.find((e) => e.provider_id === bookProviderId);
      if (!entry) return [];
      return generateTimeSlots(entry.start_time, entry.end_time);
    })(),
  );

  let isToday = $derived(selectedDate === todayLocal());

  // ── Helpers ───────────────────────────────────────────────────────────────

  function todayLocal(): string {
    return new Date().toISOString().slice(0, 10);
  }

  function tomorrowLocal(): string {
    const d = new Date();
    d.setDate(d.getDate() + 1);
    return d.toISOString().slice(0, 10);
  }

  function addDays(date: string, n: number): string {
    const d = new Date(date + "T12:00:00");
    d.setDate(d.getDate() + n);
    return d.toISOString().slice(0, 10);
  }

  function getDayName(date: string): string {
    return new Date(date + "T12:00:00").toLocaleDateString("en-US", { weekday: "long" });
  }

  function formatDisplayDate(date: string): string {
    return new Date(date + "T12:00:00").toLocaleDateString("en-JM", {
      weekday: "long",
      day: "numeric",
      month: "short",
      year: "numeric",
    });
  }

  function parseHHMM(t: string): number {
    const [h, m] = t.split(":").map(Number);
    return h * 60 + m;
  }

  function minsToHHMM(m: number): string {
    return `${Math.floor(m / 60).toString().padStart(2, "0")}:${(m % 60).toString().padStart(2, "0")}`;
  }

  function minsTo12h(m: number): string {
    const h = Math.floor(m / 60);
    const min = m % 60;
    const period = h >= 12 ? "PM" : "AM";
    const h12 = h % 12 || 12;
    return `${h12}:${min.toString().padStart(2, "0")} ${period}`;
  }

  function formatTime(isoLocal: string): string {
    const [h, m] = isoLocal.slice(11, 16).split(":").map(Number);
    const period = h >= 12 ? "PM" : "AM";
    const h12 = h % 12 || 12;
    return `${h12}:${m.toString().padStart(2, "0")} ${period}`;
  }

  function formatDate(isoLocal: string): string {
    return new Date(isoLocal.slice(0, 10) + "T12:00:00").toLocaleDateString("en-JM", {
      day: "numeric", month: "short", year: "numeric",
    });
  }

  function buildStartTime(date: string, time: string): string {
    return `${date}T${time}:00`;
  }

  function generateTimeSlots(start: string, end: string): string[] {
    const slots: string[] = [];
    let [h, m] = start.split(":").map(Number);
    const [eh, em] = end.split(":").map(Number);
    while (h * 60 + m < eh * 60 + em) {
      slots.push(`${h.toString().padStart(2, "0")}:${m.toString().padStart(2, "0")}`);
      m += 15;
      if (m >= 60) { h++; m -= 60; }
    }
    return slots;
  }

  function statusBadgeClass(status: string): string {
    const map: Record<string, string> = {
      Booked: "badge-booked", Completed: "badge-completed",
      Cancelled: "badge-cancelled", NoShow: "badge-noshow", Rescheduled: "badge-rescheduled",
    };
    return map[status] ?? "badge-booked";
  }

  function statusBlockClass(status: string): string {
    const map: Record<string, string> = {
      Booked: "appt-booked", Completed: "appt-completed",
      Cancelled: "appt-cancelled", NoShow: "appt-noshow", Rescheduled: "appt-rescheduled",
    };
    return map[status] ?? "appt-booked";
  }

  // ── Data loading ──────────────────────────────────────────────────────────

  async function loadSetupData() {
    const [officeRes, procRes, provRes] = await Promise.all([
      commands.listOffices(),
      commands.listProcedureTypes(),
      commands.listProviders(),
    ]);
    if (officeRes.status === "ok") {
      offices = officeRes.data.filter((o) => !o.archived);
      if (!selectedOfficeId && offices.length > 0) {
        selectedOfficeId = offices[0].id;
        bookOfficeId = offices[0].id;
      }
    }
    if (procRes.status === "ok") procedures = procRes.data.filter((p) => p.is_active);
    if (provRes.status === "ok") allProviders = provRes.data;
  }

  async function loadGrid() {
    if (!selectedOfficeId) return;
    scheduleLoading = true;
    const [schedRes, rosterRes] = await Promise.all([
      commands.getSchedule(selectedOfficeId, selectedDate),
      commands.getOfficeProviderSchedule(selectedOfficeId, selectedDate),
    ]);
    scheduleLoading = false;
    if (schedRes.status === "ok") schedule = schedRes.data;
    else toast.error(getErrorMessage(schedRes.error));
    if (rosterRes.status === "ok") providerRoster = rosterRes.data;
  }

  async function loadBookRoster() {
    if (!bookOfficeId || !bookStartDate) return;
    if (bookOfficeId === selectedOfficeId && bookStartDate === selectedDate) {
      bookRoster = [...providerRoster];
      return;
    }
    bookRosterLoading = true;
    const res = await commands.getOfficeProviderSchedule(bookOfficeId, bookStartDate);
    bookRosterLoading = false;
    if (res.status === "ok") bookRoster = res.data;
  }

  async function loadCallList() {
    if (!selectedOfficeId) return;
    const res = await commands.getTomorrowsCallList(selectedOfficeId, callListDate);
    if (res.status === "ok") callList = res.data;
  }

  // ── Detail drawer ─────────────────────────────────────────────────────────

  async function openDetail(apptId: string) {
    showBookForm = false;
    detailApptId = apptId;
    detailLoading = true;
    detailData = null;
    showCancelConfirm = false;
    cancelReason = "";
    noteText = "";
    noteError = "";
    const res = await commands.getAppointment(apptId);
    detailLoading = false;
    if (res.status === "ok") detailData = res.data;
  }

  function closeDetail() {
    detailApptId = null;
    detailData = null;
    showCancelConfirm = false;
    cancelReason = "";
    noteText = "";
    noteError = "";
  }

  // ── Book drawer ───────────────────────────────────────────────────────────

  function openBookDrawer(providerId = "", startTime = "") {
    closeDetail();
    bookOfficeId = selectedOfficeId;
    bookStartDate = selectedDate;
    bookRoster = [...providerRoster];
    bookProviderId = providerId;
    bookStartTime = startTime;
    bookPatientId = "";
    bookPatientName = "";
    bookPatientSearch = "";
    bookProcedureId = "";
    bookError = "";
    showBookForm = true;
  }

  // ── Grid column click → pre-fill booking drawer ───────────────────────────

  function onColumnClick(e: MouseEvent, providerId: string) {
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const slotIndex = Math.floor((e.clientY - rect.top) / SLOT_HEIGHT);
    const mins = openMins + slotIndex * 15;
    if (mins >= closeMins) return;
    openBookDrawer(providerId, minsToHHMM(mins));
  }

  // ── Patient search ────────────────────────────────────────────────────────

  async function searchPatients() {
    if (bookPatientSearch.trim().length < 2) { patientSearchResults = []; return; }
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

  function onBookProviderChange(providerId: string) {
    bookProviderId = providerId;
    const entry = bookRoster.find((e) => e.provider_id === providerId);
    if (entry) {
      bookStartTime = generateTimeSlots(entry.start_time, entry.end_time)[0] ?? "";
    } else {
      bookStartTime = "";
    }
  }

  // ── Actions ───────────────────────────────────────────────────────────────

  async function doBookAppointment() {
    if (!bookPatientId) { bookError = "Select a patient from the search results."; return; }
    if (!bookProviderId) { bookError = "Select a provider."; return; }
    if (!bookProcedureId) { bookError = "Select a procedure."; return; }
    if (!bookOfficeId) { bookError = "Select an office."; return; }
    bookLoading = true;
    bookError = "";
    const res = await commands.bookAppointment(
      bookOfficeId, bookPatientId, bookProcedureId, bookProviderId,
      buildStartTime(bookStartDate, bookStartTime), null, STAFF_ID,
    );
    bookLoading = false;
    if (res.status === "ok") {
      showBookForm = false;
      toast.success(`Appointment booked for ${bookPatientName}.`);
      await loadGrid();
    } else {
      bookError = getErrorMessage(res.error);
    }
  }

  async function doComplete(apptId: string) {
    const ok = await confirm({
      title: "Complete appointment",
      message: "Mark this appointment as completed?",
      confirmLabel: "Mark complete",
    });
    if (!ok) return;
    const res = await commands.completeAppointment(apptId, STAFF_ID);
    if (res.status === "ok") {
      toast.success("Appointment marked complete.");
      closeDetail();
      await loadGrid();
    } else {
      toast.error(getErrorMessage(res.error));
    }
  }

  async function doNoShow(apptId: string) {
    const ok = await confirm({
      title: "Mark no-show",
      message: "Mark this patient as a no-show?",
      confirmLabel: "Mark no-show",
      destructive: true,
    });
    if (!ok) return;
    const res = await commands.markAppointmentNoShow(apptId, STAFF_ID);
    if (res.status === "ok") {
      toast.success("Appointment marked no-show.");
      closeDetail();
      await loadGrid();
    } else {
      toast.error(getErrorMessage(res.error));
    }
  }

  async function doCancel(apptId: string) {
    const res = await commands.cancelAppointment(apptId, STAFF_ID, cancelReason.trim() || null);
    if (res.status === "ok") {
      toast.success("Appointment cancelled.");
      closeDetail();
      await loadGrid();
    } else {
      toast.error(getErrorMessage(res.error));
    }
  }

  async function doAddNote() {
    if (!detailApptId || !noteText.trim()) { noteError = "Note text is required."; return; }
    noteLoading = true;
    noteError = "";
    const res = await commands.addAppointmentNote(detailApptId, noteText, STAFF_ID);
    noteLoading = false;
    if (res.status === "ok") {
      noteText = "";
      const dr = await commands.getAppointment(detailApptId);
      if (dr.status === "ok") detailData = dr.data;
    } else {
      noteError = getErrorMessage(res.error);
    }
  }

  // ── Keyboard shortcuts ────────────────────────────────────────────────────

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      if (showBookForm) { showBookForm = false; return; }
      if (detailApptId) { closeDetail(); return; }
    }
  }

  // ── Init ──────────────────────────────────────────────────────────────────

  onMount(async () => {
    await loadSetupData();
    await loadGrid();
  });

  $effect(() => {
    if (selectedOfficeId && selectedDate) loadGrid();
  });

  $effect(() => {
    if (showBookForm && bookOfficeId && bookStartDate) loadBookRoster();
  });
</script>

<svelte:window onkeydown={onKeydown} />

<!-- ═══════════════════════════════════════════════════════
     BOOKING DRAWER
     ═══════════════════════════════════════════════════════ -->
{#if showBookForm}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="drawer-overlay" onclick={() => (showBookForm = false)} aria-hidden="true"></div>

  <div class="drawer" role="dialog" aria-modal="true" aria-labelledby="book-drawer-title">
    <div class="drawer-header">
      <h2 class="drawer-title" id="book-drawer-title">Book Appointment</h2>
      <button class="btn btn-ghost btn-icon btn-sm" onclick={() => (showBookForm = false)} aria-label="Close booking form">✕</button>
    </div>

    <div class="drawer-body">
      <!-- When & where -->
      <div class="book-section">
        <p class="book-section-label">When &amp; where</p>
        <div class="book-row">
          <div class="form-field" style="flex:1">
            <label class="field-label" for="book-date">Date</label>
            <input id="book-date" type="date" bind:value={bookStartDate} />
          </div>
          <div class="form-field" style="flex:1">
            <label class="field-label" for="book-office">Office</label>
            <select id="book-office" bind:value={bookOfficeId}>
              {#each offices as o}<option value={o.id}>{o.name}</option>{/each}
            </select>
          </div>
        </div>
      </div>

      <!-- Provider -->
      <div class="book-section">
        <p class="book-section-label">Provider</p>
        {#if bookRosterLoading}
          <div class="load-row"><div class="spinner spinner-sm"></div> Checking availability…</div>
        {:else if bookRoster.length === 0}
          <p class="field-hint">
            No providers scheduled on {bookStartDate} at this office.
            <a href="/setup">Set availability in Setup → Providers</a>.
          </p>
        {:else}
          <div class="chip-group">
            {#each bookRoster as entry}
              <button
                class="chip"
                class:chip-selected={bookProviderId === entry.provider_id}
                onclick={() => onBookProviderChange(entry.provider_id)}
              >
                <span class="chip-name">{entry.provider_name}</span>
                <span class="chip-hours">{entry.start_time}–{entry.end_time}</span>
              </button>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Time slot -->
      {#if bookProviderId && availableSlots.length > 0}
        <div class="book-section">
          <p class="book-section-label">Time</p>
          <div class="slot-grid">
            {#each availableSlots as slot}
              <button
                class="slot-btn"
                class:slot-selected={bookStartTime === slot}
                onclick={() => (bookStartTime = slot)}
              >{minsTo12h(parseHHMM(slot))}</button>
            {/each}
          </div>
        </div>
      {/if}

      <!-- Patient -->
      {#if bookStartTime}
        <div class="book-section">
          <p class="book-section-label">Patient</p>
          <div class="patient-search-wrap">
            <label class="sr-only" for="book-patient">Search patient by name</label>
            <input
              id="book-patient"
              type="text"
              bind:value={bookPatientSearch}
              placeholder="Type name to search…"
              oninput={searchPatients}
              autocomplete="off"
            />
            {#if patientSearchResults.length > 0}
              <ul class="patient-dropdown" role="listbox" aria-label="Patient search results">
                {#each patientSearchResults as p}
                  <li role="option" aria-selected={bookPatientId === p.patient_id}>
                    <button onclick={() => selectPatient(p)} class="dropdown-item">
                      {p.patient_name}
                      {#if p.phone}<span class="text-muted"> · {p.phone}</span>{/if}
                    </button>
                  </li>
                {/each}
              </ul>
            {:else if bookPatientSearch.trim().length >= 2 && !bookPatientId}
              <p class="field-hint">No patients found. <a href="/patients">Register patient first</a>.</p>
            {/if}
            {#if bookPatientId}
              <div class="selected-patient">
                <span class="check-icon">✓</span> <strong>{bookPatientName}</strong>
              </div>
            {/if}
          </div>
        </div>
      {/if}

      <!-- Procedure -->
      {#if bookPatientId}
        <div class="book-section">
          <p class="book-section-label">Procedure</p>
          {#if procedures.length === 0}
            <p class="field-hint">No procedures set up. <a href="/setup">Go to Setup → Procedure Types</a>.</p>
          {:else}
            <label class="sr-only" for="book-procedure">Select procedure</label>
            <select id="book-procedure" bind:value={bookProcedureId}>
              <option value="">— Select procedure —</option>
              {#each procedures as p}
                <option value={p.id}>{p.name} ({p.default_duration_minutes} min)</option>
              {/each}
            </select>
          {/if}
        </div>
      {/if}

      {#if bookError}
        <div class="field-error" role="alert">{bookError}</div>
      {/if}
    </div>

    <div class="drawer-footer">
      <button class="btn btn-ghost" onclick={() => (showBookForm = false)}>Cancel</button>
      <button
        class="btn btn-primary"
        onclick={doBookAppointment}
        disabled={bookLoading || !bookPatientId || !bookProviderId || !bookProcedureId || !bookStartTime}
      >
        {#if bookLoading}<span class="spinner" aria-hidden="true"></span><span class="sr-only">Booking</span>{:else}Book appointment{/if}
      </button>
    </div>
  </div>
{/if}

<!-- ═══════════════════════════════════════════════════════
     DETAIL DRAWER
     ═══════════════════════════════════════════════════════ -->
{#if detailApptId !== null}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="drawer-overlay" onclick={closeDetail} aria-hidden="true"></div>

  <div class="drawer" role="dialog" aria-modal="true" aria-labelledby="detail-drawer-title">
    <div class="drawer-header">
      <h2 class="drawer-title" id="detail-drawer-title">Appointment</h2>
      <button class="btn btn-ghost btn-icon btn-sm" onclick={closeDetail} aria-label="Close detail">✕</button>
    </div>

    <div class="drawer-body">
      {#if detailLoading}
        <div class="load-row" style="justify-content:center; padding: 2rem 0;">
          <div class="spinner"></div>
        </div>
      {:else if detailData}
        {@const appt = detailData.appointment}

        <!-- Key info -->
        <dl class="detail-dl">
          <dt>Patient</dt>
          <dd><strong>{appt.patient_name}</strong></dd>
          <dt>Procedure</dt>
          <dd>{appt.procedure_name} · {appt.duration_minutes} min</dd>
          <dt>Provider</dt>
          <dd>{appt.provider_name}</dd>
          <dt>Time</dt>
          <dd>{formatTime(appt.start_time)} – {formatTime(appt.end_time)}</dd>
          <dt>Status</dt>
          <dd><span class="badge {statusBadgeClass(appt.status)}">{appt.status}</span></dd>
        </dl>

        <!-- Actions for Booked appointments -->
        {#if appt.status === "Booked"}
          {#if !showCancelConfirm}
            <div class="detail-actions">
              <button class="btn btn-primary btn-sm" onclick={() => doComplete(appt.appointment_id)}>
                Mark complete
              </button>
              <button class="btn btn-ghost btn-sm" onclick={() => doNoShow(appt.appointment_id)}>
                No-show
              </button>
              <button class="btn btn-destructive btn-sm" onclick={() => (showCancelConfirm = true)}>
                Cancel appointment
              </button>
            </div>
          {:else}
            <div class="cancel-confirm-box">
              <p class="cancel-confirm-label">Cancel this appointment?</p>
              <div class="form-field">
                <label class="field-label" for="cancel-reason">Reason (optional)</label>
                <textarea id="cancel-reason" bind:value={cancelReason} rows={2} placeholder="e.g. Patient called to cancel"></textarea>
              </div>
              <div class="cancel-confirm-actions">
                <button class="btn btn-ghost btn-sm" onclick={() => (showCancelConfirm = false)}>Go back</button>
                <button class="btn btn-destructive btn-sm" onclick={() => doCancel(appt.appointment_id)}>
                  Confirm cancellation
                </button>
              </div>
            </div>
          {/if}
        {/if}

        <!-- Notes -->
        <div class="notes-section">
          <h3 class="notes-heading">Notes {detailData.notes.length > 0 ? `(${detailData.notes.length})` : ""}</h3>

          {#if detailData.notes.length === 0}
            <p class="text-muted text-sm">No notes yet.</p>
          {:else}
            <ul class="notes-list">
              {#each detailData.notes as note}
                <li class="note-item">
                  <p class="note-meta">{formatDate(note.recorded_at)} {formatTime(note.recorded_at)} · {note.recorded_by}</p>
                  <p class="note-text">{note.text}</p>
                </li>
              {/each}
            </ul>
          {/if}

          <div class="add-note-form">
            <label class="sr-only" for="note-text">Add note</label>
            <textarea
              id="note-text"
              placeholder="Add a note…"
              bind:value={noteText}
              rows={2}
            ></textarea>
            {#if noteError}<p class="field-error" role="alert">{noteError}</p>{/if}
            <button
              class="btn btn-secondary btn-sm"
              onclick={doAddNote}
              disabled={noteLoading || !noteText.trim()}
              style="margin-top: var(--space-2);"
            >
              {#if noteLoading}<span class="spinner" aria-hidden="true"></span><span class="sr-only">Saving</span>{:else}Add note{/if}
            </button>
          </div>
        </div>
      {:else}
        <p class="text-muted">Appointment not found.</p>
      {/if}
    </div>
  </div>
{/if}

<!-- ═══════════════════════════════════════════════════════
     MAIN PAGE
     ═══════════════════════════════════════════════════════ -->
<div class="page-content">

  <!-- Page header -->
  <div class="page-header">
    <h1 class="page-title">Schedule</h1>
    <div class="header-actions">
      <button
        class="btn btn-ghost btn-sm"
        onclick={() => { showCallList = !showCallList; if (showCallList) loadCallList(); }}
      >
        {showCallList ? "Hide call list" : "Tomorrow's call list"}
      </button>
      <button
        class="btn btn-primary"
        onclick={() => openBookDrawer()}
        disabled={!selectedOfficeId}
      >
        + Book appointment
      </button>
    </div>
  </div>

  {#if offices.length === 0}
    <div class="empty-state">
      <span class="empty-state-icon">🏥</span>
      <p class="empty-state-title">No offices configured</p>
      <p class="empty-state-message">Go to <a href="/setup">Setup → Offices</a> to add an office.</p>
    </div>
  {:else}
    <!-- Office tabs -->
    <div class="office-tabs" role="tablist" aria-label="Select office">
      {#each offices as o}
        <button
          class="office-tab"
          class:active={selectedOfficeId === o.id}
          role="tab"
          aria-selected={selectedOfficeId === o.id}
          onclick={() => (selectedOfficeId = o.id)}
        >{o.name}</button>
      {/each}
    </div>

    <!-- Date navigation -->
    <div class="date-nav">
      <button class="btn btn-ghost btn-icon btn-sm" onclick={() => (selectedDate = addDays(selectedDate, -7))} title="Previous week" aria-label="Previous week">«</button>
      <button class="btn btn-ghost btn-icon btn-sm" onclick={() => (selectedDate = addDays(selectedDate, -1))} title="Previous day" aria-label="Previous day">‹</button>
      <span class="date-display">
        {formatDisplayDate(selectedDate)}
        {#if isToday}<span class="today-chip">Today</span>{/if}
      </span>
      <button class="btn btn-ghost btn-icon btn-sm" onclick={() => (selectedDate = addDays(selectedDate, 1))} title="Next day" aria-label="Next day">›</button>
      <button class="btn btn-ghost btn-icon btn-sm" onclick={() => (selectedDate = addDays(selectedDate, 7))} title="Next week" aria-label="Next week">»</button>
      {#if !isToday}
        <button class="btn btn-ghost btn-sm" onclick={() => (selectedDate = todayLocal())}>Today</button>
      {/if}
    </div>

    <!-- Tomorrow's call list -->
    {#if showCallList}
      <div class="card" style="margin-bottom: var(--space-6);">
        <div class="card-header">
          <h2 class="card-title">Call list</h2>
          <input type="date" bind:value={callListDate} onchange={loadCallList} style="min-height:36px;width:auto;" />
        </div>
        {#if callList.length === 0}
          <p class="text-muted text-sm">No booked appointments for this date.</p>
        {:else}
          <div class="table-wrap">
            <table class="call-table">
              <thead>
                <tr>
                  <th>Time</th>
                  <th>Patient</th>
                  <th>Phone</th>
                  <th>Pref.</th>
                  <th>Procedure</th>
                  <th>Provider</th>
                </tr>
              </thead>
              <tbody>
                {#each callList as e}
                  <tr>
                    <td class="mono">{formatTime(e.start_time)}</td>
                    <td>{e.patient_name}</td>
                    <td class="mono">{e.patient_phone ?? "—"}</td>
                    <td>{e.preferred_contact_channel ?? "—"}</td>
                    <td>{e.procedure_name}</td>
                    <td>{e.provider_name}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      </div>
    {/if}

    <!-- Grid -->
    {#if scheduleLoading}
      <div class="load-row" style="padding: 2rem; justify-content:center;">
        <div class="spinner"></div>
        <span class="text-muted text-sm">Loading schedule…</span>
      </div>
    {:else if officeHoursEntry === null}
      <div class="empty-state">
        <span class="empty-state-icon">🔒</span>
        <p class="empty-state-title">Closed on {dayName}</p>
        <p class="empty-state-message">Set office hours in <a href="/setup">Setup → Offices</a>.</p>
      </div>
    {:else if officeProviders.length === 0}
      <div class="empty-state">
        <span class="empty-state-icon">👥</span>
        <p class="empty-state-title">No providers assigned</p>
        <p class="empty-state-message">Assign providers to this office in <a href="/setup">Setup → Providers</a>.</p>
      </div>
    {:else}
      <div class="grid-outer">
        <!-- Column headers -->
        <div class="grid-header">
          <div class="time-col-head" aria-hidden="true"></div>
          {#each officeProviders as prov}
            {@const rosterEntry = providerRoster.find((r) => r.provider_id === prov.id)}
            <div class="col-head" class:col-head-off={!rosterEntry}>
              <div class="col-head-name">{prov.name}</div>
              {#if rosterEntry}
                <div class="col-head-hours">{rosterEntry.start_time}–{rosterEntry.end_time}</div>
              {:else}
                <div class="col-head-off-label">Not working</div>
              {/if}
            </div>
          {/each}
        </div>

        <!-- Grid body -->
        <div class="grid-body">
          <!-- Time labels -->
          <div class="time-col" style="height: {gridHeight}px" aria-hidden="true">
            {#each timeTicks as tick}
              <div class="time-tick" style="top: {tick.top}px">{tick.label}</div>
            {/each}
          </div>

          <!-- Provider columns -->
          {#each officeProviders as prov}
            {@const rosterEntry = providerRoster.find((r) => r.provider_id === prov.id)}
            {@const isWorking = !!rosterEntry}
            {@const provStart = rosterEntry ? parseHHMM(rosterEntry.start_time) : openMins}
            {@const provEnd   = rosterEntry ? parseHHMM(rosterEntry.end_time)   : openMins}
            {@const appts = schedule.filter((a) => a.provider_id === prov.id)}

            {#if isWorking}
              <div
                class="provider-col"
                style="height: {gridHeight}px"
                role="button"
                tabindex="0"
                aria-label="Book appointment with {prov.name}"
                onclick={(e) => onColumnClick(e, prov.id)}
                onkeydown={(e) => { if (e.key === "Enter") onColumnClick(e as unknown as MouseEvent, prov.id); }}
              >
                {#each timeTicks as tick}
                  <div class="h-line" style="top: {tick.top}px" aria-hidden="true"></div>
                {/each}

                {#if provStart > openMins}
                  <div class="unavail" style="top: 0; height: {((provStart - openMins) / 15) * SLOT_HEIGHT}px" aria-hidden="true"></div>
                {/if}

                {#if provEnd < closeMins}
                  <div class="unavail" style="top: {((provEnd - openMins) / 15) * SLOT_HEIGHT}px; height: {((closeMins - provEnd) / 15) * SLOT_HEIGHT}px" aria-hidden="true"></div>
                {/if}

                {#each appts as appt}
                  {@const apptMins = parseHHMM(appt.start_time.slice(11, 16))}
                  {@const blockTop = ((apptMins - openMins) / 15) * SLOT_HEIGHT}
                  {@const blockH = Math.max((appt.duration_minutes / 15) * SLOT_HEIGHT - 2, 20)}
                  <button
                    class="appt-block {statusBlockClass(appt.status)}"
                    style="top: {blockTop}px; height: {blockH}px"
                    onclick={(e) => { e.stopPropagation(); openDetail(appt.appointment_id); }}
                    title="{appt.patient_name} · {appt.procedure_name} · {formatTime(appt.start_time)}"
                    aria-label="{appt.patient_name}, {appt.procedure_name}, {formatTime(appt.start_time)}"
                  >
                    <span class="appt-time">{formatTime(appt.start_time)}</span>
                    <span class="appt-patient">{appt.patient_name}</span>
                    {#if appt.duration_minutes >= 30}
                      <span class="appt-proc">{appt.procedure_name}</span>
                    {/if}
                  </button>
                {/each}
              </div>
            {:else}
              <div
                class="provider-col provider-col-off"
                style="height: {gridHeight}px"
                aria-label="{prov.name} — not working {dayName}"
              >
                {#each timeTicks as tick}
                  <div class="h-line" style="top: {tick.top}px" aria-hidden="true"></div>
                {/each}
                <div class="unavail unavail-full" style="top: 0; height: {gridHeight}px" aria-hidden="true"></div>
                <span class="off-label">Not working</span>
              </div>
            {/if}
          {/each}
        </div>
      </div>
    {/if}
  {/if}
</div>

<style>
  /* ── Page ────────────────────────────────────────────── */
  .header-actions {
    display: flex;
    gap: var(--space-3);
    align-items: center;
  }

  /* ── Office tabs ─────────────────────────────────────── */
  .office-tabs {
    display: flex;
    gap: 2px;
    flex-wrap: wrap;
    margin-bottom: var(--space-4);
    border-bottom: 2px solid var(--pearl-mist-dk);
  }
  .office-tab {
    padding: var(--space-2) var(--space-5);
    background: transparent;
    border: none;
    border-bottom: 2px solid transparent;
    margin-bottom: -2px;
    color: var(--slate-fog);
    font-family: var(--font-body);
    font-size: var(--text-sm);
    font-weight: 500;
    cursor: pointer;
    transition: color var(--transition-fast), border-color var(--transition-fast);
    border-radius: var(--radius-sm) var(--radius-sm) 0 0;
  }
  .office-tab:hover { color: var(--abyss-navy); }
  .office-tab.active {
    color: var(--caribbean-teal);
    border-bottom-color: var(--caribbean-teal);
    font-weight: 600;
  }

  /* ── Date navigation ─────────────────────────────────── */
  .date-nav {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-bottom: var(--space-5);
  }
  .date-display {
    font-family: var(--font-heading);
    font-size: var(--text-base);
    font-weight: 600;
    color: var(--abyss-navy);
    min-width: 220px;
    text-align: center;
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }
  .today-chip {
    font-size: var(--text-xs);
    font-weight: 600;
    font-family: var(--font-heading);
    background: var(--caribbean-teal-lt);
    color: var(--caribbean-teal);
    padding: 2px var(--space-2);
    border-radius: var(--radius-pill);
  }

  /* ── Call list table ─────────────────────────────────── */
  .table-wrap { overflow-x: auto; }
  .call-table {
    width: 100%;
    border-collapse: collapse;
    font-size: var(--text-sm);
  }
  .call-table th {
    text-align: left;
    padding: var(--space-2) var(--space-3);
    font-family: var(--font-heading);
    font-size: var(--text-xs);
    font-weight: 600;
    color: var(--slate-fog);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    border-bottom: 1px solid var(--pearl-mist-dk);
    white-space: nowrap;
  }
  .call-table td {
    padding: var(--space-2) var(--space-3);
    border-bottom: 1px solid var(--pearl-mist-dk);
    color: var(--abyss-navy);
    white-space: nowrap;
  }
  .call-table tbody tr:hover { background: var(--pearl-mist); }
  .mono { font-family: var(--font-mono); font-size: 0.8em; }

  /* ── Schedule grid ───────────────────────────────────── */
  .grid-outer {
    background: #fff;
    border: 1px solid var(--pearl-mist-dk);
    border-radius: var(--radius-lg);
    overflow: hidden;
    box-shadow: var(--shadow-sm);
  }
  .grid-header {
    display: flex;
    border-bottom: 2px solid var(--pearl-mist-dk);
    background: var(--pearl-mist);
    position: sticky;
    top: var(--nav-height, 56px);
    z-index: 10;
  }
  .time-col-head {
    flex: 0 0 72px;
    border-right: 1px solid var(--pearl-mist-dk);
  }
  .col-head {
    flex: 0 0 180px;
    padding: var(--space-3) var(--space-4);
    border-right: 1px solid var(--pearl-mist-dk);
    min-height: 56px;
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 2px;
  }
  .col-head:last-child { border-right: none; }
  .col-head-name {
    font-family: var(--font-heading);
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--abyss-navy);
  }
  .col-head-hours {
    font-size: var(--text-xs);
    color: var(--caribbean-teal);
    font-family: var(--font-mono);
    font-weight: 500;
  }
  .col-head.col-head-off { opacity: 0.5; }
  .col-head-off-label {
    font-size: var(--text-xs);
    color: var(--slate-fog);
    font-style: italic;
  }

  .grid-body { display: flex; overflow-x: auto; }

  /* Time labels column */
  .time-col {
    flex: 0 0 72px;
    position: relative;
    border-right: 1px solid var(--pearl-mist-dk);
    background: var(--pearl-mist);
    overflow: hidden;
    flex-shrink: 0;
  }
  .time-tick {
    position: absolute;
    left: 0;
    right: var(--space-2);
    font-size: 0.68rem;
    color: var(--slate-fog);
    text-align: right;
    transform: translateY(-50%);
    pointer-events: none;
    font-family: var(--font-mono);
    white-space: nowrap;
  }

  /* Provider columns */
  .provider-col {
    flex: 0 0 180px;
    position: relative;
    border-right: 1px solid var(--pearl-mist-dk);
    background: #fff;
    cursor: crosshair;
    overflow: visible;
    flex-shrink: 0;
  }
  .provider-col:last-child { border-right: none; }
  .provider-col:hover { background: #fafcfd; }
  .provider-col-off { cursor: default; }
  .provider-col-off:hover { background: #fff; }

  /* Horizontal grid lines */
  .h-line {
    position: absolute;
    left: 0;
    right: 0;
    height: 1px;
    background: var(--pearl-mist-dk);
    pointer-events: none;
  }

  /* Unavailable zone */
  .unavail {
    position: absolute;
    left: 0;
    right: 0;
    pointer-events: none;
    background: repeating-linear-gradient(
      135deg,
      transparent,
      transparent 4px,
      rgba(107, 124, 130, 0.08) 4px,
      rgba(107, 124, 130, 0.08) 8px
    );
    z-index: 1;
  }
  .unavail-full { background: rgba(240, 244, 245, 0.7); }
  .off-label {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: var(--text-xs);
    color: var(--slate-fog);
    font-style: italic;
    pointer-events: none;
    z-index: 2;
  }

  /* Appointment blocks */
  .appt-block {
    position: absolute;
    left: 3px;
    right: 3px;
    border: none;
    border-radius: var(--radius-sm);
    padding: 3px 6px;
    text-align: left;
    cursor: pointer;
    font-family: var(--font-body);
    overflow: hidden;
    display: flex;
    flex-direction: column;
    gap: 1px;
    z-index: 3;
    transition: filter var(--transition-fast), box-shadow var(--transition-fast);
    box-shadow: var(--shadow-sm);
  }
  .appt-block:hover {
    filter: brightness(0.93);
    box-shadow: var(--shadow-md);
    z-index: 4;
  }

  .appt-booked      { background: var(--color-booked-lt);      color: var(--color-booked);      border-left: 3px solid var(--color-booked); }
  .appt-completed   { background: var(--color-completed-lt);   color: var(--color-completed);   border-left: 3px solid var(--color-completed); }
  .appt-cancelled   { background: var(--color-cancelled-lt);   color: var(--color-cancelled);   border-left: 3px solid var(--color-cancelled); }
  .appt-noshow      { background: var(--color-noshow-lt);      color: var(--color-noshow);      border-left: 3px solid var(--color-noshow); }
  .appt-rescheduled { background: var(--color-rescheduled-lt); color: var(--color-rescheduled); border-left: 3px solid var(--color-rescheduled); }

  .appt-time    { font-size: 0.65rem; font-weight: 700; opacity: 0.8; font-family: var(--font-mono); }
  .appt-patient { font-size: 0.72rem; font-weight: 600; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .appt-proc    { font-size: 0.65rem; opacity: 0.7; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

  /* ── Booking drawer internals ────────────────────────── */
  .book-section { margin-bottom: var(--space-5); }
  .book-section-label {
    font-family: var(--font-heading);
    font-size: var(--text-xs);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--slate-fog);
    margin: 0 0 var(--space-2);
  }
  .book-row {
    display: flex;
    gap: var(--space-3);
  }

  /* Provider chips */
  .chip-group { display: flex; flex-direction: column; gap: var(--space-2); }
  .chip {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-2) var(--space-3);
    background: var(--pearl-mist);
    border: 1.5px solid var(--pearl-mist-dk);
    border-radius: var(--radius-md);
    cursor: pointer;
    text-align: left;
    transition: border-color var(--transition-fast), background var(--transition-fast);
  }
  .chip:hover { border-color: var(--caribbean-teal); background: var(--caribbean-teal-lt); }
  .chip-selected {
    border-color: var(--caribbean-teal);
    background: var(--caribbean-teal-lt);
  }
  .chip-name { font-size: var(--text-sm); font-weight: 600; color: var(--abyss-navy); }
  .chip-hours { font-size: var(--text-xs); color: var(--slate-fog); font-family: var(--font-mono); }

  /* Time slots */
  .slot-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(80px, 1fr));
    gap: var(--space-2);
  }
  .slot-btn {
    padding: var(--space-2) var(--space-1);
    background: var(--pearl-mist);
    border: 1.5px solid var(--pearl-mist-dk);
    border-radius: var(--radius-sm);
    font-size: var(--text-xs);
    font-family: var(--font-mono);
    font-weight: 500;
    color: var(--abyss-navy);
    cursor: pointer;
    text-align: center;
    transition: all var(--transition-fast);
  }
  .slot-btn:hover { border-color: var(--caribbean-teal); color: var(--caribbean-teal); }
  .slot-selected {
    background: var(--caribbean-teal);
    border-color: var(--caribbean-teal);
    color: #fff;
  }

  /* Patient search */
  .patient-search-wrap { position: relative; }
  .patient-dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    background: #fff;
    border: 1.5px solid var(--caribbean-teal);
    border-top: none;
    border-radius: 0 0 var(--radius-md) var(--radius-md);
    list-style: none;
    margin: 0;
    padding: var(--space-1) 0;
    box-shadow: var(--shadow-md);
    z-index: 10;
    max-height: 200px;
    overflow-y: auto;
  }
  .dropdown-item {
    display: block;
    width: 100%;
    text-align: left;
    padding: var(--space-2) var(--space-3);
    background: none;
    border: none;
    font-size: var(--text-sm);
    color: var(--abyss-navy);
    cursor: pointer;
  }
  .dropdown-item:hover { background: var(--caribbean-teal-lt); }
  .selected-patient {
    margin-top: var(--space-2);
    font-size: var(--text-sm);
    color: var(--island-palm);
    font-weight: 500;
  }
  .check-icon { font-weight: 700; }

  /* ── Detail drawer internals ─────────────────────────── */
  .detail-dl {
    display: grid;
    grid-template-columns: max-content 1fr;
    gap: var(--space-1) var(--space-4);
    margin: 0 0 var(--space-5);
    font-size: var(--text-sm);
  }
  .detail-dl dt {
    color: var(--slate-fog);
    font-weight: 500;
    white-space: nowrap;
    padding: var(--space-1) 0;
  }
  .detail-dl dd {
    margin: 0;
    color: var(--abyss-navy);
    padding: var(--space-1) 0;
  }

  .detail-actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    margin-bottom: var(--space-5);
    padding-top: var(--space-3);
    border-top: 1px solid var(--pearl-mist-dk);
  }

  .cancel-confirm-box {
    padding: var(--space-4);
    background: var(--healthy-coral-lt);
    border: 1.5px solid var(--healthy-coral);
    border-radius: var(--radius-md);
    margin-bottom: var(--space-5);
  }
  .cancel-confirm-label {
    font-family: var(--font-heading);
    font-weight: 600;
    color: var(--healthy-coral-dk);
    margin: 0 0 var(--space-3);
    font-size: var(--text-sm);
  }
  .cancel-confirm-actions {
    display: flex;
    gap: var(--space-2);
    margin-top: var(--space-3);
    justify-content: flex-end;
  }

  /* Notes */
  .notes-section { border-top: 1px solid var(--pearl-mist-dk); padding-top: var(--space-4); }
  .notes-heading {
    font-family: var(--font-heading);
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--abyss-navy);
    margin: 0 0 var(--space-3);
  }
  .notes-list { list-style: none; margin: 0 0 var(--space-4); padding: 0; display: flex; flex-direction: column; gap: var(--space-3); }
  .note-item { padding: var(--space-3); background: var(--pearl-mist); border-radius: var(--radius-sm); }
  .note-meta { font-size: var(--text-xs); color: var(--slate-fog); margin: 0 0 var(--space-1); }
  .note-text { font-size: var(--text-sm); color: var(--abyss-navy); margin: 0; }
  .add-note-form { display: flex; flex-direction: column; }

  /* ── Shared ───────────────────────────────────────────── */
  .load-row {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }
</style>
