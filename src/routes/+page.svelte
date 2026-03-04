<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { getErrorMessage } from "$lib/utils/api";

  interface ModuleStatusDto {
    module_name: string;
    status: string;
    expires_at: string;
    grace_period_days: number;
    grace_expires_at: string | null;
    days_remaining: number | null;
  }

  interface LicenseStatusDto {
    overall_validity: string;
    license_type: string | null;
    eval_expires_at: string | null;
    modules: ModuleStatusDto[];
    last_validated_at: string | null;
  }

  let licenseStatus = $state<LicenseStatusDto | null>(null);
  let practiceId = $state<string | null>(null);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      licenseStatus = await invoke<LicenseStatusDto>("get_license_status");
      practiceId = await invoke<string | null>("get_practice_id");
    } catch (e) {
      error = getErrorMessage(e);
    }
  });
</script>

<main class="container">
  <h1>Belsouri</h1>
  <p>Offline-First Dental Practice Platform</p>

  {#if error}
    <p class="error">Error: {error}</p>
  {:else if licenseStatus}
    <section>
      <h2>License</h2>
      <p>Validity: <strong>{licenseStatus.overall_validity}</strong></p>
      <p>Type: {licenseStatus.license_type ?? "—"}</p>
      {#if licenseStatus.eval_expires_at}
        <p>Eval expires: {licenseStatus.eval_expires_at}</p>
      {/if}
      {#each licenseStatus.modules as mod}
        <p>
          {mod.module_name}: <strong>{mod.status}</strong>
          {#if mod.days_remaining !== null}({mod.days_remaining} days remaining){/if}
        </p>
      {/each}
    </section>

    <section>
      <h2>Identity</h2>
      <p>Practice ID: <code>{practiceId ?? "—"}</code></p>
    </section>
  {:else}
    <p>Loading...</p>
  {/if}
</main>

<style>
  .container {
    padding: 2rem;
    font-family: system-ui, sans-serif;
  }
  .error {
    color: red;
  }
  section {
    margin-top: 1.5rem;
    padding: 1rem;
    border: 1px solid #ddd;
    border-radius: 8px;
  }
  code {
    font-size: 0.75rem;
    word-break: break-all;
  }
</style>
