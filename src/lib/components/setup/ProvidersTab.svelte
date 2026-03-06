<script lang="ts">
  import { commands, type StaffMemberDto, type OfficeDto } from "$lib/bindings";
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import { toast } from "$lib/stores/toast";
  import { confirm } from "$lib/stores/confirm";
  import { formatDate } from "$lib/utils/date";

  const DAYS = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
  const CLINICAL_SPECIALIZATIONS = ["Dentist", "Hygienist", "Specialist"];

  let providers = $state<StaffMemberDto[]>([]);
  let offices = $state<OfficeDto[]>([]);
  let error = $state<string | null>(null);
  let expandedId = $state<string | null>(null);

  // Per-provider action errors
  let actionError = $state<Record<string, string>>({});

  // Loading states
  let savingSpec = $state<Record<string, boolean>>({});
  let assigningOffice = $state<Record<string, boolean>>({});
  let addingException = $state<Record<string, boolean>>({});

  // Brief success flash for availability saves: "smId:officeId:day"
  let savedAvailKeys = $state<Set<string>>(new Set());

  // Availability inputs: smId → officeId → day → { start, end }
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
    const p = providers.find((p) => p.staff_member_id === id);
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

  async function changeSpecialization(provider: StaffMemberDto, spec: string) {
    if (spec === provider.clinical_specialization) return;
    savingSpec = { ...savingSpec, [provider.staff_member_id]: true };
    const r = await commands.setProviderType(provider.staff_member_id, spec);
    savingSpec = { ...savingSpec, [provider.staff_member_id]: false };
    if (r.status === "ok") {
      providers = providers.map((p) => p.staff_member_id === provider.staff_member_id ? r.data : p);
      toast.success(`${provider.name} clinical specialization set to ${spec}.`);
    } else { actionError = { ...actionError, [provider.staff_member_id]: r.error }; }
  }

  async function assignOffice(provider: StaffMemberDto, officeId: string) {
    const key = `${provider.staff_member_id}:${officeId}`;
    assigningOffice = { ...assigningOffice, [key]: true };
    const r = await commands.assignProviderToOffice(provider.staff_member_id, officeId);
    assigningOffice = { ...assigningOffice, [key]: false };
    if (r.status === "ok") {
      providers = providers.map((p) => p.staff_member_id === provider.staff_member_id ? r.data : p);
      toast.success(`${provider.name} assigned to ${officeName(officeId)}.`);
      const existing = availInputs[provider.staff_member_id] ?? {};
      const byDay: Record<string, { start: string; end: string }> = {};
      for (const day of DAYS) byDay[day] = { start: "", end: "" };
      availInputs = { ...availInputs, [provider.staff_member_id]: { ...existing, [officeId]: byDay } };
    } else { actionError = { ...actionError, [provider.staff_member_id]: r.error }; }
  }

  async function removeOffice(provider: StaffMemberDto, officeId: string) {
    const ok = await confirm({ title: "Remove office assignment", message: `Remove ${provider.name} from ${officeName(officeId)}? This will also clear availability for that office.`, confirmLabel: "Remove", destructive: true });
    if (!ok) return;
    const r = await commands.removeProviderFromOffice(provider.staff_member_id, officeId);
    if (r.status === "ok") {
      providers = providers.map((p) => p.staff_member_id === provider.staff_member_id ? r.data : p);
      const existing = { ...(availInputs[provider.staff_member_id] ?? {}) };
      delete existing[officeId];
      availInputs = { ...availInputs, [provider.staff_member_id]: existing };
      toast.success(`Removed ${provider.name} from ${officeName(officeId)}.`);
    } else { actionError = { ...actionError, [provider.staff_member_id]: r.error }; }
  }

  function flashAvailSaved(smId: string, oid: string, day: string) {
    const key = `${smId}:${oid}:${day}`;
    savedAvailKeys = new Set([...savedAvailKeys, key]);
    setTimeout(() => { savedAvailKeys = new Set([...savedAvailKeys].filter((k) => k !== key)); }, 2000);
  }

  async function setAvail(staffMemberId: string, officeId: string, day: string) {
    const inp = availInputs[staffMemberId]?.[officeId]?.[day];
    if (!inp?.start || !inp?.end) return;
    const r = await commands.setProviderAvailability(staffMemberId, officeId, day, inp.start, inp.end);
    if (r.status === "ok") {
      providers = providers.map((p) => p.staff_member_id === staffMemberId ? r.data : p);
      flashAvailSaved(staffMemberId, officeId, day);
    } else actionError = { ...actionError, [staffMemberId]: r.error };
  }

  async function clearAvail(staffMemberId: string, officeId: string, day: string) {
    const r = await commands.clearProviderAvailability(staffMemberId, officeId, day);
    if (r.status === "ok") {
      providers = providers.map((p) => p.staff_member_id === staffMemberId ? r.data : p);
      const existing = availInputs[staffMemberId] ?? {};
      const byDay = { ...(existing[officeId] ?? {}) };
      byDay[day] = { start: "", end: "" };
      availInputs = { ...availInputs, [staffMemberId]: { ...existing, [officeId]: byDay } };
    } else { actionError = { ...actionError, [staffMemberId]: r.error }; }
  }

  async function checkAvail(smId: string, oid: string, day: string, checked: boolean) {
    if (!checked) { await clearAvail(smId, oid, day); return; }
    const existing = availInputs[smId] ?? {};
    const byOffice = existing[oid] ?? {};
    availInputs = { ...availInputs, [smId]: { ...existing, [oid]: { ...byOffice, [day]: { start: "08:00", end: "17:00" } } } };
    const r = await commands.setProviderAvailability(smId, oid, day, "08:00", "17:00");
    if (r.status === "ok") {
      providers = providers.map((p) => p.staff_member_id === smId ? r.data : p);
      flashAvailSaved(smId, oid, day);
    } else actionError = { ...actionError, [smId]: r.error };
  }

  async function blurAvail(smId: string, oid: string, day: string) {
    const inp = availInputs[smId]?.[oid]?.[day];
    if (inp?.start && inp?.end) await setAvail(smId, oid, day);
  }

  function getAvailInput(smId: string, oid: string, day: string, field: "start" | "end"): string {
    return availInputs[smId]?.[oid]?.[day]?.[field] ?? "";
  }

  function setAvailInput(smId: string, oid: string, day: string, field: "start" | "end", val: string) {
    const existing = availInputs[smId] ?? {};
    const byOffice = existing[oid] ?? {};
    const byDay = byOffice[day] ?? { start: "", end: "" };
    availInputs = { ...availInputs, [smId]: { ...existing, [oid]: { ...byOffice, [day]: { ...byDay, [field]: val } } } };
  }

  function hasAvail(provider: StaffMemberDto, officeId: string, day: string) {
    return provider.availability.some((a) => a.office_id === officeId && a.day_of_week === day);
  }

  async function addException(smId: string) {
    const f = excForm[smId];
    if (!f?.start || !f?.end) { actionError = { ...actionError, [smId]: "Exception start and end dates are required." }; return; }
    const prov = providers.find((p) => p.staff_member_id === smId);
    addingException = { ...addingException, [smId]: true };
    const r = await commands.setProviderException(smId, f.start, f.end, f.reason || null);
    addingException = { ...addingException, [smId]: false };
    if (r.status === "ok") {
      providers = providers.map((p) => p.staff_member_id === smId ? r.data : p);
      toast.success(`Exception added for ${prov!.name}: ${formatDate(f.start)} → ${formatDate(f.end)}.`);
      excForm = { ...excForm, [smId]: { start: "", end: "", reason: "" } };
    } else { actionError = { ...actionError, [smId]: r.error }; }
  }

  async function removeException(smId: string, start: string, end: string) {
    const prov = providers.find((p) => p.staff_member_id === smId);
    const ok = await confirm({
      title: "Remove exception",
      message: `Remove ${prov!.name}'s exception ${formatDate(start)} → ${formatDate(end)}?`,
      confirmLabel: "Remove",
      destructive: true,
    });
    if (!ok) return;
    const r = await commands.removeProviderException(smId, start, end);
    if (r.status === "ok") {
      providers = providers.map((p) => p.staff_member_id === smId ? r.data : p);
      toast.success(`Exception removed for ${prov!.name}: ${formatDate(start)} → ${formatDate(end)}.`);
    } else { actionError = { ...actionError, [smId]: r.error }; }
  }

  async function toggleArchive(provider: StaffMemberDto) {
    const action = provider.archived ? "unarchive" : "archive";
    const capitalisedAction = action.charAt(0).toUpperCase() + action.slice(1);
    const ok = await confirm({
      title: `${capitalisedAction} ${provider.name}`,
      message: `${capitalisedAction} ${provider.name}?`,
      confirmLabel: capitalisedAction,
      destructive: !provider.archived,
    });
    if (!ok) return;
    const r = provider.archived
      ? await commands.unarchiveStaffMember(provider.staff_member_id)
      : await commands.archiveStaffMember(provider.staff_member_id);
    if (r.status === "ok") {
      providers = providers.map((p) => p.staff_member_id === provider.staff_member_id ? r.data : p);
      toast.success(`${provider.name} ${action}d.`);
    } else { actionError = { ...actionError, [provider.staff_member_id]: r.error }; }
  }

  let unassignedOffices = $derived((provider: StaffMemberDto) =>
    offices.filter((o) => !provider.office_ids.includes(o.id))
  );
</script>

<div>
  <div class="section-header">
    <h2>Providers</h2>
  </div>

  {#if error}<p class="error">{error}</p>{/if}

  {#if providers.length === 0}
    <div class="empty-state-block">
      <p class="empty">No providers configured yet.</p>
      <div class="providers-tip">
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" width="16" height="16"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
        <div class="providers-tip-body">
          <span>Assign the <strong>Provider</strong> role to a staff member, then configure their clinical settings here.</span>
          <button class="btn-tip-cta" onclick={() => goto('/staff')}>Go to Staff</button>
        </div>
      </div>
    </div>
  {/if}

  <div class="provider-list">
    {#each providers as provider (provider.staff_member_id)}
      <div class="provider-card" class:archived={provider.archived}>
        <div class="provider-row" role="button" tabindex="0"
          aria-expanded={expandedId === provider.staff_member_id}
          aria-label="Expand {provider.name} details"
          onclick={() => toggleExpand(provider.staff_member_id)}
          onkeydown={(e) => e.key === "Enter" && toggleExpand(provider.staff_member_id)}>
          <div class="provider-info">
            <span class="provider-name">{provider.name}</span>
            {#if provider.clinical_specialization}
              <span class="badge type-badge">{provider.clinical_specialization}</span>
            {:else}
              <span class="badge unset-badge">Specialization not set</span>
            {/if}
            {#if provider.office_ids.length > 0}
              <span class="meta">{provider.office_ids.map(officeName).join(", ")}</span>
            {:else}
              <span class="meta muted">No offices assigned</span>
            {/if}
            {#if provider.archived}<span class="badge archived-badge">Archived</span>{/if}
          </div>
          <span class="chevron">{expandedId === provider.staff_member_id ? "▲" : "▼"}</span>
        </div>

        {#if expandedId === provider.staff_member_id}
          <div class="provider-detail">
            {#if actionError[provider.staff_member_id]}
              <p class="error">{actionError[provider.staff_member_id]}</p>
            {/if}

            <!-- Specialization / Archive -->
            <div class="detail-row">
              <div class="field" style="max-width:180px">
                <label for="prov-type-{provider.staff_member_id}">Clinical Specialization</label>
                <select id="prov-type-{provider.staff_member_id}"
                  disabled={savingSpec[provider.staff_member_id] ?? false}
                  value={provider.clinical_specialization ?? ""}
                  onchange={(e) => changeSpecialization(provider, (e.target as HTMLSelectElement).value)}
                >
                  <option value="" disabled>— Select —</option>
                  {#each CLINICAL_SPECIALIZATIONS as t}<option>{t}</option>{/each}
                </select>
                {#if savingSpec[provider.staff_member_id]}
                  <span class="saving-hint">Saving…</span>
                {/if}
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
                  <button class="chip-remove" onclick={() => removeOffice(provider, oid)} aria-label="Remove {officeName(oid)} from {provider.name}">✕</button>
                </span>
              {/each}
              {#each unassignedOffices(provider) as office}
                {@const assignKey = `${provider.staff_member_id}:${office.id}`}
                <button class="chip chip-add" disabled={assigningOffice[assignKey] ?? false} onclick={() => assignOffice(provider, office.id)}>
                  {assigningOffice[assignKey] ? "Adding…" : `+ ${office.name}`}
                </button>
              {/each}
              {#if offices.length === 0}
                <span class="muted">Create offices first.</span>
              {/if}
            </div>

            <!-- Availability per office -->
            {#if provider.office_ids.length > 0}
              <h4>Weekly Availability</h4>
              <p class="hours-hint">Check a day to mark availability. Edit times and click away to save.</p>
              {#each provider.office_ids as oid}
                <div class="avail-section">
                  <div class="avail-office-label">{officeName(oid)}</div>
                  <div class="hours-grid">
                    <div class="hours-header">Day</div>
                    <div class="hours-header">Start</div>
                    <div class="hours-header">End</div>
                    <div></div>
                    {#each DAYS as day}
                      {@const active = hasAvail(provider, oid, day)}
                      {@const savedKey = `${provider.staff_member_id}:${oid}:${day}`}
                      <label class="day-label" class:day-open={active}>
                        <input type="checkbox" class="day-checkbox" checked={active}
                          onchange={(e) => checkAvail(provider.staff_member_id, oid, day, (e.target as HTMLInputElement).checked)} />
                        {day}
                      </label>
                      {#if active}
                        <input class="time-input" type="time"
                          aria-label="Start time for {day} at {officeName(oid)}"
                          value={getAvailInput(provider.staff_member_id, oid, day, "start")}
                          oninput={(e) => setAvailInput(provider.staff_member_id, oid, day, "start", (e.target as HTMLInputElement).value)}
                          onblur={() => blurAvail(provider.staff_member_id, oid, day)}
                        />
                        <input class="time-input" type="time"
                          aria-label="End time for {day} at {officeName(oid)}"
                          value={getAvailInput(provider.staff_member_id, oid, day, "end")}
                          oninput={(e) => setAvailInput(provider.staff_member_id, oid, day, "end", (e.target as HTMLInputElement).value)}
                          onblur={() => blurAvail(provider.staff_member_id, oid, day)}
                        />
                        {#if savedAvailKeys.has(savedKey)}
                          <span class="avail-saved" aria-label="Saved">
                            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="16" height="16" aria-hidden="true"><polyline points="20 6 9 17 4 12"/></svg>
                          </span>
                        {:else}
                          <div></div>
                        {/if}
                      {:else}
                        <div></div>
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
                    <button class="btn-sm btn-ghost" onclick={() => removeException(provider.staff_member_id, exc.start_date, exc.end_date)} aria-label="Remove {provider.name}'s exception {formatDate(exc.start_date)} to {formatDate(exc.end_date)}">✕</button>
                  </div>
                {/each}
              </div>
            {/if}
            <form class="exc-form" onsubmit={(e) => { e.preventDefault(); addException(provider.staff_member_id); }}>
              <div class="exc-field">
                <label for="exc-start-{provider.staff_member_id}" class="exc-label">From</label>
                <input id="exc-start-{provider.staff_member_id}" type="date" bind:value={excForm[provider.staff_member_id].start} />
              </div>
              <div class="exc-field">
                <label for="exc-end-{provider.staff_member_id}" class="exc-label">To</label>
                <input id="exc-end-{provider.staff_member_id}" type="date" bind:value={excForm[provider.staff_member_id].end} />
              </div>
              <div class="exc-field exc-reason">
                <label for="exc-reason-{provider.staff_member_id}" class="exc-label">Reason</label>
                <input id="exc-reason-{provider.staff_member_id}" placeholder="e.g. Holiday" bind:value={excForm[provider.staff_member_id].reason} />
              </div>
              <button type="submit" class="btn-sm exc-submit" disabled={addingException[provider.staff_member_id] ?? false}>
                {addingException[provider.staff_member_id] ? "Adding…" : "Add"}
              </button>
            </form>
          </div>
        {/if}
      </div>
    {/each}
  </div>
</div>

<style>
  .section-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--space-4); }
  h2 { margin: 0; font-size: var(--text-xl); font-family: var(--font-heading); font-weight: 600; color: var(--abyss-navy); }
  h4 { margin: var(--space-4) 0 var(--space-2); font-size: var(--text-xs); font-weight: 700; color: var(--slate-fog); text-transform: uppercase; letter-spacing: 0.04em; font-family: var(--font-body); }
  .error { color: var(--healthy-coral-dk); font-size: var(--text-sm); margin-bottom: var(--space-2); }
  .empty, .muted { color: var(--slate-fog); font-size: var(--text-sm); font-style: italic; }
  .saving-hint { font-size: var(--text-xs); color: var(--slate-fog); font-style: italic; font-family: var(--font-body); }
  .empty-state-block { display: flex; flex-direction: column; gap: var(--space-3); }
  .providers-tip {
    display: flex;
    align-items: flex-start;
    gap: var(--space-2);
    padding: var(--space-3);
    background: var(--caribbean-teal-lt);
    border: 1px solid var(--caribbean-teal);
    border-radius: var(--radius-sm);
    font-size: var(--text-xs);
    color: var(--abyss-navy);
    font-family: var(--font-body);
    line-height: 1.5;
  }
  .providers-tip svg { flex-shrink: 0; color: var(--caribbean-teal); margin-top: 1px; }
  .providers-tip strong { font-weight: 700; }
  .providers-tip-body { display: flex; flex-direction: column; gap: var(--space-2); }
  .btn-tip-cta {
    align-self: flex-start;
    padding: var(--space-1) var(--space-3);
    background: var(--caribbean-teal);
    color: white;
    border: none;
    border-radius: var(--radius-sm);
    font-size: var(--text-xs);
    font-family: var(--font-body);
    font-weight: 600;
    cursor: pointer;
    min-height: 44px;
    transition: background var(--transition-fast);
  }
  .btn-tip-cta:hover { background: var(--caribbean-teal-dk, var(--caribbean-teal)); }

  .field { display: flex; flex-direction: column; gap: var(--space-1); flex: 1; }
  .field label { font-size: var(--text-xs); font-weight: 600; color: var(--abyss-navy); font-family: var(--font-body); }
  select {
    padding: var(--space-2) var(--space-3); border: 1px solid var(--pearl-mist-dk); border-radius: var(--radius-sm);
    font-size: var(--text-sm); font-family: var(--font-body); width: 100%; box-sizing: border-box;
    background: white; cursor: pointer;
  }
  select:disabled { opacity: 0.6; cursor: not-allowed; }
  select:focus { outline: none; border-color: var(--caribbean-teal); }

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
  .type-badge { background: var(--color-provider-type-lt); color: var(--color-provider-type); }
  .unset-badge { background: var(--pearl-mist); color: var(--slate-fog); }
  .archived-badge { background: var(--color-archived-lt); color: var(--color-archived); }
  .chevron { color: var(--slate-fog); font-size: var(--text-xs); }

  .provider-detail { padding: 0 var(--space-4) var(--space-4); border-top: 1px solid var(--pearl-mist-dk); }
  .detail-row { display: flex; gap: var(--space-4); align-items: flex-start; margin-top: var(--space-3); }
  .detail-row select {
    padding: var(--space-2) var(--space-3); border: 1px solid var(--pearl-mist-dk); border-radius: var(--radius-sm);
    font-size: var(--text-sm); font-family: var(--font-body); width: 100%; box-sizing: border-box;
  }

  .office-chips { display: flex; flex-wrap: wrap; gap: var(--space-2); }
  .chip {
    display: flex; align-items: center; gap: var(--space-1);
    padding: 3px var(--space-3); background: var(--color-provider-type-lt); border-radius: var(--radius-full);
    font-size: var(--text-xs); color: var(--color-provider-type); font-weight: 600; font-family: var(--font-body);
  }
  .chip-remove {
    background: none; border: none; cursor: pointer; color: var(--color-provider-type);
    font-size: var(--text-sm); padding: 0; line-height: 1;
    min-width: 44px; min-height: 44px;
    display: inline-flex; align-items: center; justify-content: center;
  }
  .chip-add { background: white; border: 1px dashed var(--pearl-mist-dk); color: var(--slate-fog); cursor: pointer; font-family: var(--font-body); min-height: 44px; }
  .chip-add:hover:not(:disabled) { border-color: var(--abyss-navy); color: var(--abyss-navy); }
  .chip-add:disabled { opacity: 0.6; cursor: not-allowed; }

  .hours-hint { font-size: var(--text-xs); color: var(--slate-fog); margin: 0 0 var(--space-2); }
  .avail-section { margin-bottom: var(--space-3); }
  .avail-office-label { font-size: var(--text-xs); font-weight: 700; color: var(--abyss-navy); margin-bottom: var(--space-1); font-family: var(--font-body); text-transform: uppercase; letter-spacing: 0.04em; }
  .hours-grid { display: grid; grid-template-columns: 140px 110px 110px 20px; gap: var(--space-1); align-items: center; }
  .hours-header { font-size: var(--text-xs); font-weight: 600; color: var(--slate-fog); text-transform: uppercase; letter-spacing: 0.04em; font-family: var(--font-body); }
  .day-label { display: flex; align-items: center; gap: var(--space-2); font-size: var(--text-sm); color: var(--slate-fog); cursor: pointer; user-select: none; font-family: var(--font-body); }
  .day-label.day-open { font-weight: 600; color: var(--abyss-navy); }
  .day-checkbox { cursor: pointer; }
  .time-input { padding: var(--space-1) var(--space-2); border: 1px solid var(--pearl-mist-dk); border-radius: var(--radius-sm); font-size: var(--text-sm); font-family: var(--font-body); }
  .avail-saved { display: flex; align-items: center; justify-content: center; color: var(--island-palm); }

  .exception-list { display: flex; flex-direction: column; gap: var(--space-2); margin-bottom: var(--space-2); }
  .exception-item { display: flex; align-items: center; gap: var(--space-2); font-size: var(--text-sm); font-family: var(--font-body); color: var(--abyss-navy); }
  .exc-form { display: flex; gap: var(--space-2); align-items: flex-end; flex-wrap: wrap; }
  .exc-field { display: flex; flex-direction: column; gap: 2px; }
  .exc-field.exc-reason { flex: 1; }
  .exc-label { font-size: var(--text-xs); font-weight: 600; color: var(--slate-fog); font-family: var(--font-body); }
  .exc-form input[type="date"] { padding: var(--space-1) var(--space-2); border: 1px solid var(--pearl-mist-dk); border-radius: var(--radius-sm); font-size: var(--text-sm); font-family: var(--font-body); }
  .exc-form input:not([type="date"]) { padding: var(--space-1) var(--space-2); border: 1px solid var(--pearl-mist-dk); border-radius: var(--radius-sm); font-size: var(--text-sm); font-family: var(--font-body); width: 100%; }
  .exc-submit { align-self: flex-end; }

  .btn-sm {
    display: inline-flex; align-items: center; min-height: 44px; padding: 0 var(--space-3);
    background: var(--caribbean-teal); color: white; border: none;
    border-radius: var(--radius-sm); font-size: var(--text-xs); font-family: var(--font-body); font-weight: 600; cursor: pointer;
    transition: background var(--transition-fast);
  }
  .btn-sm:disabled { opacity: 0.6; cursor: not-allowed; }
  .btn-sm.btn-ghost { background: var(--pearl-mist); color: var(--slate-fog); border: 1px solid var(--pearl-mist-dk); }
  .btn-sm.btn-ghost:hover { background: var(--pearl-mist-dk); color: var(--abyss-navy); }
  .btn-danger-sm {
    display: inline-flex; align-items: center; min-height: 44px; padding: 0 var(--space-3);
    background: white; color: var(--healthy-coral-dk);
    border: 1px solid var(--healthy-coral); border-radius: var(--radius-md);
    font-size: var(--text-xs); font-family: var(--font-body); font-weight: 600; cursor: pointer;
    transition: background var(--transition-fast);
  }
  .btn-danger-sm:hover { background: var(--healthy-coral-lt); }
</style>
