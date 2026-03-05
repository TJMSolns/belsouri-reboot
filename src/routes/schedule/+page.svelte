<script lang="ts">
  import { commands } from "$lib/bindings";
  import type {
    AppointmentDto, AppointmentWithNotesDto, CallListEntryDto,
    ProviderScheduleEntry, OfficeDto, ProviderDto, StaffShiftDto, StaffMemberDto,
  } from "$lib/bindings";
  import { getErrorMessage } from "$lib/utils/api";
  import { toast } from "$lib/stores/toast";
  import { confirm } from "$lib/stores/confirm";
  import { onMount } from "svelte";

  const STAFF_ID = "staff-system";
  const SLOT_HEIGHT = 30; // px per 15-min slot

  // ── Data ──────────────────────────────────────────────────────────────────

  let offices = $state<OfficeDto[]>([]);
  let allProviders = $state<ProviderDto[]>([]);
  let procedures = $state<{ id: string; name: string; default_duration_minutes: number; is_active: boolean; required_provider_type: string | null }[]>([]);

  // ── Grid view ─────────────────────────────────────────────────────────────

  let selectedOfficeId = $state("");
  let selectedDate = $state(todayLocal());
  let schedule = $state<AppointmentDto[]>([]);
  let providerRoster = $state<ProviderScheduleEntry[]>([]);
  let scheduleLoading = $state(false);

  // ── Detail drawer ─────────────────────────────────────────────────────────

  let detailApptId = $state<string | null>(null);
  let detailData = $state<AppointmentWithNotesDto | null>(null);
  let detailLoading = $state(false);
  let showCancelConfirm = $state(false);
  let cancelReason = $state("");
  let completingAppt = $state(false);
  let noShowingAppt = $state(false);
  let cancellingAppt = $state(false);

  // ── Reschedule form (in detail drawer) ────────────────────────────────────

  let showReschedule = $state(false);
  let reschedOfficeId = $state("");
  let reschedProviderId = $state("");
  let reschedDate = $state("");
  let reschedTime = $state("");
  let reschedLoading = $state(false);
  let reschedError = $state("");
  let reschedRoster = $state<ProviderScheduleEntry[]>([]);
  let reschedRosterLoading = $state(false);

  // ── Provider grid visibility (SCH-2) ──────────────────────────────────────

  let showAllProviders = $state(false);

  // ── Roster tab (SCH-5) ────────────────────────────────────────────────────

  let scheduleView = $state<"grid" | "roster">("grid");
  let allStaff = $state<StaffMemberDto[]>([]);
  let shiftRoster = $state<StaffShiftDto[]>([]);
  let rosterLoading = $state(false);
  let showPlanShift = $state(false);
  let planShiftStaffId = $state("");
  let planShiftOfficeId = $state("");
  let planShiftDate = $state(todayLocal());
  let planShiftStart = $state("09:00");
  let planShiftEnd = $state("17:00");
  let planShiftRole = $state("");
  let planShiftLoading = $state(false);
  let planShiftError = $state("");
  let cancellingShiftId = $state<string | null>(null);

  // ── Book drawer ───────────────────────────────────────────────────────────

  let showBookForm = $state(false);
  let bookOfficeId = $state("");
  let bookPatientSearch = $state("");
  let bookPatientId = $state("");
  let bookPatientName = $state("");
  let bookProviderId = $state("");
  let bookProcedureId = $state("");
  let bookStartDate = $state(todayLocal());
  let bookStartTime = $state("");
  let bookError = $state("");
  let bookLoading = $state(false);
  let patientSearchDebounce = $state<ReturnType<typeof setTimeout> | null>(null);
  let patientSearchResults = $state<{ patient_id: string; patient_name: string; first_name: string; last_name: string; phone: string | null }[]>([]);
  let bookRoster = $state<ProviderScheduleEntry[]>([]);
  let bookRosterLoading = $state(false);

  // ── Notes (in detail drawer) ──────────────────────────────────────────────

  let noteText = $state("");
  let noteError = $state("");
  let noteLoading = $state(false);

  // ── Call list ─────────────────────────────────────────────────────────────

  let showCallList = $state(false);
  let callList = $state<CallListEntryDto[]>([]);
  let callListDate = $state(tomorrowLocal());

  // ── Derived grid values ───────────────────────────────────────────────────

  let currentOffice = $derived(offices.find((o) => o.id === selectedOfficeId) ?? null);
  let dayName = $derived(getDayName(selectedDate));
  let officeHoursEntry = $derived(
    currentOffice?.hours.find((h) => h.day_of_week === dayName) ?? null,
  );
  let openMins = $derived(officeHoursEntry ? parseHHMM(officeHoursEntry.open_time) : 480);
  let closeMins = $derived(officeHoursEntry ? parseHHMM(officeHoursEntry.close_time) : 1020);
  let gridHeight = $derived(Math.max(0, ((closeMins - openMins) / 15) * SLOT_HEIGHT));

  let officeProviders = $derived(
    allProviders
      .filter((p) => !p.archived && p.office_ids.includes(selectedOfficeId))
      .sort((a, b) => a.name.localeCompare(b.name)),
  );

  let timeTicks = $derived(
    (() => {
      if (!officeHoursEntry) return [];
      const ticks: { label: string; top: number }[] = [];
      for (let m = openMins; m <= closeMins; m += 60) {
        ticks.push({ label: minsTo12h(m), top: ((m - openMins) / 15) * SLOT_HEIGHT });
      }
      return ticks;
    })(),
  );

  let isToday = $derived(selectedDate === todayLocal());

  // Capability levels matching the Rust implementation
  const CAPABILITY_LEVELS: Record<string, number> = { Specialist: 3, Dentist: 2, Hygienist: 1 };

  let eligibleBookRoster = $derived((() => {
    const proc = procedures.find((p) => p.id === bookProcedureId);
    if (!proc?.required_provider_type) return bookRoster;
    const req = CAPABILITY_LEVELS[proc.required_provider_type] ?? 0;
    return bookRoster.filter((entry) => {
      const prov = allProviders.find((p) => p.id === entry.provider_id);
      return (CAPABILITY_LEVELS[prov?.provider_type ?? ""] ?? 0) >= req;
    });
  })());

  let availableSlots = $derived(
    (() => {
      const entry = eligibleBookRoster.find((e) => e.provider_id === bookProviderId);
      if (!entry) return [];
      return generateTimeSlots(entry.start_time, entry.end_time);
    })(),
  );

  let reschedAvailableSlots = $derived(
    (() => {
      const entry = reschedRoster.find((e) => e.provider_id === reschedProviderId);
      if (!entry) return [];
      return generateTimeSlots(entry.start_time, entry.end_time);
    })(),
  );

  let visibleProviders = $derived(
    showAllProviders
      ? officeProviders
      : officeProviders.filter((p) => providerRoster.some((r) => r.provider_id === p.id)),
  );

  // ── Helpers ───────────────────────────────────────────────────────────────

  function todayLocal(): string {
    return new Date().toISOString().slice(0, 10);
  }

  function tomorrowLocal(): string {
    const d = new Date();
    d.setDate(d.getDate() + 1);
    return d.toISOString().slice(0, 10);
  }

  function addDays(date: string, n: number): string {
    const d = new Date(date + "T12:00:00");
    d.setDate(d.getDate() + n);
    return d.toISOString().slice(0, 10);
  }

  function getDayName(date: string): string {
    return new Date(date + "T12:00:00").toLocaleDateString("en-US", { weekday: "long" });
  }

  function formatDisplayDate(date: string): string {
    return new Date(date + "T12:00:00").toLocaleDateString("en-JM", {
      weekday: "long",
      day: "numeric",
      month: "short",
      year: "numeric",
    });
  }

  function parseHHMM(t: string): number {
    const [h, m] = t.split(":").map(Number);
    return h * 60 + m;
  }

  function minsToHHMM(m: number): string {
    return `${Math.floor(m / 60).toString().padStart(2, "0")}:${(m % 60).toString().padStart(2, "0")}`;
  }

  function minsTo12h(m: number): string {
    const h = Math.floor(m / 60);
    const min = m % 60;
    const period = h >= 12 ? "PM" : "AM";
    const h12 = h % 12 || 12;
    return `${h12}:${min.toString().padStart(2, "0")} ${period}`;
  }

  function formatTime(isoLocal: string): string {
    const [h, m] = isoLocal.slice(11, 16).split(":").map(Number);
    const period = h >= 12 ? "PM" : "AM";
    const h12 = h % 12 || 12;
    return `${h12}:${m.toString().padStart(2, "0")} ${period}`;
  }

  function formatDate(isoLocal: string): string {
    return new Date(isoLocal.slice(0, 10) + "T12:00:00").toLocaleDateString("en-JM", {
      day: "numeric", month: "short", year: "numeric",
    });
  }

  function buildStartTime(date: string, time: string): string {
    return `${date}T${time}:00`;
  }

  function generateTimeSlots(start: string, end: string): string[] {
    const slots: string[] = [];
    let [h, m] = start.split(":").map(Number);
    const [eh, em] = end.split(":").map(Number);
    while (h * 60 + m < eh * 60 + em) {
      slots.push(`${h.toString().padStart(2, "0")}:${m.toString().padStart(2, "0")}`);
      m += 15;
      if (m >= 60) { h++; m -= 60; }
    }
    return slots;
  }

  function statusBadgeClass(status: string): string {
    const map: Record<string, string> = {
      Booked: "badge-booked", Completed: "badge-completed",
      Cancelled: "badge-cancelled", NoShow: "badge-noshow", Rescheduled: "badge-rescheduled",
    };
    return map[status] ?? "badge-booked";
  }

  function statusBlockClass(status: string): string {
    const map: Record<string, string> = {
      Booked: "appt-booked", Completed: "appt-completed",
      Cancelled: "appt-cancelled", NoShow: "appt-noshow", Rescheduled: "appt-rescheduled",
    };
    return map[status] ?? "appt-booked";
  }

  function getWeekStart(date: string): string {
    const d = new Date(date + "T12:00:00");
    const day = d.getDay(); // 0=Sun
    const diff = day === 0 ? -6 : 1 - day;
    d.setDate(d.getDate() + diff);
    return d.toISOString().slice(0, 10);
  }

  function getWeekDays(weekStart: string): string[] {
    const days: string[] = [];
    for (let i = 0; i < 7; i++) {
      days.push(addDays(weekStart, i));
    }
    return days;
  }

  function formatWeekRange(weekStart: string): string {
    const weekEnd = addDays(weekStart, 6);
    const startD = new Date(weekStart + "T12:00:00");
    const endD = new Date(weekEnd + "T12:00:00");
    return `${startD.toLocaleDateString("en-JM", { month: "short", day: "numeric" })} – ${endD.toLocaleDateString("en-JM", { month: "short", day: "numeric", year: "numeric" })}`;
  }

  function dayAbbr(date: string): string {
    return new Date(date + "T12:00:00").toLocaleDateString("en-US", { weekday: "short" });
  }

  function dayNum(date: string): string {
    return new Date(date + "T12:00:00").getDate().toString();
  }

  // ── Data loading ──────────────────────────────────────────────────────────

  async function loadSetupData() {
    const [officeRes, procRes, provRes, staffRes] = await Promise.all([
      commands.listOffices(),
      commands.listProcedureTypes(),
      commands.listProviders(),
      commands.listStaffMembers(),
    ]);
    if (officeRes.status === "ok") {
      offices = officeRes.data.filter((o) => !o.archived);
      if (!selectedOfficeId && offices.length > 0) {
        selectedOfficeId = offices[0].id;
        bookOfficeId = offices[0].id;
        planShiftOfficeId = offices[0].id;
      }
    }
    if (procRes.status === "ok") procedures = procRes.data.filter((p) => p.is_active);
    if (provRes.status === "ok") allProviders = provRes.data;
    if (staffRes.status === "ok") allStaff = staffRes.data.filter((s) => !s.archived);
  }

  async function loadGrid() {
    if (!selectedOfficeId) return;
    scheduleLoading = true;
    const [schedRes, rosterRes] = await Promise.all([
      commands.getSchedule(selectedOfficeId, selectedDate),
      commands.getOfficeProviderSchedule(selectedOfficeId, selectedDate),
    ]);
    scheduleLoading = false;
    if (schedRes.status === "ok") schedule = schedRes.data;
    else toast.error(getErrorMessage(schedRes.error));
    if (rosterRes.status === "ok") providerRoster = rosterRes.data;
  }

  async function loadBookRoster() {
    if (!bookOfficeId || !bookStartDate) return;
    if (bookOfficeId === selectedOfficeId && bookStartDate === selectedDate) {
      bookRoster = [...providerRoster];
      return;
    }
    bookRosterLoading = true;
    const res = await commands.getOfficeProviderSchedule(bookOfficeId, bookStartDate);
    bookRosterLoading = false;
    if (res.status === "ok") bookRoster = res.data;
  }

  async function loadCallList() {
    if (!selectedOfficeId) return;
    const res = await commands.getTomorrowsCallList(selectedOfficeId, callListDate);
    if (res.status === "ok") callList = res.data;
  }

  async function loadRoster() {
    rosterLoading = true;
    const weekStart = getWeekStart(selectedDate);
    const res = await commands.getShiftRoster(weekStart, null);
    rosterLoading = false;
    if (res.status === "ok") shiftRoster = res.data;
    else toast.error(getErrorMessage(res.error));
  }

  async function doPlanShift() {
    if (!planShiftStaffId) { planShiftError = "Select a staff member."; return; }
    if (!planShiftOfficeId) { planShiftError = "Select an office."; return; }
    if (!planShiftRole.trim()) { planShiftError = "Enter a role for this shift."; return; }
    planShiftLoading = true;
    planShiftError = "";
    const staffMember = allStaff.find((s) => s.staff_member_id === planShiftStaffId);
    const res = await commands.planStaffShift(
      planShiftStaffId, planShiftOfficeId, planShiftDate,
      planShiftStart, planShiftEnd, planShiftRole, STAFF_ID,
    );
    planShiftLoading = false;
    if (res.status === "ok") {
      showPlanShift = false;
      const staffName = staffMember?.name ?? "Staff member";
      toast.success(`Shift planned for ${staffName} on ${formatDisplayDate(planShiftDate)} (${planShiftStart}–${planShiftEnd}).`);
      await loadRoster();
    } else {
      planShiftError = getErrorMessage(res.error);
    }
  }

  async function doCancelShift(shift: StaffShiftDto) {
    const ok = await confirm({
      title: "Cancel shift",
      message: `Cancel ${shift.staff_name}'s shift on ${formatDisplayDate(shift.date)} (${shift.start_time}–${shift.end_time})?`,
      confirmLabel: "Cancel shift",
      destructive: true,
    });
    if (!ok) return;
    cancellingShiftId = shift.shift_id;
    const res = await commands.cancelStaffShift(shift.shift_id, null, STAFF_ID);
    cancellingShiftId = null;
    if (res.status === "ok") {
      toast.success(`${shift.staff_name}'s shift on ${formatDisplayDate(shift.date)} cancelled.`);
      await loadRoster();
    } else {
      toast.error(getErrorMessage(res.error));
    }
  }

  // ── Detail drawer ─────────────────────────────────────────────────────────

  async function openDetail(apptId: string) {
    showBookForm = false;
    detailApptId = apptId;
    detailLoading = true;
    detailData = null;
    showCancelConfirm = false;
    cancelReason = "";
    noteText = "";
    noteError = "";
    const res = await commands.getAppointment(apptId);
    detailLoading = false;
    if (res.status === "ok") detailData = res.data;
  }

  function closeDetail() {
    detailApptId = null;
    detailData = null;
    showCancelConfirm = false;
    cancelReason = "";
    noteText = "";
    noteError = "";
    showReschedule = false;
    reschedError = "";
    reschedRoster = [];
  }

  // ── Book drawer ───────────────────────────────────────────────────────────

  function openBookDrawer(providerId = "", startTime = "") {
    closeDetail();
    bookOfficeId = selectedOfficeId;
    bookStartDate = selectedDate;
    bookRoster = [...providerRoster];
    bookProviderId = providerId;
    bookStartTime = startTime;
    bookPatientId = "";
    bookPatientName = "";
    bookPatientSearch = "";
    bookProcedureId = "";
    bookError = "";
    showBookForm = true;
  }

  // ── Grid column click → pre-fill booking drawer ───────────────────────────

  function onColumnClick(e: MouseEvent, providerId: string) {
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const slotIndex = Math.floor((e.clientY - rect.top) / SLOT_HEIGHT);
    const mins = openMins + slotIndex * 15;
    if (mins >= closeMins) return;
    openBookDrawer(providerId, minsToHHMM(mins));
  }

  // ── Patient search ────────────────────────────────────────────────────────

  async function searchPatients() {
    if (bookPatientSearch.trim().length < 2) { patientSearchResults = []; return; }
    const res = await commands.searchPatients(bookPatientSearch, null, null, false);
    if (res.status === "ok") {
      patientSearchResults = res.data.map((p) => ({
        patient_id: p.patient_id,
        patient_name: p.full_name_display,
        first_name: p.first_name,
        last_name: p.last_name,
        phone: p.phone,
      }));
    }
  }

  function onPatientSearchInput() {
    if (patientSearchDebounce) clearTimeout(patientSearchDebounce);
    patientSearchDebounce = setTimeout(searchPatients, 250);
  }

  function selectPatient(p: typeof patientSearchResults[0]) {
    bookPatientId = p.patient_id;
    bookPatientName = p.patient_name;
    bookPatientSearch = p.patient_name;
    patientSearchResults = [];
  }

  function onBookProviderChange(providerId: string) {
    bookProviderId = providerId;
    const entry = bookRoster.find((e) => e.provider_id === providerId);
    if (entry) {
      bookStartTime = generateTimeSlots(entry.start_time, entry.end_time)[0] ?? "";
    } else {
      bookStartTime = "";
    }
  }

  // ── Actions ───────────────────────────────────────────────────────────────

  async function doBookAppointment() {
    if (!bookPatientId) { bookError = "Select a patient from the search results."; return; }
    if (!bookProviderId) { bookError = "Select a provider."; return; }
    if (!bookProcedureId) { bookError = "Select a procedure."; return; }
    if (!bookOfficeId) { bookError = "Select an office."; return; }
    bookLoading = true;
    bookError = "";
    const res = await commands.bookAppointment(
      bookOfficeId, bookPatientId, bookProcedureId, bookProviderId,
      buildStartTime(bookStartDate, bookStartTime), null, STAFF_ID,
    );
    bookLoading = false;
    if (res.status === "ok") {
      showBookForm = false;
      toast.success(`Appointment booked for ${bookPatientName} on ${formatDisplayDate(bookStartDate)} at ${minsTo12h(parseHHMM(bookStartTime))}.`);
      await loadGrid();
    } else {
      bookError = getErrorMessage(res.error);
    }
  }

  async function doComplete(apptId: string) {
    const patientName = detailData?.appointment.patient_name ?? "Appointment";
    const apptTimeLabel = detailData ? ` at ${formatTime(detailData.appointment.start_time)}` : "";
    const ok = await confirm({
      title: "Mark appointment complete",
      message: `Mark ${patientName}${apptTimeLabel} as completed?`,
      confirmLabel: "Mark complete",
    });
    if (!ok) return;
    completingAppt = true;
    const res = await commands.completeAppointment(apptId, STAFF_ID);
    completingAppt = false;
    if (res.status === "ok") {
      toast.success(`${patientName}${apptTimeLabel} marked complete.`);
      closeDetail();
      await loadGrid();
    } else {
      toast.error(getErrorMessage(res.error));
    }
  }

  async function doNoShow(apptId: string) {
    const patientName = detailData?.appointment.patient_name ?? "Appointment";
    const apptTimeLabel = detailData ? ` at ${formatTime(detailData.appointment.start_time)}` : "";
    const ok = await confirm({
      title: "Mark no-show",
      message: `Mark ${patientName}${apptTimeLabel} as a no-show?`,
      confirmLabel: "Mark no-show",
      destructive: true,
    });
    if (!ok) return;
    noShowingAppt = true;
    const res = await commands.markAppointmentNoShow(apptId, STAFF_ID);
    noShowingAppt = false;
    if (res.status === "ok") {
      toast.success(`${patientName}${apptTimeLabel} marked no-show.`);
      closeDetail();
      await loadGrid();
    } else {
      toast.error(getErrorMessage(res.error));
    }
  }

  async function doCancel(apptId: string) {
    const patientName = detailData?.appointment.patient_name ?? "Appointment";
    const apptTime = detailData ? ` at ${formatTime(detailData.appointment.start_time)}` : "";
    cancellingAppt = true;
    const res = await commands.cancelAppointment(apptId, STAFF_ID, cancelReason.trim() || null);
    cancellingAppt = false;
    if (res.status === "ok") {
      toast.success(`${patientName}${apptTime} cancelled.`);
      closeDetail();
      await loadGrid();
    } else {
      toast.error(getErrorMessage(res.error));
    }
  }

  async function doAddNote() {
    if (!detailApptId || !noteText.trim()) { noteError = "Note text is required."; return; }
    noteLoading = true;
    noteError = "";
    const res = await commands.addAppointmentNote(detailApptId, noteText, STAFF_ID);
    noteLoading = false;
    if (res.status === "ok") {
      toast.success("Note added.");
      noteText = "";
      const dr = await commands.getAppointment(detailApptId);
      if (dr.status === "ok") detailData = dr.data;
    } else {
      noteError = getErrorMessage(res.error);
    }
  }

  // ── Reschedule ────────────────────────────────────────────────────────────

  function openReschedule() {
    if (!detailData) return;
    const appt = detailData.appointment;
    showReschedule = true;
    showCancelConfirm = false;
    reschedOfficeId = appt.office_id;
    reschedProviderId = appt.provider_id;
    reschedDate = appt.start_time.slice(0, 10);
    reschedTime = appt.start_time.slice(11, 16);
    reschedError = "";
    reschedRoster = [];
  }

  function closeReschedule() {
    showReschedule = false;
    reschedError = "";
    reschedRoster = [];
  }

  async function loadReschedRoster() {
    if (!reschedOfficeId || !reschedDate) return;
    reschedRosterLoading = true;
    const res = await commands.getOfficeProviderSchedule(reschedOfficeId, reschedDate);
    reschedRosterLoading = false;
    if (res.status === "ok") reschedRoster = res.data;
  }

  async function doReschedule(apptId: string) {
    if (!reschedProviderId || !reschedDate || !reschedTime) {
      reschedError = "Select a new date, provider, and time slot.";
      return;
    }
    reschedLoading = true;
    reschedError = "";
    const patientName = detailData?.appointment.patient_name ?? "Appointment";
    const res = await commands.rescheduleAppointment(
      apptId, reschedOfficeId, reschedProviderId,
      buildStartTime(reschedDate, reschedTime), null, STAFF_ID,
    );
    reschedLoading = false;
    if (res.status === "ok") {
      toast.success(`${patientName} rescheduled to ${formatDisplayDate(reschedDate)} at ${minsTo12h(parseHHMM(reschedTime))}.`);
      closeDetail();
      await loadGrid();
    } else {
      reschedError = getErrorMessage(res.error);
    }
  }

  // ── Keyboard shortcuts ────────────────────────────────────────────────────

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      if (showBookForm) { showBookForm = false; return; }
      if (detailApptId) { closeDetail(); return; }
    }
  }

  // ── Init ──────────────────────────────────────────────────────────────────

  onMount(async () => {
    await loadSetupData();
    await loadGrid();
  });

  $effect(() => {
    if (selectedOfficeId && selectedDate) { showAllProviders = false; loadGrid(); }
  });

  $effect(() => {
    if (showBookForm && bookOfficeId && bookStartDate) loadBookRoster();
  });

  // When procedure changes and the selected provider is no longer eligible, deselect them.
  $effect(() => {
    if (bookProviderId && eligibleBookRoster.length > 0) {
      const stillEligible = eligibleBookRoster.some((e) => e.provider_id === bookProviderId);
      if (!stillEligible) { bookProviderId = ""; bookStartTime = ""; }
    } else if (bookProviderId && eligibleBookRoster.length === 0 && bookProcedureId) {
      bookProviderId = ""; bookStartTime = "";
    }
  });

  $effect(() => {
    if (showReschedule && reschedOfficeId && reschedDate) loadReschedRoster();
  });

  $effect(() => {
    if (scheduleView === "roster" && selectedDate) loadRoster();
  });

  // Pre-fill planShiftRole when staff changes
  $effect(() => {
    if (planShiftStaffId) {
      const s = allStaff.find((m) => m.staff_member_id === planShiftStaffId);
      if (s && s.roles.length > 0) {
        planShiftRole = s.roles[0];
      }
    }
  });
</script>

<svelte:window onkeydown={onKeydown} />

<!-- ═══════════════════════════════════════════════════════
     BOOKING DRAWER
     ═══════════════════════════════════════════════════════ -->
{#if showBookForm}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="drawer-overlay" onclick={() => (showBookForm = false)} aria-hidden="true"></div>

  <div class="drawer" role="dialog" aria-modal="true" aria-labelledby="book-drawer-title">
    <div class="drawer-header">
      <h2 class="drawer-title" id="book-drawer-title">Book appointment</h2>
      <button class="btn btn-ghost btn-icon btn-sm" onclick={() => (showBookForm = false)} aria-label="Close booking form">✕</button>
    </div>

    <div class="drawer-body">
      <!-- When & where -->
      <div class="book-section">
        <p class="book-section-label">When &amp; where</p>
        <div class="book-row">
          <div class="form-field" style="flex:1">
            <label class="field-label" for="book-date">Date</label>
            <input id="book-date" type="date" bind:value={bookStartDate} />
          </div>
          <div class="form-field" style="flex:1">
            <label class="field-label" for="book-office">Office</label>
            <select id="book-office" bind:value={bookOfficeId}>
              {#each offices as o}<option value={o.id}>{o.name}</option>{/each}
            </select>
          </div>
        </div>
      </div>

      <!-- Provider -->
      <div class="book-section">
        <p class="book-section-label">Provider</p>
        {#if bookRosterLoading}
          <div class="load-row"><div class="spinner spinner-sm"></div> Checking availability…</div>
        {:else if bookRoster.length === 0}
          <p class="field-hint">
            No providers scheduled on {formatDisplayDate(bookStartDate)} at this office.
            <a href="/setup">Set availability in Setup → Providers</a>.
          </p>
        {:else if eligibleBookRoster.length === 0 && bookProcedureId}
          {@const proc = procedures.find((p) => p.id === bookProcedureId)}
          <div class="field-error" role="alert">
            <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true" style="flex-shrink:0"><circle cx="8" cy="8" r="6"/><line x1="8" y1="5" x2="8" y2="8"/><line x1="8" y1="11" x2="8" y2="11.5" stroke-width="2"/></svg>
            No eligible providers for {proc?.name ?? "this procedure"} on {formatDisplayDate(bookStartDate)}.
            {proc?.required_provider_type ? `${proc.name} requires a ${proc.required_provider_type} or higher.` : ""}
            Try a different day or procedure.
          </div>
        {:else}
          <div class="chip-group">
            {#each eligibleBookRoster as entry}
              <button
                class="chip"
                class:chip-selected={bookProviderId === entry.provider_id}
                onclick={() => onBookProviderChange(entry.provider_id)}
              >
                <span class="chip-name">{entry.provider_name}</span>
                <span class="chip-hours">{entry.start_time}–{entry.end_time}</span>
              </button>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Time slot -->
      {#if bookProviderId && availableSlots.length > 0}
        <div class="book-section">
          <p class="book-section-label">Time</p>
          <div class="slot-grid">
            {#each availableSlots as slot}
              <button
                class="slot-btn"
                class:slot-selected={bookStartTime === slot}
                onclick={() => (bookStartTime = slot)}
              >{minsTo12h(parseHHMM(slot))}</button>
            {/each}
          </div>
        </div>
      {/if}

      <!-- Patient -->
      {#if bookStartTime}
        <div class="book-section">
          <p class="book-section-label">Patient</p>
          <div class="patient-search-wrap">
            <label class="field-label" for="book-patient">Search by name</label>
            <input
              id="book-patient"
              type="text"
              bind:value={bookPatientSearch}
              placeholder="Type name to search…"
              oninput={onPatientSearchInput}
              autocomplete="off"
            />
            {#if patientSearchResults.length > 0}
              <ul class="patient-dropdown" role="listbox" aria-label="Patient search results">
                {#each patientSearchResults as p}
                  <li role="option" aria-selected={bookPatientId === p.patient_id}>
                    <button onclick={() => selectPatient(p)} class="dropdown-item">
                      {p.patient_name}
                      {#if p.phone}<span class="text-muted"> · {p.phone}</span>{/if}
                    </button>
                  </li>
                {/each}
              </ul>
            {:else if bookPatientSearch.trim().length >= 2 && !bookPatientId}
              <p class="field-hint">No patients found. <a href="/patients">Register a patient first</a>.</p>
            {/if}
            {#if bookPatientId}
              <div class="selected-patient">
                <span class="check-icon">✓</span> <strong>{bookPatientName}</strong>
              </div>
            {/if}
          </div>
        </div>
      {/if}

      <!-- Procedure -->
      {#if bookPatientId}
        <div class="book-section">
          <p class="book-section-label">Procedure</p>
          {#if procedures.length === 0}
            <p class="field-hint">No procedures set up. <a href="/setup">Go to Setup → Procedure Types</a>.</p>
          {:else}
            <label class="field-label" for="book-procedure">Select procedure</label>
            <select id="book-procedure" bind:value={bookProcedureId}>
              <option value="">— Select procedure —</option>
              {#each procedures as p}
                <option value={p.id}>{p.name} ({p.default_duration_minutes} min)</option>
              {/each}
            </select>
          {/if}
        </div>
      {/if}

      {#if bookError}
        <div class="field-error" role="alert">{bookError}</div>
      {/if}
    </div>

    <div class="drawer-footer">
      <button class="btn btn-ghost" onclick={() => (showBookForm = false)}>Cancel</button>
      <button
        class="btn btn-primary"
        onclick={doBookAppointment}
        disabled={bookLoading || !bookPatientId || !bookProviderId || !bookProcedureId || !bookStartTime}
      >
        {#if bookLoading}<span class="spinner" aria-hidden="true"></span><span class="sr-only">Booking</span>{:else}Book appointment{/if}
      </button>
    </div>
  </div>
{/if}

<!-- ═══════════════════════════════════════════════════════
     DETAIL DRAWER
     ═══════════════════════════════════════════════════════ -->
{#if detailApptId !== null}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="drawer-overlay" onclick={closeDetail} aria-hidden="true"></div>

  <div class="drawer" role="dialog" aria-modal="true" aria-labelledby="detail-drawer-title">
    <div class="drawer-header">
      <h2 class="drawer-title" id="detail-drawer-title">Appointment</h2>
      <button class="btn btn-ghost btn-icon btn-sm" onclick={closeDetail} aria-label="Close detail">✕</button>
    </div>

    <div class="drawer-body">
      {#if detailLoading}
        <div class="load-row" style="justify-content:center; padding: 2rem 0;">
          <div class="spinner"></div>
        </div>
      {:else if detailData}
        {@const appt = detailData.appointment}

        <!-- Key info -->
        <dl class="detail-dl">
          <dt>Patient</dt>
          <dd><strong>{appt.patient_name}</strong></dd>
          <dt>Procedure</dt>
          <dd>{appt.procedure_name} · {appt.duration_minutes} min</dd>
          <dt>Provider</dt>
          <dd>{appt.provider_name}</dd>
          <dt>Time</dt>
          <dd>{formatTime(appt.start_time)} – {formatTime(appt.end_time)}</dd>
          <dt>Status</dt>
          <dd><span class="badge {statusBadgeClass(appt.status)}">{appt.status}</span></dd>
        </dl>

        <!-- Actions for Booked appointments -->
        {#if appt.status === "Booked"}
          {#if !showCancelConfirm}
            <div class="detail-actions">
              <button class="btn btn-primary btn-sm" onclick={() => doComplete(appt.appointment_id)} disabled={completingAppt || noShowingAppt || cancellingAppt || reschedLoading}>
                {#if completingAppt}<span class="spinner" aria-hidden="true"></span><span class="sr-only">Completing</span>{:else}Mark complete{/if}
              </button>
              <button class="btn btn-ghost btn-sm" onclick={() => doNoShow(appt.appointment_id)} disabled={completingAppt || noShowingAppt || cancellingAppt || reschedLoading}>
                {#if noShowingAppt}<span class="spinner" aria-hidden="true"></span><span class="sr-only">Saving</span>{:else}No-show{/if}
              </button>
              <button class="btn btn-ghost btn-sm" onclick={() => { if (showReschedule) closeReschedule(); else openReschedule(); }} disabled={completingAppt || noShowingAppt || cancellingAppt || reschedLoading}>
                Reschedule
              </button>
              <button class="btn btn-destructive btn-sm" onclick={() => (showCancelConfirm = true)} disabled={completingAppt || noShowingAppt || cancellingAppt || reschedLoading}>
                Cancel appointment
              </button>
            </div>
          {:else}
            <div class="cancel-confirm-box">
              <p class="cancel-confirm-label">Cancel {appt.patient_name}'s appointment?</p>
              <div class="form-field">
                <label class="field-label" for="cancel-reason">Reason (optional)</label>
                <textarea id="cancel-reason" bind:value={cancelReason} rows={2} placeholder="e.g. Patient called to cancel"></textarea>
              </div>
              <div class="cancel-confirm-actions">
                <button class="btn btn-ghost btn-sm" onclick={() => (showCancelConfirm = false)}>Go back</button>
                <button class="btn btn-destructive btn-sm" onclick={() => doCancel(appt.appointment_id)} disabled={cancellingAppt}>
                  {#if cancellingAppt}<span class="spinner" aria-hidden="true"></span><span class="sr-only">Cancelling</span>{:else}Confirm cancellation{/if}
                </button>
              </div>
            </div>
          {/if}
        {/if}

        <!-- Reschedule form -->
        {#if showReschedule}
          <div class="reschedule-box">
            <p class="reschedule-label">Reschedule appointment</p>
            <div class="book-row" style="margin-bottom: var(--space-3);">
              <div class="form-field" style="flex:1">
                <label class="field-label" for="resched-date">New date</label>
                <input id="resched-date" type="date" bind:value={reschedDate} onchange={() => loadReschedRoster()} />
              </div>
              <div class="form-field" style="flex:1">
                <label class="field-label" for="resched-office">Office</label>
                <select id="resched-office" bind:value={reschedOfficeId} onchange={() => loadReschedRoster()}>
                  {#each offices as o}<option value={o.id}>{o.name}</option>{/each}
                </select>
              </div>
            </div>
            <p class="book-section-label">Provider</p>
            {#if reschedRosterLoading}
              <div class="load-row"><div class="spinner spinner-sm"></div> Checking availability…</div>
            {:else if reschedRoster.length === 0 && reschedDate}
              <p class="field-hint">No providers scheduled on {formatDisplayDate(reschedDate)} at this office. <a href="/setup">Set availability in Setup → Providers</a>.</p>
            {:else}
              <div class="chip-group" style="margin-bottom: var(--space-3);">
                {#each reschedRoster as entry}
                  <button
                    class="chip"
                    class:chip-selected={reschedProviderId === entry.provider_id}
                    onclick={() => { reschedProviderId = entry.provider_id; reschedTime = generateTimeSlots(entry.start_time, entry.end_time)[0] ?? ""; }}
                  >
                    <span class="chip-name">{entry.provider_name}</span>
                    <span class="chip-hours">{entry.start_time}–{entry.end_time}</span>
                  </button>
                {/each}
              </div>
            {/if}
            {#if reschedProviderId && reschedAvailableSlots.length > 0}
              <p class="book-section-label">New time</p>
              <div class="slot-grid" style="margin-bottom: var(--space-3);">
                {#each reschedAvailableSlots as slot}
                  <button
                    class="slot-btn"
                    class:slot-selected={reschedTime === slot}
                    onclick={() => (reschedTime = slot)}
                  >{minsTo12h(parseHHMM(slot))}</button>
                {/each}
              </div>
            {/if}
            {#if reschedError}
              <div class="field-error" role="alert" style="margin-bottom: var(--space-2);">{reschedError}</div>
            {/if}
            <div class="reschedule-actions">
              <button class="btn btn-ghost btn-sm" onclick={closeReschedule}>Cancel</button>
              <button
                class="btn btn-primary btn-sm"
                onclick={() => doReschedule(appt.appointment_id)}
                disabled={reschedLoading || !reschedProviderId || !reschedTime}
              >
                {#if reschedLoading}<span class="spinner" aria-hidden="true"></span><span class="sr-only">Rescheduling</span>{:else}Confirm reschedule{/if}
              </button>
            </div>
          </div>
        {/if}

        <!-- Notes -->
        <div class="notes-section">
          <h3 class="notes-heading">Notes {detailData.notes.length > 0 ? `(${detailData.notes.length})` : ""}</h3>

          {#if detailData.notes.length === 0}
            <p class="text-muted text-sm">No notes yet.</p>
          {:else}
            <ul class="notes-list">
              {#each detailData.notes as note}
                <li class="note-item">
                  <p class="note-meta">{formatDate(note.recorded_at)} {formatTime(note.recorded_at)} · {note.recorded_by}</p>
                  <p class="note-text">{note.text}</p>
                </li>
              {/each}
            </ul>
          {/if}

          <div class="add-note-form">
            <label class="field-label" for="note-text">Add note</label>
            <textarea
              id="note-text"
              placeholder="Add a note…"
              bind:value={noteText}
              rows={2}
            ></textarea>
            {#if noteError}<p class="field-error" role="alert">{noteError}</p>{/if}
            <button
              class="btn btn-secondary btn-sm"
              onclick={doAddNote}
              disabled={noteLoading || !noteText.trim()}
              style="margin-top: var(--space-2);"
            >
              {#if noteLoading}<span class="spinner" aria-hidden="true"></span><span class="sr-only">Saving</span>{:else}Add note{/if}
            </button>
          </div>
        </div>
      {:else}
        <p class="text-muted">Appointment not found.</p>
      {/if}
    </div>
  </div>
{/if}

<!-- ═══════════════════════════════════════════════════════
     MAIN PAGE
     ═══════════════════════════════════════════════════════ -->
<div class="page-content">

  <!-- Page header -->
  <div class="page-header">
    <h1 class="page-title">Schedule</h1>
    <div class="header-actions">
      <!-- View switcher -->
      <div class="view-tabs" role="tablist" aria-label="Schedule view">
        <button
          class="view-tab"
          class:active={scheduleView === "grid"}
          role="tab"
          aria-selected={scheduleView === "grid"}
          onclick={() => (scheduleView = "grid")}
        >
          <svg viewBox="0 0 24 24" stroke-width="1.75" stroke="currentColor" fill="none" aria-hidden="true" class="icon-sm"><rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/></svg>
          Schedule
        </button>
        <button
          class="view-tab"
          class:active={scheduleView === "roster"}
          role="tab"
          aria-selected={scheduleView === "roster"}
          onclick={() => (scheduleView = "roster")}
        >
          <svg viewBox="0 0 24 24" stroke-width="1.75" stroke="currentColor" fill="none" aria-hidden="true" class="icon-sm"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/></svg>
          Roster
        </button>
      </div>
      {#if scheduleView === "grid"}
        <button
          class="btn btn-ghost btn-sm"
          onclick={() => { showCallList = !showCallList; if (showCallList) loadCallList(); }}
        >
          {showCallList ? "Hide call list" : "Call list"}
        </button>
        <button
          class="btn btn-primary"
          onclick={() => openBookDrawer()}
          disabled={!selectedOfficeId}
        >
          + Book appointment
        </button>
      {:else}
        <button
          class="btn btn-primary"
          onclick={() => { showPlanShift = true; planShiftError = ""; planShiftDate = selectedDate; if (allStaff.length > 0 && !planShiftStaffId) planShiftStaffId = allStaff[0].staff_member_id; if (offices.length > 0 && !planShiftOfficeId) planShiftOfficeId = offices[0].id; }}
          disabled={allStaff.length === 0 || offices.length === 0}
        >
          + Plan shift
        </button>
      {/if}
    </div>
  </div>

  {#if offices.length === 0}
    <div class="empty-state">
      <span class="empty-state-icon" aria-hidden="true">🏥</span>
      <p class="empty-state-title">No offices configured</p>
      <p class="empty-state-message">Go to <a href="/setup">Setup → Offices</a> to add an office.</p>
    </div>
  {:else if scheduleView === "roster"}
    <!-- ══ ROSTER VIEW ══ -->
    <div class="date-nav">
      <button class="btn btn-ghost btn-icon btn-sm" onclick={() => (selectedDate = addDays(getWeekStart(selectedDate), -7))} title="Previous week" aria-label="Previous week">«</button>
      <span class="date-display">{formatWeekRange(getWeekStart(selectedDate))}</span>
      <button class="btn btn-ghost btn-icon btn-sm" onclick={() => (selectedDate = addDays(getWeekStart(selectedDate), 7))} title="Next week" aria-label="Next week">»</button>
      <button class="btn btn-ghost btn-sm" onclick={() => (selectedDate = todayLocal())}>This week</button>
    </div>

    {#if showPlanShift}
      <!-- Plan shift form panel -->
      <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
      <div class="drawer-overlay" onclick={() => (showPlanShift = false)} aria-hidden="true"></div>
      <div class="drawer" role="dialog" aria-modal="true" aria-labelledby="plan-shift-title">
        <div class="drawer-header">
          <h2 class="drawer-title" id="plan-shift-title">Plan shift</h2>
          <button class="btn btn-ghost btn-icon btn-sm" onclick={() => (showPlanShift = false)} aria-label="Close plan shift form">
            <svg viewBox="0 0 24 24" stroke-width="1.75" stroke="currentColor" fill="none" aria-hidden="true" class="icon-sm"><path d="M18 6 6 18M6 6l12 12"/></svg>
          </button>
        </div>
        <div class="drawer-body">
          <div class="form-field">
            <label class="field-label" for="ps-staff">Staff member</label>
            <select id="ps-staff" bind:value={planShiftStaffId} onchange={() => { const s = allStaff.find(m => m.staff_member_id === planShiftStaffId); if (s && s.roles.length > 0) planShiftRole = s.roles[0]; else planShiftRole = ""; }}>
              <option value="">— Select staff member —</option>
              {#each allStaff as s}
                <option value={s.staff_member_id}>{s.name} ({s.roles.join(", ")})</option>
              {/each}
            </select>
          </div>
          <div class="form-field">
            <label class="field-label" for="ps-office">Office</label>
            <select id="ps-office" bind:value={planShiftOfficeId}>
              {#each offices as o}
                <option value={o.id}>{o.name}</option>
              {/each}
            </select>
          </div>
          <div class="book-row">
            <div class="form-field" style="flex:1">
              <label class="field-label" for="ps-date">Date</label>
              <input id="ps-date" type="date" bind:value={planShiftDate} />
            </div>
          </div>
          <div class="book-row">
            <div class="form-field" style="flex:1">
              <label class="field-label" for="ps-start">Start time</label>
              <input id="ps-start" type="time" bind:value={planShiftStart} />
            </div>
            <div class="form-field" style="flex:1">
              <label class="field-label" for="ps-end">End time</label>
              <input id="ps-end" type="time" bind:value={planShiftEnd} />
            </div>
          </div>
          <div class="form-field">
            <label class="field-label" for="ps-role">Role for this shift</label>
            {#if planShiftStaffId}
              {@const staffMember = allStaff.find((s) => s.staff_member_id === planShiftStaffId)}
              {#if staffMember && staffMember.roles.length > 0}
                <select id="ps-role" bind:value={planShiftRole}>
                  {#each staffMember.roles as r}
                    <option value={r}>{r}</option>
                  {/each}
                </select>
              {:else}
                <input id="ps-role" type="text" bind:value={planShiftRole} placeholder="e.g. Staff" />
              {/if}
            {:else}
              <input id="ps-role" type="text" bind:value={planShiftRole} placeholder="Select a staff member first" disabled />
            {/if}
          </div>
          {#if planShiftError}
            <div class="field-error" role="alert">
              <svg viewBox="0 0 24 24" stroke-width="1.75" stroke="currentColor" fill="none" aria-hidden="true" class="icon-sm"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
              {planShiftError}
            </div>
          {/if}
        </div>
        <div class="drawer-footer">
          <button class="btn btn-ghost" onclick={() => (showPlanShift = false)}>Cancel</button>
          <button
            class="btn btn-primary"
            onclick={doPlanShift}
            disabled={planShiftLoading || !planShiftStaffId || !planShiftOfficeId || !planShiftRole.trim()}
          >
            {#if planShiftLoading}
              <span class="spinner" aria-hidden="true"></span><span class="sr-only">Saving</span>
            {:else}
              <svg viewBox="0 0 24 24" stroke-width="1.75" stroke="currentColor" fill="none" aria-hidden="true" class="icon-sm"><path d="M12 5v14M5 12h14"/></svg>
              Plan shift
            {/if}
          </button>
        </div>
      </div>
    {/if}

    {#if rosterLoading}
      <div class="load-row" style="padding: 2rem; justify-content:center;">
        <div class="spinner"></div>
        <span class="text-muted text-sm">Loading roster…</span>
      </div>
    {:else}
      {@const weekStart = getWeekStart(selectedDate)}
      {@const weekDays = getWeekDays(weekStart)}
      {@const activeStaff = allStaff.filter((s) => !s.archived)}

      {#if activeStaff.length === 0}
        <div class="empty-state">
          <svg viewBox="0 0 24 24" stroke-width="1.75" stroke="currentColor" fill="none" aria-hidden="true" class="empty-state-icon-svg"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/></svg>
          <p class="empty-state-title">No staff members</p>
          <p class="empty-state-message">Register staff members in <a href="/staff">Staff</a> to plan shifts.</p>
        </div>
      {:else}
        <div class="roster-table-wrap">
          <table class="roster-table" aria-label="Staff shift roster">
            <thead>
              <tr>
                <th class="roster-name-col">Staff member</th>
                {#each weekDays as day}
                  <th class="roster-day-col" class:roster-today={day === todayLocal()}>
                    <span class="roster-day-abbr">{dayAbbr(day)}</span>
                    <span class="roster-day-num">{dayNum(day)}</span>
                  </th>
                {/each}
              </tr>
            </thead>
            <tbody>
              {#each activeStaff as member}
                {@const memberShifts = shiftRoster.filter((s) => s.staff_member_id === member.staff_member_id)}
                <tr>
                  <td class="roster-name-col">
                    <span class="roster-staff-name">{member.name}</span>
                    <span class="roster-staff-roles text-muted text-xs">{member.roles.join(", ")}</span>
                  </td>
                  {#each weekDays as day}
                    {@const dayShifts = memberShifts.filter((s) => s.date === day)}
                    <td class="roster-day-col" class:roster-today={day === todayLocal()}>
                      {#each dayShifts as shift}
                        <div class="shift-cell" class:shift-cancelled={shift.cancelled}>
                          <span class="shift-times">
                            {#if shift.cancelled}
                              <svg viewBox="0 0 24 24" stroke-width="1.75" stroke="currentColor" fill="none" aria-hidden="true" class="icon-xs shift-cancelled-icon"><path d="M18 6 6 18M6 6l12 12"/></svg>
                            {:else}
                              <svg viewBox="0 0 24 24" stroke-width="1.75" stroke="currentColor" fill="none" aria-hidden="true" class="icon-xs shift-active-icon"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
                            {/if}
                            {shift.start_time}–{shift.end_time}
                          </span>
                          <span class="shift-role">{shift.role}</span>
                          {#if !shift.cancelled}
                            <button
                              class="shift-cancel-btn"
                              onclick={() => doCancelShift(shift)}
                              disabled={cancellingShiftId === shift.shift_id}
                              aria-label="Cancel shift for {shift.staff_name} on {shift.date}"
                              title="Cancel shift"
                            >
                              {#if cancellingShiftId === shift.shift_id}
                                <span class="spinner spinner-xs" aria-hidden="true"></span>
                              {:else}
                                <svg viewBox="0 0 24 24" stroke-width="1.75" stroke="currentColor" fill="none" aria-hidden="true" class="icon-xs"><path d="M18 6 6 18M6 6l12 12"/></svg>
                              {/if}
                            </button>
                          {/if}
                        </div>
                      {/each}
                    </td>
                  {/each}
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    {/if}

  {:else}
    <!-- ══ GRID VIEW ══ -->
    <!-- Office tabs -->
    <div class="office-tabs" role="tablist" aria-label="Select office">
      {#each offices as o}
        <button
          class="office-tab"
          class:active={selectedOfficeId === o.id}
          role="tab"
          aria-selected={selectedOfficeId === o.id}
          onclick={() => (selectedOfficeId = o.id)}
        >{o.name}</button>
      {/each}
    </div>

    <!-- Date navigation -->
    <div class="date-nav">
      <button class="btn btn-ghost btn-icon btn-sm" onclick={() => (selectedDate = addDays(selectedDate, -7))} title="Previous week" aria-label="Previous week">«</button>
      <button class="btn btn-ghost btn-icon btn-sm" onclick={() => (selectedDate = addDays(selectedDate, -1))} title="Previous day" aria-label="Previous day">‹</button>
      <span class="date-display">
        {formatDisplayDate(selectedDate)}
        {#if isToday}<span class="today-chip">Today</span>{/if}
      </span>
      <button class="btn btn-ghost btn-icon btn-sm" onclick={() => (selectedDate = addDays(selectedDate, 1))} title="Next day" aria-label="Next day">›</button>
      <button class="btn btn-ghost btn-icon btn-sm" onclick={() => (selectedDate = addDays(selectedDate, 7))} title="Next week" aria-label="Next week">»</button>
      {#if !isToday}
        <button class="btn btn-ghost btn-sm" onclick={() => (selectedDate = todayLocal())}>Today</button>
      {/if}
    </div>

    <!-- Tomorrow's call list -->
    {#if showCallList}
      <div class="card" style="margin-bottom: var(--space-6);">
        <div class="card-header">
          <h2 class="card-title">Call list<span class="print-only-date">&nbsp;—&nbsp;{formatDisplayDate(callListDate)}</span></h2>
          <input type="date" bind:value={callListDate} onchange={loadCallList} style="min-height:36px;width:auto;" />
          <button class="btn btn-ghost btn-sm print-btn" onclick={() => window.print()}>Print</button>
        </div>
        {#if callList.length === 0}
          <p class="text-muted text-sm">No booked appointments for this date.</p>
        {:else}
          <div class="table-wrap">
            <table class="call-table">
              <thead>
                <tr>
                  <th>Time</th>
                  <th>Patient</th>
                  <th>Phone</th>
                  <th>Pref. channel</th>
                  <th>Procedure</th>
                  <th>Provider</th>
                </tr>
              </thead>
              <tbody>
                {#each callList as e}
                  <tr>
                    <td class="mono">{formatTime(e.start_time)}</td>
                    <td>{e.patient_name}</td>
                    <td class="mono">{e.patient_phone ?? "—"}</td>
                    <td>{e.preferred_contact_channel ?? "—"}</td>
                    <td>{e.procedure_name}</td>
                    <td>{e.provider_name}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      </div>
    {/if}

    <!-- Grid -->
    {#if scheduleLoading}
      <div class="load-row" style="padding: 2rem; justify-content:center;">
        <div class="spinner"></div>
        <span class="text-muted text-sm">Loading schedule…</span>
      </div>
    {:else if officeHoursEntry === null}
      <div class="empty-state">
        <span class="empty-state-icon" aria-hidden="true">🔒</span>
        <p class="empty-state-title">Closed on {dayName}</p>
        <p class="empty-state-message">Set office hours in <a href="/setup">Setup → Offices</a>.</p>
      </div>
    {:else if officeProviders.length === 0}
      <div class="empty-state">
        <span class="empty-state-icon" aria-hidden="true">👥</span>
        <p class="empty-state-title">No providers assigned</p>
        <p class="empty-state-message">Assign providers to this office in <a href="/setup">Setup → Providers</a>.</p>
      </div>
    {:else}
      <!-- Provider visibility toggle (SCH-2) -->
      {#if providerRoster.length < officeProviders.length}
        <div class="provider-toggle-row">
          <button class="btn btn-ghost btn-sm" onclick={() => (showAllProviders = !showAllProviders)}>
            {showAllProviders
              ? "Show scheduled only"
              : `Show all providers (${officeProviders.length - providerRoster.length} not scheduled today)`}
          </button>
        </div>
      {/if}

      {#if visibleProviders.length === 0}
        <div class="empty-state">
          <span class="empty-state-icon" aria-hidden="true">📅</span>
          <p class="empty-state-title">No providers scheduled on {dayName}</p>
          <p class="empty-state-message">
            <button class="link-btn" onclick={() => (showAllProviders = true)}>Show all providers</button>
            or set availability in <a href="/setup">Setup → Providers</a>.
          </p>
        </div>
      {:else}
        <div class="grid-outer">
          <!-- Column headers -->
          <div class="grid-header">
            <div class="time-col-head" aria-hidden="true"></div>
            {#each visibleProviders as prov}
              {@const rosterEntry = providerRoster.find((r) => r.provider_id === prov.id)}
              <div class="col-head" class:col-head-off={!rosterEntry}>
                <div class="col-head-name">{prov.name}</div>
                {#if rosterEntry}
                  <div class="col-head-hours">{rosterEntry.start_time}–{rosterEntry.end_time}</div>
                {:else}
                  <div class="col-head-off-label">Not scheduled</div>
                {/if}
              </div>
            {/each}
          </div>

          <!-- Grid body -->
          <div class="grid-body">
            <!-- Time labels -->
            <div class="time-col" style="height: {gridHeight}px" aria-hidden="true">
              {#each timeTicks as tick}
                <div class="time-tick" style="top: {tick.top}px">{tick.label}</div>
              {/each}
            </div>

            <!-- Provider columns -->
            {#each visibleProviders as prov}
              {@const rosterEntry = providerRoster.find((r) => r.provider_id === prov.id)}
              {@const isWorking = !!rosterEntry}
              {@const provStart = rosterEntry ? parseHHMM(rosterEntry.start_time) : openMins}
              {@const provEnd   = rosterEntry ? parseHHMM(rosterEntry.end_time)   : openMins}
              {@const appts = schedule.filter((a) => a.provider_id === prov.id)}

              {#if isWorking}
                <div
                  class="provider-col"
                  style="height: {gridHeight}px"
                  role="button"
                  tabindex="0"
                  aria-label="Book appointment with {prov.name}"
                  onclick={(e) => onColumnClick(e, prov.id)}
                  onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); onColumnClick(e as unknown as MouseEvent, prov.id); } }}
                >
                  {#each timeTicks as tick}
                    <div class="h-line" style="top: {tick.top}px" aria-hidden="true"></div>
                  {/each}

                  {#if provStart > openMins}
                    <div class="unavail" style="top: 0; height: {((provStart - openMins) / 15) * SLOT_HEIGHT}px" aria-hidden="true"></div>
                  {/if}

                  {#if provEnd < closeMins}
                    <div class="unavail" style="top: {((provEnd - openMins) / 15) * SLOT_HEIGHT}px; height: {((closeMins - provEnd) / 15) * SLOT_HEIGHT}px" aria-hidden="true"></div>
                  {/if}

                  {#each appts as appt}
                    {@const apptMins = parseHHMM(appt.start_time.slice(11, 16))}
                    {@const blockTop = ((apptMins - openMins) / 15) * SLOT_HEIGHT}
                    {@const blockH = Math.max((appt.duration_minutes / 15) * SLOT_HEIGHT - 2, 20)}
                    <button
                      class="appt-block {statusBlockClass(appt.status)}"
                      style="top: {blockTop}px; height: {blockH}px"
                      onclick={(e) => { e.stopPropagation(); openDetail(appt.appointment_id); }}
                      title="{appt.patient_name} · {appt.procedure_name} · {formatTime(appt.start_time)}"
                      aria-label="{appt.patient_name}, {appt.procedure_name}, {formatTime(appt.start_time)}"
                    >
                      <span class="appt-time">{formatTime(appt.start_time)}</span>
                      <span class="appt-patient">{appt.patient_name}</span>
                      {#if appt.duration_minutes >= 30}
                        <span class="appt-proc">{appt.procedure_name}</span>
                      {/if}
                    </button>
                  {/each}
                </div>
              {:else}
                <div
                  class="provider-col provider-col-off"
                  style="height: {gridHeight}px"
                  aria-label="{prov.name} — not working {dayName}"
                >
                  {#each timeTicks as tick}
                    <div class="h-line" style="top: {tick.top}px" aria-hidden="true"></div>
                  {/each}
                  <div class="unavail unavail-full" style="top: 0; height: {gridHeight}px" aria-hidden="true"></div>
                  <span class="off-label">Not scheduled</span>
                </div>
              {/if}
            {/each}
          </div>
        </div>
      {/if}
    {/if}
  {/if}
</div>

<style>
  /* ── Page ────────────────────────────────────────────── */
  .header-actions {
    display: flex;
    gap: var(--space-3);
    align-items: center;
  }

  /* ── Office tabs ─────────────────────────────────────── */
  .office-tabs {
    display: flex;
    gap: 2px;
    flex-wrap: wrap;
    margin-bottom: var(--space-4);
    border-bottom: 2px solid var(--pearl-mist-dk);
  }
  .office-tab {
    padding: var(--space-2) var(--space-5);
    background: transparent;
    border: none;
    border-bottom: 2px solid transparent;
    margin-bottom: -2px;
    color: var(--slate-fog);
    font-family: var(--font-body);
    font-size: var(--text-sm);
    font-weight: 500;
    cursor: pointer;
    transition: color var(--transition-fast), border-color var(--transition-fast);
    border-radius: var(--radius-sm) var(--radius-sm) 0 0;
  }
  .office-tab:hover { color: var(--abyss-navy); }
  .office-tab.active {
    color: var(--caribbean-teal);
    border-bottom-color: var(--caribbean-teal);
    font-weight: 600;
  }

  /* ── Date navigation ─────────────────────────────────── */
  .date-nav {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-bottom: var(--space-5);
  }
  .date-display {
    font-family: var(--font-heading);
    font-size: var(--text-base);
    font-weight: 600;
    color: var(--abyss-navy);
    min-width: 220px;
    text-align: center;
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }
  .today-chip {
    font-size: var(--text-xs);
    font-weight: 600;
    font-family: var(--font-heading);
    background: var(--caribbean-teal-lt);
    color: var(--caribbean-teal);
    padding: 2px var(--space-2);
    border-radius: var(--radius-pill);
  }

  /* ── Call list table ─────────────────────────────────── */
  .table-wrap { overflow-x: auto; }
  .call-table {
    width: 100%;
    border-collapse: collapse;
    font-size: var(--text-sm);
  }
  .call-table th {
    text-align: left;
    padding: var(--space-2) var(--space-3);
    font-family: var(--font-heading);
    font-size: var(--text-xs);
    font-weight: 600;
    color: var(--slate-fog);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    border-bottom: 1px solid var(--pearl-mist-dk);
    white-space: nowrap;
  }
  .call-table td {
    padding: var(--space-2) var(--space-3);
    border-bottom: 1px solid var(--pearl-mist-dk);
    color: var(--abyss-navy);
    white-space: nowrap;
  }
  .call-table tbody tr:hover { background: var(--pearl-mist); }
  .mono { font-family: var(--font-mono); font-size: 0.8em; }

  /* ── Schedule grid ───────────────────────────────────── */
  .grid-outer {
    background: #fff;
    border: 1px solid var(--pearl-mist-dk);
    border-radius: var(--radius-lg);
    overflow: hidden;
    box-shadow: var(--shadow-sm);
  }
  .grid-header {
    display: flex;
    border-bottom: 2px solid var(--pearl-mist-dk);
    background: var(--pearl-mist);
    position: sticky;
    top: var(--nav-height, 56px);
    z-index: 10;
  }
  .time-col-head {
    flex: 0 0 72px;
    border-right: 1px solid var(--pearl-mist-dk);
  }
  .col-head {
    flex: 0 0 180px;
    padding: var(--space-3) var(--space-4);
    border-right: 1px solid var(--pearl-mist-dk);
    min-height: 56px;
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 2px;
  }
  .col-head:last-child { border-right: none; }
  .col-head-name {
    font-family: var(--font-heading);
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--abyss-navy);
  }
  .col-head-hours {
    font-size: var(--text-xs);
    color: var(--caribbean-teal);
    font-family: var(--font-mono);
    font-weight: 500;
  }
  .col-head.col-head-off { opacity: 0.5; }
  .col-head-off-label {
    font-size: var(--text-xs);
    color: var(--slate-fog);
    font-style: italic;
  }

  .grid-body { display: flex; overflow-x: auto; }

  /* Time labels column */
  .time-col {
    flex: 0 0 72px;
    position: relative;
    border-right: 1px solid var(--pearl-mist-dk);
    background: var(--pearl-mist);
    overflow: hidden;
    flex-shrink: 0;
  }
  .time-tick {
    position: absolute;
    left: 0;
    right: var(--space-2);
    font-size: 0.68rem;
    color: var(--slate-fog);
    text-align: right;
    transform: translateY(-50%);
    pointer-events: none;
    font-family: var(--font-mono);
    white-space: nowrap;
  }

  /* Provider columns */
  .provider-col {
    flex: 0 0 180px;
    position: relative;
    border-right: 1px solid var(--pearl-mist-dk);
    background: #fff;
    cursor: crosshair;
    overflow: visible;
    flex-shrink: 0;
  }
  .provider-col:last-child { border-right: none; }
  .provider-col:hover { background: #fafcfd; }
  .provider-col-off { cursor: default; }
  .provider-col-off:hover { background: #fff; }

  /* Horizontal grid lines */
  .h-line {
    position: absolute;
    left: 0;
    right: 0;
    height: 1px;
    background: var(--pearl-mist-dk);
    pointer-events: none;
  }

  /* Unavailable zone */
  .unavail {
    position: absolute;
    left: 0;
    right: 0;
    pointer-events: none;
    background: repeating-linear-gradient(
      135deg,
      transparent,
      transparent 4px,
      rgba(107, 124, 130, 0.08) 4px,
      rgba(107, 124, 130, 0.08) 8px
    );
    z-index: 1;
  }
  .unavail-full { background: rgba(240, 244, 245, 0.7); }
  .off-label {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: var(--text-xs);
    color: var(--slate-fog);
    font-style: italic;
    pointer-events: none;
    z-index: 2;
  }

  /* Appointment blocks */
  .appt-block {
    position: absolute;
    left: 3px;
    right: 3px;
    border: none;
    border-radius: var(--radius-sm);
    padding: 3px 6px;
    text-align: left;
    cursor: pointer;
    font-family: var(--font-body);
    overflow: hidden;
    display: flex;
    flex-direction: column;
    gap: 1px;
    z-index: 3;
    transition: filter var(--transition-fast), box-shadow var(--transition-fast);
    box-shadow: var(--shadow-sm);
  }
  .appt-block:hover {
    filter: brightness(0.93);
    box-shadow: var(--shadow-md);
    z-index: 4;
  }

  .appt-booked      { background: var(--color-booked-lt);      color: var(--color-booked);      border-left: 3px solid var(--color-booked); }
  .appt-completed   { background: var(--color-completed-lt);   color: var(--color-completed);   border-left: 3px solid var(--color-completed); }
  .appt-cancelled   { background: var(--color-cancelled-lt);   color: var(--color-cancelled);   border-left: 3px solid var(--color-cancelled); }
  .appt-noshow      { background: var(--color-noshow-lt);      color: var(--color-noshow);      border-left: 3px solid var(--color-noshow); }
  .appt-rescheduled { background: var(--color-rescheduled-lt); color: var(--color-rescheduled); border-left: 3px solid var(--color-rescheduled); }

  .appt-time    { font-size: 0.65rem; font-weight: 700; opacity: 0.8; font-family: var(--font-mono); }
  .appt-patient { font-size: 0.72rem; font-weight: 600; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .appt-proc    { font-size: 0.65rem; opacity: 0.7; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

  /* ── Booking drawer internals ────────────────────────── */
  .book-section { margin-bottom: var(--space-5); }
  .book-section-label {
    font-family: var(--font-heading);
    font-size: var(--text-xs);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--slate-fog);
    margin: 0 0 var(--space-2);
  }
  .book-row {
    display: flex;
    gap: var(--space-3);
  }

  /* Provider chips */
  .chip-group { display: flex; flex-direction: column; gap: var(--space-2); }
  .chip {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-2) var(--space-3);
    background: var(--pearl-mist);
    border: 1.5px solid var(--pearl-mist-dk);
    border-radius: var(--radius-md);
    cursor: pointer;
    text-align: left;
    transition: border-color var(--transition-fast), background var(--transition-fast);
  }
  .chip:hover { border-color: var(--caribbean-teal); background: var(--caribbean-teal-lt); }
  .chip-selected {
    border-color: var(--caribbean-teal);
    background: var(--caribbean-teal-lt);
  }
  .chip-name { font-size: var(--text-sm); font-weight: 600; color: var(--abyss-navy); }
  .chip-hours { font-size: var(--text-xs); color: var(--slate-fog); font-family: var(--font-mono); }

  /* Time slots */
  .slot-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(80px, 1fr));
    gap: var(--space-2);
  }
  .slot-btn {
    padding: var(--space-2) var(--space-1);
    background: var(--pearl-mist);
    border: 1.5px solid var(--pearl-mist-dk);
    border-radius: var(--radius-sm);
    font-size: var(--text-xs);
    font-family: var(--font-mono);
    font-weight: 500;
    color: var(--abyss-navy);
    cursor: pointer;
    text-align: center;
    transition: all var(--transition-fast);
  }
  .slot-btn:hover { border-color: var(--caribbean-teal); color: var(--caribbean-teal); }
  .slot-selected {
    background: var(--caribbean-teal);
    border-color: var(--caribbean-teal);
    color: #fff;
  }

  /* Patient search */
  .patient-search-wrap { position: relative; }
  .patient-dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    background: #fff;
    border: 1.5px solid var(--caribbean-teal);
    border-top: none;
    border-radius: 0 0 var(--radius-md) var(--radius-md);
    list-style: none;
    margin: 0;
    padding: var(--space-1) 0;
    box-shadow: var(--shadow-md);
    z-index: 10;
    max-height: 200px;
    overflow-y: auto;
  }
  .dropdown-item {
    display: block;
    width: 100%;
    text-align: left;
    padding: var(--space-2) var(--space-3);
    background: none;
    border: none;
    font-size: var(--text-sm);
    color: var(--abyss-navy);
    cursor: pointer;
  }
  .dropdown-item:hover { background: var(--caribbean-teal-lt); }
  .selected-patient {
    margin-top: var(--space-2);
    font-size: var(--text-sm);
    color: var(--island-palm);
    font-weight: 500;
  }
  .check-icon { font-weight: 700; }

  /* ── Detail drawer internals ─────────────────────────── */
  .detail-dl {
    display: grid;
    grid-template-columns: max-content 1fr;
    gap: var(--space-1) var(--space-4);
    margin: 0 0 var(--space-5);
    font-size: var(--text-sm);
  }
  .detail-dl dt {
    color: var(--slate-fog);
    font-weight: 500;
    white-space: nowrap;
    padding: var(--space-1) 0;
  }
  .detail-dl dd {
    margin: 0;
    color: var(--abyss-navy);
    padding: var(--space-1) 0;
  }

  .detail-actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    margin-bottom: var(--space-5);
    padding-top: var(--space-3);
    border-top: 1px solid var(--pearl-mist-dk);
  }

  .cancel-confirm-box {
    padding: var(--space-4);
    background: var(--healthy-coral-lt);
    border: 1.5px solid var(--healthy-coral);
    border-radius: var(--radius-md);
    margin-bottom: var(--space-5);
  }
  .cancel-confirm-label {
    font-family: var(--font-heading);
    font-weight: 600;
    color: var(--healthy-coral-dk);
    margin: 0 0 var(--space-3);
    font-size: var(--text-sm);
  }
  .cancel-confirm-actions {
    display: flex;
    gap: var(--space-2);
    margin-top: var(--space-3);
    justify-content: flex-end;
  }

  /* Notes */
  .notes-section { border-top: 1px solid var(--pearl-mist-dk); padding-top: var(--space-4); }
  .notes-heading {
    font-family: var(--font-heading);
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--abyss-navy);
    margin: 0 0 var(--space-3);
  }
  .notes-list { list-style: none; margin: 0 0 var(--space-4); padding: 0; display: flex; flex-direction: column; gap: var(--space-3); }
  .note-item { padding: var(--space-3); background: var(--pearl-mist); border-radius: var(--radius-sm); }
  .note-meta { font-size: var(--text-xs); color: var(--slate-fog); margin: 0 0 var(--space-1); }
  .note-text { font-size: var(--text-sm); color: var(--abyss-navy); margin: 0; }
  .add-note-form { display: flex; flex-direction: column; }

  /* ── Reschedule box ──────────────────────────────────── */
  .reschedule-box {
    padding: var(--space-4);
    background: var(--pearl-mist);
    border: 1.5px solid var(--pearl-mist-dk);
    border-radius: var(--radius-md);
    margin-bottom: var(--space-5);
  }
  .reschedule-label {
    font-family: var(--font-heading);
    font-weight: 600;
    color: var(--abyss-navy);
    margin: 0 0 var(--space-3);
    font-size: var(--text-sm);
  }
  .reschedule-actions {
    display: flex;
    gap: var(--space-2);
    justify-content: flex-end;
    margin-top: var(--space-3);
  }

  /* ── Provider toggle row (SCH-2) ─────────────────────── */
  .provider-toggle-row {
    display: flex;
    justify-content: flex-end;
    margin-bottom: var(--space-3);
  }

  /* ── Link button (looks like inline link) ────────────── */
  .link-btn {
    background: none;
    border: none;
    padding: 0;
    color: var(--caribbean-teal);
    text-decoration: underline;
    cursor: pointer;
    font-size: inherit;
    font-family: inherit;
  }
  .link-btn:hover { color: var(--caribbean-teal-dk); }

  /* ── Shared ───────────────────────────────────────────── */
  .load-row {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  /* ── Print support ─────────────────────────────────────── */
  .print-only-date {
    display: none;
  }

  @media print {
    /* Hide everything except the call list card */
    :global(.app-nav),
    :global(.skip-link),
    .page-header,
    .office-tabs,
    .date-nav,
    .grid-outer,
    .grid-header,
    .grid-body,
    .empty-state,
    .load-row {
      display: none !important;
    }

    /* Hide the call list toggle button row */
    .header-actions {
      display: none !important;
    }

    /* Hide the date picker — date is shown in the card title */
    input[type="date"] {
      display: none !important;
    }

    /* Hide the Print button itself when printing */
    .print-btn {
      display: none !important;
    }

    /* Show the inline date text only when printing */
    .print-only-date {
      display: inline;
      margin-left: 4px;
      font-size: 11pt;
      color: #333;
    }

    /* Make the call list card print cleanly */
    .page-content {
      padding: 0;
      background: white;
    }

    .card {
      box-shadow: none;
      border: none;
      margin: 0;
    }

    .call-table {
      width: 100%;
      font-size: 11pt;
    }
  }

  /* ── View switcher tabs ───────────────────────────────── */
  .view-tabs {
    display: flex;
    gap: 2px;
    background: var(--pearl-mist);
    border-radius: var(--radius-md);
    padding: 2px;
  }
  .view-tab {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-4);
    border: none;
    border-radius: calc(var(--radius-md) - 2px);
    background: transparent;
    color: var(--slate-fog);
    font-family: var(--font-body);
    font-size: var(--text-sm);
    font-weight: 500;
    cursor: pointer;
    transition: background var(--transition-fast), color var(--transition-fast);
  }
  .view-tab:hover { color: var(--abyss-navy); }
  .view-tab.active {
    background: var(--surface-raised);
    color: var(--caribbean-teal);
    font-weight: 600;
    box-shadow: var(--shadow-card);
  }

  /* ── Roster table ─────────────────────────────────────── */
  .roster-table-wrap {
    overflow-x: auto;
    border-radius: var(--radius-md);
    border: 1.5px solid var(--pearl-mist-dk);
    background: var(--surface-raised);
  }
  .roster-table {
    width: 100%;
    border-collapse: collapse;
    font-family: var(--font-body);
    font-size: var(--text-sm);
  }
  .roster-table th {
    padding: var(--space-2) var(--space-3);
    font-family: var(--font-heading);
    font-size: var(--text-xs);
    font-weight: 600;
    color: var(--slate-fog);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    border-bottom: 1px solid var(--pearl-mist-dk);
    background: var(--pearl-mist);
    white-space: nowrap;
  }
  .roster-table td {
    padding: var(--space-2) var(--space-3);
    border-bottom: 1px solid var(--pearl-mist-dk);
    vertical-align: top;
  }
  .roster-table tr:last-child td { border-bottom: none; }
  .roster-name-col {
    min-width: 140px;
    max-width: 200px;
  }
  .roster-day-col {
    min-width: 90px;
    text-align: center;
  }
  th.roster-today, td.roster-today {
    background: var(--caribbean-teal-lt);
  }
  .roster-day-abbr {
    display: block;
    font-size: var(--text-xs);
    font-weight: 500;
    color: var(--slate-fog);
  }
  .roster-day-num {
    display: block;
    font-size: var(--text-sm);
    font-weight: 700;
    color: var(--abyss-navy);
  }
  .roster-staff-name {
    display: block;
    font-weight: 600;
    color: var(--abyss-navy);
  }
  .roster-staff-roles {
    display: block;
    margin-top: 2px;
  }
  .shift-cell {
    display: flex;
    flex-direction: column;
    gap: 1px;
    background: var(--caribbean-teal-lt);
    border-left: 3px solid var(--caribbean-teal);
    border-radius: 0 var(--radius-sm) var(--radius-sm) 0;
    padding: var(--space-1) var(--space-2);
    margin-bottom: var(--space-1);
    position: relative;
  }
  .shift-cell.shift-cancelled {
    background: var(--pearl-mist);
    border-left-color: var(--slate-fog);
    opacity: 0.6;
  }
  .shift-times {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    font-size: var(--text-xs);
    font-weight: 600;
    color: var(--caribbean-teal);
  }
  .shift-cell.shift-cancelled .shift-times {
    color: var(--slate-fog);
    text-decoration: line-through;
  }
  .shift-role {
    font-size: var(--text-xs);
    color: var(--slate-fog);
  }
  .shift-active-icon {
    color: var(--caribbean-teal);
    flex-shrink: 0;
  }
  .shift-cancelled-icon {
    color: var(--slate-fog);
    flex-shrink: 0;
  }
  .shift-cancel-btn {
    position: absolute;
    top: var(--space-1);
    right: var(--space-1);
    background: none;
    border: none;
    cursor: pointer;
    color: var(--slate-fog);
    padding: 2px;
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0;
    transition: opacity var(--transition-fast), color var(--transition-fast);
  }
  .shift-cell:hover .shift-cancel-btn { opacity: 1; }
  .shift-cancel-btn:hover { color: var(--healthy-coral); }
  .shift-cancel-btn:disabled { cursor: not-allowed; opacity: 0.5; }

  /* ── Icon sizes ──────────────────────────────────────── */
  .icon-sm { width: 16px; height: 16px; flex-shrink: 0; }
  .icon-xs { width: 12px; height: 12px; flex-shrink: 0; }

  /* ── Roster empty state SVG icon ─────────────────────── */
  .empty-state-icon-svg {
    width: 48px;
    height: 48px;
    color: var(--slate-fog);
    margin-bottom: var(--space-3);
  }
</style>
