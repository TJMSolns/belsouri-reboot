<script lang="ts">
  import "../app.css";
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { getErrorMessage } from "$lib/utils/api";
  import { page } from "$app/stores";
  import Toast from "$lib/components/Toast.svelte";
  import ConfirmDialog from "$lib/components/ConfirmDialog.svelte";

  let { children } = $props();

  onMount(async () => {
    try {
      await invoke("startup_license_check");
    } catch (e) {
      console.error("Startup license check failed:", getErrorMessage(e));
    }
  });
</script>

<a href="#main-content" class="skip-link">Skip to main content</a>

<nav class="app-nav" aria-label="Main navigation">
  <a href="/" class="brand" aria-label="Belsouri home">Belsouri</a>
  <div class="nav-links">
    <a href="/patients"
       class:active={$page.url.pathname.startsWith("/patients")}
       aria-current={$page.url.pathname.startsWith("/patients") ? "page" : undefined}>
      Patients
    </a>
    <a href="/staff"
       class:active={$page.url.pathname.startsWith("/staff")}
       aria-current={$page.url.pathname.startsWith("/staff") ? "page" : undefined}>
      Staff
    </a>
    <a href="/schedule"
       class:active={$page.url.pathname.startsWith("/schedule")}
       aria-current={$page.url.pathname.startsWith("/schedule") ? "page" : undefined}>
      Schedule
    </a>
    <a href="/setup"
       class:active={$page.url.pathname.startsWith("/setup")}
       aria-current={$page.url.pathname.startsWith("/setup") ? "page" : undefined}>
      Setup
    </a>
  </div>
</nav>

<main id="main-content">{@render children()}</main>

<Toast />
<ConfirmDialog />

<style>
  .skip-link {
    position: absolute;
    top: -100%;
    left: var(--space-4, 16px);
    z-index: 200;
    padding: var(--space-2, 8px) var(--space-4, 16px);
    background: var(--caribbean-teal, #008B99);
    color: #fff;
    font-family: var(--font-body, 'Inter', sans-serif);
    font-size: var(--text-sm, 0.875rem);
    font-weight: 600;
    border-radius: 0 0 var(--radius-sm, 4px) var(--radius-sm, 4px);
    text-decoration: none;
    transition: top 0.1s;
  }
  .skip-link:focus { top: 0; }

  .app-nav {
    position: sticky;
    top: 0;
    z-index: 100;
    display: flex;
    align-items: center;
    gap: 0;
    height: var(--nav-height, 56px);
    padding: 0 var(--space-6, 24px);
    background: var(--abyss-navy, #1A2D33);
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.25);
  }

  .brand {
    font-family: var(--font-heading, 'Lexend', sans-serif);
    font-size: 1.125rem;
    font-weight: 700;
    color: var(--caribbean-teal, #008B99);
    text-decoration: none;
    margin-right: var(--space-8, 32px);
    letter-spacing: -0.01em;
    flex-shrink: 0;
  }
  .brand:hover { color: #00a8b8; }

  .nav-links {
    display: flex;
    align-items: stretch;
    gap: 0;
    height: 100%;
  }

  .nav-links a {
    display: flex;
    align-items: center;
    padding: 0 var(--space-5, 20px);
    color: rgba(240, 244, 245, 0.65);
    text-decoration: none;
    font-family: var(--font-body, 'Inter', sans-serif);
    font-size: var(--text-sm, 0.875rem);
    font-weight: 500;
    border-bottom: 3px solid transparent;
    transition: color var(--transition-fast, 100ms), border-color var(--transition-fast, 100ms);
    white-space: nowrap;
  }

  .nav-links a:hover {
    color: #fff;
  }

  .nav-links a.active {
    color: #fff;
    border-bottom-color: var(--caribbean-teal, #008B99);
  }
</style>
