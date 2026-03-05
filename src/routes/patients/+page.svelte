<script lang="ts">
  import { commands, type PatientDto, type PatientWithNotesDto, type PatientNoteDto } from "$lib/bindings";
  import { onMount } from "svelte";
  import { formatDate } from "$lib/utils/date";

  // ── Search state ─────────────────────────────────────────────────────────────
  let searchName = $state("");
  let searchPhone = $state("");
  let includeArchived = $state(false);
  let patients = $state<PatientDto[]>([]);
  let searchError = $state<string | null>(null);
  let searched = $state(false);
  let searchTimeout = $state<ReturnType<typeof setTimeout> | null>(null);

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

  function onNameInput(e: Event) {
    const value = (e.target as HTMLInputElement).value;
    searchName = value;
    if (searchTimeout) clearTimeout(searchTimeout);
    if (value.length === 0) {
      // Clear results immediately when query cleared
      patients = [];
      searched = false;
      searchTimeout = setTimeout(() => runSearch(), 0);
      return;
    }
    if (value.length < 2) {
      // Don't fire search — just show the hint
      patients = [];
      searched = false;
      return;
    }
    searchTimeout = setTimeout(() => runSearch(), 250);
  }

  function onPhoneInput() {
    if (searchTimeout) clearTimeout(searchTimeout);
    searchTimeout = setTimeout(() => runSearch(), 250);
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
          {#if registering}<span class="spinner" aria-hidden="true"></span><span class="sr-only">Registering</span>{:else}Register{/if}
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
    <div class="search-field-wrap">
      <label for="search-name" class="sr-only">Search by name</label>
      <input
        id="search-name"
        class="search-input"
        placeholder="Search by name…"
        value={searchName}
        oninput={onNameInput}
      />
      {#if searchName.length === 1}
        <p class="search-hint">Type at least 2 characters to search</p>
      {/if}
    </div>
    <label for="search-phone" class="sr-only">Search by phone</label>
    <input
      id="search-phone"
      class="search-input"
      placeholder="Search by phone…"
      bind:value={searchPhone}
      oninput={onPhoneInput}
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
          aria-label="Expand {patient.full_name_display} details"
          onclick={() => toggleExpand(patient.patient_id)}
          onkeydown={(e) => e.key === "Enter" && toggleExpand(patient.patient_id)}
        >
          <div class="patient-info">
            <span class="patient-name">{patient.full_name_display}</span>
            {#if patient.phone}<span class="meta">{patient.phone}</span>{/if}
            {#if patient.email}<span class="meta">{patient.email}</span>{/if}
            {#if patient.date_of_birth}<span class="meta">DOB: {formatDate(patient.date_of_birth)}</span>{/if}
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
                      {#if demoSaving}<span class="spinner" aria-hidden="true"></span><span class="sr-only">Saving</span>{:else}Save{/if}
                    </button>
                    <button class="btn-sm btn-ghost" onclick={() => (editingSection = null)}>Cancel</button>
                  </div>
                {:else}
                  <dl class="info-list">
                    <dt>Name</dt><dd>{detailData.patient.full_name_display}</dd>
                    <dt>Date of Birth</dt><dd>{formatDate(detailData.patient.date_of_birth)}</dd>
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
                      {#if contSaving}<span class="spinner" aria-hidden="true"></span><span class="sr-only">Saving</span>{:else}Save{/if}
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
                    {#if noteAdding}<span class="spinner" aria-hidden="true"></span><span class="sr-only">Adding</span>{:else}Add Note{/if}
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
  /* ── Layout ──────────────────────────────────────────── */
  .page { padding: var(--space-6); max-width: 900px; }
  .page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--space-5); }
  h1 { margin: 0; font-size: var(--text-2xl); font-family: var(--font-heading); font-weight: 700; color: var(--abyss-navy); }
  h3 { margin: 0 0 var(--space-4); font-size: var(--text-base); font-family: var(--font-heading); font-weight: 600; color: var(--abyss-navy); }
  h4 { margin: 0; font-size: var(--text-xs); font-weight: 700; text-transform: uppercase; letter-spacing: 0.06em; color: var(--slate-fog); font-family: var(--font-heading); }
  .error { color: var(--healthy-coral-dk); font-size: var(--text-sm); margin-bottom: var(--space-3); }
  .warning { color: #7A5A00; background: #FFF8E7; border: 1px solid #F0C040; border-radius: var(--radius-md); padding: var(--space-2) var(--space-3); font-size: var(--text-sm); margin-bottom: var(--space-3); }
  .empty { color: var(--slate-fog); font-style: italic; font-size: var(--text-sm); }
  .loading { color: var(--slate-fog); font-size: var(--text-sm); }

  /* ── Cards / forms ───────────────────────────────────── */
  .card { background: #fff; border: 1px solid var(--pearl-mist-dk); border-radius: var(--radius-lg); padding: var(--space-6); box-shadow: var(--shadow-sm); }
  .form-card { margin-bottom: var(--space-5); }
  .form-row { display: flex; gap: var(--space-4); flex-wrap: wrap; margin-bottom: var(--space-4); }
  .form-actions { display: flex; gap: var(--space-3); align-items: center; margin-top: var(--space-4); }

  /* ── Fields ──────────────────────────────────────────── */
  .field { display: flex; flex-direction: column; gap: var(--space-1); flex: 1; min-width: 160px; }
  .field label { font-size: var(--text-xs); font-weight: 600; color: var(--abyss-navy); font-family: var(--font-body); }

  /* ── Search ──────────────────────────────────────────── */
  .search-bar { display: flex; gap: var(--space-3); align-items: flex-start; margin-bottom: var(--space-4); flex-wrap: wrap; }
  .search-field-wrap { display: flex; flex-direction: column; flex: 1; min-width: 200px; }
  .search-hint { font-size: var(--text-xs); color: var(--slate-fog); margin-top: var(--space-1); font-family: var(--font-body); margin-bottom: 0; }
  .search-input {
    min-height: 44px; padding: var(--space-2) var(--space-3);
    border: 1.5px solid var(--pearl-mist-dk); border-radius: var(--radius-md);
    font-size: var(--text-sm); font-family: var(--font-body); flex: 1; min-width: 200px; width: 100%;
    transition: border-color var(--transition-fast);
  }
  .search-input:focus { outline: none; border-color: var(--caribbean-teal); box-shadow: 0 0 0 3px rgba(0,139,153,0.15); }
  .archived-toggle { display: flex; align-items: center; gap: var(--space-2); font-size: var(--text-sm); color: var(--slate-fog); cursor: pointer; white-space: nowrap; }

  /* ── Patient list ────────────────────────────────────── */
  .patient-list { display: flex; flex-direction: column; gap: var(--space-2); }
  .patient-card { border: 1px solid var(--pearl-mist-dk); border-radius: var(--radius-lg); overflow: hidden; background: #fff; box-shadow: var(--shadow-sm); }
  .patient-card.archived { opacity: 0.65; }
  .patient-row { display: flex; justify-content: space-between; align-items: center; padding: var(--space-4) var(--space-5); cursor: pointer; user-select: none; transition: background var(--transition-fast); }
  .patient-row:hover { background: var(--pearl-mist); }
  .patient-info { display: flex; align-items: center; gap: var(--space-3); flex-wrap: wrap; }
  .patient-name { font-weight: 600; font-size: var(--text-sm); font-family: var(--font-heading); color: var(--abyss-navy); }
  .meta { font-size: var(--text-xs); color: var(--slate-fog); }
  .badge { font-size: var(--text-xs); padding: 2px var(--space-2); border-radius: var(--radius-pill); font-weight: 600; font-family: var(--font-heading); }
  .archived-badge { background: var(--pearl-mist-dk); color: var(--slate-fog); }
  .chevron { color: var(--slate-fog); font-size: var(--text-xs); }

  /* ── Detail panel ────────────────────────────────────── */
  .detail-panel { border-top: 1px solid var(--pearl-mist-dk); padding: var(--space-4) var(--space-5); }
  .detail-section { margin-bottom: var(--space-5); }
  .detail-section:last-child { margin-bottom: 0; }
  .section-title-row { display: flex; align-items: center; gap: var(--space-4); margin-bottom: var(--space-2); }

  /* Info list (dl) */
  .info-list { display: grid; grid-template-columns: 130px 1fr; gap: var(--space-1) var(--space-4); margin: var(--space-2) 0 0; font-size: var(--text-sm); }
  dt { color: var(--slate-fog); font-weight: 500; }
  dd { margin: 0; color: var(--abyss-navy); }

  /* Edit grid */
  .edit-grid { display: flex; flex-wrap: wrap; gap: var(--space-3); margin-top: var(--space-2); }
  .edit-grid .field { min-width: 150px; }
  .edit-actions { display: flex; gap: var(--space-2); margin-top: var(--space-4); }

  /* Notes */
  .empty-notes { color: var(--slate-fog); font-size: var(--text-sm); font-style: italic; margin: var(--space-1) 0 var(--space-4); }
  .notes-list { list-style: none; margin: var(--space-2) 0 var(--space-4); padding: 0; display: flex; flex-direction: column; gap: var(--space-2); }
  .note-item { background: var(--pearl-mist); border-radius: var(--radius-sm); padding: var(--space-3) var(--space-4); }
  .note-text { margin: 0 0 var(--space-1); font-size: var(--text-sm); color: var(--abyss-navy); }
  .note-meta { font-size: var(--text-xs); color: var(--slate-fog); }
  .note-form { display: flex; gap: var(--space-2); align-items: flex-start; margin-top: var(--space-2); }
  .note-input { flex: 1; }

  /* Archive section */
  .archive-section { border-top: 1px solid var(--pearl-mist-dk); padding-top: var(--space-4); }

  /* ── Buttons ─────────────────────────────────────────── */
  .btn-primary {
    display: inline-flex; align-items: center; min-height: 44px; padding: 0 var(--space-5);
    background: var(--caribbean-teal); color: #fff; border: none;
    border-radius: var(--radius-md); font-family: var(--font-heading); font-size: var(--text-sm);
    font-weight: 600; cursor: pointer; white-space: nowrap;
    transition: background var(--transition-fast);
  }
  .btn-primary:hover:not(:disabled) { background: var(--caribbean-teal-dk); }
  .btn-primary:disabled { opacity: 0.45; cursor: not-allowed; }

  .btn-secondary {
    display: inline-flex; align-items: center; min-height: 44px; padding: 0 var(--space-5);
    background: transparent; color: var(--caribbean-teal); border: 1.5px solid var(--caribbean-teal);
    border-radius: var(--radius-md); font-family: var(--font-heading); font-size: var(--text-sm);
    font-weight: 600; cursor: pointer; white-space: nowrap;
    transition: background var(--transition-fast);
  }
  .btn-secondary:hover { background: var(--caribbean-teal-lt); }

  .btn-sm {
    display: inline-flex; align-items: center; min-height: 36px; padding: 0 var(--space-4);
    background: var(--caribbean-teal); color: #fff; border: none;
    border-radius: var(--radius-md); font-family: var(--font-heading); font-size: var(--text-xs);
    font-weight: 600; cursor: pointer; white-space: nowrap;
    transition: background var(--transition-fast);
  }
  .btn-sm:disabled { opacity: 0.45; cursor: not-allowed; }
  .btn-sm.btn-ghost { background: transparent; color: var(--slate-fog); border: 1.5px solid var(--pearl-mist-dk); }
  .btn-sm.btn-ghost:hover { background: var(--pearl-mist); color: var(--abyss-navy); border-color: var(--abyss-navy); }

  .btn-danger-sm {
    display: inline-flex; align-items: center; min-height: 36px; padding: 0 var(--space-4);
    background: transparent; color: var(--healthy-coral-dk); border: 1.5px solid var(--healthy-coral);
    border-radius: var(--radius-md); font-family: var(--font-heading); font-size: var(--text-xs);
    font-weight: 600; cursor: pointer;
    transition: background var(--transition-fast);
  }
  .btn-danger-sm:hover { background: var(--healthy-coral-lt); }
</style>
