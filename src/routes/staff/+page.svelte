<script lang="ts">
  import { commands, type StaffMemberDto, type AppointmentDto, type StaffShiftDto } from "$lib/bindings";
  import { onMount } from "svelte";
  import { toast } from "$lib/stores/toast";
  import { confirm } from "$lib/stores/confirm";

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

  // ── Provider schedule section ─────────────────────────────────────────────

  let providers = $state<StaffMemberDto[]>([]);
  let expandedProviderId = $state<string | null>(null);
  let providerWeekStart = $state(getMondayOfWeek(todayStr()));
  let providerSchedule = $state<AppointmentDto[]>([]);
  let providerScheduleLoading = $state(false);
  let officeMap = $state<Record<string, string>>({});

  // STAFF-1: Sort / filter
  let searchQuery = $state("");
  let roleFilter = $state<"all" | "PracticeManager" | "Provider" | "Staff">("all");
  let sortAsc = $state(true);

  // STAFF-4: Shift grid in expanded card
  let memberShifts = $state<StaffShiftDto[]>([]);
  let memberShiftsLoading = $state(false);
  let memberShiftsWeekStart = $state(getMondayOfWeek(todayStr()));

  const ROLES = ["PracticeManager", "Provider", "Staff"];
  const CHANNELS = ["", "WhatsApp", "SMS", "Phone", "Email"];

  onMount(load);

  // ── Provider schedule helpers ──────────────────────────────────────────────

  function todayStr(): string { return new Date().toISOString().slice(0, 10); }

  function addDays(date: string, n: number): string {
    const d = new Date(date + "T12:00:00");
    d.setDate(d.getDate() + n);
    return d.toISOString().slice(0, 10);
  }

  function getMondayOfWeek(date: string): string {
    const d = new Date(date + "T12:00:00");
    const day = d.getDay(); // 0=Sun, 1=Mon, …
    const diff = day === 0 ? -6 : 1 - day;
    d.setDate(d.getDate() + diff);
    return d.toISOString().slice(0, 10);
  }

  function getWeekEnd(weekStart: string): string {
    return addDays(weekStart, 6);
  }

  function formatWeekRange(weekStart: string): string {
    const s = new Date(weekStart + "T12:00:00");
    const e = new Date(weekStart + "T12:00:00");
    e.setDate(e.getDate() + 6);
    const sFmt = s.toLocaleDateString("en-JM", { day: "numeric", month: "short" });
    const eFmt = e.toLocaleDateString("en-JM", { day: "numeric", month: "short", year: "numeric" });
    return `${sFmt} – ${eFmt}`;
  }

  function getWeekDays(weekStart: string): string[] {
    return Array.from({ length: 7 }, (_, i) => addDays(weekStart, i));
  }

  function formatDayHeader(date: string): string {
    return new Date(date + "T12:00:00").toLocaleDateString("en-JM", {
      weekday: "long", day: "numeric", month: "short",
    });
  }

  function formatApptTime(isoLocal: string): string {
    const [h, m] = isoLocal.slice(11, 16).split(":").map(Number);
    const period = h < 12 ? "AM" : "PM";
    const h12 = h % 12 === 0 ? 12 : h % 12;
    return `${h12}:${m.toString().padStart(2, "0")} ${period}`;
  }

  function parseHHMM(t: string): number {
    const [h, m] = t.split(":").map(Number);
    return h * 60 + m;
  }

  function minsTo12h(m: number): string {
    const h = Math.floor(m / 60);
    const min = m % 60;
    const period = h >= 12 ? "PM" : "AM";
    const h12 = h % 12 || 12;
    return `${h12}:${min.toString().padStart(2, "0")} ${period}`;
  }

  function dayAbbr(date: string): string {
    return new Date(date + "T12:00:00").toLocaleDateString("en-JM", { weekday: "short" });
  }

  function formatRoleName(role: string): string {
    const labels: Record<string, string> = {
      PracticeManager: "Practice Manager",
      Provider: "Provider",
      Staff: "Staff",
    };
    return labels[role] ?? role;
  }

  async function loadProviderSchedule(providerId: string) {
    const weekEnd = getWeekEnd(providerWeekStart);
    providerScheduleLoading = true;
    const res = await commands.getProviderSchedule(providerId, providerWeekStart, weekEnd);
    providerScheduleLoading = false;
    if (res.status === "ok") providerSchedule = res.data;
  }

  async function toggleProvider(providerId: string) {
    if (expandedProviderId === providerId) {
      expandedProviderId = null;
      providerSchedule = [];
      return;
    }
    expandedProviderId = providerId;
    await loadProviderSchedule(providerId);
  }

  async function navigateProviderWeek(delta: number) {
    providerWeekStart = addDays(providerWeekStart, delta);
    if (expandedProviderId) await loadProviderSchedule(expandedProviderId);
  }

  // ── Load ──────────────────────────────────────────────────────────────────

  async function load() {
    error = null;
    const [staffR, statusR, providerR, officeR] = await Promise.all([
      commands.listStaffMembers(),
      commands.getStaffSetupStatus(),
      commands.listProviders(),
      commands.listOffices(),
    ]);
    if (staffR.status === "ok") staff = staffR.data;
    else error = staffR.error;
    if (statusR.status === "ok") setupStatus = statusR.data;
    if (providerR.status === "ok") providers = providerR.data;
    if (officeR.status === "ok") {
      officeMap = Object.fromEntries(officeR.data.map((o) => [o.id, o.name]));
    }
  }

  function activeStaff() { return staff.filter((s) => !s.archived); }
  function archivedStaff() { return staff.filter((s) => s.archived); }
  let hasActivePM = $derived(staff.some((s) => !s.archived && s.roles.includes("PracticeManager")));

  // STAFF-1: filtered + sorted active staff
  let filteredActiveStaff = $derived(
    staff
      .filter((s) => !s.archived)
      .filter((s) => !searchQuery.trim() || s.name.toLowerCase().includes(searchQuery.trim().toLowerCase()))
      .filter((s) => roleFilter === "all" || s.roles.includes(roleFilter))
      .sort((a, b) => { const c = a.name.localeCompare(b.name); return sortAsc ? c : -c; }),
  );

  // STAFF-4: shift loading
  async function loadMemberShifts(id: string) {
    const memberName = staff.find((s) => s.staff_member_id === id)?.name ?? "this staff member";
    memberShiftsLoading = true;
    const res = await commands.getShiftRoster(memberShiftsWeekStart, null);
    memberShiftsLoading = false;
    if (res.status === "ok") {
      memberShifts = res.data.filter((s) => s.staff_member_id === id && !s.cancelled);
    } else {
      toast.error(`Could not load shift schedule for ${memberName}. Check your connection and try again.`);
    }
  }

  async function navigateMemberShiftsWeek(delta: number, id: string) {
    memberShiftsWeekStart = addDays(memberShiftsWeekStart, delta * 7);
    await loadMemberShifts(id);
  }

  async function claim() {
    if (!claimName.trim()) { claimError = "Practice Manager name is required to claim this role."; return; }
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
    if (!regName.trim()) { regError = "Staff member name is required."; return; }
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
      toast.success(`${regName} registered as ${formatRoleName(regRole)}.`);
      regName = ""; regPhone = ""; regEmail = ""; regChannel = ""; regRole = "Staff";
      showRegister = false;
    } else { regError = r.error; }
  }

  async function toggleExpand(id: string) {
    if (expandedId === id) {
      expandedId = null;
      pinSection = null;
      verifyResult = null;
      memberShifts = [];
    } else {
      expandedId = id;
      pinSection = null;
      pinError = null;
      roleError = null;
      verifyResult = null;
      memberShiftsWeekStart = getMondayOfWeek(todayStr());
      await loadMemberShifts(id);
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
    const targetName = staff.find((s) => s.staff_member_id === target_id)?.name ?? "this staff member";
    const ok = await confirm({
      title: "Reset PIN",
      message: `Reset ${targetName}'s PIN? They will need to set a new one before they can sign in.`,
      confirmLabel: "Reset PIN",
      destructive: true,
    });
    if (!ok) return;
    // The PM executing this action — we'd normally have a current user ID.
    // For MVP without full auth, we use the first active PM.
    const pm = staff.find((s) => !s.archived && s.roles.includes("PracticeManager"));
    if (!pm) { toast.error("No active Practice Manager found."); return; }
    pinSaving = true; pinError = null;
    const r = await commands.resetPin(target_id, pm.staff_member_id);
    pinSaving = false;
    if (r.status === "ok") {
      staff = staff.map((s) => s.staff_member_id === target_id ? r.data : s);
      toast.success(`${targetName}'s PIN has been reset.`);
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
    const memberName = staff.find((s) => s.staff_member_id === id)?.name ?? "this staff member";
    const ok = await confirm({
      title: `Archive ${memberName}?`,
      message: `This will deactivate ${memberName}'s account. You can restore it later.`,
      confirmLabel: "Archive",
      destructive: true,
    });
    if (!ok) return;
    const r = await commands.archiveStaffMember(id);
    if (r.status === "ok") {
      staff = staff.map((s) => s.staff_member_id === id ? r.data : s);
      if (expandedId === id) expandedId = null;
      toast.success(`${memberName} archived.`);
    } else { toast.error(r.error); }
  }

  async function doUnarchive(id: string) {
    const memberName = staff.find((s) => s.staff_member_id === id)?.name ?? "Staff member";
    const r = await commands.unarchiveStaffMember(id);
    if (r.status === "ok") {
      staff = staff.map((s) => s.staff_member_id === id ? r.data : s);
      toast.success(`${memberName} restored to active staff.`);
    } else {
      toast.error(r.error);
    }
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
        <label for="claim-name">Name</label>
        <input id="claim-name" bind:value={claimName} placeholder="Dr. Spence" />
      </div>
      <div class="form-actions">
        <button class="btn-primary" onclick={claim} disabled={claiming}>
          {#if claiming}<span class="spinner" aria-hidden="true"></span><span class="sr-only">Claiming</span>{:else}Claim{/if}
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
          <label for="reg-name">Name *</label>
          <input id="reg-name" bind:value={regName} placeholder="Maria Brown" />
        </div>
        <div class="field" style="max-width:140px">
          <label for="reg-role">Initial Role</label>
          <select id="reg-role" bind:value={regRole}>
            {#each ROLES as r}<option value={r}>{formatRoleName(r)}</option>{/each}
          </select>
        </div>
      </div>
      <div class="form-row">
        <div class="field">
          <label for="reg-phone">Phone</label>
          <input id="reg-phone" bind:value={regPhone} placeholder="+1-876-555-0100" />
        </div>
        <div class="field">
          <label for="reg-email">Email</label>
          <input id="reg-email" type="email" bind:value={regEmail} />
        </div>
        <div class="field" style="max-width:140px">
          <label for="reg-channel">Preferred Channel</label>
          <select id="reg-channel" bind:value={regChannel}>
            {#each CHANNELS as c}<option value={c}>{c || "—"}</option>{/each}
          </select>
        </div>
      </div>
      <div class="form-actions">
        <button class="btn-primary" onclick={registerStaff} disabled={registering}>
          {#if registering}<span class="spinner" aria-hidden="true"></span><span class="sr-only">Registering</span>{:else}Register{/if}
        </button>
      </div>
    </div>
  {/if}

  <!-- STAFF-1: Sort / filter bar -->
  {#if activeStaff().length > 0}
    <div class="filter-bar">
      <label for="staff-search" class="sr-only">Search staff by name</label>
      <input
        id="staff-search"
        type="search"
        bind:value={searchQuery}
        placeholder="Search by name…"
        class="search-input"
      />
      <div class="filter-chips" role="group" aria-label="Filter by role">
        {#each [
          { value: "all", label: "All" },
          { value: "PracticeManager", label: "Practice Manager" },
          { value: "Provider", label: "Provider" },
          { value: "Staff", label: "Staff" },
        ] as chip}
          <button
            class="chip-btn"
            class:active={roleFilter === chip.value}
            onclick={() => (roleFilter = chip.value as typeof roleFilter)}
            aria-pressed={roleFilter === chip.value}
          >{chip.label}</button>
        {/each}
      </div>
      <button
        class="sort-btn"
        onclick={() => (sortAsc = !sortAsc)}
        aria-label={sortAsc ? "Currently A to Z — click to sort Z to A" : "Currently Z to A — click to sort A to Z"}
      >{sortAsc ? "A→Z" : "Z→A"}</button>
      <span class="filter-count">{filteredActiveStaff.length} member{filteredActiveStaff.length === 1 ? "" : "s"}</span>
    </div>
  {/if}

  <!-- Active staff list -->
  {#if filteredActiveStaff.length === 0 && !showClaim && !showRegister}
    <p class="empty">
      {#if activeStaff().length === 0}
        {#if !hasActivePM}
          No staff yet. Click <strong>Claim Practice Manager Role</strong> to get started.
        {:else}
          No staff registered yet. Click <strong>+ Register Staff</strong> to add a staff member.
        {/if}
      {:else}
        No staff match your filter.
        <button class="btn-link" onclick={() => { searchQuery = ""; roleFilter = "all"; }}>Clear filter</button>
      {/if}
    </p>
  {/if}

  <div class="staff-list">
    {#each filteredActiveStaff as sm (sm.staff_member_id)}
      <div class="staff-card">
        <div
          class="staff-row"
          role="button"
          tabindex="0"
          aria-expanded={expandedId === sm.staff_member_id}
          aria-label="Expand {sm.name} details"
          onclick={() => toggleExpand(sm.staff_member_id)}
          onkeydown={(e) => (e.key === "Enter" || e.key === " ") && toggleExpand(sm.staff_member_id)}
        >
          <div class="staff-info">
            <span class="staff-name">{sm.name}</span>
            {#each sm.roles as role}
              <span class="role-badge role-{role.toLowerCase()}">{formatRoleName(role)}</span>
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
                    {formatRoleName(role)}
                    <button
                      class="remove-role-btn"
                      onclick={() => doRemoveRole(sm.staff_member_id, role)}
                      title="Remove {formatRoleName(role)} role"
                      aria-label="Remove {formatRoleName(role)} role from {sm.name}"
                    >✕</button>
                  </span>
                {/each}
              </div>
              <div class="add-role-row">
                <label for="add-role-{sm.staff_member_id}" class="sr-only">Select role to add</label>
                <select id="add-role-{sm.staff_member_id}" bind:value={roleToAdd}>
                  {#each ROLES.filter((r) => !sm.roles.includes(r)) as r}<option value={r}>{formatRoleName(r)}</option>{/each}
                </select>
                <button class="btn-sm" onclick={() => doAssignRole(sm.staff_member_id)} disabled={roleAdding}>
                  {#if roleAdding}<span class="spinner" aria-hidden="true"></span><span class="sr-only">Adding role</span>{:else}Add Role{/if}
                </button>
              </div>
              <!-- Provider role guidance (SCH-3) -->
              {#if sm.roles.includes("Provider")}
                <div class="provider-role-hint">
                  <svg class="hint-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" width="16" height="16"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
                  <span>
                    Provider role · To schedule appointments for {sm.name},
                    register them in <a href="/setup">Setup → Providers</a>
                    and assign an office with availability.
                  </span>
                </div>
              {/if}
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
                  <label for="pin-new-{sm.staff_member_id}" class="sr-only">New PIN (4–6 digits)</label>
                  <input id="pin-new-{sm.staff_member_id}" type="password" inputmode="numeric" maxlength="6" placeholder="4–6 digits" bind:value={newPin} class="pin-input" />
                  <button class="btn-sm" onclick={() => doSetPin(sm.staff_member_id)} disabled={pinSaving}>
                    {#if pinSaving}<span class="spinner" aria-hidden="true"></span><span class="sr-only">Saving</span>{:else}Save{/if}
                  </button>
                  <button class="btn-sm btn-ghost" onclick={() => pinSection = null}>Cancel</button>
                </div>
              {:else if pinSection === "change"}
                <div class="pin-form">
                  <label for="pin-current-{sm.staff_member_id}" class="sr-only">Current PIN</label>
                  <input id="pin-current-{sm.staff_member_id}" type="password" inputmode="numeric" maxlength="6" placeholder="Current PIN" bind:value={currentPin} class="pin-input" />
                  <label for="pin-new-change-{sm.staff_member_id}" class="sr-only">New PIN</label>
                  <input id="pin-new-change-{sm.staff_member_id}" type="password" inputmode="numeric" maxlength="6" placeholder="New PIN" bind:value={newPin} class="pin-input" />
                  <button class="btn-sm" onclick={() => doChangePin(sm.staff_member_id)} disabled={pinSaving}>
                    {#if pinSaving}<span class="spinner" aria-hidden="true"></span><span class="sr-only">Saving</span>{:else}Save{/if}
                  </button>
                  <button class="btn-sm btn-ghost" onclick={() => pinSection = null}>Cancel</button>
                </div>
              {/if}

              <!-- PIN verify (identity switch test) -->
              {#if sm.has_pin}
                <div class="verify-row">
                  <label for="pin-verify-{sm.staff_member_id}" class="sr-only">Verify PIN</label>
                  <input id="pin-verify-{sm.staff_member_id}" type="password" inputmode="numeric" maxlength="6" placeholder="Verify PIN" bind:value={verifyPin} class="pin-input" />
                  <button class="btn-sm btn-ghost" onclick={() => doVerifyPin(sm.staff_member_id)} disabled={verifying}>
                    {#if verifying}<span class="spinner" aria-hidden="true"></span><span class="sr-only">Verifying</span>{:else}Verify{/if}
                  </button>
                  {#if verifyResult === true}<span class="verify-ok">✓ Correct</span>{/if}
                  {#if verifyResult === false}<span class="verify-fail">✗ Incorrect</span>{/if}
                </div>
              {/if}
            </section>

            <!-- STAFF-4: Shifts this week -->
            <section class="detail-section">
              <div class="section-row">
                <h4>Shifts this week</h4>
                <div class="week-nav-compact">
                  <button class="nav-btn-sm" onclick={() => navigateMemberShiftsWeek(-1, sm.staff_member_id)} title="Previous week" aria-label="Previous week">‹</button>
                  <span class="week-label-sm">{formatWeekRange(memberShiftsWeekStart)}</span>
                  <button class="nav-btn-sm" onclick={() => navigateMemberShiftsWeek(1, sm.staff_member_id)} title="Next week" aria-label="Next week">›</button>
                </div>
              </div>
              {#if memberShiftsLoading}
                <div class="load-row">
                  <span class="spinner spinner-sm" aria-hidden="true"></span>
                  <span>Loading shifts…</span>
                </div>
              {:else}
                <div class="shift-week-grid" role="table" aria-label="Shift schedule for {sm.name}">
                  {#each getWeekDays(memberShiftsWeekStart) as day}
                    {@const shift = memberShifts.find((s) => s.date === day)}
                    <div class="shift-day-cell" class:shift-active={!!shift} role="cell">
                      <div class="shift-day-label">{dayAbbr(day)}</div>
                      <div class="shift-day-num">{new Date(day + "T12:00:00").getDate()}</div>
                      {#if shift}
                        <div class="shift-time">{minsTo12h(parseHHMM(shift.start_time))}–{minsTo12h(parseHHMM(shift.end_time))}</div>
                        <div class="shift-office-abbr" title={shift.office_name}>{shift.office_name.split(" ")[0]}</div>
                      {:else}
                        <div class="shift-none" aria-label="No shift">—</div>
                      {/if}
                    </div>
                  {/each}
                </div>
              {/if}
            </section>

            <!-- STAFF-3: Plan shift link -->
            <section class="detail-section plan-shift-section">
              <a href="/schedule?view=roster&planFor={sm.staff_member_id}" class="btn-sm btn-ghost plan-shift-link">
                <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><rect x="3" y="4" width="18" height="18" rx="2" ry="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/></svg>
                Plan shift in Roster
              </a>
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

  <!-- Clinical Staff (Providers) -->
  {#if providers.filter((p) => !p.archived).length > 0 || providers.length > 0}
    <div class="providers-section">
      <h3 class="section-heading">Clinical Staff (Providers)</h3>

      <div class="providers-list">
        {#each providers.filter((p) => !p.archived) as prov (prov.staff_member_id)}
          <div class="provider-card">
            <div
              class="provider-row"
              role="button"
              tabindex="0"
              aria-expanded={expandedProviderId === prov.staff_member_id}
              aria-label="Expand {prov.name} schedule"
              onclick={() => toggleProvider(prov.staff_member_id)}
              onkeydown={(e) => (e.key === "Enter" || e.key === " ") && toggleProvider(prov.staff_member_id)}
            >
              <div class="provider-info">
                <span class="provider-name">{prov.name}</span>
                <span class="provider-type-badge">{prov.clinical_specialization ?? "—"}</span>
              </div>
              <span class="chevron">{expandedProviderId === prov.staff_member_id ? "▲" : "▼"}</span>
            </div>

            {#if expandedProviderId === prov.staff_member_id}
              <div class="provider-schedule-panel">
                <!-- Week navigation -->
                <div class="week-nav">
                  <button class="nav-btn-sm" onclick={() => navigateProviderWeek(-28)} title="Previous month" aria-label="Previous month">«</button>
                  <button class="nav-btn-sm" onclick={() => navigateProviderWeek(-7)} title="Previous week" aria-label="Previous week">‹</button>
                  <span class="week-label">Week of {formatWeekRange(providerWeekStart)}</span>
                  <button class="nav-btn-sm" onclick={() => navigateProviderWeek(7)} title="Next week" aria-label="Next week">›</button>
                  <button class="nav-btn-sm" onclick={() => navigateProviderWeek(28)} title="Next month" aria-label="Next month">»</button>
                </div>

                {#if providerScheduleLoading}
                  <p class="prov-muted">Loading schedule…</p>
                {:else}
                  {#each getWeekDays(providerWeekStart) as day}
                    {@const dayAppts = providerSchedule.filter((a) => a.start_time.slice(0, 10) === day).sort((a, b) => a.start_time.localeCompare(b.start_time))}
                    <div class="week-day">
                      <div class="week-day-header">{formatDayHeader(day)}</div>
                      {#if dayAppts.length === 0}
                        <p class="prov-muted">(no appointments)</p>
                      {:else}
                        <ul class="prov-appt-list">
                          {#each dayAppts as appt}
                            <li class="prov-appt-item">
                              <span class="prov-appt-time">{formatApptTime(appt.start_time)}</span>
                              <span class="prov-appt-patient">{appt.patient_name}</span>
                              <span class="prov-appt-sep">—</span>
                              <span class="prov-appt-proc">{appt.procedure_name}</span>
                              {#if appt.duration_minutes}
                                <span class="prov-appt-dur">({appt.duration_minutes} min)</span>
                              {/if}
                              <span class="prov-appt-office">@ {officeMap[appt.office_id] ?? appt.office_id}</span>
                              <span class="prov-appt-status prov-status-{appt.status.toLowerCase()}">{appt.status}</span>
                            </li>
                          {/each}
                        </ul>
                      {/if}
                    </div>
                  {/each}
                {/if}
              </div>
            {/if}
          </div>
        {/each}
      </div>

      <!-- Archived providers (collapsed) -->
      {#if providers.filter((p) => p.archived).length > 0}
        <div class="archived-providers">
          <h4 class="archived-heading">Inactive Providers</h4>
          <div class="providers-list">
            {#each providers.filter((p) => p.archived) as prov (prov.staff_member_id)}
              <div class="provider-card archived">
                <div class="provider-row">
                  <div class="provider-info">
                    <span class="provider-name">{prov.name}</span>
                    <span class="provider-type-badge">{prov.clinical_specialization ?? "—"}</span>
                    <span class="badge archived-badge">Archived</span>
                  </div>
                </div>
              </div>
            {/each}
          </div>
        </div>
      {/if}
    </div>
  {/if}

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
              <button class="btn-sm btn-ghost" onclick={() => doUnarchive(sm.staff_member_id)}>Restore</button>
            </div>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  /* ── Layout ──────────────────────────────────────────── */
  .page { padding: var(--space-6); max-width: 840px; }
  .page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--space-5); }
  h1 { margin: 0; font-size: var(--text-2xl); font-family: var(--font-heading); font-weight: 700; color: var(--abyss-navy); }
  h3 { margin: 0 0 var(--space-4); font-size: var(--text-base); font-family: var(--font-heading); font-weight: 600; color: var(--abyss-navy); }
  h4 { margin: 0; font-size: var(--text-xs); font-weight: 700; text-transform: uppercase;
       letter-spacing: 0.06em; color: var(--slate-fog); font-family: var(--font-heading); }
  .header-actions { display: flex; gap: var(--space-3); }
  .error { color: var(--healthy-coral-dk); font-size: var(--text-sm); margin-bottom: var(--space-3); font-family: var(--font-body); }
  .empty { color: var(--slate-fog); font-style: italic; font-family: var(--font-body); font-size: var(--text-sm); }
  .hint { font-size: var(--text-sm); color: var(--slate-fog); margin-bottom: var(--space-4); }
  .meta { font-size: var(--text-xs); color: var(--slate-fog); }

  /* ── Setup status banner ─────────────────────────────── */
  .setup-status {
    padding: var(--space-3) var(--space-4); border-radius: var(--radius-md); font-size: var(--text-sm);
    margin-bottom: var(--space-5); font-family: var(--font-body);
    background: #FFF8E7; border: 1px solid #F0C040; color: #7A5A00;
  }
  .setup-status.complete { background: var(--island-palm-lt); border-color: #A9DFBF; color: var(--island-palm); }

  /* ── Cards / forms ───────────────────────────────────── */
  .card { background: #fff; border: 1px solid var(--pearl-mist-dk); border-radius: var(--radius-lg); padding: var(--space-6); }
  .form-card { margin-bottom: var(--space-5); box-shadow: var(--shadow-sm); }
  .form-row { display: flex; gap: var(--space-4); flex-wrap: wrap; margin-bottom: var(--space-4); }
  .form-actions { display: flex; gap: var(--space-3); margin-top: var(--space-4); }

  .field { display: flex; flex-direction: column; gap: var(--space-1); flex: 1; min-width: 140px; }
  .field label { font-size: var(--text-xs); font-weight: 600; color: var(--abyss-navy); font-family: var(--font-body); }

  /* ── Staff list ──────────────────────────────────────── */
  .staff-list { display: flex; flex-direction: column; gap: var(--space-2); }
  .staff-card { border: 1px solid var(--pearl-mist-dk); border-radius: var(--radius-lg); overflow: hidden; background: #fff; box-shadow: var(--shadow-sm); }
  .staff-card.archived { opacity: 0.65; }
  .staff-row {
    display: flex; justify-content: space-between; align-items: center;
    padding: var(--space-4) var(--space-5); cursor: pointer; user-select: none;
    transition: background var(--transition-fast);
  }
  .staff-card.archived .staff-row { cursor: default; }
  .staff-row:hover { background: var(--pearl-mist); }
  .staff-info { display: flex; align-items: center; gap: var(--space-2); flex-wrap: wrap; }
  .staff-name { font-weight: 600; font-size: var(--text-sm); font-family: var(--font-heading); color: var(--abyss-navy); }
  .chevron { color: var(--slate-fog); font-size: var(--text-xs); }

  /* Badges (local overrides to match role naming) */
  .role-badge { font-size: var(--text-xs); padding: 2px var(--space-2); border-radius: var(--radius-pill); font-weight: 600; font-family: var(--font-heading); }
  .role-practicemanager { background: var(--color-role-pm-lt); color: var(--color-role-pm); }
  .role-provider { background: var(--color-role-provider-lt); color: var(--color-role-provider); }
  .role-staff { background: var(--color-role-staff-lt); color: var(--color-role-staff); }
  .no-pin-badge { font-size: var(--text-xs); padding: 2px var(--space-2); border-radius: var(--radius-pill); font-weight: 600; background: #FFF8E7; color: #7A5A00; }
  .archived-badge { font-size: var(--text-xs); padding: 2px var(--space-2); border-radius: var(--radius-pill); font-weight: 600; background: var(--pearl-mist-dk); color: var(--slate-fog); }

  /* ── Detail panel (expanded inside card) ─────────────── */
  .detail-panel { border-top: 1px solid var(--pearl-mist-dk); padding: var(--space-4) var(--space-5); }
  .detail-section { margin-bottom: var(--space-5); }
  .detail-section:last-child { margin-bottom: 0; }
  .info-list { display: grid; grid-template-columns: 80px 1fr; gap: var(--space-1) var(--space-4);
               margin: var(--space-2) 0 0; font-size: var(--text-sm); font-family: var(--font-body); }
  dt { color: var(--slate-fog); font-weight: 500; }
  dd { margin: 0; color: var(--abyss-navy); }

  /* ── Roles ───────────────────────────────────────────── */
  .roles-list { display: flex; gap: var(--space-2); flex-wrap: wrap; margin: var(--space-2) 0; }
  .role-chip {
    display: flex; align-items: center; gap: var(--space-1); background: var(--pearl-mist);
    border: 1px solid var(--pearl-mist-dk); border-radius: var(--radius-pill);
    padding: 2px var(--space-3); font-size: var(--text-xs); font-family: var(--font-body); color: var(--abyss-navy);
  }
  .remove-role-btn { background: none; border: none; cursor: pointer; color: var(--slate-fog); font-size: 0.7rem; padding: 0; line-height: 1; }
  .remove-role-btn:hover { color: var(--healthy-coral-dk); }
  .add-role-row { display: flex; gap: var(--space-2); align-items: center; margin-top: var(--space-2); }

  /* ── Provider role hint (SCH-3) ─────────────────────── */
  .provider-role-hint {
    display: flex;
    align-items: flex-start;
    gap: var(--space-2);
    margin-top: var(--space-3);
    padding: var(--space-3);
    background: var(--caribbean-teal-lt, #e6f7f8);
    border: 1px solid var(--caribbean-teal);
    border-radius: var(--radius-md);
    font-size: var(--text-xs);
    color: var(--abyss-navy);
    font-family: var(--font-body);
    line-height: 1.5;
  }
  .hint-icon {
    flex-shrink: 0;
    color: var(--caribbean-teal);
    margin-top: 1px;
  }
  .provider-role-hint a {
    color: var(--caribbean-teal);
    font-weight: 600;
    text-decoration: underline;
  }
  .provider-role-hint a:hover { color: var(--caribbean-teal-dk); }

  /* ── PIN ─────────────────────────────────────────────── */
  .pin-input {
    width: 120px; min-height: 40px; padding: var(--space-2) var(--space-3);
    border: 1.5px solid var(--pearl-mist-dk); border-radius: var(--radius-md);
    font-size: var(--text-sm); font-family: var(--font-mono);
  }
  .pin-input:focus { outline: none; border-color: var(--caribbean-teal); box-shadow: 0 0 0 3px rgba(0,139,153,0.15); }
  .pin-actions { display: flex; gap: var(--space-2); margin-top: var(--space-2); }
  .pin-form { display: flex; gap: var(--space-2); align-items: center; flex-wrap: wrap; margin-top: var(--space-2); }
  .verify-row { display: flex; gap: var(--space-2); align-items: center; margin-top: var(--space-4); padding-top: var(--space-3); border-top: 1px dashed var(--pearl-mist-dk); }
  .verify-ok { font-size: var(--text-sm); color: var(--island-palm); font-weight: 600; }
  .verify-fail { font-size: var(--text-sm); color: var(--healthy-coral-dk); font-weight: 600; }

  /* ── Archive ─────────────────────────────────────────── */
  .archive-section { border-top: 1px solid var(--pearl-mist-dk); padding-top: var(--space-4); }
  .archived-section { margin-top: var(--space-8); }
  .archived-heading { font-size: var(--text-xs); color: var(--slate-fog); text-transform: uppercase;
                      letter-spacing: 0.06em; margin-bottom: var(--space-3); font-family: var(--font-heading); }

  /* ── Providers section ───────────────────────────────── */
  .providers-section { margin-top: var(--space-8); }
  .section-heading {
    font-size: var(--text-xs); color: var(--slate-fog); text-transform: uppercase;
    letter-spacing: 0.08em; margin-bottom: var(--space-4); font-family: var(--font-heading); font-weight: 700;
    border-bottom: 1px solid var(--pearl-mist-dk); padding-bottom: var(--space-2);
  }
  .providers-list { display: flex; flex-direction: column; gap: var(--space-2); }
  .provider-card { border: 1px solid var(--pearl-mist-dk); border-radius: var(--radius-lg); overflow: hidden; background: #fff; box-shadow: var(--shadow-sm); }
  .provider-card.archived { opacity: 0.6; }
  .provider-row { display: flex; justify-content: space-between; align-items: center; padding: var(--space-3) var(--space-5); cursor: pointer; user-select: none; transition: background var(--transition-fast); }
  .provider-card.archived .provider-row { cursor: default; }
  .provider-row:hover { background: var(--pearl-mist); }
  .provider-info { display: flex; align-items: center; gap: var(--space-2); flex-wrap: wrap; }
  .provider-name { font-weight: 600; font-size: var(--text-sm); font-family: var(--font-heading); color: var(--abyss-navy); }
  .provider-type-badge {
    font-size: var(--text-xs); padding: 2px var(--space-2); border-radius: var(--radius-pill); font-weight: 600;
    background: var(--color-role-provider-lt); color: var(--color-role-provider); font-family: var(--font-heading);
  }

  /* ── Provider schedule panel ─────────────────────────── */
  .provider-schedule-panel { border-top: 1px solid var(--pearl-mist-dk); padding: var(--space-4) var(--space-5) var(--space-5); }
  .week-nav { display: flex; align-items: center; gap: var(--space-2); margin-bottom: var(--space-4); flex-wrap: wrap; }
  .nav-btn-sm {
    background: var(--pearl-mist); border: 1.5px solid var(--pearl-mist-dk); border-radius: var(--radius-sm);
    color: var(--slate-fog); font-size: var(--text-sm); min-width: 40px; min-height: 40px;
    cursor: pointer; display: flex; align-items: center; justify-content: center; padding: 0;
    transition: all var(--transition-fast);
  }
  .nav-btn-sm:hover { border-color: var(--caribbean-teal); color: var(--caribbean-teal); }
  .week-label { font-size: var(--text-sm); font-weight: 600; color: var(--abyss-navy); font-family: var(--font-heading); flex: 1; text-align: center; }
  .week-day { margin-bottom: var(--space-4); }
  .week-day:last-child { margin-bottom: 0; }
  .week-day-header { font-size: var(--text-xs); font-weight: 700; color: var(--slate-fog); text-transform: uppercase; letter-spacing: 0.06em; margin-bottom: var(--space-2); font-family: var(--font-heading); }
  .prov-muted { font-size: var(--text-xs); color: var(--slate-fog); margin: var(--space-1) 0; font-style: italic; }
  .prov-appt-list { list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: var(--space-1); }
  .prov-appt-item {
    display: flex; align-items: center; gap: var(--space-2); flex-wrap: wrap;
    font-size: var(--text-sm); font-family: var(--font-body);
    background: var(--pearl-mist); border-radius: var(--radius-sm); padding: var(--space-2) var(--space-3);
  }
  .prov-appt-time { font-family: var(--font-mono); font-weight: 600; color: var(--caribbean-teal); white-space: nowrap; font-size: var(--text-xs); }
  .prov-appt-patient { font-weight: 600; color: var(--abyss-navy); }
  .prov-appt-sep { color: var(--pearl-mist-dk); }
  .prov-appt-proc { color: var(--slate-fog); }
  .prov-appt-dur { color: var(--slate-fog); font-size: var(--text-xs); }
  .prov-appt-office { color: var(--slate-fog); font-size: var(--text-xs); margin-left: auto; }
  .prov-appt-status { font-size: 0.68rem; font-weight: 700; padding: 2px 6px; border-radius: var(--radius-pill); text-transform: uppercase; letter-spacing: 0.03em; white-space: nowrap; }
  .prov-status-booked      { background: var(--color-booked-lt);      color: var(--color-booked); }
  .prov-status-completed   { background: var(--color-completed-lt);   color: var(--color-completed); }
  .prov-status-cancelled   { background: var(--color-cancelled-lt);   color: var(--color-cancelled); }
  .prov-status-noshow      { background: var(--color-noshow-lt);      color: var(--color-noshow); }
  .prov-status-rescheduled { background: var(--color-rescheduled-lt); color: var(--color-rescheduled); }
  .archived-providers { margin-top: var(--space-4); }

  /* ── Buttons (page-local, designed to match global .btn style) ── */
  .btn-primary {
    display: inline-flex; align-items: center; gap: var(--space-2);
    min-height: 44px; padding: 0 var(--space-5);
    background: var(--caribbean-teal); color: #fff; border: none;
    border-radius: var(--radius-md); font-family: var(--font-heading); font-size: var(--text-sm);
    font-weight: 600; cursor: pointer; white-space: nowrap;
    transition: background var(--transition-fast);
  }
  .btn-primary:hover:not(:disabled) { background: var(--caribbean-teal-dk); }
  .btn-primary:disabled { opacity: 0.45; cursor: not-allowed; }

  .btn-sm {
    display: inline-flex; align-items: center; gap: var(--space-1);
    min-height: 36px; padding: 0 var(--space-4);
    background: var(--caribbean-teal); color: #fff; border: none;
    border-radius: var(--radius-md); font-family: var(--font-heading); font-size: var(--text-xs);
    font-weight: 600; cursor: pointer; white-space: nowrap;
    transition: background var(--transition-fast);
  }
  .btn-sm:disabled { opacity: 0.45; cursor: not-allowed; }
  .btn-sm.btn-ghost {
    background: transparent; color: var(--slate-fog);
    border: 1.5px solid var(--pearl-mist-dk);
  }
  .btn-sm.btn-ghost:hover { background: var(--pearl-mist); color: var(--abyss-navy); border-color: var(--abyss-navy); }

  .btn-danger-sm {
    display: inline-flex; align-items: center; min-height: 36px; padding: 0 var(--space-4);
    background: transparent; color: var(--healthy-coral-dk);
    border: 1.5px solid var(--healthy-coral); border-radius: var(--radius-md);
    font-family: var(--font-heading); font-size: var(--text-xs); font-weight: 600;
    cursor: pointer;
    transition: background var(--transition-fast);
  }
  .btn-danger-sm:hover { background: var(--healthy-coral-lt); }

  /* ── STAFF-1: Filter bar ─────────────────────────────── */
  .filter-bar {
    display: flex; align-items: center; gap: var(--space-3); flex-wrap: wrap;
    margin-bottom: var(--space-4);
    padding: var(--space-3) var(--space-4);
    background: var(--pearl-mist); border: 1px solid var(--pearl-mist-dk); border-radius: var(--radius-md);
  }
  .search-input {
    min-width: 180px; height: 36px; padding: 0 var(--space-3);
    border: 1.5px solid var(--pearl-mist-dk); border-radius: var(--radius-md);
    font-size: var(--text-sm); font-family: var(--font-body); background: #fff;
  }
  .search-input:focus { outline: none; border-color: var(--caribbean-teal); box-shadow: 0 0 0 3px rgba(0,139,153,0.15); }
  .filter-chips { display: flex; gap: var(--space-1); }
  .chip-btn {
    min-height: 40px; padding: 0 var(--space-3);
    border: 1.5px solid var(--pearl-mist-dk); border-radius: var(--radius-pill);
    background: #fff; font-size: var(--text-xs); font-family: var(--font-heading);
    font-weight: 600; color: var(--slate-fog); cursor: pointer;
    transition: all var(--transition-fast);
  }
  .chip-btn:hover { border-color: var(--caribbean-teal); color: var(--caribbean-teal); }
  .chip-btn.active { background: var(--caribbean-teal); border-color: var(--caribbean-teal); color: #fff; }
  .sort-btn {
    min-height: 40px; padding: 0 var(--space-3);
    border: 1.5px solid var(--pearl-mist-dk); border-radius: var(--radius-md);
    background: #fff; font-size: var(--text-xs); font-family: var(--font-heading);
    font-weight: 600; color: var(--slate-fog); cursor: pointer;
    transition: all var(--transition-fast);
  }
  .sort-btn:hover { border-color: var(--caribbean-teal); color: var(--caribbean-teal); }
  .filter-count { margin-left: auto; font-size: var(--text-xs); color: var(--slate-fog); font-family: var(--font-body); white-space: nowrap; }
  .btn-link {
    background: none; border: none; color: var(--caribbean-teal);
    font-size: inherit; font-family: var(--font-body); cursor: pointer; text-decoration: underline; padding: 0;
  }
  .btn-link:hover { color: var(--caribbean-teal-dk); }

  /* ── STAFF-4: Shift week grid ────────────────────────── */
  .section-row { display: flex; align-items: center; justify-content: space-between; margin-bottom: var(--space-3); }
  .week-nav-compact { display: flex; align-items: center; gap: var(--space-1); }
  .week-label-sm { font-size: var(--text-xs); color: var(--slate-fog); font-family: var(--font-body); white-space: nowrap; }
  .shift-week-grid {
    display: grid; grid-template-columns: repeat(7, 1fr); gap: var(--space-1); margin-bottom: var(--space-2);
  }
  .shift-day-cell {
    display: flex; flex-direction: column; align-items: center; gap: 2px;
    padding: var(--space-2) var(--space-1);
    border-radius: var(--radius-sm); background: var(--pearl-mist); border: 1px solid var(--pearl-mist-dk);
    text-align: center;
  }
  .shift-day-cell.shift-active { background: var(--caribbean-teal-lt, #e6f7f8); border-color: var(--caribbean-teal); }
  .shift-day-label { font-size: 0.65rem; font-weight: 700; color: var(--slate-fog); font-family: var(--font-heading); text-transform: uppercase; letter-spacing: 0.04em; }
  .shift-day-num { font-size: var(--text-xs); font-weight: 600; color: var(--abyss-navy); font-family: var(--font-heading); }
  .shift-time { font-size: var(--text-xs); font-family: var(--font-mono); color: var(--caribbean-teal); font-weight: 600; line-height: 1.2; }
  .shift-office-abbr { font-size: 0.6rem; color: var(--slate-fog); font-family: var(--font-body); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 100%; }
  .shift-none { font-size: var(--text-xs); color: var(--pearl-mist-dk); font-family: var(--font-body); }
  .load-row { display: flex; align-items: center; gap: var(--space-2); font-size: var(--text-xs); color: var(--slate-fog); font-family: var(--font-body); }

  /* ── STAFF-3: Plan shift link ────────────────────────── */
  .plan-shift-section { border-top: 1px solid var(--pearl-mist-dk); padding-top: var(--space-3); }
  .plan-shift-link { text-decoration: none; display: inline-flex; align-items: center; gap: var(--space-2); }
  .plan-shift-link:hover { background: var(--pearl-mist); border-color: var(--caribbean-teal); color: var(--caribbean-teal); }
</style>
