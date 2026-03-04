<script lang="ts">
  import { commands, type PracticeDto } from "$lib/bindings";
  import { onMount } from "svelte";

  let name = $state("");
  let phone = $state("");
  let email = $state("");
  let website = $state("");
  let address_line_1 = $state("");
  let address_line_2 = $state("");
  let city_town = $state("");
  let subdivision = $state("");
  let country = $state("Jamaica");

  let saving = $state(false);
  let saved = $state(false);
  let error = $state<string | null>(null);

  let subdivisionLabel = $derived(country === "Jamaica" ? "Parish" : "Region");

  onMount(async () => {
    const r = await commands.getPractice();
    if (r.status === "ok" && r.data) {
      populate(r.data);
    } else if (r.status === "error") {
      error = r.error;
    }
  });

  function populate(p: PracticeDto) {
    name = p.name;
    phone = p.phone ?? "";
    email = p.email ?? "";
    website = p.website ?? "";
    address_line_1 = p.address_line_1 ?? "";
    address_line_2 = p.address_line_2 ?? "";
    city_town = p.city_town ?? "";
    subdivision = p.subdivision ?? "";
    country = p.country ?? "Jamaica";
  }

  async function save() {
    if (!name.trim()) { error = "Practice name is required"; return; }
    saving = true;
    error = null;
    const r = await commands.updatePracticeDetails(
      name.trim(),
      phone.trim() || null,
      email.trim() || null,
      website.trim() || null,
      address_line_1.trim() || null,
      address_line_2.trim() || null,
      city_town.trim() || null,
      subdivision.trim() || null,
      country.trim() || null,
    );
    saving = false;
    if (r.status === "ok") {
      populate(r.data);
      saved = true;
      setTimeout(() => (saved = false), 2500);
    } else {
      error = r.error;
    }
  }
</script>

<form class="form" onsubmit={(e) => { e.preventDefault(); save(); }}>
  <h2>Practice Details</h2>

  {#if error}
    <p class="form-error">{error}</p>
  {/if}

  <div class="field">
    <label for="pname">Practice Name <span class="req">*</span></label>
    <input id="pname" bind:value={name} placeholder="e.g. Smile Dental" />
  </div>

  <div class="row">
    <div class="field">
      <label for="phone">Phone</label>
      <input id="phone" bind:value={phone} placeholder="+1-876-555-0100" />
    </div>
    <div class="field">
      <label for="email">Email</label>
      <input id="email" type="email" bind:value={email} placeholder="info@smildental.com" />
    </div>
  </div>

  <div class="field">
    <label for="website">Website</label>
    <input id="website" bind:value={website} placeholder="https://smilental.com" />
  </div>

  <div class="field">
    <label for="addr1">Address Line 1</label>
    <input id="addr1" bind:value={address_line_1} placeholder="123 Main Street" />
  </div>

  <div class="field">
    <label for="addr2">Address Line 2</label>
    <input id="addr2" bind:value={address_line_2} placeholder="Suite 4" />
  </div>

  <div class="row">
    <div class="field">
      <label for="city">City / Town</label>
      <input id="city" bind:value={city_town} placeholder="Kingston" />
    </div>
    <div class="field">
      <label for="subdivision">{subdivisionLabel}</label>
      <input id="subdivision" bind:value={subdivision} placeholder="Kingston" />
    </div>
  </div>

  <div class="field" style="max-width: 200px">
    <label for="country">Country</label>
    <input id="country" bind:value={country} placeholder="Jamaica" />
  </div>

  <div class="actions">
    <button type="submit" class="btn-primary" disabled={saving}>
      {saving ? "Saving…" : "Save"}
    </button>
    {#if saved}
      <span class="saved-msg">Saved!</span>
    {/if}
  </div>
</form>

<style>
  .form { max-width: 560px; }
  h2 { margin: 0 0 1.25rem; font-size: 1.1rem; color: #222; }
  .form-error { color: #c0392b; font-size: 0.875rem; margin-bottom: 0.75rem; }
  .field { display: flex; flex-direction: column; gap: 0.3rem; margin-bottom: 0.9rem; }
  .field label { font-size: 0.8rem; font-weight: 600; color: #555; text-transform: uppercase; letter-spacing: 0.03em; }
  .req { color: #c0392b; }
  input {
    padding: 0.5rem 0.7rem;
    border: 1px solid #ccc;
    border-radius: 6px;
    font-size: 0.9rem;
    font-family: system-ui, sans-serif;
    width: 100%;
    box-sizing: border-box;
  }
  input:focus { outline: none; border-color: #1a1a2e; box-shadow: 0 0 0 2px rgba(26,26,46,0.15); }
  .row { display: flex; gap: 1rem; }
  .row .field { flex: 1; }
  .actions { display: flex; align-items: center; gap: 1rem; margin-top: 1.25rem; }
  .btn-primary {
    padding: 0.5rem 1.5rem;
    background: #1a1a2e;
    color: white;
    border: none;
    border-radius: 6px;
    font-size: 0.9rem;
    cursor: pointer;
    font-family: system-ui, sans-serif;
  }
  .btn-primary:hover:not(:disabled) { background: #2a2a4e; }
  .btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
  .saved-msg { color: #27ae60; font-size: 0.875rem; font-weight: 600; }
</style>
