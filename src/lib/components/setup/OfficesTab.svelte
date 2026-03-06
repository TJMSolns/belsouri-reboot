<script lang="ts">
  import { commands, type OfficeDto } from "$lib/bindings";
  import { onMount } from "svelte";
  import { toast } from "$lib/stores/toast";
  import { confirm } from "$lib/stores/confirm";

  const DAYS = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];

  let offices = $state<OfficeDto[]>([]);
  let error = $state<string | null>(null);
  let expandedId = $state<string | null>(null);

  // Create form
  let showCreate = $state(false);
  let newName = $state("");
  let newChairs = $state(1);
  let creating = $state(false);
  let createError = $state<string | null>(null);

  // Hours editor state per expanded office
  let hoursInputs = $state<Record<string, { open: string; close: string }>>({});
  let hoursError = $state<Record<string, string>>({});
  let hoursSaving = $state<Record<string, boolean>>({});

  // Address editor state per expanded office
  type AddrFields = { addr1: string; addr2: string; city: string; sub: string; country: string };
  let addrInputs = $state<Record<string, AddrFields>>({});
  let addrSaving = $state<Record<string, boolean>>({});
  let addrError = $state<Record<string, string>>({});

  onMount(load);

  async function load() {
    const r = await commands.listOffices();
    if (r.status === "ok") {
      offices = r.data;
    } else {
      error = r.error;
    }
  }

  function toggleExpand(id: string) {
    if (expandedId === id) {
      expandedId = null;
    } else {
      expandedId = id;
      // Initialise hours inputs from current state
      const office = offices.find((o) => o.id === id);
      if (office) {
        const inputs: Record<string, { open: string; close: string }> = {};
        for (const d of DAYS) {
          const h = office.hours.find((h) => h.day_of_week === d);
          inputs[d] = { open: h?.open_time ?? "", close: h?.close_time ?? "" };
        }
        hoursInputs = { ...hoursInputs, [id]: inputs as any };
        addrInputs = { ...addrInputs, [id]: {
          addr1: office.address_line_1 ?? "",
          addr2: office.address_line_2 ?? "",
          city: office.city_town ?? "",
          sub: office.subdivision ?? "",
          country: office.country ?? "",
        } };
      }
    }
  }

  async function createOffice() {
    if (!newName.trim()) { createError = "Office name is required."; return; }
    if (newChairs < 1) { createError = "Chair count must be at least 1."; return; }
    creating = true;
    createError = null;
    const r = await commands.createOffice(newName.trim(), newChairs);
    creating = false;
    if (r.status === "ok") {
      offices = [...offices, r.data].sort((a, b) => a.name.localeCompare(b.name));
      toast.success(`Office "${r.data.name}" created.`);
      newName = "";
      newChairs = 1;
      showCreate = false;
    } else {
      createError = r.error;
    }
  }

  async function renameOffice(office: OfficeDto, newName: string) {
    if (!newName.trim() || newName.trim() === office.name) return;
    const r = await commands.renameOffice(office.id, newName.trim());
    if (r.status === "ok") {
      offices = offices.map((o) => (o.id === office.id ? r.data : o));
      toast.success(`Office renamed to ${r.data.name}.`);
    } else {
      error = r.error;
    }
  }

  async function updateChairs(office: OfficeDto, count: number) {
    if (count < 1 || count === office.chair_count) return;
    const r = await commands.updateOfficeChairCount(office.id, count);
    if (r.status === "ok") {
      offices = offices.map((o) => (o.id === office.id ? r.data : o));
      toast.success(`Chair count for ${r.data.name} updated to ${r.data.chair_count}.`);
    } else {
      error = r.error;
    }
  }

  async function setHours(officeId: string, day: string) {
    const inputs = (hoursInputs[officeId] as any)?.[day];
    if (!inputs) return;
    const { open, close } = inputs;
    if (!open || !close) return;
    hoursSaving = { ...hoursSaving, [`${officeId}-${day}`]: true };
    hoursError = { ...hoursError, [`${officeId}-${day}`]: "" };
    const r = await commands.setOfficeHours(officeId, day, open, close);
    hoursSaving = { ...hoursSaving, [`${officeId}-${day}`]: false };
    if (r.status === "ok") {
      offices = offices.map((o) => (o.id === officeId ? r.data : o));
    } else {
      hoursError = { ...hoursError, [`${officeId}-${day}`]: r.error };
    }
  }

  async function closeDay(officeId: string, day: string) {
    hoursSaving = { ...hoursSaving, [`${officeId}-${day}`]: true };
    const r = await commands.closeOfficeDay(officeId, day);
    hoursSaving = { ...hoursSaving, [`${officeId}-${day}`]: false };
    if (r.status === "ok") {
      offices = offices.map((o) => (o.id === officeId ? r.data : o));
      const existing = (hoursInputs[officeId] as any) ?? {};
      hoursInputs = { ...hoursInputs, [officeId]: { ...existing, [day]: { open: "", close: "" } } };
    } else {
      hoursError = { ...hoursError, [`${officeId}-${day}`]: r.error };
    }
  }

  async function checkDay(officeId: string, day: string, checked: boolean) {
    if (!checked) { await closeDay(officeId, day); return; }
    const existing = (hoursInputs[officeId] as any) ?? {};
    hoursInputs = { ...hoursInputs, [officeId]: { ...existing, [day]: { open: "08:00", close: "17:00" } } };
    hoursSaving = { ...hoursSaving, [`${officeId}-${day}`]: true };
    hoursError = { ...hoursError, [`${officeId}-${day}`]: "" };
    const r = await commands.setOfficeHours(officeId, day, "08:00", "17:00");
    hoursSaving = { ...hoursSaving, [`${officeId}-${day}`]: false };
    if (r.status === "ok") offices = offices.map((o) => (o.id === officeId ? r.data : o));
    else hoursError = { ...hoursError, [`${officeId}-${day}`]: r.error };
  }

  async function blurHours(officeId: string, day: string) {
    const inputs = (hoursInputs[officeId] as any)?.[day];
    if (inputs?.open && inputs?.close) await setHours(officeId, day);
  }

  async function saveAddress(officeId: string) {
    const addr = addrInputs[officeId];
    if (!addr) return;
    addrSaving = { ...addrSaving, [officeId]: true };
    addrError = { ...addrError, [officeId]: "" };
    const r = await commands.setOfficeAddress(
      officeId,
      addr.addr1 || null,
      addr.addr2 || null,
      addr.city || null,
      addr.sub || null,
      addr.country || null,
    );
    addrSaving = { ...addrSaving, [officeId]: false };
    if (r.status === "ok") {
      offices = offices.map((o) => (o.id === officeId ? r.data : o));
      toast.success(`Address saved for ${r.data.name}.`);
    } else {
      addrError = { ...addrError, [officeId]: r.error };
    }
  }

  function setAddrInput(officeId: string, field: keyof AddrFields, val: string) {
    const existing = addrInputs[officeId] ?? { addr1: "", addr2: "", city: "", sub: "", country: "" };
    addrInputs = { ...addrInputs, [officeId]: { ...existing, [field]: val } };
  }

  async function archiveOffice(id: string) {
    const officeName = offices.find((o) => o.id === id)?.name ?? "this office";
    const ok = await confirm({ title: `Archive ${officeName}`, message: `${officeName} will be hidden from active scheduling lists. You can restore it later.`, confirmLabel: "Archive", destructive: true });
    if (!ok) return;
    const r = await commands.archiveOffice(id);
    if (r.status === "ok") {
      offices = offices.map((o) => (o.id === id ? r.data : o));
    } else {
      error = r.error;
    }
  }

  function getHoursInput(officeId: string, day: string, field: "open" | "close"): string {
    return (hoursInputs[officeId] as any)?.[day]?.[field] ?? "";
  }

  function setHoursInput(officeId: string, day: string, field: "open" | "close", val: string) {
    const existing = (hoursInputs[officeId] as any) ?? {};
    const dayExisting = existing[day] ?? { open: "", close: "" };
    hoursInputs = { ...hoursInputs, [officeId]: { ...existing, [day]: { ...dayExisting, [field]: val } } };
  }

  function hasHours(office: OfficeDto, day: string) {
    return office.hours.some((h) => h.day_of_week === day);
  }
</script>

<div>
  <div class="section-header">
    <h2>Offices</h2>
    <button class="btn-primary" onclick={() => { showCreate = !showCreate; if (!showCreate) { newName = ""; newChairs = 1; createError = null; } }}>
      {showCreate ? "Cancel" : "+ New Office"}
    </button>
  </div>

  {#if error}
    <p class="error">{error}</p>
  {/if}

  {#if showCreate}
    <form class="create-form" onsubmit={(e) => { e.preventDefault(); createOffice(); }}>
      {#if createError}<p class="error">{createError}</p>{/if}
      <div class="row">
        <div class="field">
          <label for="new-office-name">Name <span class="required-mark" aria-hidden="true">*</span></label>
          <input id="new-office-name" bind:value={newName} placeholder="e.g. Kingston" />
        </div>
        <div class="field" style="max-width:120px">
          <label for="new-office-chairs">Chairs <span class="required-mark" aria-hidden="true">*</span></label>
          <input id="new-office-chairs" type="number" min="1" bind:value={newChairs} />
        </div>
        <div class="field" style="justify-content:flex-end; padding-top:1.4rem">
          <button type="submit" class="btn-primary" disabled={creating}>
            {#if creating}<span class="spinner-btn" aria-hidden="true"></span>{/if}
            {creating ? "Creating…" : "Create"}
          </button>
        </div>
      </div>
    </form>
  {/if}

  {#if offices.length === 0 && !showCreate}
    <p class="empty">No offices yet. Use + New Office to create your first office.</p>
  {/if}

  <div class="office-list">
    {#each offices as office (office.id)}
      <div class="office-card" class:archived={office.archived}>
        <div class="office-row" role="button" tabindex="0"
          aria-expanded={expandedId === office.id}
          aria-label="Expand {office.name} details"
          onclick={() => toggleExpand(office.id)}
          onkeydown={(e) => e.key === "Enter" && toggleExpand(office.id)}>
          <div class="office-info">
            <span class="office-name">{office.name}</span>
            <span class="office-meta">{office.chair_count} chair{office.chair_count !== 1 ? "s" : ""}</span>
            <span class="office-meta">
              {office.hours.length} day{office.hours.length !== 1 ? "s" : ""} set
            </span>
            {#if office.city_town || office.country}
              <span class="office-meta office-addr">
                {[office.city_town, office.country].filter(Boolean).join(", ")}
              </span>
            {/if}
            {#if office.archived}
              <span class="badge archived-badge">Archived</span>
            {/if}
          </div>
          <span class="chevron">{expandedId === office.id ? "▲" : "▼"}</span>
        </div>

        {#if expandedId === office.id}
          <div class="office-detail">
            <!-- Rename & chairs inline -->
            <div class="detail-row">
              <div class="field">
                <label for="office-name-{office.id}">Name</label>
                <input id="office-name-{office.id}"
                  value={office.name}
                  onblur={(e) => renameOffice(office, (e.target as HTMLInputElement).value)}
                  onkeydown={(e) => e.key === "Enter" && renameOffice(office, (e.target as HTMLInputElement).value)}
                />
              </div>
              <div class="field" style="max-width:110px">
                <label for="office-chairs-{office.id}">Chairs</label>
                <input id="office-chairs-{office.id}"
                  type="number" min="1"
                  value={office.chair_count}
                  onblur={(e) => updateChairs(office, Number((e.target as HTMLInputElement).value))}
                  onkeydown={(e) => e.key === "Enter" && updateChairs(office, Number((e.target as HTMLInputElement).value))}
                />
              </div>
              {#if !office.archived}
                <div class="field" style="justify-content:flex-end; padding-top:1.4rem">
                  <button class="btn-danger-sm" onclick={() => archiveOffice(office.id)}>Archive</button>
                </div>
              {/if}
            </div>

            <!-- Address -->
            <h4>Address</h4>
            {#if addrError[office.id]}
              <p class="field-error">{addrError[office.id]}</p>
            {/if}
            <div class="addr-grid">
              <div class="field addr-full">
                <label for="office-addr1-{office.id}">Address Line 1</label>
                <input id="office-addr1-{office.id}"
                  value={addrInputs[office.id]?.addr1 ?? ""}
                  placeholder="e.g. 12 Harbour Street"
                  oninput={(e) => setAddrInput(office.id, "addr1", (e.target as HTMLInputElement).value)}
                  onblur={() => saveAddress(office.id)}
                />
              </div>
              <div class="field addr-full">
                <label for="office-addr2-{office.id}">Address Line 2</label>
                <input id="office-addr2-{office.id}"
                  value={addrInputs[office.id]?.addr2 ?? ""}
                  placeholder="Suite, floor, etc. (optional)"
                  oninput={(e) => setAddrInput(office.id, "addr2", (e.target as HTMLInputElement).value)}
                  onblur={() => saveAddress(office.id)}
                />
              </div>
              <div class="field">
                <label for="office-city-{office.id}">City / Town</label>
                <input id="office-city-{office.id}"
                  value={addrInputs[office.id]?.city ?? ""}
                  placeholder="e.g. Kingston"
                  oninput={(e) => setAddrInput(office.id, "city", (e.target as HTMLInputElement).value)}
                  onblur={() => saveAddress(office.id)}
                />
              </div>
              <div class="field">
                <label for="office-sub-{office.id}">Parish / Region</label>
                <input id="office-sub-{office.id}"
                  value={addrInputs[office.id]?.sub ?? ""}
                  placeholder="e.g. Kingston"
                  oninput={(e) => setAddrInput(office.id, "sub", (e.target as HTMLInputElement).value)}
                  onblur={() => saveAddress(office.id)}
                />
              </div>
              <div class="field">
                <label for="office-country-{office.id}">Country</label>
                <input id="office-country-{office.id}"
                  value={addrInputs[office.id]?.country ?? ""}
                  placeholder="e.g. Jamaica"
                  oninput={(e) => setAddrInput(office.id, "country", (e.target as HTMLInputElement).value)}
                  onblur={() => saveAddress(office.id)}
                />
              </div>
              {#if addrSaving[office.id]}
                <div class="addr-saving">Saving…</div>
              {/if}
            </div>

            <!-- Hours editor -->
            <h4>Operating Hours</h4>
            <p class="hours-hint">Check a day to mark it open. Edit times and click away to save.</p>
            <div class="hours-grid">
              <div class="hours-header">Day</div>
              <div class="hours-header">Open</div>
              <div class="hours-header">Close</div>

              {#each DAYS as day}
                {@const key = `${office.id}-${day}`}
                {@const isOpen = hasHours(office, day)}
                <label class="day-label" class:day-open={isOpen}>
                  <input
                    type="checkbox"
                    class="day-checkbox"
                    checked={isOpen}
                    disabled={!!hoursSaving[key]}
                    onchange={(e) => checkDay(office.id, day, (e.target as HTMLInputElement).checked)}
                  />
                  {day}
                  {#if hoursSaving[key]}<span class="saving-indicator"> Saving…</span>{/if}
                </label>
                {#if isOpen}
                  <input
                    class="time-input"
                    type="time"
                    aria-label="Open time for {day}"
                    value={getHoursInput(office.id, day, "open")}
                    oninput={(e) => setHoursInput(office.id, day, "open", (e.target as HTMLInputElement).value)}
                    onblur={() => blurHours(office.id, day)}
                  />
                  <input
                    class="time-input"
                    type="time"
                    aria-label="Close time for {day}"
                    value={getHoursInput(office.id, day, "close")}
                    oninput={(e) => setHoursInput(office.id, day, "close", (e.target as HTMLInputElement).value)}
                    onblur={() => blurHours(office.id, day)}
                  />
                {:else}
                  <div></div>
                  <div></div>
                {/if}
                {#if hoursError[key]}
                  <div class="hours-err" style="grid-column:1/-1">{hoursError[key]}</div>
                {/if}
              {/each}
            </div>
          </div>
        {/if}
      </div>
    {/each}
  </div>
</div>

<style>
  .section-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--space-4); }
  h2 { margin: 0; font-size: var(--text-xl); font-family: var(--font-heading); font-weight: 600; color: var(--abyss-navy); }
  h4 { margin: var(--space-4) 0 var(--space-2); font-size: var(--text-xs); font-weight: 700; color: var(--slate-fog); text-transform: uppercase; letter-spacing: 0.06em; font-family: var(--font-body); }
  .error { color: var(--healthy-coral-dk); font-size: var(--text-sm); margin-bottom: var(--space-3); }
  .empty { color: var(--slate-fog); font-style: italic; font-size: var(--text-sm); }

  .create-form {
    background: var(--pearl-mist); border: 1px solid var(--pearl-mist-dk);
    border-radius: var(--radius-md); padding: var(--space-4); margin-bottom: var(--space-4);
  }

  .row { display: flex; gap: var(--space-4); align-items: flex-start; }
  .field { display: flex; flex-direction: column; gap: var(--space-1); flex: 1; }
  .field label { font-size: var(--text-xs); font-weight: 600; color: var(--abyss-navy); font-family: var(--font-body); }
  input:not([type="number"]):not([type="time"]) {
    padding: var(--space-2) var(--space-3); border: 1px solid var(--pearl-mist-dk); border-radius: var(--radius-sm);
    font-size: var(--text-sm); font-family: var(--font-body); width: 100%; box-sizing: border-box;
  }
  input[type="number"] {
    padding: var(--space-2) var(--space-3); border: 1px solid var(--pearl-mist-dk); border-radius: var(--radius-sm);
    font-size: var(--text-sm); font-family: var(--font-body); width: 100%; box-sizing: border-box;
  }
  input:focus { outline: none; border-color: var(--caribbean-teal); }

  .office-list { display: flex; flex-direction: column; gap: var(--space-3); }

  .office-card {
    border: 1px solid var(--pearl-mist-dk); border-radius: var(--radius-md);
    overflow: hidden; background: white;
  }
  .office-card.archived { opacity: 0.6; }

  .office-row {
    display: flex; justify-content: space-between; align-items: center;
    padding: var(--space-3) var(--space-4); cursor: pointer; user-select: none;
  }
  .office-row:hover { background: var(--pearl-mist); }

  .office-info { display: flex; align-items: center; gap: var(--space-3); }
  .office-name { font-weight: 600; font-size: var(--text-sm); color: var(--abyss-navy); font-family: var(--font-body); }
  .office-meta { font-size: var(--text-xs); color: var(--slate-fog); }
  .badge { font-size: var(--text-xs); padding: 2px var(--space-2); border-radius: var(--radius-full); font-weight: 600; font-family: var(--font-body); }
  .archived-badge { background: var(--color-archived-lt); color: var(--color-archived); }
  .chevron { color: var(--slate-fog); font-size: var(--text-xs); }

  .office-detail { padding: 0 var(--space-4) var(--space-4); border-top: 1px solid var(--pearl-mist-dk); }

  .detail-row { display: flex; gap: var(--space-4); align-items: flex-start; margin-top: var(--space-3); }
  .detail-row .field { flex: 1; }
  .detail-row input {
    padding: var(--space-2) var(--space-3); border: 1px solid var(--pearl-mist-dk); border-radius: var(--radius-sm);
    font-size: var(--text-sm); font-family: var(--font-body); width: 100%; box-sizing: border-box;
  }

  .office-addr { font-style: italic; }
  .field-error { font-size: var(--text-xs); color: var(--healthy-coral-dk); margin: 0 0 var(--space-2); }
  .addr-grid { display: grid; grid-template-columns: 1fr 1fr; gap: var(--space-2) var(--space-4); margin-bottom: var(--space-4); }
  .addr-full { grid-column: 1 / -1; }
  .addr-saving { grid-column: 1 / -1; font-size: var(--text-xs); color: var(--slate-fog); font-style: italic; }

  .hours-hint { font-size: var(--text-xs); color: var(--slate-fog); margin: 0 0 var(--space-3); }
  .hours-grid { display: grid; grid-template-columns: 140px 110px 110px; gap: var(--space-1); align-items: center; }
  .hours-header { font-size: var(--text-xs); font-weight: 600; color: var(--slate-fog); text-transform: uppercase; letter-spacing: 0.04em; font-family: var(--font-body); }
  .day-label { display: flex; align-items: center; gap: var(--space-2); font-size: var(--text-sm); color: var(--slate-fog); cursor: pointer; user-select: none; font-family: var(--font-body); }
  .day-label.day-open { font-weight: 600; color: var(--abyss-navy); }
  .day-checkbox { cursor: pointer; }
  .time-input { padding: var(--space-1) var(--space-2); border: 1px solid var(--pearl-mist-dk); border-radius: var(--radius-sm); font-size: var(--text-sm); font-family: var(--font-body); }
  .saving-indicator { font-size: var(--text-xs); color: var(--slate-fog); font-style: italic; }
  .hours-err { font-size: var(--text-xs); color: var(--healthy-coral-dk); padding-left: 2px; }

  .btn-primary {
    display: inline-flex; align-items: center; min-height: 44px; padding: 0 var(--space-4);
    background: var(--caribbean-teal); color: white; border: none;
    border-radius: var(--radius-md); font-family: var(--font-heading); font-size: var(--text-sm);
    font-weight: 600; cursor: pointer; transition: background var(--transition-fast);
  }
  .btn-primary:hover:not(:disabled) { background: var(--caribbean-teal-dk); }
  .btn-primary:disabled { opacity: 0.45; cursor: not-allowed; }

  .btn-danger-sm {
    display: inline-flex; align-items: center; min-height: 44px; padding: 0 var(--space-3);
    background: white; color: var(--healthy-coral-dk);
    border: 1px solid var(--healthy-coral); border-radius: var(--radius-md);
    font-size: var(--text-xs); font-family: var(--font-body); font-weight: 600; cursor: pointer;
    transition: background var(--transition-fast);
  }
  .btn-danger-sm:hover { background: var(--healthy-coral-lt); }
</style>
