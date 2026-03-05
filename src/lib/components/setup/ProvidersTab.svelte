<script lang="ts">
  import { commands, type ProviderDto, type OfficeDto } from "$lib/bindings";
  import { onMount } from "svelte";
  import { toast } from "$lib/stores/toast";
  import { confirm } from "$lib/stores/confirm";
  import { formatDate } from "$lib/utils/date";

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
    const ok = await confirm({ title: "Remove office assignment", message: `Remove ${provider.name} from ${officeName(officeId)}? This will also clear availability for that office.`, confirmLabel: "Remove", destructive: true });
    if (!ok) return;
    const r = await commands.removeProviderFromOffice(provider.id, officeId);
    if (r.status === "ok") {
      providers = providers.map((p) => p.id === provider.id ? r.data : p);
      const existing = { ...(availInputs[provider.id] ?? {}) };
      delete existing[officeId];
      availInputs = { ...availInputs, [provider.id]: existing };
      toast.success(`Removed ${provider.name} from ${officeName(officeId)}.`);
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

  async function checkAvail(pid: string, oid: string, day: string, checked: boolean) {
    if (!checked) { await clearAvail(pid, oid, day); return; }
    const existing = availInputs[pid] ?? {};
    const byOffice = existing[oid] ?? {};
    availInputs = { ...availInputs, [pid]: { ...existing, [oid]: { ...byOffice, [day]: { start: "08:00", end: "17:00" } } } };
    const r = await commands.setProviderAvailability(pid, oid, day, "08:00", "17:00");
    if (r.status === "ok") providers = providers.map((p) => p.id === pid ? r.data : p);
    else actionError = { ...actionError, [pid]: r.error };
  }

  async function blurAvail(pid: string, oid: string, day: string) {
    const inp = availInputs[pid]?.[oid]?.[day];
    if (inp?.start && inp?.end) await setAvail(pid, oid, day);
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
    const ok = await confirm({
      title: `${action.charAt(0).toUpperCase() + action.slice(1)} provider`,
      message: `${action.charAt(0).toUpperCase() + action.slice(1)} ${provider.name}?`,
      confirmLabel: action.charAt(0).toUpperCase() + action.slice(1),
      destructive: !provider.archived,
    });
    if (!ok) return;
    const r = provider.archived
      ? await commands.unarchiveProvider(provider.id)
      : await commands.archiveProvider(provider.id);
    if (r.status === "ok") {
      providers = providers.map((p) => p.id === provider.id ? r.data : p);
      toast.success(`${provider.name} ${action}d.`);
    } else { actionError = { ...actionError, [provider.id]: r.error }; }
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
          <label for="new-provider-name">Name</label>
          <input id="new-provider-name" bind:value={newName} placeholder="Dr. Brown" />
        </div>
        <div class="field" style="max-width:150px">
          <label for="new-provider-type">Type</label>
          <select id="new-provider-type" bind:value={newType}>
            {#each PROVIDER_TYPES as t}<option>{t}</option>{/each}
          </select>
        </div>
        <div class="field" style="justify-content:flex-end; padding-top:1.4rem">
          <button type="submit" class="btn-primary" disabled={registering}>
            {#if registering}<span class="spinner" aria-hidden="true"></span><span class="sr-only">Registering</span>{:else}Register{/if}
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
          aria-expanded={expandedId === provider.id}
          aria-label="Expand {provider.name} details"
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
                <label for="prov-name-{provider.id}">Name</label>
                <input id="prov-name-{provider.id}"
                  value={provider.name}
                  onblur={(e) => rename(provider, (e.target as HTMLInputElement).value)}
                  onkeydown={(e) => e.key === "Enter" && rename(provider, (e.target as HTMLInputElement).value)}
                />
              </div>
              <div class="field" style="max-width:150px">
                <label for="prov-type-{provider.id}">Type</label>
                <select id="prov-type-{provider.id}"
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
                  <button class="chip-remove" onclick={() => removeOffice(provider, oid)} aria-label="Remove {officeName(oid)}">✕</button>
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
              <p class="hours-hint">Check a day to mark availability. Edit times and tab away to save.</p>
              {#each provider.office_ids as oid}
                <div class="avail-section">
                  <div class="avail-office-label">{officeName(oid)}</div>
                  <div class="hours-grid">
                    <div class="hours-header">Day</div>
                    <div class="hours-header">Start</div>
                    <div class="hours-header">End</div>
                    {#each DAYS as day}
                      {@const active = hasAvail(provider, oid, day)}
                      <label class="day-label" class:day-open={active}>
                        <input type="checkbox" class="day-checkbox" checked={active}
                          onchange={(e) => checkAvail(provider.id, oid, day, (e.target as HTMLInputElement).checked)} />
                        {day}
                      </label>
                      {#if active}
                        <input class="time-input" type="time"
                          aria-label="Start time for {day} at {officeName(oid)}"
                          value={getAvailInput(provider.id, oid, day, "start")}
                          oninput={(e) => setAvailInput(provider.id, oid, day, "start", (e.target as HTMLInputElement).value)}
                          onblur={() => blurAvail(provider.id, oid, day)}
                        />
                        <input class="time-input" type="time"
                          aria-label="End time for {day} at {officeName(oid)}"
                          value={getAvailInput(provider.id, oid, day, "end")}
                          oninput={(e) => setAvailInput(provider.id, oid, day, "end", (e.target as HTMLInputElement).value)}
                          onblur={() => blurAvail(provider.id, oid, day)}
                        />
                      {:else}
                        <div></div>
                        <div></div>
                      {/if}
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
                    <span>{formatDate(exc.start_date)} → {formatDate(exc.end_date)}</span>
                    {#if exc.reason}<span class="muted">({exc.reason})</span>{/if}
                    <button class="btn-sm btn-ghost" onclick={() => removeException(provider.id, exc.start_date, exc.end_date)} aria-label="Remove exception">✕</button>
                  </div>
                {/each}
              </div>
            {/if}
            <form class="exc-form" onsubmit={(e) => { e.preventDefault(); addException(provider.id); }}>
              <label for="exc-start-{provider.id}" class="sr-only">Exception start date</label>
              <input id="exc-start-{provider.id}" type="date" bind:value={excForm[provider.id].start} />
              <span aria-hidden="true">→</span>
              <label for="exc-end-{provider.id}" class="sr-only">Exception end date</label>
              <input id="exc-end-{provider.id}" type="date" bind:value={excForm[provider.id].end} />
              <label for="exc-reason-{provider.id}" class="sr-only">Reason (optional)</label>
              <input id="exc-reason-{provider.id}" placeholder="Reason (optional)" bind:value={excForm[provider.id].reason} style="flex:1" />
              <button type="submit" class="btn-sm">Add</button>
            </form>
          </div>
        {/if}
      </div>
    {/each}
  </div>
</div>

<style>
  .sr-only { position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px; overflow: hidden; clip: rect(0,0,0,0); white-space: nowrap; border: 0; }
  .section-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--space-4); }
  h2 { margin: 0; font-size: var(--text-xl); font-family: var(--font-heading); font-weight: 600; color: var(--abyss-navy); }
  h4 { margin: var(--space-4) 0 var(--space-2); font-size: var(--text-xs); font-weight: 700; color: var(--slate-fog); text-transform: uppercase; letter-spacing: 0.04em; font-family: var(--font-body); }
  .error { color: var(--healthy-coral-dk); font-size: var(--text-sm); margin-bottom: var(--space-2); }
  .empty, .muted { color: var(--slate-fog); font-size: var(--text-sm); font-style: italic; }

  .create-form {
    background: var(--pearl-mist); border: 1px solid var(--pearl-mist-dk);
    border-radius: var(--radius-md); padding: var(--space-4); margin-bottom: var(--space-4);
  }
  .row { display: flex; gap: var(--space-4); align-items: flex-start; }
  .field { display: flex; flex-direction: column; gap: var(--space-1); flex: 1; }
  .field label { font-size: var(--text-xs); font-weight: 600; color: var(--abyss-navy); font-family: var(--font-body); }
  input:not([type="date"]):not([type="time"]), select {
    padding: var(--space-2) var(--space-3); border: 1px solid var(--pearl-mist-dk); border-radius: var(--radius-sm);
    font-size: var(--text-sm); font-family: var(--font-body); width: 100%; box-sizing: border-box;
  }
  select { background: white; cursor: pointer; }
  input:focus, select:focus { outline: none; border-color: var(--caribbean-teal); }

  .provider-list { display: flex; flex-direction: column; gap: var(--space-3); }
  .provider-card { border: 1px solid var(--pearl-mist-dk); border-radius: var(--radius-md); overflow: hidden; background: white; }
  .provider-card.archived { opacity: 0.6; }

  .provider-row {
    display: flex; justify-content: space-between; align-items: center;
    padding: var(--space-3) var(--space-4); cursor: pointer; user-select: none;
  }
  .provider-row:hover { background: var(--pearl-mist); }
  .provider-info { display: flex; align-items: center; gap: var(--space-2); flex-wrap: wrap; }
  .provider-name { font-weight: 600; font-size: var(--text-sm); color: var(--abyss-navy); font-family: var(--font-body); }
  .meta { font-size: var(--text-xs); color: var(--slate-fog); }
  .badge { font-size: var(--text-xs); padding: 2px var(--space-2); border-radius: var(--radius-full); font-weight: 600; font-family: var(--font-body); }
  .type-badge { background: #e8f0fe; color: #1a5cb3; }
  .archived-badge { background: #f0e6d3; color: #a06030; }
  .chevron { color: var(--slate-fog); font-size: var(--text-xs); }

  .provider-detail { padding: 0 var(--space-4) var(--space-4); border-top: 1px solid var(--pearl-mist-dk); }
  .detail-row { display: flex; gap: var(--space-4); align-items: flex-start; margin-top: var(--space-3); }
  .detail-row input, .detail-row select {
    padding: var(--space-2) var(--space-3); border: 1px solid var(--pearl-mist-dk); border-radius: var(--radius-sm);
    font-size: var(--text-sm); font-family: var(--font-body); width: 100%; box-sizing: border-box;
  }

  .office-chips { display: flex; flex-wrap: wrap; gap: var(--space-2); }
  .chip {
    display: flex; align-items: center; gap: var(--space-1);
    padding: 3px var(--space-3); background: #e8f0fe; border-radius: var(--radius-full);
    font-size: var(--text-xs); color: #1a5cb3; font-weight: 600; font-family: var(--font-body);
  }
  .chip-remove { background: none; border: none; cursor: pointer; color: #1a5cb3; font-size: var(--text-sm); padding: 0; line-height: 1; }
  .chip-add { background: white; border: 1px dashed var(--pearl-mist-dk); color: var(--slate-fog); cursor: pointer; font-family: var(--font-body); }
  .chip-add:hover { border-color: var(--abyss-navy); color: var(--abyss-navy); }

  .hours-hint { font-size: var(--text-xs); color: var(--slate-fog); margin: 0 0 var(--space-2); }
  .avail-section { margin-bottom: var(--space-3); }
  .avail-office-label { font-size: var(--text-xs); font-weight: 700; color: var(--abyss-navy); margin-bottom: var(--space-1); font-family: var(--font-body); text-transform: uppercase; letter-spacing: 0.04em; }
  .hours-grid { display: grid; grid-template-columns: 140px 110px 110px; gap: var(--space-1); align-items: center; }
  .hours-header { font-size: var(--text-xs); font-weight: 600; color: var(--slate-fog); text-transform: uppercase; letter-spacing: 0.04em; font-family: var(--font-body); }
  .day-label { display: flex; align-items: center; gap: var(--space-2); font-size: var(--text-sm); color: var(--slate-fog); cursor: pointer; user-select: none; font-family: var(--font-body); }
  .day-label.day-open { font-weight: 600; color: var(--abyss-navy); }
  .day-checkbox { cursor: pointer; }
  .time-input { padding: var(--space-1) var(--space-2); border: 1px solid var(--pearl-mist-dk); border-radius: var(--radius-sm); font-size: var(--text-sm); font-family: var(--font-body); }

  .exception-list { display: flex; flex-direction: column; gap: var(--space-2); margin-bottom: var(--space-2); }
  .exception-item { display: flex; align-items: center; gap: var(--space-2); font-size: var(--text-sm); font-family: var(--font-body); color: var(--abyss-navy); }
  .exc-form { display: flex; gap: var(--space-2); align-items: center; flex-wrap: wrap; }
  .exc-form input[type="date"] { padding: var(--space-1) var(--space-2); border: 1px solid var(--pearl-mist-dk); border-radius: var(--radius-sm); font-size: var(--text-sm); font-family: var(--font-body); }
  .exc-form input:not([type="date"]) { padding: var(--space-1) var(--space-2); border: 1px solid var(--pearl-mist-dk); border-radius: var(--radius-sm); font-size: var(--text-sm); font-family: var(--font-body); }

  .btn-primary {
    display: inline-flex; align-items: center; min-height: 36px; padding: 0 var(--space-4);
    background: var(--caribbean-teal); color: #fff; border: none;
    border-radius: var(--radius-md); font-family: var(--font-heading); font-size: var(--text-sm);
    font-weight: 600; cursor: pointer; transition: background var(--transition-fast);
  }
  .btn-primary:hover:not(:disabled) { background: var(--caribbean-teal-dk); }
  .btn-primary:disabled { opacity: 0.45; cursor: not-allowed; }
  .btn-sm {
    display: inline-flex; align-items: center; min-height: 28px; padding: 0 var(--space-3);
    background: var(--caribbean-teal); color: white; border: none;
    border-radius: var(--radius-sm); font-size: var(--text-xs); font-family: var(--font-body); font-weight: 600; cursor: pointer;
    transition: background var(--transition-fast);
  }
  .btn-sm.btn-ghost { background: var(--pearl-mist); color: var(--slate-fog); border: 1px solid var(--pearl-mist-dk); }
  .btn-sm.btn-ghost:hover { background: var(--pearl-mist-dk); color: var(--abyss-navy); }
  .btn-danger-sm {
    display: inline-flex; align-items: center; min-height: 32px; padding: 0 var(--space-3);
    background: white; color: var(--healthy-coral-dk);
    border: 1px solid var(--healthy-coral); border-radius: var(--radius-md);
    font-size: var(--text-xs); font-family: var(--font-body); font-weight: 600; cursor: pointer;
    transition: background var(--transition-fast);
  }
  .btn-danger-sm:hover { background: #fef2f0; }
</style>
