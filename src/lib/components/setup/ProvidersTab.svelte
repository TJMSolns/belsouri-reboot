<script lang="ts">
  import { commands, type ProviderDto, type OfficeDto } from "$lib/bindings";
  import { onMount } from "svelte";

  const DAYS = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
  const PROVIDER_TYPES = ["Dentist", "Hygienist", "Specialist"];

  let providers = $state<ProviderDto[]>([]);
  let offices = $state<OfficeDto[]>([]);
  let error = $state<string | null>(null);
  let expandedId = $state<string | null>(null);

  // Register form
  let showRegister = $state(false);
  let newName = $state("");
  let newType = $state("Dentist");
  let registering = $state(false);
  let registerError = $state<string | null>(null);

  // Per-provider action errors
  let actionError = $state<Record<string, string>>({});

  // Availability inputs: provId → officeId → day → { start, end }
  let availInputs = $state<Record<string, Record<string, Record<string, { start: string; end: string }>>>>({});

  // Exception form per provider
  let excForm = $state<Record<string, { start: string; end: string; reason: string }>>({});

  onMount(load);

  async function load() {
    const [pr, or] = await Promise.all([commands.listProviders(), commands.listOffices()]);
    if (pr.status === "ok") providers = pr.data;
    else error = pr.error;
    if (or.status === "ok") offices = or.data.filter((o) => !o.archived);
    else error = or.error;
  }

  function officeName(id: string) {
    return offices.find((o) => o.id === id)?.name ?? id.slice(0, 8);
  }

  function toggleExpand(id: string) {
    if (expandedId === id) { expandedId = null; return; }
    expandedId = id;
    // Seed availability inputs from current state
    const p = providers.find((p) => p.id === id);
    if (!p) return;
    const byOffice: Record<string, Record<string, { start: string; end: string }>> = {};
    for (const offId of p.office_ids) {
      byOffice[offId] = {};
      for (const day of DAYS) {
        const w = p.availability.find((a) => a.office_id === offId && a.day_of_week === day);
        byOffice[offId][day] = { start: w?.start_time ?? "", end: w?.end_time ?? "" };
      }
    }
    availInputs = { ...availInputs, [id]: byOffice };
    if (!excForm[id]) excForm = { ...excForm, [id]: { start: "", end: "", reason: "" } };
  }

  async function register() {
    if (!newName.trim()) { registerError = "Name is required"; return; }
    registering = true; registerError = null;
    const r = await commands.registerProvider(newName.trim(), newType);
    registering = false;
    if (r.status === "ok") {
      providers = [...providers, r.data].sort((a, b) => a.name.localeCompare(b.name));
      newName = ""; showRegister = false;
    } else { registerError = r.error; }
  }

  async function rename(provider: ProviderDto, val: string) {
    if (!val.trim() || val.trim() === provider.name) return;
    const r = await commands.renameProvider(provider.id, val.trim());
    if (r.status === "ok") providers = providers.map((p) => p.id === provider.id ? r.data : p);
    else actionError = { ...actionError, [provider.id]: r.error };
  }

  async function changeType(provider: ProviderDto, type: string) {
    if (type === provider.provider_type) return;
    const r = await commands.changeProviderType(provider.id, type);
    if (r.status === "ok") providers = providers.map((p) => p.id === provider.id ? r.data : p);
    else actionError = { ...actionError, [provider.id]: r.error };
  }

  async function assignOffice(provider: ProviderDto, officeId: string) {
    const r = await commands.assignProviderToOffice(provider.id, officeId);
    if (r.status === "ok") {
      providers = providers.map((p) => p.id === provider.id ? r.data : p);
      // Seed availability inputs for new office
      const existing = availInputs[provider.id] ?? {};
      const byDay: Record<string, { start: string; end: string }> = {};
      for (const day of DAYS) byDay[day] = { start: "", end: "" };
      availInputs = { ...availInputs, [provider.id]: { ...existing, [officeId]: byDay } };
    } else { actionError = { ...actionError, [provider.id]: r.error }; }
  }

  async function removeOffice(provider: ProviderDto, officeId: string) {
    if (!confirm(`Remove ${provider.name} from ${officeName(officeId)}? This will also clear availability for that office.`)) return;
    const r = await commands.removeProviderFromOffice(provider.id, officeId);
    if (r.status === "ok") {
      providers = providers.map((p) => p.id === provider.id ? r.data : p);
      const existing = { ...(availInputs[provider.id] ?? {}) };
      delete existing[officeId];
      availInputs = { ...availInputs, [provider.id]: existing };
    } else { actionError = { ...actionError, [provider.id]: r.error }; }
  }

  async function setAvail(providerId: string, officeId: string, day: string) {
    const inp = availInputs[providerId]?.[officeId]?.[day];
    if (!inp?.start || !inp?.end) return;
    const r = await commands.setProviderAvailability(providerId, officeId, day, inp.start, inp.end);
    if (r.status === "ok") providers = providers.map((p) => p.id === providerId ? r.data : p);
    else actionError = { ...actionError, [providerId]: r.error };
  }

  async function clearAvail(providerId: string, officeId: string, day: string) {
    const r = await commands.clearProviderAvailability(providerId, officeId, day);
    if (r.status === "ok") {
      providers = providers.map((p) => p.id === providerId ? r.data : p);
      const existing = availInputs[providerId] ?? {};
      const byDay = { ...(existing[officeId] ?? {}) };
      byDay[day] = { start: "", end: "" };
      availInputs = { ...availInputs, [providerId]: { ...existing, [officeId]: byDay } };
    } else { actionError = { ...actionError, [providerId]: r.error }; }
  }

  function getAvailInput(pid: string, oid: string, day: string, field: "start" | "end"): string {
    return availInputs[pid]?.[oid]?.[day]?.[field] ?? "";
  }

  function setAvailInput(pid: string, oid: string, day: string, field: "start" | "end", val: string) {
    const existing = availInputs[pid] ?? {};
    const byOffice = existing[oid] ?? {};
    const byDay = byOffice[day] ?? { start: "", end: "" };
    availInputs = { ...availInputs, [pid]: { ...existing, [oid]: { ...byOffice, [day]: { ...byDay, [field]: val } } } };
  }

  function hasAvail(provider: ProviderDto, officeId: string, day: string) {
    return provider.availability.some((a) => a.office_id === officeId && a.day_of_week === day);
  }

  async function addException(pid: string) {
    const f = excForm[pid];
    if (!f?.start || !f?.end) { actionError = { ...actionError, [pid]: "Start and end date are required" }; return; }
    const r = await commands.setProviderException(pid, f.start, f.end, f.reason || null);
    if (r.status === "ok") {
      providers = providers.map((p) => p.id === pid ? r.data : p);
      excForm = { ...excForm, [pid]: { start: "", end: "", reason: "" } };
    } else { actionError = { ...actionError, [pid]: r.error }; }
  }

  async function removeException(pid: string, start: string, end: string) {
    const r = await commands.removeProviderException(pid, start, end);
    if (r.status === "ok") providers = providers.map((p) => p.id === pid ? r.data : p);
    else actionError = { ...actionError, [pid]: r.error };
  }

  async function toggleArchive(provider: ProviderDto) {
    const action = provider.archived ? "unarchive" : "archive";
    if (!confirm(`${action.charAt(0).toUpperCase() + action.slice(1)} ${provider.name}?`)) return;
    const r = provider.archived
      ? await commands.unarchiveProvider(provider.id)
      : await commands.archiveProvider(provider.id);
    if (r.status === "ok") providers = providers.map((p) => p.id === provider.id ? r.data : p);
    else actionError = { ...actionError, [provider.id]: r.error };
  }

  let unassignedOffices = $derived((provider: ProviderDto) =>
    offices.filter((o) => !provider.office_ids.includes(o.id))
  );
</script>

<div>
  <div class="section-header">
    <h2>Providers</h2>
    <button class="btn-primary" onclick={() => (showRegister = !showRegister)}>
      {showRegister ? "Cancel" : "+ Register Provider"}
    </button>
  </div>

  {#if error}<p class="error">{error}</p>{/if}

  {#if showRegister}
    <form class="create-form" onsubmit={(e) => { e.preventDefault(); register(); }}>
      {#if registerError}<p class="error">{registerError}</p>{/if}
      <div class="row">
        <div class="field">
          <label>Name</label>
          <input bind:value={newName} placeholder="Dr. Brown" />
        </div>
        <div class="field" style="max-width:150px">
          <label>Type</label>
          <select bind:value={newType}>
            {#each PROVIDER_TYPES as t}<option>{t}</option>{/each}
          </select>
        </div>
        <div class="field" style="justify-content:flex-end; padding-top:1.4rem">
          <button type="submit" class="btn-primary" disabled={registering}>
            {registering ? "Registering…" : "Register"}
          </button>
        </div>
      </div>
    </form>
  {/if}

  {#if providers.length === 0 && !showRegister}
    <p class="empty">No providers yet.</p>
  {/if}

  <div class="provider-list">
    {#each providers as provider (provider.id)}
      <div class="provider-card" class:archived={provider.archived}>
        <div class="provider-row" role="button" tabindex="0"
          onclick={() => toggleExpand(provider.id)}
          onkeydown={(e) => e.key === "Enter" && toggleExpand(provider.id)}>
          <div class="provider-info">
            <span class="provider-name">{provider.name}</span>
            <span class="badge type-badge">{provider.provider_type}</span>
            {#if provider.office_ids.length > 0}
              <span class="meta">{provider.office_ids.map(officeName).join(", ")}</span>
            {:else}
              <span class="meta muted">No offices assigned</span>
            {/if}
            {#if provider.archived}<span class="badge archived-badge">Archived</span>{/if}
          </div>
          <span class="chevron">{expandedId === provider.id ? "▲" : "▼"}</span>
        </div>

        {#if expandedId === provider.id}
          <div class="provider-detail">
            {#if actionError[provider.id]}
              <p class="error">{actionError[provider.id]}</p>
            {/if}

            <!-- Name / Type -->
            <div class="detail-row">
              <div class="field">
                <label>Name</label>
                <input
                  value={provider.name}
                  onblur={(e) => rename(provider, (e.target as HTMLInputElement).value)}
                  onkeydown={(e) => e.key === "Enter" && rename(provider, (e.target as HTMLInputElement).value)}
                />
              </div>
              <div class="field" style="max-width:150px">
                <label>Type</label>
                <select
                  value={provider.provider_type}
                  onchange={(e) => changeType(provider, (e.target as HTMLSelectElement).value)}
                >
                  {#each PROVIDER_TYPES as t}<option>{t}</option>{/each}
                </select>
              </div>
              <div class="field" style="justify-content:flex-end; padding-top:1.4rem">
                <button
                  class={provider.archived ? "btn-sm" : "btn-danger-sm"}
                  onclick={() => toggleArchive(provider)}
                >{provider.archived ? "Unarchive" : "Archive"}</button>
              </div>
            </div>

            <!-- Office Assignments -->
            <h4>Office Assignments</h4>
            <div class="office-chips">
              {#each provider.office_ids as oid}
                <span class="chip">
                  {officeName(oid)}
                  <button class="chip-remove" onclick={() => removeOffice(provider, oid)}>✕</button>
                </span>
              {/each}
              {#each unassignedOffices(provider) as office}
                <button class="chip chip-add" onclick={() => assignOffice(provider, office.id)}>
                  + {office.name}
                </button>
              {/each}
              {#if offices.length === 0}
                <span class="muted">Create offices first</span>
              {/if}
            </div>

            <!-- Availability per office -->
            {#if provider.office_ids.length > 0}
              <h4>Weekly Availability</h4>
              {#each provider.office_ids as oid}
                <div class="avail-section">
                  <div class="avail-office-label">{officeName(oid)}</div>
                  <div class="hours-grid">
                    <div class="hours-header">Day</div>
                    <div class="hours-header">Start</div>
                    <div class="hours-header">End</div>
                    <div class="hours-header"></div>
                    {#each DAYS as day}
                      {@const active = hasAvail(provider, oid, day)}
                      <div class="day-label" class:day-open={active}>{day}</div>
                      <input class="time-input" type="time"
                        value={getAvailInput(provider.id, oid, day, "start")}
                        oninput={(e) => setAvailInput(provider.id, oid, day, "start", (e.target as HTMLInputElement).value)}
                      />
                      <input class="time-input" type="time"
                        value={getAvailInput(provider.id, oid, day, "end")}
                        oninput={(e) => setAvailInput(provider.id, oid, day, "end", (e.target as HTMLInputElement).value)}
                      />
                      <div class="hours-actions">
                        <button class="btn-sm" onclick={() => setAvail(provider.id, oid, day)}>Set</button>
                        {#if active}
                          <button class="btn-sm btn-ghost" onclick={() => clearAvail(provider.id, oid, day)}>✕</button>
                        {/if}
                      </div>
                    {/each}
                  </div>
                </div>
              {/each}
            {/if}

            <!-- Exceptions -->
            <h4>Date Exceptions (Vacation / Time Off)</h4>
            {#if provider.exceptions.length > 0}
              <div class="exception-list">
                {#each provider.exceptions as exc}
                  <div class="exception-item">
                    <span>{exc.start_date} → {exc.end_date}</span>
                    {#if exc.reason}<span class="muted">({exc.reason})</span>{/if}
                    <button class="btn-sm btn-ghost" onclick={() => removeException(provider.id, exc.start_date, exc.end_date)}>✕</button>
                  </div>
                {/each}
              </div>
            {/if}
            <form class="exc-form" onsubmit={(e) => { e.preventDefault(); addException(provider.id); }}>
              <input type="date" bind:value={excForm[provider.id].start} />
              <span>→</span>
              <input type="date" bind:value={excForm[provider.id].end} />
              <input placeholder="Reason (optional)" bind:value={excForm[provider.id].reason} style="flex:1" />
              <button type="submit" class="btn-sm">Add</button>
            </form>
          </div>
        {/if}
      </div>
    {/each}
  </div>
</div>

<style>
  .section-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem; }
  h2 { margin: 0; font-size: 1.1rem; color: #222; }
  h4 { margin: 1rem 0 0.4rem; font-size: 0.82rem; color: #555; text-transform: uppercase; letter-spacing: 0.04em; }
  .error { color: #c0392b; font-size: 0.875rem; margin-bottom: 0.5rem; }
  .empty, .muted { color: #999; font-size: 0.875rem; font-style: italic; }

  .create-form {
    background: #f7f8fa; border: 1px solid #e0e0e0; border-radius: 8px;
    padding: 1rem; margin-bottom: 1rem;
  }
  .row { display: flex; gap: 1rem; align-items: flex-start; }
  .field { display: flex; flex-direction: column; gap: 0.3rem; flex: 1; }
  .field label { font-size: 0.78rem; font-weight: 600; color: #555; text-transform: uppercase; letter-spacing: 0.03em; }
  input:not([type="date"]):not([type="time"]), select {
    padding: 0.45rem 0.6rem; border: 1px solid #ccc; border-radius: 6px;
    font-size: 0.9rem; font-family: system-ui, sans-serif; width: 100%; box-sizing: border-box;
  }
  select { background: white; cursor: pointer; }
  input:focus, select:focus { outline: none; border-color: #1a1a2e; }

  .provider-list { display: flex; flex-direction: column; gap: 0.75rem; }
  .provider-card { border: 1px solid #ddd; border-radius: 8px; overflow: hidden; background: white; }
  .provider-card.archived { opacity: 0.6; }

  .provider-row {
    display: flex; justify-content: space-between; align-items: center;
    padding: 0.75rem 1rem; cursor: pointer; user-select: none;
  }
  .provider-row:hover { background: #f7f8fa; }
  .provider-info { display: flex; align-items: center; gap: 0.65rem; flex-wrap: wrap; }
  .provider-name { font-weight: 600; font-size: 0.95rem; }
  .meta { font-size: 0.8rem; color: #777; }
  .badge { font-size: 0.72rem; padding: 0.15rem 0.5rem; border-radius: 20px; font-weight: 600; }
  .type-badge { background: #e8f0fe; color: #1a5cb3; }
  .archived-badge { background: #f0e6d3; color: #a06030; }
  .chevron { color: #aaa; font-size: 0.8rem; }

  .provider-detail { padding: 0 1rem 1rem; border-top: 1px solid #eee; }
  .detail-row { display: flex; gap: 1rem; align-items: flex-start; margin-top: 0.75rem; }
  .detail-row input, .detail-row select { padding: 0.4rem 0.6rem; border: 1px solid #ccc; border-radius: 6px; font-size: 0.9rem; width: 100%; box-sizing: border-box; }

  .office-chips { display: flex; flex-wrap: wrap; gap: 0.5rem; }
  .chip {
    display: flex; align-items: center; gap: 0.35rem;
    padding: 0.25rem 0.6rem; background: #e8f0fe; border-radius: 20px;
    font-size: 0.82rem; color: #1a5cb3; font-weight: 600;
  }
  .chip-remove { background: none; border: none; cursor: pointer; color: #1a5cb3; font-size: 0.85rem; padding: 0; line-height: 1; }
  .chip-add { background: white; border: 1px dashed #aaa; color: #555; cursor: pointer; font-family: system-ui, sans-serif; }
  .chip-add:hover { border-color: #1a1a2e; color: #1a1a2e; }

  .avail-section { margin-bottom: 0.75rem; }
  .avail-office-label { font-size: 0.82rem; font-weight: 700; color: #444; margin-bottom: 0.35rem; }
  .hours-grid { display: grid; grid-template-columns: 110px 110px 110px 1fr; gap: 0.3rem; align-items: center; }
  .hours-header { font-size: 0.72rem; font-weight: 600; color: #888; text-transform: uppercase; letter-spacing: 0.04em; }
  .day-label { font-size: 0.85rem; color: #666; }
  .day-label.day-open { font-weight: 600; color: #1a1a2e; }
  .time-input { padding: 0.28rem 0.4rem; border: 1px solid #ccc; border-radius: 5px; font-size: 0.84rem; }
  .hours-actions { display: flex; gap: 0.35rem; }

  .exception-list { display: flex; flex-direction: column; gap: 0.4rem; margin-bottom: 0.5rem; }
  .exception-item { display: flex; align-items: center; gap: 0.5rem; font-size: 0.85rem; }
  .exc-form { display: flex; gap: 0.5rem; align-items: center; flex-wrap: wrap; }
  .exc-form input[type="date"] { padding: 0.3rem 0.5rem; border: 1px solid #ccc; border-radius: 5px; font-size: 0.85rem; }
  .exc-form input:not([type="date"]) { padding: 0.3rem 0.5rem; border: 1px solid #ccc; border-radius: 5px; font-size: 0.85rem; }

  .btn-primary {
    padding: 0.45rem 1.1rem; background: #1a1a2e; color: white;
    border: none; border-radius: 6px; font-size: 0.875rem; cursor: pointer; font-family: system-ui, sans-serif;
  }
  .btn-primary:hover:not(:disabled) { background: #2a2a4e; }
  .btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn-sm { padding: 0.25rem 0.6rem; background: #1a1a2e; color: white; border: none; border-radius: 4px; font-size: 0.78rem; cursor: pointer; font-family: system-ui, sans-serif; }
  .btn-sm.btn-ghost { background: #eee; color: #555; }
  .btn-sm.btn-ghost:hover { background: #ddd; }
  .btn-danger-sm { padding: 0.35rem 0.75rem; background: white; color: #c0392b; border: 1px solid #c0392b; border-radius: 6px; font-size: 0.8rem; cursor: pointer; font-family: system-ui, sans-serif; }
  .btn-danger-sm:hover { background: #fdf0ef; }
</style>
