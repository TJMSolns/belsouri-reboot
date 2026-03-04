<script lang="ts">
  import { commands, type ProcedureTypeDto } from "$lib/bindings";
  import { onMount } from "svelte";

  const CATEGORIES = ["Consult", "Preventive", "Restorative", "Invasive", "Cosmetic", "Diagnostic"];
  const CATEGORY_COLORS: Record<string, string> = {
    Consult: "#f0c040",
    Preventive: "#4a90d9",
    Restorative: "#27ae60",
    Invasive: "#e74c3c",
    Cosmetic: "#9b59b6",
    Diagnostic: "#95a5a6",
  };

  let types = $state<ProcedureTypeDto[]>([]);
  let error = $state<string | null>(null);
  let seeding = $state(false);

  // Define form
  let showDefine = $state(false);
  let newName = $state("");
  let newCategory = $state("Preventive");
  let newDuration = $state(30);
  let defining = $state(false);
  let defineError = $state<string | null>(null);

  // Editing state per type
  let editingId = $state<string | null>(null);
  let editName = $state("");
  let editCategory = $state("");
  let editDuration = $state(30);
  let editError = $state<string | null>(null);

  onMount(load);

  async function load() {
    const r = await commands.listProcedureTypes();
    if (r.status === "ok") types = r.data;
    else error = r.error;
  }

  async function seed() {
    if (types.length > 0) {
      if (!confirm("Add the 10 default procedure types? (Existing types won't be affected)")) return;
    }
    seeding = true; error = null;
    const r = await commands.seedDefaultProcedureTypes();
    seeding = false;
    if (r.status === "ok") types = r.data;
    else error = r.error;
  }

  async function define() {
    if (!newName.trim()) { defineError = "Name is required"; return; }
    defining = true; defineError = null;
    const r = await commands.defineProcedureType(newName.trim(), newCategory, newDuration);
    defining = false;
    if (r.status === "ok") {
      types = [...types, r.data].sort((a, b) => a.name.localeCompare(b.name));
      newName = ""; newDuration = 30; showDefine = false;
    } else { defineError = r.error; }
  }

  function startEdit(pt: ProcedureTypeDto) {
    editingId = pt.id;
    editName = pt.name;
    editCategory = pt.category;
    editDuration = pt.default_duration_minutes;
    editError = null;
  }

  async function saveEdit(id: string) {
    editError = null;
    const r = await commands.updateProcedureType(id, editName.trim() || null, editCategory, editDuration);
    if (r.status === "ok") {
      types = types.map((t) => t.id === id ? r.data : t);
      editingId = null;
    } else { editError = r.error; }
  }

  async function toggleActive(pt: ProcedureTypeDto) {
    const r = pt.is_active
      ? await commands.deactivateProcedureType(pt.id)
      : await commands.reactivateProcedureType(pt.id);
    if (r.status === "ok") types = types.map((t) => t.id === pt.id ? r.data : t);
    else error = r.error;
  }

  let activeTypes = $derived(types.filter((t) => t.is_active));
  let inactiveTypes = $derived(types.filter((t) => !t.is_active));
</script>

<div>
  <div class="section-header">
    <h2>Procedure Types</h2>
    <div class="header-actions">
      <button class="btn-secondary" onclick={seed} disabled={seeding}>
        {seeding ? "Seeding…" : "Seed Defaults"}
      </button>
      <button class="btn-primary" onclick={() => (showDefine = !showDefine)}>
        {showDefine ? "Cancel" : "+ Define"}
      </button>
    </div>
  </div>

  {#if error}<p class="error">{error}</p>{/if}

  {#if types.length === 0 && !showDefine}
    <div class="empty-state">
      <p>No procedure types defined yet.</p>
      <button class="btn-primary" onclick={seed} disabled={seeding}>
        {seeding ? "Seeding…" : "Seed 10 defaults"}
      </button>
    </div>
  {/if}

  {#if showDefine}
    <form class="create-form" onsubmit={(e) => { e.preventDefault(); define(); }}>
      {#if defineError}<p class="error">{defineError}</p>{/if}
      <div class="row">
        <div class="field">
          <label>Name</label>
          <input bind:value={newName} placeholder="e.g. Root Canal" />
        </div>
        <div class="field" style="max-width:150px">
          <label>Category</label>
          <select bind:value={newCategory}>
            {#each CATEGORIES as c}<option>{c}</option>{/each}
          </select>
        </div>
        <div class="field" style="max-width:100px">
          <label>Duration (min)</label>
          <input type="number" min="15" max="240" bind:value={newDuration} />
        </div>
        <div class="field" style="justify-content:flex-end; padding-top:1.4rem">
          <button type="submit" class="btn-primary" disabled={defining}>
            {defining ? "Defining…" : "Define"}
          </button>
        </div>
      </div>
    </form>
  {/if}

  {#if activeTypes.length > 0}
    <div class="type-list">
      {#each activeTypes as pt (pt.id)}
        <div class="type-row">
          {#if editingId === pt.id}
            <div class="edit-row">
              {#if editError}<p class="error">{editError}</p>{/if}
              <input bind:value={editName} placeholder="Name" style="flex:1" />
              <select bind:value={editCategory}>
                {#each CATEGORIES as c}<option>{c}</option>{/each}
              </select>
              <input type="number" min="15" max="240" bind:value={editDuration} style="width:80px" />
              <span class="duration-label">min</span>
              <button class="btn-sm" onclick={() => saveEdit(pt.id)}>Save</button>
              <button class="btn-sm btn-ghost" onclick={() => (editingId = null)}>Cancel</button>
            </div>
          {:else}
            <div class="type-info">
              <span class="cat-dot" style="background:{CATEGORY_COLORS[pt.category] ?? '#ccc'}"></span>
              <span class="type-name">{pt.name}</span>
              <span class="badge cat-badge" style="background:{CATEGORY_COLORS[pt.category]}22; color:{CATEGORY_COLORS[pt.category]}">{pt.category}</span>
              <span class="meta">{pt.default_duration_minutes} min</span>
            </div>
            <div class="type-actions">
              <button class="btn-sm btn-ghost" onclick={() => startEdit(pt)}>Edit</button>
              <button class="btn-sm btn-ghost" onclick={() => toggleActive(pt)}>Deactivate</button>
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}

  {#if inactiveTypes.length > 0}
    <div class="inactive-section">
      <h4>Inactive</h4>
      <div class="type-list">
        {#each inactiveTypes as pt (pt.id)}
          <div class="type-row inactive">
            <div class="type-info">
              <span class="cat-dot" style="background:#ccc"></span>
              <span class="type-name">{pt.name}</span>
              <span class="badge cat-badge" style="background:#f0f0f0; color:#999">{pt.category}</span>
              <span class="meta">{pt.default_duration_minutes} min</span>
            </div>
            <div class="type-actions">
              <button class="btn-sm btn-ghost" onclick={() => toggleActive(pt)}>Reactivate</button>
            </div>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .section-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem; }
  h2 { margin: 0; font-size: 1.1rem; color: #222; }
  h4 { margin: 1.25rem 0 0.5rem; font-size: 0.82rem; color: #888; text-transform: uppercase; letter-spacing: 0.04em; }
  .header-actions { display: flex; gap: 0.5rem; }
  .error { color: #c0392b; font-size: 0.875rem; margin-bottom: 0.5rem; }

  .empty-state { text-align: center; padding: 2rem; color: #888; }
  .empty-state p { margin-bottom: 1rem; }

  .create-form { background: #f7f8fa; border: 1px solid #e0e0e0; border-radius: 8px; padding: 1rem; margin-bottom: 1rem; }
  .row { display: flex; gap: 1rem; align-items: flex-start; }
  .field { display: flex; flex-direction: column; gap: 0.3rem; flex: 1; }
  .field label { font-size: 0.78rem; font-weight: 600; color: #555; text-transform: uppercase; letter-spacing: 0.03em; }
  input:not([type="number"]), select {
    padding: 0.45rem 0.6rem; border: 1px solid #ccc; border-radius: 6px;
    font-size: 0.9rem; font-family: system-ui, sans-serif; width: 100%; box-sizing: border-box; background: white;
  }
  input[type="number"] {
    padding: 0.45rem 0.6rem; border: 1px solid #ccc; border-radius: 6px;
    font-size: 0.9rem; font-family: system-ui, sans-serif; width: 100%; box-sizing: border-box;
  }
  input:focus, select:focus { outline: none; border-color: #1a1a2e; }

  .type-list { display: flex; flex-direction: column; gap: 0; border: 1px solid #e8e8e8; border-radius: 8px; overflow: hidden; }
  .type-row { display: flex; justify-content: space-between; align-items: center; padding: 0.6rem 1rem; border-bottom: 1px solid #f0f0f0; background: white; }
  .type-row:last-child { border-bottom: none; }
  .type-row.inactive { background: #fafafa; }
  .type-row:hover { background: #f7f8fa; }

  .type-info { display: flex; align-items: center; gap: 0.65rem; }
  .cat-dot { width: 10px; height: 10px; border-radius: 50%; flex-shrink: 0; }
  .type-name { font-weight: 500; font-size: 0.9rem; }
  .meta { font-size: 0.8rem; color: #888; }
  .badge { font-size: 0.72rem; padding: 0.15rem 0.5rem; border-radius: 20px; font-weight: 600; }
  .type-actions { display: flex; gap: 0.4rem; }

  .edit-row { display: flex; align-items: center; gap: 0.5rem; flex: 1; flex-wrap: wrap; }
  .edit-row input { padding: 0.3rem 0.5rem; border: 1px solid #ccc; border-radius: 5px; font-size: 0.85rem; }
  .edit-row select { padding: 0.3rem 0.5rem; border: 1px solid #ccc; border-radius: 5px; font-size: 0.85rem; background: white; }
  .duration-label { font-size: 0.8rem; color: #777; }
  .inactive-section { margin-top: 1rem; }

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
  .btn-secondary:hover:not(:disabled) { background: #f5f5f5; }
  .btn-secondary:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn-sm { padding: 0.25rem 0.6rem; background: #1a1a2e; color: white; border: none; border-radius: 4px; font-size: 0.78rem; cursor: pointer; font-family: system-ui, sans-serif; }
  .btn-sm.btn-ghost { background: #eee; color: #555; }
  .btn-sm.btn-ghost:hover { background: #ddd; }
</style>
