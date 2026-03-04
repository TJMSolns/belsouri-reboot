<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { getErrorMessage } from "$lib/utils/api";
  import { page } from "$app/stores";

  let { children } = $props();

  onMount(async () => {
    try {
      await invoke("startup_license_check");
    } catch (e) {
      console.error("Startup license check failed:", getErrorMessage(e));
    }
  });
</script>

<nav class="app-nav">
  <a href="/" class="brand">Belsouri</a>
  <a href="/setup" class:active={$page.url.pathname.startsWith("/setup")}>Setup</a>
</nav>

{@render children()}

<style>
  .app-nav {
    display: flex;
    align-items: center;
    gap: 1.5rem;
    padding: 0.6rem 2rem;
    background: #1a1a2e;
    border-bottom: 1px solid #333;
  }
  .brand {
    font-weight: bold;
    font-size: 1rem;
    margin-right: 0.5rem;
    color: #7eb8f7;
    text-decoration: none;
    font-family: system-ui, sans-serif;
  }
  .app-nav a {
    color: #aaa;
    text-decoration: none;
    font-family: system-ui, sans-serif;
    font-size: 0.875rem;
    padding: 0.2rem 0.5rem;
    border-radius: 4px;
  }
  .app-nav a:hover { color: #fff; }
  .app-nav a.active { color: #fff; background: rgba(255,255,255,0.1); }
</style>
