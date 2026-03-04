<script lang="ts">
  import { commands, type StaffMemberDto } from "$lib/bindings";
  import { onMount } from "svelte";

  let staff = $state<StaffMemberDto[]>([]);
  let error = $state<string | null>(null);
  let setupStatus = $state<{ complete: boolean } | null>(null);

  // Claim PM form (first-run bootstrap)
  let showClaim = $state(false);
  let claimName = $state("");
  let claiming = $state(false);
  let claimError = $state<string | null>(null);

  // Register form
  let showRegister = $state(false);
  let regName = $state("");
  let regPhone = $state("");
  let regEmail = $state("");
  let regChannel = $state("");
  let regRole = $state("Staff");
  let registering = $state(false);
  let regError = $state<string | null>(null);

  // Expanded staff member
  let expandedId = $state<string | null>(null);
  let expandError = $state<string | null>(null);

  // PIN operations
  let pinSection = $state<"set" | "change" | null>(null);
  let newPin = $state("");
  let currentPin = $state("");
  let pinSaving = $state(false);
  let pinError = $state<string | null>(null);

  // Role operations
  let roleToAdd = $state("Staff");
  let roleAdding = $state(false);
  let roleError = $state<string | null>(null);

  // PIN verify (identity switch)
  let verifyPin = $state("");
  let verifyResult = $state<boolean | null>(null);
  let verifying = $state(false);

  const ROLES = ["PracticeManager", "Provider", "Staff"];
  const CHANNELS = ["", "WhatsApp", "SMS", "Phone", "Email"];

  onMount(load);

  async function load() {
    error = null;
    const [staffR, statusR] = await Promise.all([
      commands.listStaffMembers(),
      commands.getStaffSetupStatus(),
    ]);
    if (staffR.status === "ok") staff = staffR.data;
    else error = staffR.error;
    if (statusR.status === "ok") setupStatus = statusR.data;
  }

  function activeStaff() { return staff.filter((s) => !s.archived); }
  function archivedStaff() { return staff.filter((s) => s.archived); }
  let hasActivePM = $derived(staff.some((s) => !s.archived && s.roles.includes("PracticeManager")));

  async function claim() {
    if (!claimName.trim()) { claimError = "Name is required"; return; }
    claiming = true; claimError = null;
    const r = await commands.claimPracticeManagerRole(claimName.trim());
    claiming = false;
    if (r.status === "ok") {
      staff = [r.data, ...staff];
      claimName = ""; showClaim = false;
      await load(); // refresh setup status
    } else { claimError = r.error; }
  }

  async function registerStaff() {
    if (!regName.trim()) { regError = "Name is required"; return; }
    registering = true; regError = null;
    const r = await commands.registerStaffMember(
      regName.trim(),
      regPhone.trim() || null,
      regEmail.trim() || null,
      regChannel || null,
      regRole,
    );
    registering = false;
    if (r.status === "ok") {
      staff = [...staff, r.data].sort((a, b) => a.name.localeCompare(b.name));
      regName = ""; regPhone = ""; regEmail = ""; regChannel = ""; regRole = "Staff";
      showRegister = false;
    } else { regError = r.error; }
  }

  function toggleExpand(id: string) {
    if (expandedId === id) {
      expandedId = null;
      pinSection = null;
      verifyResult = null;
    } else {
      expandedId = id;
      pinSection = null;
      pinError = null;
      roleError = null;
      verifyResult = null;
    }
  }

  async function doSetPin(id: string) {
    if (!newPin) return;
    pinSaving = true; pinError = null;
    const r = await commands.setPin(id, newPin);
    pinSaving = false;
    if (r.status === "ok") {
      staff = staff.map((s) => s.staff_member_id === id ? r.data : s);
      newPin = ""; pinSection = null;
    } else { pinError = r.error; }
  }

  async function doChangePin(id: string) {
    if (!currentPin || !newPin) return;
    pinSaving = true; pinError = null;
    const r = await commands.changePin(id, currentPin, newPin);
    pinSaving = false;
    if (r.status === "ok") {
      staff = staff.map((s) => s.staff_member_id === id ? r.data : s);
      currentPin = ""; newPin = ""; pinSection = null;
    } else { pinError = r.error; }
  }

  async function doResetPin(target_id: string) {
    if (!confirm("Reset this staff member's PIN? They will need to set a new one before switching identity.")) return;
    // The PM executing this action — we'd normally have a current user ID.
    // For MVP without full auth, we use the first active PM.
    const pm = staff.find((s) => !s.archived && s.roles.includes("PracticeManager"));
    if (!pm) { expandError = "No active Practice Manager found"; return; }
    pinSaving = true; pinError = null;
    const r = await commands.resetPin(target_id, pm.staff_member_id);
    pinSaving = false;
    if (r.status === "ok") {
      staff = staff.map((s) => s.staff_member_id === target_id ? r.data : s);
    } else { pinError = r.error; }
  }

  async function doAssignRole(id: string) {
    roleAdding = true; roleError = null;
    const r = await commands.assignRole(id, roleToAdd);
    roleAdding = false;
    if (r.status === "ok") {
      staff = staff.map((s) => s.staff_member_id === id ? r.data : s);
    } else { roleError = r.error; }
  }

  async function doRemoveRole(id: string, role: string) {
    roleError = null;
    const r = await commands.removeRole(id, role);
    if (r.status === "ok") {
      staff = staff.map((s) => s.staff_member_id === id ? r.data : s);
    } else { roleError = r.error; }
  }

  async function doVerifyPin(id: string) {
    if (!verifyPin) return;
    verifying = true; verifyResult = null;
    const r = await commands.verifyStaffPin(id, verifyPin);
    verifying = false;
    if (r.status === "ok") {
      verifyResult = r.data;
    }
  }

  async function doArchive(id: string) {
    if (!confirm("Archive this staff member?")) return;
    const r = await commands.archiveStaffMember(id);
    if (r.status === "ok") {
      staff = staff.map((s) => s.staff_member_id === id ? r.data : s);
      if (expandedId === id) expandedId = null;
    } else { expandError = r.error; }
  }

  async function doUnarchive(id: string) {
    const r = await commands.unarchiveStaffMember(id);
    if (r.status === "ok") {
      staff = staff.map((s) => s.staff_member_id === id ? r.data : s);
    } else { expandError = r.error; }
  }

  function roleLabel(roles: string[]): string {
    return roles.join(", ") || "—";
  }
</script>

<div class="page">
  <div class="page-header">
    <h1>Staff</h1>
    <div class="header-actions">
      {#if !hasActivePM}
        <button class="btn-primary" onclick={() => { showClaim = !showClaim; claimError = null; }}>
          {showClaim ? "Cancel" : "Claim Practice Manager Role"}
        </button>
      {:else}
        <button class="btn-primary" onclick={() => { showRegister = !showRegister; regError = null; }}>
          {showRegister ? "Cancel" : "+ Register Staff"}
        </button>
      {/if}
    </div>
  </div>

  {#if setupStatus !== null}
    <div class="setup-status" class:complete={setupStatus.complete}>
      {setupStatus.complete
        ? "✓ Staff setup complete — at least one Practice Manager has a PIN set."
        : "⚠ Staff setup incomplete — set a PIN for at least one Practice Manager."}
    </div>
  {/if}

  {#if error}<p class="error">{error}</p>{/if}
  {#if expandError}<p class="error">{expandError}</p>{/if}

  <!-- Claim PM form (first run) -->
  {#if showClaim}
    <div class="card form-card">
      <h3>Claim Practice Manager Role</h3>
      <p class="hint">No Practice Manager exists yet. Enter your name to become the first Practice Manager.</p>
      {#if claimError}<p class="error">{claimError}</p>{/if}
      <div class="field">
        <label>Name</label>
        <input bind:value={claimName} placeholder="Dr. Spence" />
      </div>
      <div class="form-actions">
        <button class="btn-primary" onclick={claim} disabled={claiming}>
          {claiming ? "Claiming…" : "Claim"}
        </button>
      </div>
    </div>
  {/if}

  <!-- Register staff form -->
  {#if showRegister}
    <div class="card form-card">
      <h3>Register Staff Member</h3>
      {#if regError}<p class="error">{regError}</p>{/if}
      <div class="form-row">
        <div class="field">
          <label>Name *</label>
          <input bind:value={regName} placeholder="Maria Brown" />
        </div>
        <div class="field" style="max-width:140px">
          <label>Initial Role</label>
          <select bind:value={regRole}>
            {#each ROLES as r}<option>{r}</option>{/each}
          </select>
        </div>
      </div>
      <div class="form-row">
        <div class="field">
          <label>Phone</label>
          <input bind:value={regPhone} placeholder="+1-876-555-0100" />
        </div>
        <div class="field">
          <label>Email</label>
          <input type="email" bind:value={regEmail} />
        </div>
        <div class="field" style="max-width:140px">
          <label>Preferred Channel</label>
          <select bind:value={regChannel}>
            {#each CHANNELS as c}<option value={c}>{c || "—"}</option>{/each}
          </select>
        </div>
      </div>
      <div class="form-actions">
        <button class="btn-primary" onclick={registerStaff} disabled={registering}>
          {registering ? "Registering…" : "Register"}
        </button>
      </div>
    </div>
  {/if}

  <!-- Active staff list -->
  {#if activeStaff().length === 0 && !showClaim && !showRegister}
    <p class="empty">No staff registered yet.</p>
  {/if}

  <div class="staff-list">
    {#each activeStaff() as sm (sm.staff_member_id)}
      <div class="staff-card">
        <div
          class="staff-row"
          role="button"
          tabindex="0"
          onclick={() => toggleExpand(sm.staff_member_id)}
          onkeydown={(e) => e.key === "Enter" && toggleExpand(sm.staff_member_id)}
        >
          <div class="staff-info">
            <span class="staff-name">{sm.name}</span>
            {#each sm.roles as role}
              <span class="role-badge role-{role.toLowerCase()}">{role}</span>
            {/each}
            {#if !sm.has_pin}
              <span class="badge no-pin-badge">No PIN</span>
            {/if}
          </div>
          <span class="chevron">{expandedId === sm.staff_member_id ? "▲" : "▼"}</span>
        </div>

        {#if expandedId === sm.staff_member_id}
          <div class="detail-panel">
            <!-- Contact info -->
            <section class="detail-section">
              <h4>Contact</h4>
              <dl class="info-list">
                <dt>Phone</dt><dd>{sm.phone ?? "—"}</dd>
                <dt>Email</dt><dd>{sm.email ?? "—"}</dd>
                <dt>Channel</dt><dd>{sm.preferred_contact_channel ?? "—"}</dd>
              </dl>
            </section>

            <!-- Roles -->
            <section class="detail-section">
              <h4>Roles</h4>
              {#if roleError}<p class="error">{roleError}</p>{/if}
              <div class="roles-list">
                {#each sm.roles as role}
                  <span class="role-chip">
                    {role}
                    <button
                      class="remove-role-btn"
                      onclick={() => doRemoveRole(sm.staff_member_id, role)}
                      title="Remove role"
                    >✕</button>
                  </span>
                {/each}
              </div>
              <div class="add-role-row">
                <select bind:value={roleToAdd}>
                  {#each ROLES.filter((r) => !sm.roles.includes(r)) as r}<option>{r}</option>{/each}
                </select>
                <button class="btn-sm" onclick={() => doAssignRole(sm.staff_member_id)} disabled={roleAdding}>
                  {roleAdding ? "…" : "Add Role"}
                </button>
              </div>
            </section>

            <!-- PIN -->
            <section class="detail-section">
              <h4>PIN</h4>
              {#if pinError && expandedId === sm.staff_member_id}
                <p class="error">{pinError}</p>
              {/if}
              {#if pinSection === null}
                <div class="pin-actions">
                  {#if !sm.has_pin}
                    <button class="btn-sm" onclick={() => { pinSection = "set"; newPin = ""; }}>Set PIN</button>
                  {:else}
                    <button class="btn-sm btn-ghost" onclick={() => { pinSection = "change"; currentPin = ""; newPin = ""; }}>Change PIN</button>
                    <button class="btn-sm btn-ghost" onclick={() => doResetPin(sm.staff_member_id)} disabled={pinSaving}>Reset PIN</button>
                  {/if}
                </div>
              {:else if pinSection === "set"}
                <div class="pin-form">
                  <input type="password" inputmode="numeric" maxlength="6" placeholder="4–6 digits" bind:value={newPin} class="pin-input" />
                  <button class="btn-sm" onclick={() => doSetPin(sm.staff_member_id)} disabled={pinSaving}>
                    {pinSaving ? "Saving…" : "Save"}
                  </button>
                  <button class="btn-sm btn-ghost" onclick={() => pinSection = null}>Cancel</button>
                </div>
              {:else if pinSection === "change"}
                <div class="pin-form">
                  <input type="password" inputmode="numeric" maxlength="6" placeholder="Current PIN" bind:value={currentPin} class="pin-input" />
                  <input type="password" inputmode="numeric" maxlength="6" placeholder="New PIN" bind:value={newPin} class="pin-input" />
                  <button class="btn-sm" onclick={() => doChangePin(sm.staff_member_id)} disabled={pinSaving}>
                    {pinSaving ? "Saving…" : "Save"}
                  </button>
                  <button class="btn-sm btn-ghost" onclick={() => pinSection = null}>Cancel</button>
                </div>
              {/if}

              <!-- PIN verify (identity switch test) -->
              {#if sm.has_pin}
                <div class="verify-row">
                  <input type="password" inputmode="numeric" maxlength="6" placeholder="Verify PIN" bind:value={verifyPin} class="pin-input" />
                  <button class="btn-sm btn-ghost" onclick={() => doVerifyPin(sm.staff_member_id)} disabled={verifying}>
                    {verifying ? "…" : "Verify"}
                  </button>
                  {#if verifyResult === true}<span class="verify-ok">✓ Correct</span>{/if}
                  {#if verifyResult === false}<span class="verify-fail">✗ Incorrect</span>{/if}
                </div>
              {/if}
            </section>

            <!-- Archive -->
            <section class="detail-section archive-section">
              <button class="btn-danger-sm" onclick={() => doArchive(sm.staff_member_id)}>Archive Staff Member</button>
            </section>
          </div>
        {/if}
      </div>
    {/each}
  </div>

  <!-- Archived staff -->
  {#if archivedStaff().length > 0}
    <div class="archived-section">
      <h3 class="archived-heading">Archived</h3>
      <div class="staff-list">
        {#each archivedStaff() as sm (sm.staff_member_id)}
          <div class="staff-card archived">
            <div class="staff-row">
              <div class="staff-info">
                <span class="staff-name">{sm.name}</span>
                <span class="badge archived-badge">Archived</span>
                <span class="meta">{roleLabel(sm.roles)}</span>
              </div>
              <button class="btn-sm btn-ghost" onclick={() => doUnarchive(sm.staff_member_id)}>Unarchive</button>
            </div>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .page { padding: 1.5rem 2rem; max-width: 800px; }
  .page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.75rem; }
  h1 { margin: 0; font-size: 1.25rem; color: #222; font-family: system-ui, sans-serif; }
  h3 { margin: 0 0 0.75rem; font-size: 1rem; color: #222; font-family: system-ui, sans-serif; }
  h4 { margin: 0; font-size: 0.82rem; font-weight: 700; text-transform: uppercase;
       letter-spacing: 0.04em; color: #666; font-family: system-ui, sans-serif; }
  .header-actions { display: flex; gap: 0.5rem; }
  .error { color: #c0392b; font-size: 0.875rem; margin-bottom: 0.5rem; font-family: system-ui, sans-serif; }
  .empty { color: #999; font-style: italic; font-family: system-ui, sans-serif; }
  .hint { font-size: 0.875rem; color: #666; margin-bottom: 0.75rem; font-family: system-ui, sans-serif; }
  .meta { font-size: 0.82rem; color: #777; font-family: system-ui, sans-serif; }

  .setup-status {
    padding: 0.6rem 1rem; border-radius: 6px; font-size: 0.875rem;
    margin-bottom: 1rem; font-family: system-ui, sans-serif;
    background: #fff8e1; border: 1px solid #f0c040; color: #a06030;
  }
  .setup-status.complete { background: #eafaf1; border-color: #a9dfbf; color: #1e8449; }

  .card { background: white; border: 1px solid #e0e0e0; border-radius: 8px; padding: 1.25rem; }
  .form-card { margin-bottom: 1.25rem; }
  .form-row { display: flex; gap: 1rem; flex-wrap: wrap; margin-bottom: 0.75rem; }
  .form-actions { display: flex; gap: 0.75rem; margin-top: 0.75rem; }

  .field { display: flex; flex-direction: column; gap: 0.3rem; flex: 1; min-width: 140px; }
  .field label { font-size: 0.78rem; font-weight: 600; color: #555;
                 text-transform: uppercase; letter-spacing: 0.03em; font-family: system-ui, sans-serif; }
  input:not([type="password"]):not([type="email"]):not([type="checkbox"]),
  select {
    padding: 0.45rem 0.6rem; border: 1px solid #ccc; border-radius: 6px;
    font-size: 0.9rem; font-family: system-ui, sans-serif; width: 100%; box-sizing: border-box;
    background: white;
  }
  input:focus, select:focus { outline: none; border-color: #1a1a2e; }

  .staff-list { display: flex; flex-direction: column; gap: 0.6rem; }
  .staff-card { border: 1px solid #ddd; border-radius: 8px; overflow: hidden; background: white; }
  .staff-card.archived { opacity: 0.65; }
  .staff-row { display: flex; justify-content: space-between; align-items: center;
               padding: 0.75rem 1rem; cursor: pointer; user-select: none; }
  .staff-card.archived .staff-row { cursor: default; }
  .staff-row:hover { background: #f7f8fa; }
  .staff-info { display: flex; align-items: center; gap: 0.5rem; flex-wrap: wrap; }
  .staff-name { font-weight: 600; font-size: 0.95rem; font-family: system-ui, sans-serif; }
  .chevron { color: #aaa; font-size: 0.8rem; }

  .badge { font-size: 0.72rem; padding: 0.15rem 0.5rem; border-radius: 20px; font-weight: 600; font-family: system-ui, sans-serif; }
  .role-badge { font-size: 0.72rem; padding: 0.15rem 0.5rem; border-radius: 20px; font-weight: 600; font-family: system-ui, sans-serif; }
  .role-practicemanager { background: #e8f4f8; color: #1a7aae; }
  .role-provider { background: #eafaf1; color: #1e8449; }
  .role-staff { background: #f4f4f4; color: #555; }
  .no-pin-badge { background: #fff3cd; color: #856404; }
  .archived-badge { background: #f0e6d3; color: #a06030; }

  /* Detail panel */
  .detail-panel { border-top: 1px solid #eee; padding: 1rem; }
  .detail-section { margin-bottom: 1.25rem; }
  .detail-section:last-child { margin-bottom: 0; }
  .info-list { display: grid; grid-template-columns: 80px 1fr; gap: 0.3rem 0.75rem;
               margin: 0.5rem 0 0; font-size: 0.875rem; font-family: system-ui, sans-serif; }
  dt { color: #888; font-weight: 500; }
  dd { margin: 0; color: #222; }

  /* Roles */
  .roles-list { display: flex; gap: 0.4rem; flex-wrap: wrap; margin: 0.5rem 0; }
  .role-chip { display: flex; align-items: center; gap: 0.3rem; background: #f0f0f0;
               border-radius: 20px; padding: 0.2rem 0.5rem; font-size: 0.82rem; font-family: system-ui, sans-serif; }
  .remove-role-btn { background: none; border: none; cursor: pointer; color: #999;
                     font-size: 0.7rem; padding: 0; line-height: 1; }
  .remove-role-btn:hover { color: #c0392b; }
  .add-role-row { display: flex; gap: 0.5rem; align-items: center; margin-top: 0.5rem; }
  .add-role-row select { padding: 0.3rem 0.5rem; border: 1px solid #ccc; border-radius: 5px;
                         font-size: 0.85rem; background: white; font-family: system-ui, sans-serif; }

  /* PIN */
  .pin-input { padding: 0.4rem 0.6rem; border: 1px solid #ccc; border-radius: 6px;
               font-size: 0.9rem; font-family: monospace; width: 100px; box-sizing: border-box; }
  .pin-input:focus { outline: none; border-color: #1a1a2e; }
  .pin-actions { display: flex; gap: 0.5rem; margin-top: 0.4rem; }
  .pin-form { display: flex; gap: 0.5rem; align-items: center; flex-wrap: wrap; margin-top: 0.4rem; }
  .verify-row { display: flex; gap: 0.5rem; align-items: center; margin-top: 0.75rem; }
  .verify-ok { font-size: 0.85rem; color: #1e8449; font-weight: 600; font-family: system-ui, sans-serif; }
  .verify-fail { font-size: 0.85rem; color: #c0392b; font-weight: 600; font-family: system-ui, sans-serif; }

  /* Archive */
  .archive-section { border-top: 1px solid #f0f0f0; padding-top: 0.75rem; }
  .archived-section { margin-top: 1.5rem; }
  .archived-heading { font-size: 0.85rem; color: #aaa; text-transform: uppercase;
                      letter-spacing: 0.04em; margin-bottom: 0.5rem; font-family: system-ui, sans-serif; }

  /* Buttons */
  .btn-primary {
    padding: 0.45rem 1.1rem; background: #1a1a2e; color: white;
    border: none; border-radius: 6px; font-size: 0.875rem; cursor: pointer; font-family: system-ui, sans-serif;
  }
  .btn-primary:hover:not(:disabled) { background: #2a2a4e; }
  .btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn-sm {
    padding: 0.25rem 0.6rem; background: #1a1a2e; color: white;
    border: none; border-radius: 4px; font-size: 0.78rem; cursor: pointer; font-family: system-ui, sans-serif;
    white-space: nowrap;
  }
  .btn-sm:disabled { opacity: 0.4; cursor: not-allowed; }
  .btn-sm.btn-ghost { background: #eee; color: #555; }
  .btn-sm.btn-ghost:hover { background: #ddd; }
  .btn-danger-sm {
    padding: 0.35rem 0.75rem; background: white; color: #c0392b;
    border: 1px solid #c0392b; border-radius: 6px; font-size: 0.8rem; cursor: pointer; font-family: system-ui, sans-serif;
  }
  .btn-danger-sm:hover { background: #fdf0ef; }
</style>
