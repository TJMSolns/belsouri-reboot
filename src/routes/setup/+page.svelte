<script lang="ts">
  import PracticeTab from "$lib/components/setup/PracticeTab.svelte";
  import OfficesTab from "$lib/components/setup/OfficesTab.svelte";
  import ProvidersTab from "$lib/components/setup/ProvidersTab.svelte";
  import ProcedureTypesTab from "$lib/components/setup/ProcedureTypesTab.svelte";

  type Tab = "practice" | "offices" | "providers" | "procedures";
  let activeTab = $state<Tab>("practice");

  const tabs: { id: Tab; label: string }[] = [
    { id: "practice", label: "Practice" },
    { id: "offices", label: "Offices" },
    { id: "providers", label: "Providers" },
    { id: "procedures", label: "Procedure Types" },
  ];
</script>

<div class="setup-page">
  <header class="setup-header">
    <h1>Practice Setup</h1>
  </header>

  <div class="tab-bar" role="tablist" aria-label="Setup sections">
    {#each tabs as tab}
      <button
        class="tab-btn"
        class:active={activeTab === tab.id}
        role="tab"
        aria-selected={activeTab === tab.id}
        onclick={() => (activeTab = tab.id)}
      >
        {tab.label}
      </button>
    {/each}
  </div>

  <div class="tab-content">
    {#if activeTab === "practice"}
      <PracticeTab />
    {:else if activeTab === "offices"}
      <OfficesTab />
    {:else if activeTab === "providers"}
      <ProvidersTab />
    {:else if activeTab === "procedures"}
      <ProcedureTypesTab />
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
