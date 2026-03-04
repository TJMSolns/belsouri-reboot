<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { getErrorMessage } from "$lib/utils/api";

  let { children } = $props();

  onMount(async () => {
    try {
      await invoke("startup_license_check");
    } catch (e) {
      console.error("Startup license check failed:", getErrorMessage(e));
    }
  });
</script>

{@render children()}
