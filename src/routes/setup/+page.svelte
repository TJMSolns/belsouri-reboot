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
    font-family: system-ui, sans-serif;
    max-width: 900px;
    margin: 0 auto;
    padding: 1.5rem 2rem;
  }
  .setup-header h1 {
    margin: 0 0 1rem;
    font-size: 1.5rem;
    color: #1a1a2e;
  }
  .tab-bar {
    display: flex;
    gap: 0;
    border-bottom: 2px solid #ddd;
    margin-bottom: 1.5rem;
  }
  .tab-btn {
    padding: 0.6rem 1.25rem;
    border: none;
    background: none;
    cursor: pointer;
    font-size: 0.9rem;
    font-family: system-ui, sans-serif;
    color: #666;
    border-bottom: 2px solid transparent;
    margin-bottom: -2px;
  }
  .tab-btn:hover { color: #333; }
  .tab-btn.active {
    color: #1a1a2e;
    font-weight: 600;
    border-bottom-color: #1a1a2e;
  }
  .tab-content {
    min-height: 400px;
  }
</style>
