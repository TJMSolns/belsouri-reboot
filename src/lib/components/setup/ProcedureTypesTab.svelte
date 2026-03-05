<script lang="ts">
  import { commands, type ProcedureTypeDto } from "$lib/bindings";
  import { onMount } from "svelte";
  import { toast } from "$lib/stores/toast";
  import { confirm } from "$lib/stores/confirm";

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
      const ok = await confirm({ title: "Add default procedure types", message: "Add the 10 default procedure types? Existing types won't be affected.", confirmLabel: "Add defaults" });
      if (!ok) return;
    }
    seeding = true; error = null;
    const r = await commands.seedDefaultProcedureTypes();
    seeding = false;
    if (r.status === "ok") {
      types = r.data;
      toast.success("Default procedure types added.");
    } else { error = r.error; }
  }

  async function define() {
    if (!newName.trim()) { defineError = "Name is required."; return; }
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
        {#if seeding}<span class="spinner" aria-hidden="true"></span><span class="sr-only">Adding defaults</span>{:else}Add Defaults{/if}
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
        {#if seeding}<span class="spinner" aria-hidden="true"></span><span class="sr-only">Adding defaults</span>{:else}Add 10 defaults{/if}
      </button>
    </div>
  {/if}

  {#if showDefine}
    <form class="create-form" onsubmit={(e) => { e.preventDefault(); define(); }}>
      {#if defineError}<p class="error">{defineError}</p>{/if}
      <div class="row">
        <div class="field">
          <label for="proc-name">Name</label>
          <input id="proc-name" bind:value={newName} placeholder="e.g. Root Canal" />
        </div>
        <div class="field" style="max-width:150px">
          <label for="proc-category">Category</label>
          <select id="proc-category" bind:value={newCategory}>
            {#each CATEGORIES as c}<option>{c}</option>{/each}
          </select>
        </div>
        <div class="field" style="max-width:100px">
          <label for="proc-duration">Duration (min)</label>
          <input id="proc-duration" type="number" min="15" max="240" bind:value={newDuration} />
        </div>
        <div class="field" style="justify-content:flex-end; padding-top:1.4rem">
          <button type="submit" class="btn-primary" disabled={defining}>
            {#if defining}<span class="spinner" aria-hidden="true"></span><span class="sr-only">Defining</span>{:else}Define{/if}
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
  .section-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--space-4); }
  h2 { margin: 0; font-size: var(--text-xl); font-family: var(--font-heading); font-weight: 600; color: var(--abyss-navy); }
  h4 { margin: var(--space-5) 0 var(--space-2); font-size: var(--text-xs); font-weight: 700; color: var(--slate-fog); text-transform: uppercase; letter-spacing: 0.04em; font-family: var(--font-body); }
  .header-actions { display: flex; gap: var(--space-2); }
  .error { color: var(--healthy-coral-dk); font-size: var(--text-sm); margin-bottom: var(--space-2); }

  .empty-state { text-align: center; padding: var(--space-6); color: var(--slate-fog); font-family: var(--font-body); }
  .empty-state p { margin-bottom: var(--space-4); font-size: var(--text-sm); }

  .create-form { background: var(--pearl-mist); border: 1px solid var(--pearl-mist-dk); border-radius: var(--radius-md); padding: var(--space-4); margin-bottom: var(--space-4); }
  .row { display: flex; gap: var(--space-4); align-items: flex-start; }
  .field { display: flex; flex-direction: column; gap: var(--space-1); flex: 1; }
  .field label { font-size: var(--text-xs); font-weight: 600; color: var(--abyss-navy); font-family: var(--font-body); }
  input:not([type="number"]), select {
    padding: var(--space-2) var(--space-3); border: 1px solid var(--pearl-mist-dk); border-radius: var(--radius-sm);
    font-size: var(--text-sm); font-family: var(--font-body); width: 100%; box-sizing: border-box; background: white;
  }
  input[type="number"] {
    padding: var(--space-2) var(--space-3); border: 1px solid var(--pearl-mist-dk); border-radius: var(--radius-sm);
    font-size: var(--text-sm); font-family: var(--font-body); width: 100%; box-sizing: border-box;
  }
  input:focus, select:focus { outline: none; border-color: var(--caribbean-teal); }

  .type-list { display: flex; flex-direction: column; gap: 0; border: 1px solid var(--pearl-mist-dk); border-radius: var(--radius-md); overflow: hidden; }
  .type-row { display: flex; justify-content: space-between; align-items: center; padding: var(--space-2) var(--space-4); border-bottom: 1px solid var(--pearl-mist); background: white; }
  .type-row:last-child { border-bottom: none; }
  .type-row.inactive { background: var(--pearl-mist); }
  .type-row:hover { background: var(--pearl-mist); }

  .type-info { display: flex; align-items: center; gap: var(--space-2); }
  .cat-dot { width: 10px; height: 10px; border-radius: 50%; flex-shrink: 0; }
  .type-name { font-weight: 500; font-size: var(--text-sm); color: var(--abyss-navy); font-family: var(--font-body); }
  .meta { font-size: var(--text-xs); color: var(--slate-fog); font-family: var(--font-body); }
  .badge { font-size: var(--text-xs); padding: 2px var(--space-2); border-radius: var(--radius-full); font-weight: 600; font-family: var(--font-body); }
  .type-actions { display: flex; gap: var(--space-1); }

  .edit-row { display: flex; align-items: center; gap: var(--space-2); flex: 1; flex-wrap: wrap; }
  .edit-row input { padding: var(--space-1) var(--space-2); border: 1px solid var(--pearl-mist-dk); border-radius: var(--radius-sm); font-size: var(--text-sm); font-family: var(--font-body); }
  .edit-row select { padding: var(--space-1) var(--space-2); border: 1px solid var(--pearl-mist-dk); border-radius: var(--radius-sm); font-size: var(--text-sm); font-family: var(--font-body); background: white; }
  .duration-label { font-size: var(--text-xs); color: var(--slate-fog); font-family: var(--font-body); }
  .inactive-section { margin-top: var(--space-4); }

  .btn-primary {
    display: inline-flex; align-items: center; min-height: 36px; padding: 0 var(--space-4);
    background: var(--caribbean-teal); color: #fff; border: none;
    border-radius: var(--radius-md); font-family: var(--font-heading); font-size: var(--text-sm);
    font-weight: 600; cursor: pointer; transition: background var(--transition-fast);
  }
  .btn-primary:hover:not(:disabled) { background: var(--caribbean-teal-dk); }
  .btn-primary:disabled { opacity: 0.45; cursor: not-allowed; }
  .btn-secondary {
    display: inline-flex; align-items: center; min-height: 36px; padding: 0 var(--space-4);
    background: white; color: var(--abyss-navy);
    border: 1px solid var(--pearl-mist-dk); border-radius: var(--radius-md);
    font-family: var(--font-heading); font-size: var(--text-sm); font-weight: 600; cursor: pointer;
    transition: background var(--transition-fast), border-color var(--transition-fast);
  }
  .btn-secondary:hover:not(:disabled) { background: var(--pearl-mist); border-color: var(--slate-fog); }
  .btn-secondary:disabled { opacity: 0.45; cursor: not-allowed; }
  .btn-sm {
    display: inline-flex; align-items: center; min-height: 28px; padding: 0 var(--space-3);
    background: var(--caribbean-teal); color: white; border: none;
    border-radius: var(--radius-sm); font-size: var(--text-xs); font-family: var(--font-body); font-weight: 600; cursor: pointer;
    transition: background var(--transition-fast);
  }
  .btn-sm.btn-ghost { background: var(--pearl-mist); color: var(--slate-fog); border: 1px solid var(--pearl-mist-dk); }
  .btn-sm.btn-ghost:hover { background: var(--pearl-mist-dk); color: var(--abyss-navy); }
</style>
