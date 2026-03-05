<script lang="ts">
  import { commands, type PatientDto, type PatientWithNotesDto, type PatientNoteDto } from "$lib/bindings";
  import { onMount } from "svelte";

  // ── Search state ─────────────────────────────────────────────────────────────
  let searchName = $state("");
  let searchPhone = $state("");
  let includeArchived = $state(false);
  let patients = $state<PatientDto[]>([]);
  let searchError = $state<string | null>(null);
  let searched = $state(false);

  // ── Register form ─────────────────────────────────────────────────────────────
  let showRegister = $state(false);
  let regFirstName = $state("");
  let regLastName = $state("");
  let regPhone = $state("");
  let regEmail = $state("");
  let regChannel = $state("");
  let regDob = $state("");
  let registering = $state(false);
  let regError = $state<string | null>(null);
  let regWarning = $state<string | null>(null);

  // ── Expanded patient detail ───────────────────────────────────────────────────
  let expandedId = $state<string | null>(null);
  let detailData = $state<PatientWithNotesDto | null>(null);
  let detailLoading = $state(false);
  let detailError = $state<string | null>(null);

  // Edit modes
  let editingSection = $state<"demographics" | "contact" | null>(null);
  // demographics edit fields
  let demoFirstName = $state("");
  let demoLastName = $state("");
  let demoDob = $state("");
  let demoAddr1 = $state("");
  let demoCity = $state("");
  let demoSubdiv = $state("");
  let demoCountry = $state("");
  let demoSaving = $state(false);
  let demoError = $state<string | null>(null);
  // contact edit fields
  let contPhone = $state("");
  let contEmail = $state("");
  let contChannel = $state("");
  let contSaving = $state(false);
  let contError = $state<string | null>(null);

  // Note form
  let noteText = $state("");
  let noteAdding = $state(false);
  let noteError = $state<string | null>(null);

  const STAFF_ID = "staff-system"; // placeholder until auth exists
  const CHANNELS = ["", "Phone", "Email", "WhatsApp"];

  onMount(() => runSearch());

  async function runSearch() {
    searchError = null;
    const r = await commands.searchPatients(
      searchName.trim() || null,
      searchPhone.trim() || null,
      null,
      includeArchived,
    );
    searched = true;
    if (r.status === "ok") {
      patients = r.data;
    } else {
      searchError = r.error;
    }
  }

  async function register() {
    if (!regFirstName.trim() || !regLastName.trim()) {
      regError = "First and last name are required"; return;
    }
    if (!regPhone.trim() && !regEmail.trim()) {
      regError = "At least one of phone or email is required"; return;
    }
    registering = true;
    regError = null;
    regWarning = null;
    const r = await commands.registerPatient(
      regFirstName.trim(),
      regLastName.trim(),
      regPhone.trim() || null,
      regEmail.trim() || null,
      regChannel || null,
      null,
      regDob.trim() || null,
      STAFF_ID,
    );
    registering = false;
    if (r.status === "ok") {
      const { patient, duplicate_warning } = r.data;
      regWarning = duplicate_warning ?? null;
      // Add to list or refresh
      patients = [patient, ...patients];
      regFirstName = ""; regLastName = ""; regPhone = ""; regEmail = "";
      regChannel = ""; regDob = "";
      if (!duplicate_warning) showRegister = false;
    } else {
      regError = r.error;
    }
  }

  async function toggleExpand(id: string) {
    if (expandedId === id) {
      expandedId = null;
      detailData = null;
      editingSection = null;
      return;
    }
    expandedId = id;
    editingSection = null;
    detailData = null;
    detailLoading = true;
    detailError = null;
    const r = await commands.getPatient(id);
    detailLoading = false;
    if (r.status === "ok") {
      detailData = r.data;
    } else {
      detailError = r.error;
    }
  }

  function startEditDemographics() {
    if (!detailData) return;
    const p = detailData.patient;
    demoFirstName = p.first_name;
    demoLastName = p.last_name;
    demoDob = p.date_of_birth ?? "";
    demoAddr1 = p.address_line_1 ?? "";
    demoCity = p.city_town ?? "";
    demoSubdiv = p.subdivision ?? "";
    demoCountry = p.country ?? "";
    demoError = null;
    editingSection = "demographics";
  }

  async function saveDemographics() {
    if (!detailData) return;
    demoSaving = true; demoError = null;
    const r = await commands.updatePatientDemographics(
      detailData.patient.patient_id,
      demoFirstName.trim(),
      demoLastName.trim(),
      demoDob.trim() || null,
      demoAddr1.trim() || null,
      demoCity.trim() || null,
      demoSubdiv.trim() || null,
      demoCountry.trim() || null,
      STAFF_ID,
    );
    demoSaving = false;
    if (r.status === "ok") {
      detailData = { ...detailData, patient: r.data };
      patients = patients.map((p) => p.patient_id === r.data.patient_id ? r.data : p);
      editingSection = null;
    } else {
      demoError = r.error;
    }
  }

  function startEditContact() {
    if (!detailData) return;
    const p = detailData.patient;
    contPhone = p.phone ?? "";
    contEmail = p.email ?? "";
    contChannel = p.preferred_contact_channel ?? "";
    contError = null;
    editingSection = "contact";
  }

  async function saveContact() {
    if (!detailData) return;
    contSaving = true; contError = null;
    const r = await commands.updatePatientContactInfo(
      detailData.patient.patient_id,
      contPhone.trim() || null,
      contEmail.trim() || null,
      contChannel || null,
      STAFF_ID,
    );
    contSaving = false;
    if (r.status === "ok") {
      detailData = { ...detailData, patient: r.data };
      patients = patients.map((p) => p.patient_id === r.data.patient_id ? r.data : p);
      editingSection = null;
    } else {
      contError = r.error;
    }
  }

  async function addNote() {
    if (!detailData || !noteText.trim()) return;
    noteAdding = true; noteError = null;
    const r = await commands.addPatientNote(
      detailData.patient.patient_id,
      noteText.trim(),
      STAFF_ID,
    );
    noteAdding = false;
    if (r.status === "ok") {
      detailData = { ...detailData, notes: [...detailData.notes, r.data] };
      noteText = "";
    } else {
      noteError = r.error;
    }
  }

  async function archivePatient(id: string) {
    if (!confirm("Archive this patient?")) return;
    const r = await commands.archivePatient(id, STAFF_ID);
    if (r.status === "ok") {
      patients = patients.map((p) => p.patient_id === id ? r.data : p);
      if (detailData?.patient.patient_id === id) {
        detailData = { ...detailData, patient: r.data };
      }
    } else {
      detailError = r.error;
    }
  }

  async function unarchivePatient(id: string) {
    const r = await commands.unarchivePatient(id, STAFF_ID);
    if (r.status === "ok") {
      patients = patients.map((p) => p.patient_id === id ? r.data : p);
      if (detailData?.patient.patient_id === id) {
        detailData = { ...detailData, patient: r.data };
      }
    } else {
      detailError = r.error;
    }
  }

  function formatDate(iso: string | null | undefined): string {
    if (!iso) return "—";
    return iso.length === 10 ? iso : iso.slice(0, 10);
  }
</script>

<div class="page">
  <!-- Header -->
  <div class="page-header">
    <h1>Patients</h1>
    <button class="btn-primary" onclick={() => { showRegister = !showRegister; regError = null; regWarning = null; }}>
      {showRegister ? "Cancel" : "+ Register Patient"}
    </button>
  </div>

  <!-- Register form -->
  {#if showRegister}
    <div class="card form-card">
      <h3>Register Patient</h3>
      {#if regError}<p class="error">{regError}</p>{/if}
      {#if regWarning}<p class="warning">⚠ {regWarning}</p>{/if}
      <div class="form-row">
        <div class="field">
          <label for="reg-first-name">First Name *</label>
          <input id="reg-first-name" bind:value={regFirstName} placeholder="Maria" />
        </div>
        <div class="field">
          <label for="reg-last-name">Last Name *</label>
          <input id="reg-last-name" bind:value={regLastName} placeholder="Brown" />
        </div>
        <div class="field">
          <label for="reg-dob">Date of Birth</label>
          <input id="reg-dob" type="date" bind:value={regDob} />
        </div>
      </div>
      <div class="form-row">
        <div class="field">
          <label for="reg-phone">Phone</label>
          <input id="reg-phone" bind:value={regPhone} placeholder="+1-876-555-0100" />
        </div>
        <div class="field">
          <label for="reg-email">Email</label>
          <input id="reg-email" type="email" bind:value={regEmail} placeholder="maria@example.com" />
        </div>
        <div class="field">
          <label for="reg-channel">Preferred Channel</label>
          <select id="reg-channel" bind:value={regChannel}>
            {#each CHANNELS as c}<option value={c}>{c || "—"}</option>{/each}
          </select>
        </div>
      </div>
      <div class="form-actions">
        <button class="btn-primary" onclick={register} disabled={registering}>
          {registering ? "Registering…" : "Register"}
        </button>
        {#if regWarning}
          <button class="btn-secondary" onclick={() => { showRegister = false; regWarning = null; }}>
            Close Anyway
          </button>
        {/if}
      </div>
    </div>
  {/if}

  <!-- Search -->
  <div class="search-bar">
    <label for="search-name" class="sr-only">Search by name</label>
    <input
      id="search-name"
      class="search-input"
      placeholder="Search by name…"
      bind:value={searchName}
      oninput={runSearch}
    />
    <label for="search-phone" class="sr-only">Search by phone</label>
    <input
      id="search-phone"
      class="search-input"
      placeholder="Search by phone…"
      bind:value={searchPhone}
      oninput={runSearch}
    />
    <label class="archived-toggle">
      <input type="checkbox" bind:checked={includeArchived} onchange={runSearch} />
      Show archived
    </label>
  </div>

  {#if searchError}
    <p class="error">{searchError}</p>
  {/if}

  <!-- Patient list -->
  {#if searched && patients.length === 0}
    <p class="empty">
      {#if searchName || searchPhone}
        No patients match that search. Try a different name or phone number.
      {:else}
        No patients registered yet. Click <strong>+ Register Patient</strong> to add your first patient.
      {/if}
    </p>
  {/if}

  <div class="patient-list">
    {#each patients as patient (patient.patient_id)}
      <div class="patient-card" class:archived={patient.archived}>
        <!-- Summary row -->
        <div
          class="patient-row"
          role="button"
          tabindex="0"
          aria-expanded={expandedId === patient.patient_id}
          onclick={() => toggleExpand(patient.patient_id)}
          onkeydown={(e) => e.key === "Enter" && toggleExpand(patient.patient_id)}
        >
          <div class="patient-info">
            <span class="patient-name">{patient.full_name_display}</span>
            {#if patient.phone}<span class="meta">{patient.phone}</span>{/if}
            {#if patient.email}<span class="meta">{patient.email}</span>{/if}
            {#if patient.date_of_birth}<span class="meta">DOB: {patient.date_of_birth}</span>{/if}
            {#if patient.archived}<span class="badge archived-badge">Archived</span>{/if}
          </div>
          <span class="chevron">{expandedId === patient.patient_id ? "▲" : "▼"}</span>
        </div>

        <!-- Detail panel -->
        {#if expandedId === patient.patient_id}
          <div class="detail-panel">
            {#if detailLoading}
              <p class="loading">Loading…</p>
            {:else if detailError}
              <p class="error">{detailError}</p>
            {:else if detailData}
              <!-- ── Demographics section ── -->
              <section class="detail-section">
                <div class="section-title-row">
                  <h4>Demographics</h4>
                  {#if editingSection !== "demographics"}
                    <button class="btn-sm btn-ghost" onclick={startEditDemographics}>Edit</button>
                  {/if}
                </div>
                {#if editingSection === "demographics"}
                  {#if demoError}<p class="error">{demoError}</p>{/if}
                  <div class="edit-grid">
                    <div class="field">
                      <label for="demo-first-{expandedId}">First Name</label>
                      <input id="demo-first-{expandedId}" bind:value={demoFirstName} />
                    </div>
                    <div class="field">
                      <label for="demo-last-{expandedId}">Last Name</label>
                      <input id="demo-last-{expandedId}" bind:value={demoLastName} />
                    </div>
                    <div class="field">
                      <label for="demo-dob-{expandedId}">Date of Birth</label>
                      <input id="demo-dob-{expandedId}" type="date" bind:value={demoDob} />
                    </div>
                    <div class="field">
                      <label for="demo-addr1-{expandedId}">Address Line 1</label>
                      <input id="demo-addr1-{expandedId}" bind:value={demoAddr1} />
                    </div>
                    <div class="field">
                      <label for="demo-city-{expandedId}">City / Town</label>
                      <input id="demo-city-{expandedId}" bind:value={demoCity} />
                    </div>
                    <div class="field">
                      <label for="demo-subdiv-{expandedId}">Parish / Region</label>
                      <input id="demo-subdiv-{expandedId}" bind:value={demoSubdiv} />
                    </div>
                    <div class="field">
                      <label for="demo-country-{expandedId}">Country</label>
                      <input id="demo-country-{expandedId}" bind:value={demoCountry} />
                    </div>
                  </div>
                  <div class="edit-actions">
                    <button class="btn-sm" onclick={saveDemographics} disabled={demoSaving}>
                      {demoSaving ? "Saving…" : "Save"}
                    </button>
                    <button class="btn-sm btn-ghost" onclick={() => (editingSection = null)}>Cancel</button>
                  </div>
                {:else}
                  <dl class="info-list">
                    <dt>Name</dt><dd>{detailData.patient.full_name_display}</dd>
                    <dt>Date of Birth</dt><dd>{detailData.patient.date_of_birth ?? "—"}</dd>
                    <dt>Address</dt><dd>{detailData.patient.address_line_1 ?? "—"}</dd>
                    <dt>City</dt><dd>{detailData.patient.city_town ?? "—"}</dd>
                    <dt>Parish</dt><dd>{detailData.patient.subdivision ?? "—"}</dd>
                    <dt>Country</dt><dd>{detailData.patient.country ?? "—"}</dd>
                    <dt>Registered by</dt><dd>{detailData.patient.registered_by}</dd>
                    <dt>Registered at</dt><dd>{formatDate(detailData.patient.registered_at)}</dd>
                  </dl>
                {/if}
              </section>

              <!-- ── Contact section ── -->
              <section class="detail-section">
                <div class="section-title-row">
                  <h4>Contact</h4>
                  {#if editingSection !== "contact"}
                    <button class="btn-sm btn-ghost" onclick={startEditContact}>Edit</button>
                  {/if}
                </div>
                {#if editingSection === "contact"}
                  {#if contError}<p class="error">{contError}</p>{/if}
                  <div class="edit-grid">
                    <div class="field">
                      <label for="cont-phone-{expandedId}">Phone</label>
                      <input id="cont-phone-{expandedId}" bind:value={contPhone} placeholder="+1-876-555-0100" />
                    </div>
                    <div class="field">
                      <label for="cont-email-{expandedId}">Email</label>
                      <input id="cont-email-{expandedId}" type="email" bind:value={contEmail} />
                    </div>
                    <div class="field">
                      <label for="cont-channel-{expandedId}">Preferred Channel</label>
                      <select id="cont-channel-{expandedId}" bind:value={contChannel}>
                        {#each CHANNELS as c}<option value={c}>{c || "—"}</option>{/each}
                      </select>
                    </div>
                  </div>
                  <div class="edit-actions">
                    <button class="btn-sm" onclick={saveContact} disabled={contSaving}>
                      {contSaving ? "Saving…" : "Save"}
                    </button>
                    <button class="btn-sm btn-ghost" onclick={() => (editingSection = null)}>Cancel</button>
                  </div>
                {:else}
                  <dl class="info-list">
                    <dt>Phone</dt><dd>{detailData.patient.phone ?? "—"}</dd>
                    <dt>Email</dt><dd>{detailData.patient.email ?? "—"}</dd>
                    <dt>Preferred Channel</dt><dd>{detailData.patient.preferred_contact_channel ?? "—"}</dd>
                  </dl>
                {/if}
              </section>

              <!-- ── Notes section ── -->
              <section class="detail-section">
                <h4>Notes</h4>
                {#if detailData.notes.length === 0}
                  <p class="empty-notes">No notes yet.</p>
                {:else}
                  <ul class="notes-list">
                    {#each detailData.notes as note (note.note_id)}
                      <li class="note-item">
                        <p class="note-text">{note.text}</p>
                        <span class="note-meta">{note.recorded_by} · {formatDate(note.recorded_at)}</span>
                      </li>
                    {/each}
                  </ul>
                {/if}
                {#if noteError}<p class="error">{noteError}</p>{/if}
                <div class="note-form">
                  <textarea
                    class="note-input"
                    placeholder="Add a note…"
                    bind:value={noteText}
                    rows={2}
                  ></textarea>
                  <button class="btn-sm" onclick={addNote} disabled={noteAdding || !noteText.trim()}>
                    {noteAdding ? "Adding…" : "Add Note"}
                  </button>
                </div>
              </section>

              <!-- ── Archive / unarchive ── -->
              <section class="detail-section archive-section">
                {#if detailData.patient.archived}
                  <button class="btn-secondary" onclick={() => unarchivePatient(detailData!.patient.patient_id)}>
                    Unarchive Patient
                  </button>
                {:else}
                  <button class="btn-danger-sm" onclick={() => archivePatient(detailData!.patient.patient_id)}>
                    Archive Patient
                  </button>
                {/if}
              </section>
            {/if}
          </div>
        {/if}
      </div>
    {/each}
  </div>
</div>

<style>
  .page { padding: 1.5rem 2rem; max-width: 900px; }
  .page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 1.25rem; }
  h1 { margin: 0; font-size: 1.25rem; color: #222; font-family: system-ui, sans-serif; }
  h3 { margin: 0 0 1rem; font-size: 1rem; color: #222; }
  h4 { margin: 0; font-size: 0.82rem; font-weight: 700; text-transform: uppercase;
       letter-spacing: 0.04em; color: #666; }
  .error { color: #c0392b; font-size: 0.875rem; margin-bottom: 0.5rem; }
  .warning { color: #a06030; background: #fff8e1; border: 1px solid #f0c040;
             border-radius: 6px; padding: 0.5rem 0.75rem; font-size: 0.875rem; margin-bottom: 0.5rem; }
  .empty { color: #999; font-style: italic; font-family: system-ui, sans-serif; }
  .loading { color: #999; font-family: system-ui, sans-serif; }

  /* Register card */
  .card { background: white; border: 1px solid #e0e0e0; border-radius: 8px; padding: 1.25rem; }
  .form-card { margin-bottom: 1.25rem; }
  .form-row { display: flex; gap: 1rem; flex-wrap: wrap; margin-bottom: 0.75rem; }
  .form-actions { display: flex; gap: 0.75rem; align-items: center; margin-top: 0.75rem; }

  /* Fields */
  .field { display: flex; flex-direction: column; gap: 0.3rem; flex: 1; min-width: 160px; }
  .field label { font-size: 0.78rem; font-weight: 600; color: #555;
                 text-transform: uppercase; letter-spacing: 0.03em; font-family: system-ui, sans-serif; }
  input:not([type="checkbox"]):not([type="date"]), select, textarea {
    padding: 0.45rem 0.6rem; border: 1px solid #ccc; border-radius: 6px;
    font-size: 0.9rem; font-family: system-ui, sans-serif; width: 100%; box-sizing: border-box;
    background: white;
  }
  input[type="date"] {
    padding: 0.45rem 0.6rem; border: 1px solid #ccc; border-radius: 6px;
    font-size: 0.9rem; font-family: system-ui, sans-serif; width: 100%; box-sizing: border-box;
  }
  input:focus, select:focus, textarea:focus { outline: none; border-color: #1a1a2e; }
  textarea { resize: vertical; }

  /* Search */
  .search-bar { display: flex; gap: 0.75rem; align-items: center; margin-bottom: 1rem; flex-wrap: wrap; }
  .search-input { padding: 0.5rem 0.75rem; border: 1px solid #ccc; border-radius: 6px;
                  font-size: 0.9rem; font-family: system-ui, sans-serif; flex: 1; min-width: 180px; box-sizing: border-box; }
  .search-input:focus { outline: none; border-color: #1a1a2e; }
  .archived-toggle { display: flex; align-items: center; gap: 0.4rem;
                     font-size: 0.85rem; color: #666; font-family: system-ui, sans-serif;
                     cursor: pointer; white-space: nowrap; }

  /* Patient list */
  .patient-list { display: flex; flex-direction: column; gap: 0.6rem; }
  .patient-card { border: 1px solid #ddd; border-radius: 8px; overflow: hidden; background: white; }
  .patient-card.archived { opacity: 0.65; }
  .patient-row { display: flex; justify-content: space-between; align-items: center;
                 padding: 0.75rem 1rem; cursor: pointer; user-select: none; }
  .patient-row:hover { background: #f7f8fa; }
  .patient-info { display: flex; align-items: center; gap: 0.75rem; flex-wrap: wrap; }
  .patient-name { font-weight: 600; font-size: 0.95rem; font-family: system-ui, sans-serif; }
  .meta { font-size: 0.82rem; color: #777; font-family: system-ui, sans-serif; }
  .badge { font-size: 0.72rem; padding: 0.15rem 0.5rem; border-radius: 20px; font-weight: 600; font-family: system-ui, sans-serif; }
  .archived-badge { background: #f0e6d3; color: #a06030; }
  .chevron { color: #aaa; font-size: 0.8rem; }

  /* Detail panel */
  .detail-panel { border-top: 1px solid #eee; padding: 1rem; }
  .detail-section { margin-bottom: 1.25rem; }
  .detail-section:last-child { margin-bottom: 0; }
  .section-title-row { display: flex; align-items: center; gap: 0.75rem; margin-bottom: 0.5rem; }

  /* Info list (dl) */
  .info-list { display: grid; grid-template-columns: 130px 1fr; gap: 0.3rem 0.75rem;
               margin: 0.5rem 0 0; font-size: 0.875rem; font-family: system-ui, sans-serif; }
  dt { color: #888; font-weight: 500; }
  dd { margin: 0; color: #222; }

  /* Edit grid */
  .edit-grid { display: flex; flex-wrap: wrap; gap: 0.75rem; margin-top: 0.5rem; }
  .edit-grid .field { min-width: 150px; }
  .edit-actions { display: flex; gap: 0.5rem; margin-top: 0.75rem; }

  /* Notes */
  .empty-notes { color: #bbb; font-size: 0.85rem; font-style: italic; font-family: system-ui, sans-serif; margin: 0.25rem 0 0.75rem; }
  .notes-list { list-style: none; margin: 0.25rem 0 0.75rem; padding: 0;
                display: flex; flex-direction: column; gap: 0.5rem; }
  .note-item { background: #f7f8fa; border: 1px solid #eee; border-radius: 6px; padding: 0.6rem 0.75rem; }
  .note-text { margin: 0 0 0.25rem; font-size: 0.9rem; color: #222; font-family: system-ui, sans-serif; }
  .note-meta { font-size: 0.75rem; color: #999; font-family: system-ui, sans-serif; }
  .note-form { display: flex; gap: 0.5rem; align-items: flex-start; margin-top: 0.5rem; }
  .note-input { flex: 1; }

  /* Archive section */
  .archive-section { border-top: 1px solid #f0f0f0; padding-top: 0.75rem; }

  /* Buttons */
  .btn-primary {
    padding: 0.45rem 1.1rem; background: #1a1a2e; color: white;
    border: none; border-radius: 6px; font-size: 0.875rem; cursor: pointer; font-family: system-ui, sans-serif;
  }
  .btn-primary:hover:not(:disabled) { background: #2a2a4e; }
  .btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn-secondary {
    padding: 0.45rem 1.1rem; background: white; color: #444;
    border: 1px solid #ccc; border-radius: 6px; font-size: 0.875rem; cursor: pointer; font-family: system-ui, sans-serif;
  }
  .btn-secondary:hover { background: #f5f5f5; }
  .btn-sm {
    padding: 0.25rem 0.6rem; background: #1a1a2e; color: white;
    border: none; border-radius: 4px; font-size: 0.78rem; cursor: pointer; font-family: system-ui, sans-serif;
    white-space: nowrap;
  }
  .btn-sm:disabled { opacity: 0.4; cursor: not-allowed; }
  .btn-sm.btn-ghost { background: #eee; color: #555; }
  .btn-sm.btn-ghost:hover { background: #ddd; }
  .btn-danger-sm {
    padding: 0.35rem 0.75rem; background: white; color: #c0392b;
    border: 1px solid #c0392b; border-radius: 6px; font-size: 0.8rem; cursor: pointer; font-family: system-ui, sans-serif;
  }
  .btn-danger-sm:hover { background: #fdf0ef; }
</style>
