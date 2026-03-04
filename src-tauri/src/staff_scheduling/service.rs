use chrono::NaiveDate;
use super::types::{ProviderAvailabilityResult, ProviderScheduleEntry};

/// Returns true if `date` (YYYY-MM-DD) is within [start_date, end_date] inclusive.
fn date_in_range(date: &str, start_date: &str, end_date: &str) -> bool {
    date >= start_date && date <= end_date
}

/// Returns the weekday name ("Monday"…"Sunday") for a YYYY-MM-DD date string.
pub fn weekday_of(date: &str) -> Option<String> {
    NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .ok()
        .map(|d| format!("{}", d.format("%A")))
}

/// Pure function: check provider availability at a specific office/date/time.
///
/// Check priority (SS5g — provider archived takes precedence):
/// 1. provider archived → false, "provider archived"
/// 2. not assigned to office → false, "not assigned"
/// 3. office archived OR no hours for day → false, "office closed"
/// 4. active exception covers date → false, "exception"
/// 5. no availability window for day-of-week → false, "no availability"
/// 6. time outside availability window → false, "no availability"
/// 7. → true, null
pub fn compute_provider_availability(
    provider_archived: bool,
    assigned_to_office: bool,
    office_archived: bool,
    office_has_hours_for_day: bool,
    exceptions: &[(String, String)],       // (start_date, end_date) pairs
    availability_for_day: Option<(&str, &str)>, // (start_time, end_time) or None
    date: &str,
    time: &str, // "HH:MM"
) -> ProviderAvailabilityResult {
    if provider_archived {
        return ProviderAvailabilityResult {
            available: false,
            reason: Some("provider archived".into()),
        };
    }
    if !assigned_to_office {
        return ProviderAvailabilityResult {
            available: false,
            reason: Some("not assigned".into()),
        };
    }
    if office_archived || !office_has_hours_for_day {
        return ProviderAvailabilityResult {
            available: false,
            reason: Some("office closed".into()),
        };
    }
    for (start, end) in exceptions {
        if date_in_range(date, start, end) {
            return ProviderAvailabilityResult {
                available: false,
                reason: Some("exception".into()),
            };
        }
    }
    match availability_for_day {
        None => ProviderAvailabilityResult {
            available: false,
            reason: Some("no availability".into()),
        },
        Some((avail_start, avail_end)) => {
            // Time must be >= window start and strictly < window end (SS1d)
            if time >= avail_start && time < avail_end {
                ProviderAvailabilityResult { available: true, reason: None }
            } else {
                ProviderAvailabilityResult {
                    available: false,
                    reason: Some("no availability".into()),
                }
            }
        }
    }
}

/// Input record for compute_office_schedule.
pub struct OfficeProviderData {
    pub provider_id: String,
    pub provider_name: String,
    pub provider_archived: bool,
    pub exceptions: Vec<(String, String)>,          // (start_date, end_date)
    pub availability_for_day: Option<(String, String)>, // (start_time, end_time) for the queried day-of-week
}

/// Pure function: build schedule entries for an office on a given date.
/// Excludes archived providers and those with an active exception on the date.
/// Only includes providers that have an availability window for the day-of-week.
pub fn compute_office_schedule(
    providers: &[OfficeProviderData],
    date: &str,
) -> Vec<ProviderScheduleEntry> {
    let mut entries = Vec::new();
    for p in providers {
        if p.provider_archived {
            continue;
        }
        if p.exceptions.iter().any(|(s, e)| date_in_range(date, s, e)) {
            continue;
        }
        if let Some((start, end)) = &p.availability_for_day {
            entries.push(ProviderScheduleEntry {
                provider_id: p.provider_id.clone(),
                provider_name: p.provider_name.clone(),
                start_time: start.clone(),
                end_time: end.clone(),
            });
        }
    }
    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    fn avail(
        provider_archived: bool,
        assigned_to_office: bool,
        office_archived: bool,
        office_has_hours: bool,
        exceptions: &[(String, String)],
        avail_for_day: Option<(&str, &str)>,
        date: &str,
        time: &str,
    ) -> ProviderAvailabilityResult {
        compute_provider_availability(
            provider_archived,
            assigned_to_office,
            office_archived,
            office_has_hours,
            exceptions,
            avail_for_day,
            date,
            time,
        )
    }

    fn make_provider(
        name: &str,
        archived: bool,
        exceptions: Vec<(&str, &str)>,
        avail: Option<(&str, &str)>,
    ) -> OfficeProviderData {
        OfficeProviderData {
            provider_id: format!("pid-{name}"),
            provider_name: name.to_string(),
            provider_archived: archived,
            exceptions: exceptions.into_iter().map(|(s, e)| (s.to_string(), e.to_string())).collect(),
            availability_for_day: avail.map(|(s, e)| (s.to_string(), e.to_string())),
        }
    }

    // ── weekday_of ──────────────────────────────────────────────────────────

    #[test]
    fn test_weekday_monday() {
        assert_eq!(weekday_of("2026-12-21"), Some("Monday".to_string()));
    }

    #[test]
    fn test_weekday_friday() {
        assert_eq!(weekday_of("2026-12-25"), Some("Friday".to_string()));
    }

    #[test]
    fn test_weekday_invalid() {
        assert_eq!(weekday_of("not-a-date"), None);
    }

    // ── SS1: weekly availability window ─────────────────────────────────────

    #[test] // SS1a: available within window
    fn test_available_within_window() {
        let r = avail(false, true, false, true, &[], Some(("09:00", "17:00")), "2026-12-21", "10:00");
        assert!(r.available);
        assert!(r.reason.is_none());
    }

    #[test] // SS1b: no window for day
    fn test_no_window_for_day() {
        let r = avail(false, true, false, true, &[], None, "2026-12-21", "10:00");
        assert!(!r.available);
        assert_eq!(r.reason.as_deref(), Some("no availability"));
    }

    #[test] // SS1c: at start boundary → available
    fn test_available_at_start_boundary() {
        let r = avail(false, true, false, true, &[], Some(("09:00", "17:00")), "2026-12-21", "09:00");
        assert!(r.available);
    }

    #[test] // SS1d: at end boundary → not available
    fn test_not_available_at_end_boundary() {
        let r = avail(false, true, false, true, &[], Some(("09:00", "17:00")), "2026-12-21", "17:00");
        assert!(!r.available);
        assert_eq!(r.reason.as_deref(), Some("no availability"));
    }

    // ── SS2: exceptions ──────────────────────────────────────────────────────

    #[test] // SS2a: exception blocks date
    fn test_exception_blocks_date() {
        let exc = vec![("2026-12-20".to_string(), "2026-12-31".to_string())];
        let r = avail(false, true, false, true, &exc, Some(("09:00", "17:00")), "2026-12-22", "10:00");
        assert!(!r.available);
        assert_eq!(r.reason.as_deref(), Some("exception"));
    }

    #[test] // SS2b: day before exception → available
    fn test_available_before_exception() {
        let exc = vec![("2026-12-20".to_string(), "2026-12-31".to_string())];
        let r = avail(false, true, false, true, &exc, Some(("09:00", "17:00")), "2026-12-19", "10:00");
        assert!(r.available);
    }

    #[test] // SS2c: day after exception → available
    fn test_available_after_exception() {
        let exc = vec![("2026-12-20".to_string(), "2026-12-31".to_string())];
        let r = avail(false, true, false, true, &exc, Some(("09:00", "17:00")), "2027-01-02", "10:00");
        assert!(r.available);
    }

    #[test] // SS2d: single-day exception
    fn test_single_day_exception() {
        let exc = vec![("2026-12-28".to_string(), "2026-12-28".to_string())];
        let r = avail(false, true, false, true, &exc, Some(("09:00", "17:00")), "2026-12-28", "10:00");
        assert!(!r.available);
        assert_eq!(r.reason.as_deref(), Some("exception"));
    }

    // ── SS3: office assignment ───────────────────────────────────────────────

    #[test] // SS3a: not assigned to office
    fn test_not_assigned() {
        let r = avail(false, false, false, true, &[], Some(("09:00", "17:00")), "2026-12-21", "10:00");
        assert!(!r.available);
        assert_eq!(r.reason.as_deref(), Some("not assigned"));
    }

    // ── SS4: office hours ────────────────────────────────────────────────────

    #[test] // SS4a/b: office has no hours for day
    fn test_office_no_hours_for_day() {
        let r = avail(false, true, false, false, &[], Some(("09:00", "17:00")), "2026-12-21", "10:00");
        assert!(!r.available);
        assert_eq!(r.reason.as_deref(), Some("office closed"));
    }

    #[test] // SS4d: office archived
    fn test_office_archived() {
        let r = avail(false, true, true, true, &[], Some(("09:00", "17:00")), "2026-12-21", "10:00");
        assert!(!r.available);
        assert_eq!(r.reason.as_deref(), Some("office closed"));
    }

    // ── SS5: reason codes and priority ───────────────────────────────────────

    #[test] // SS5a: available → null reason
    fn test_available_null_reason() {
        let r = avail(false, true, false, true, &[], Some(("09:00", "17:00")), "2026-12-21", "10:00");
        assert!(r.available);
        assert!(r.reason.is_none());
    }

    #[test] // SS5f: provider archived
    fn test_provider_archived() {
        let r = avail(true, true, false, true, &[], Some(("09:00", "17:00")), "2026-12-21", "10:00");
        assert!(!r.available);
        assert_eq!(r.reason.as_deref(), Some("provider archived"));
    }

    #[test] // SS5g: provider archived takes priority over exception
    fn test_provider_archived_priority_over_exception() {
        let exc = vec![("2026-12-21".to_string(), "2026-12-21".to_string())];
        let r = avail(true, true, false, true, &exc, Some(("09:00", "17:00")), "2026-12-21", "10:00");
        assert!(!r.available);
        assert_eq!(r.reason.as_deref(), Some("provider archived"));
    }

    #[test] // SS5g: provider archived takes priority over not assigned
    fn test_provider_archived_priority_over_not_assigned() {
        let r = avail(true, false, false, true, &[], None, "2026-12-21", "10:00");
        assert!(!r.available);
        assert_eq!(r.reason.as_deref(), Some("provider archived"));
    }

    // ── SS6: office schedule ─────────────────────────────────────────────────

    #[test] // SS6a: all providers available
    fn test_schedule_all_available() {
        let providers = vec![
            make_provider("Dr. A", false, vec![], Some(("09:00", "17:00"))),
            make_provider("Dr. B", false, vec![], Some(("09:00", "17:00"))),
        ];
        let entries = compute_office_schedule(&providers, "2026-12-21");
        assert_eq!(entries.len(), 2);
    }

    #[test] // SS6b: exception excludes one provider
    fn test_schedule_exception_excluded() {
        let providers = vec![
            make_provider("Dr. A", false, vec![], Some(("09:00", "17:00"))),
            make_provider("Dr. B", false, vec![("2026-12-21", "2026-12-21")], Some(("09:00", "17:00"))),
        ];
        let entries = compute_office_schedule(&providers, "2026-12-21");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].provider_name, "Dr. A");
    }

    #[test] // SS6c: archived provider excluded
    fn test_schedule_archived_excluded() {
        let providers = vec![
            make_provider("Dr. A", false, vec![], Some(("09:00", "17:00"))),
            make_provider("Dr. B", true, vec![], Some(("09:00", "17:00"))),
        ];
        let entries = compute_office_schedule(&providers, "2026-12-21");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].provider_name, "Dr. A");
    }

    #[test] // SS6d: no providers → empty
    fn test_schedule_no_providers() {
        let entries = compute_office_schedule(&[], "2026-12-21");
        assert!(entries.is_empty());
    }

    #[test] // SS6e: all providers have exceptions → empty
    fn test_schedule_all_exceptions_empty() {
        let providers = vec![
            make_provider("Dr. A", false, vec![("2026-12-21", "2026-12-21")], Some(("09:00", "17:00"))),
            make_provider("Dr. B", false, vec![("2026-12-21", "2026-12-21")], Some(("09:00", "17:00"))),
        ];
        let entries = compute_office_schedule(&providers, "2026-12-21");
        assert!(entries.is_empty());
    }

    #[test] // SS6f: entry reflects specific day's hours
    fn test_schedule_reflects_hours() {
        let providers = vec![
            make_provider("Dr. A", false, vec![], Some(("09:00", "13:00"))),
        ];
        let entries = compute_office_schedule(&providers, "2026-12-21");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].start_time, "09:00");
        assert_eq!(entries[0].end_time, "13:00");
    }

    #[test] // provider with no availability for day is excluded
    fn test_schedule_no_avail_for_day_excluded() {
        let providers = vec![
            make_provider("Dr. A", false, vec![], None), // no avail for Monday
        ];
        let entries = compute_office_schedule(&providers, "2026-12-21");
        assert!(entries.is_empty());
    }
}
