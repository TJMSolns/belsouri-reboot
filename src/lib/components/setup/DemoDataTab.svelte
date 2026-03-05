<script lang="ts">
  import { commands } from "$lib/bindings";
  import { toast } from "$lib/stores/toast";

  let seeding = $state(false);
  let archiving = $state(false);
  let seedError = $state<string | null>(null);
  let archiveError = $state<string | null>(null);

  async function doSeed() {
    seeding = true;
    seedError = null;
    const r = await commands.seedDemoData();
    seeding = false;
    if (r.status === "ok") {
      toast.success(
        `Demo data seeded: ${r.data.patients_created} patients, ${r.data.providers_created} providers, ${r.data.staff_created} staff members.`
      );
    } else {
      seedError = r.error;
    }
  }

  async function doArchive() {
    archiving = true;
    archiveError = null;
    const r = await commands.archiveDemoData();
    archiving = false;
    if (r.status === "ok") {
      toast.success(
        `Demo data archived: ${r.data.patients_archived} patients, ${r.data.providers_archived} providers, ${r.data.staff_archived} staff members.`
      );
    } else {
      archiveError = r.error;
    }
  }
</script>

<div class="demo-tab">
  <h2>Demo Data</h2>
  <p class="demo-desc">
    Seed the practice with sample Caribbean names for testing and demonstration. All records use
    role-as-last-name so they are easy to identify and remove.
  </p>

  <div class="demo-cards">
    <div class="demo-card">
      <div class="demo-card-header">
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor"
          stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <circle cx="12" cy="8" r="4"/>
          <path d="M6 20v-2a4 4 0 0 1 4-4h4a4 4 0 0 1 4 4v2"/>
        </svg>
        <h3>Seed Demo Data</h3>
      </div>
      <ul class="demo-list">
        <li>10 patients — e.g. <em>Marcus Patient, Asha Patient</em></li>
        <li>6 providers — 2 Specialists, 2 Dentists, 2 Hygienists</li>
        <li>2 staff members — e.g. <em>Andre Staff, Yolanda Staff</em></li>
      </ul>
      {#if seedError}
        <p class="field-error">{seedError}</p>
      {/if}
      <button class="btn-primary" onclick={doSeed} disabled={seeding || archiving}>
        {#if seeding}
          <span class="spinner" aria-hidden="true"></span> Seeding…
        {:else}
          Seed Demo Data
        {/if}
      </button>
    </div>

    <div class="demo-card demo-card--danger">
      <div class="demo-card-header">
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor"
          stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <polyline points="3 6 5 6 21 6"/>
          <path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"/>
          <path d="M10 11v6"/>
          <path d="M14 11v6"/>
          <path d="M9 6V4a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2"/>
        </svg>
        <h3>Archive Demo Data</h3>
      </div>
      <p class="demo-card-note">
        Archives all patients with last name "Patient", providers ending in Specialist / Dentist /
        Hygienist, and staff ending in "Staff". Real records are not affected.
      </p>
      {#if archiveError}
        <p class="field-error">{archiveError}</p>
      {/if}
      <button class="btn-outline btn-outline--danger" onclick={doArchive} disabled={seeding || archiving}>
        {#if archiving}
          <span class="spinner" aria-hidden="true"></span> Archiving…
        {:else}
          Archive Demo Data
        {/if}
      </button>
    </div>
  </div>
</div>

<style>
  .demo-tab {
    max-width: 680px;
  }
  .demo-tab h2 {
    margin: 0 0 var(--space-2);
    font-size: var(--text-xl);
    font-family: var(--font-heading);
    font-weight: 700;
    color: var(--abyss-navy);
  }
  .demo-desc {
    margin: 0 0 var(--space-6);
    font-size: var(--text-sm);
    color: var(--slate-fog);
    font-family: var(--font-body);
  }
  .demo-cards {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-4);
  }
  .demo-card {
    padding: var(--space-5);
    border: 1px solid var(--pearl-mist-dk);
    border-radius: var(--radius-lg);
    background: var(--pearl-mist);
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }
  .demo-card--danger {
    border-color: var(--healthy-coral);
    background: var(--healthy-coral-lt, #fff5f5);
  }
  .demo-card-header {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    color: var(--abyss-navy);
  }
  .demo-card-header h3 {
    margin: 0;
    font-size: var(--text-base);
    font-family: var(--font-heading);
    font-weight: 600;
    color: var(--abyss-navy);
  }
  .demo-list {
    margin: 0;
    padding-left: var(--space-4);
    font-size: var(--text-sm);
    font-family: var(--font-body);
    color: var(--abyss-navy);
    line-height: 1.7;
  }
  .demo-card-note {
    margin: 0;
    font-size: var(--text-sm);
    font-family: var(--font-body);
    color: var(--slate-fog);
    line-height: 1.5;
  }
  .field-error {
    margin: 0;
    font-size: var(--text-sm);
    font-family: var(--font-body);
    color: var(--healthy-coral);
  }
  .btn-outline--danger {
    color: var(--healthy-coral);
    border-color: var(--healthy-coral);
  }
  .btn-outline--danger:hover:not(:disabled) {
    background: var(--healthy-coral);
    color: white;
  }
  .spinner {
    display: inline-block;
    width: 14px;
    height: 14px;
    border: 2px solid currentColor;
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
    vertical-align: middle;
  }
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
