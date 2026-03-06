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

  import { toast } from "$lib/stores/toast";

  let saving = $state(false);
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
    if (!name.trim()) { error = "Practice name is required."; return; }
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
      toast.success(`Practice details saved for "${name.trim()}".`);
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
    <label for="pname">Practice Name <span class="required-mark" aria-hidden="true">*</span></label>
    <input id="pname" bind:value={name} placeholder="e.g. Smile Dental" />
  </div>

  <div class="row">
    <div class="field">
      <label for="phone">Phone</label>
      <input id="phone" bind:value={phone} placeholder="+1-876-555-0100" />
    </div>
    <div class="field">
      <label for="email">Email</label>
      <input id="email" type="email" bind:value={email} placeholder="info@smiledental.com" />
    </div>
  </div>

  <div class="field">
    <label for="website">Website</label>
    <input id="website" bind:value={website} placeholder="https://smiledental.com" />
  </div>

  <div class="field">
    <label for="addr1">Address Line 1</label>
    <input id="addr1" bind:value={address_line_1} placeholder="e.g. 12 Harbour Street" />
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
      {#if saving}<span class="spinner-btn" aria-hidden="true"></span>{/if}
      {saving ? "Saving…" : "Save"}
    </button>
  </div>
</form>

<style>
  .form { max-width: 560px; }
  h2 { margin: 0 0 var(--space-5); font-size: var(--text-xl); font-family: var(--font-heading); font-weight: 600; color: var(--abyss-navy); }
  .form-error { color: var(--healthy-coral-dk); font-size: var(--text-sm); margin-bottom: var(--space-3); }
  .field { display: flex; flex-direction: column; gap: var(--space-1); margin-bottom: var(--space-4); }
  .field label { font-size: var(--text-xs); font-weight: 600; color: var(--abyss-navy); font-family: var(--font-body); }
  .row { display: flex; gap: var(--space-4); }
  .row .field { flex: 1; }
  .actions { display: flex; align-items: center; gap: var(--space-4); margin-top: var(--space-5); }
  .btn-primary {
    display: inline-flex; align-items: center; min-height: 44px; padding: 0 var(--space-6);
    background: var(--caribbean-teal); color: white; border: none;
    border-radius: var(--radius-md); font-family: var(--font-heading); font-size: var(--text-sm);
    font-weight: 600; cursor: pointer; transition: background var(--transition-fast);
  }
  .btn-primary:hover:not(:disabled) { background: var(--caribbean-teal-dk); }
  .btn-primary:disabled { opacity: 0.45; cursor: not-allowed; }
</style>
