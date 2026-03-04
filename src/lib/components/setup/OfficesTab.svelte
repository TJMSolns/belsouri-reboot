<script lang="ts">
  import { commands, type OfficeDto } from "$lib/bindings";
  import { onMount } from "svelte";

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
      }
    }
  }

  async function createOffice() {
    if (!newName.trim()) { createError = "Name is required"; return; }
    if (newChairs < 1) { createError = "Chair count must be at least 1"; return; }
    creating = true;
    createError = null;
    const r = await commands.createOffice(newName.trim(), newChairs);
    creating = false;
    if (r.status === "ok") {
      offices = [...offices, r.data].sort((a, b) => a.name.localeCompare(b.name));
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
    } else {
      error = r.error;
    }
  }

  async function updateChairs(office: OfficeDto, count: number) {
    if (count < 1 || count === office.chair_count) return;
    const r = await commands.updateOfficeChairCount(office.id, count);
    if (r.status === "ok") {
      offices = offices.map((o) => (o.id === office.id ? r.data : o));
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
      // Clear the inputs for this day
      const existing = (hoursInputs[officeId] as any) ?? {};
      hoursInputs = { ...hoursInputs, [officeId]: { ...existing, [day]: { open: "", close: "" } } };
    } else {
      hoursError = { ...hoursError, [`${officeId}-${day}`]: r.error };
    }
  }

  async function archiveOffice(id: string) {
    if (!confirm("Archive this office? It will be hidden from active lists.")) return;
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
    <button class="btn-primary" onclick={() => (showCreate = !showCreate)}>
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
          <label>Name</label>
          <input bind:value={newName} placeholder="e.g. Kingston" />
        </div>
        <div class="field" style="max-width:120px">
          <label>Chairs</label>
          <input type="number" min="1" bind:value={newChairs} />
        </div>
        <div class="field" style="justify-content:flex-end; padding-top:1.4rem">
          <button type="submit" class="btn-primary" disabled={creating}>
            {creating ? "Creating…" : "Create"}
          </button>
        </div>
      </div>
    </form>
  {/if}

  {#if offices.length === 0 && !showCreate}
    <p class="empty">No offices yet. Create one above.</p>
  {/if}

  <div class="office-list">
    {#each offices as office (office.id)}
      <div class="office-card" class:archived={office.archived}>
        <div class="office-row" role="button" tabindex="0"
          onclick={() => toggleExpand(office.id)}
          onkeydown={(e) => e.key === "Enter" && toggleExpand(office.id)}>
          <div class="office-info">
            <span class="office-name">{office.name}</span>
            <span class="office-meta">{office.chair_count} chair{office.chair_count !== 1 ? "s" : ""}</span>
            <span class="office-meta">
              {office.hours.length} day{office.hours.length !== 1 ? "s" : ""} set
            </span>
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
                <label>Name</label>
                <input
                  value={office.name}
                  onblur={(e) => renameOffice(office, (e.target as HTMLInputElement).value)}
                  onkeydown={(e) => e.key === "Enter" && renameOffice(office, (e.target as HTMLInputElement).value)}
                />
              </div>
              <div class="field" style="max-width:110px">
                <label>Chairs</label>
                <input
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

            <!-- Hours editor -->
            <h4>Operating Hours</h4>
            <div class="hours-grid">
              <div class="hours-header">Day</div>
              <div class="hours-header">Open</div>
              <div class="hours-header">Close</div>
              <div class="hours-header"></div>

              {#each DAYS as day}
                {@const key = `${office.id}-${day}`}
                {@const isOpen = hasHours(office, day)}
                <div class="day-label" class:day-open={isOpen}>{day}</div>
                <input
                  class="time-input"
                  type="time"
                  value={getHoursInput(office.id, day, "open")}
                  oninput={(e) => setHoursInput(office.id, day, "open", (e.target as HTMLInputElement).value)}
                />
                <input
                  class="time-input"
                  type="time"
                  value={getHoursInput(office.id, day, "close")}
                  oninput={(e) => setHoursInput(office.id, day, "close", (e.target as HTMLInputElement).value)}
                />
                <div class="hours-actions">
                  <button
                    class="btn-sm"
                    disabled={hoursSaving[key]}
                    onclick={() => setHours(office.id, day)}
                  >Set</button>
                  {#if isOpen}
                    <button
                      class="btn-sm btn-ghost"
                      disabled={hoursSaving[key]}
                      onclick={() => closeDay(office.id, day)}
                    >✕</button>
                  {/if}
                </div>
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
  .section-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem; }
  h2 { margin: 0; font-size: 1.1rem; color: #222; }
  h4 { margin: 1rem 0 0.5rem; font-size: 0.85rem; color: #555; text-transform: uppercase; letter-spacing: 0.04em; }
  .error { color: #c0392b; font-size: 0.875rem; margin-bottom: 0.75rem; }
  .empty { color: #999; font-style: italic; }

  .create-form {
    background: #f7f8fa;
    border: 1px solid #e0e0e0;
    border-radius: 8px;
    padding: 1rem;
    margin-bottom: 1rem;
  }

  .row { display: flex; gap: 1rem; align-items: flex-start; }
  .field { display: flex; flex-direction: column; gap: 0.3rem; flex: 1; }
  .field label { font-size: 0.78rem; font-weight: 600; color: #555; text-transform: uppercase; letter-spacing: 0.03em; }
  input:not([type="number"]):not([type="time"]) {
    padding: 0.45rem 0.6rem; border: 1px solid #ccc; border-radius: 6px;
    font-size: 0.9rem; font-family: system-ui, sans-serif; width: 100%; box-sizing: border-box;
  }
  input[type="number"] {
    padding: 0.45rem 0.6rem; border: 1px solid #ccc; border-radius: 6px;
    font-size: 0.9rem; font-family: system-ui, sans-serif; width: 100%; box-sizing: border-box;
  }
  input:focus { outline: none; border-color: #1a1a2e; }

  .office-list { display: flex; flex-direction: column; gap: 0.75rem; }

  .office-card {
    border: 1px solid #ddd;
    border-radius: 8px;
    overflow: hidden;
    background: white;
  }
  .office-card.archived { opacity: 0.6; }

  .office-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1rem;
    cursor: pointer;
    user-select: none;
  }
  .office-row:hover { background: #f7f8fa; }

  .office-info { display: flex; align-items: center; gap: 0.75rem; }
  .office-name { font-weight: 600; font-size: 0.95rem; }
  .office-meta { font-size: 0.8rem; color: #777; }
  .badge { font-size: 0.72rem; padding: 0.15rem 0.5rem; border-radius: 20px; font-weight: 600; }
  .archived-badge { background: #f0e6d3; color: #a06030; }
  .chevron { color: #aaa; font-size: 0.8rem; }

  .office-detail {
    padding: 0 1rem 1rem;
    border-top: 1px solid #eee;
  }

  .detail-row { display: flex; gap: 1rem; align-items: flex-start; margin-top: 0.75rem; }
  .detail-row .field { flex: 1; }
  .detail-row input { padding: 0.4rem 0.6rem; border: 1px solid #ccc; border-radius: 6px; font-size: 0.9rem; width: 100%; box-sizing: border-box; }

  .hours-grid {
    display: grid;
    grid-template-columns: 110px 110px 110px 1fr;
    gap: 0.35rem;
    align-items: center;
  }
  .hours-header { font-size: 0.75rem; font-weight: 600; color: #888; text-transform: uppercase; letter-spacing: 0.04em; }
  .day-label { font-size: 0.85rem; color: #444; }
  .day-label.day-open { font-weight: 600; color: #1a1a2e; }
  .time-input { padding: 0.3rem 0.4rem; border: 1px solid #ccc; border-radius: 5px; font-size: 0.85rem; }
  .hours-actions { display: flex; gap: 0.4rem; }
  .hours-err { font-size: 0.78rem; color: #c0392b; padding-left: 2px; }

  .btn-primary {
    padding: 0.45rem 1.1rem; background: #1a1a2e; color: white;
    border: none; border-radius: 6px; font-size: 0.875rem; cursor: pointer; font-family: system-ui, sans-serif;
  }
  .btn-primary:hover:not(:disabled) { background: #2a2a4e; }
  .btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }

  .btn-sm {
    padding: 0.25rem 0.6rem; background: #1a1a2e; color: white;
    border: none; border-radius: 4px; font-size: 0.78rem; cursor: pointer; font-family: system-ui, sans-serif;
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
