<script lang="ts">
  import { onMount } from "svelte";
  import { commands } from "$lib/bindings";
  import type { PracticeDto, OfficeDto, StaffMemberDto, ProcedureTypeDto } from "$lib/bindings";
  import PracticeTab from "$lib/components/setup/PracticeTab.svelte";
  import OfficesTab from "$lib/components/setup/OfficesTab.svelte";
  import ProvidersTab from "$lib/components/setup/ProvidersTab.svelte";
  import ProcedureTypesTab from "$lib/components/setup/ProcedureTypesTab.svelte";
  import DemoDataTab from "$lib/components/setup/DemoDataTab.svelte";
  import SetupChecklist from "$lib/components/setup/SetupChecklist.svelte";

  type Tab = "practice" | "offices" | "providers" | "procedures" | "demo";
  let activeTab = $state<Tab>("practice");

  const tabs: { id: Tab; label: string }[] = [
    { id: "practice", label: "Practice" },
    { id: "offices", label: "Offices" },
    { id: "providers", label: "Providers" },
    { id: "procedures", label: "Procedure Types" },
    { id: "demo", label: "Demo Data" },
  ];

  // ── Checklist data ──────────────────────────────────────────────
  let practice = $state<PracticeDto | null>(null);
  let offices = $state<OfficeDto[]>([]);
  let providers = $state<StaffMemberDto[]>([]);
  let procedureTypes = $state<ProcedureTypeDto[]>([]);

  async function reloadSetupData() {
    const [pRes, oRes, pvRes, ptRes] = await Promise.all([
      commands.getPractice(),
      commands.listOffices(),
      commands.listProviders(),
      commands.listProcedureTypes(),
    ]);
    if (pRes.status === "ok") practice = pRes.data;
    if (oRes.status === "ok") offices = oRes.data;
    if (pvRes.status === "ok") providers = pvRes.data;
    if (ptRes.status === "ok") procedureTypes = ptRes.data;
  }

  onMount(() => {
    reloadSetupData();
  });

  // Reload checklist data whenever the active tab changes so the checklist
  // reflects any changes the user just made on a tab.
  $effect(() => {
    if (activeTab) reloadSetupData();
  });

  let practiceComplete = $derived(!!practice?.name);
  let officesComplete = $derived(offices.filter((o) => !o.archived).length > 0);
  let providersComplete = $derived(providers.filter((p) =>
    !p.archived &&
    p.roles.includes("Provider") &&
    p.clinical_specialization != null &&
    p.office_ids.length > 0 &&
    p.availability.length > 0
  ).length > 0);
  let proceduresComplete = $derived(procedureTypes.filter((pt) => pt.is_active).length > 0);
</script>

<div class="setup-page">
  <header class="setup-header">
    <h1>Practice Setup</h1>
  </header>

  <SetupChecklist
    {practiceComplete}
    {officesComplete}
    {providersComplete}
    {proceduresComplete}
    onGoTo={(tab) => (activeTab = tab as Tab)}
  />

  <div class="tab-bar" role="tablist" aria-label="Setup sections">
    {#each tabs as tab}
      <button
        class="tab-btn"
        class:active={activeTab === tab.id}
        id="tab-{tab.id}"
        role="tab"
        aria-selected={activeTab === tab.id}
        aria-controls="tabpanel-{tab.id}"
        onclick={() => (activeTab = tab.id)}
      >
        {tab.label}
      </button>
    {/each}
  </div>

  <div class="tab-content" id="tabpanel-{activeTab}" role="tabpanel" aria-labelledby="tab-{activeTab}">
    {#if activeTab === "practice"}
      <PracticeTab />
    {:else if activeTab === "offices"}
      <OfficesTab />
    {:else if activeTab === "providers"}
      <ProvidersTab />
    {:else if activeTab === "procedures"}
      <ProcedureTypesTab />
    {:else if activeTab === "demo"}
      <DemoDataTab />
    {/if}
  </div>
</div>

<style>
  .setup-page {
    max-width: 960px;
    margin: 0 auto;
    padding: var(--space-6);
  }
  .setup-header h1 {
    margin: 0 0 var(--space-5);
    font-size: var(--text-2xl);
    font-family: var(--font-heading);
    font-weight: 700;
    color: var(--abyss-navy);
  }
  .tab-bar {
    display: flex;
    gap: 0;
    border-bottom: 2px solid var(--pearl-mist-dk);
    margin-bottom: var(--space-6);
  }
  .tab-btn {
    padding: var(--space-2) var(--space-5);
    border: none;
    background: none;
    cursor: pointer;
    font-size: var(--text-sm);
    font-family: var(--font-body);
    font-weight: 500;
    color: var(--slate-fog);
    border-bottom: 2px solid transparent;
    margin-bottom: -2px;
    transition: color var(--transition-fast), border-color var(--transition-fast);
  }
  .tab-btn:hover { color: var(--abyss-navy); }
  .tab-btn.active {
    color: var(--caribbean-teal);
    font-weight: 600;
    border-bottom-color: var(--caribbean-teal);
  }
  .tab-content { min-height: 400px; }
</style>
