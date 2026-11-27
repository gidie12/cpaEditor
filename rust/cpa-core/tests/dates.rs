//! TestCpaDates (TestDescribeCpaDate and the logic of TestDateLabels) ported to Rust.
//!
//! Python used ZoneInfo('Europe/Amsterdam') and expected the names CEST and CET; a FixedOffset has no such names,
//! so the offsets of Amsterdam on those dates are used and named like Python's datetime.timezone: UTC+02:00.

use chrono::{FixedOffset, NaiveDateTime};
use cpa_core::dates::{describe_cpa_date, now_cpa_date};
use cpa_core::CPA_DATE_FORMAT;

fn summer() -> Option<FixedOffset> {
    FixedOffset::east_opt(2 * 3600)
}

fn winter() -> Option<FixedOffset> {
    FixedOffset::east_opt(3600)
}

#[test]
fn describe_cpa_date_zulu_and_local() {
    assert_eq!(
        describe_cpa_date("2026-10-09T10:44:32Z", summer()),
        "Zulu: 2026-10-09T10:44:32Z    Local: 2026-10-09 12:44:32 UTC+02:00 (UTC+02:00)"
    );
    assert_eq!(
        describe_cpa_date("2026-12-31T23:30:00Z", winter()),
        "Zulu: 2026-12-31T23:30:00Z    Local: 2027-01-01 00:30:00 UTC+01:00 (UTC+01:00)"
    );
    assert_eq!(
        describe_cpa_date("2026-10-09T10:44:32Z", FixedOffset::east_opt(0)),
        "Zulu: 2026-10-09T10:44:32Z    Local: 2026-10-09 10:44:32 UTC (UTC+00:00)"
    );
}

#[test]
fn describe_cpa_date_with_offset() {
    assert_eq!(
        describe_cpa_date("2026-10-09T12:44:32+02:00", summer()),
        "Zulu: 2026-10-09T10:44:32Z    Local: 2026-10-09 12:44:32 UTC+02:00 (UTC+02:00)"
    );
}

#[test]
fn describe_cpa_date_without_timezone() {
    assert_eq!(describe_cpa_date("2026-10-09T10:44:32", summer()), "Date has no timezone: add Z for Zulu time (UTC)");
}

#[test]
fn describe_cpa_date_invalid() {
    assert!(describe_cpa_date("09-10-2026", summer()).contains("Not a valid date"));
}

#[test]
fn describe_cpa_date_empty() {
    assert_eq!(describe_cpa_date("", summer()), "");
    assert_eq!(describe_cpa_date("  ", summer()), "");
}

#[test]
fn update_date_labels_shows_zulu_and_local() {
    // The computer's own timezone (None)
    assert!(describe_cpa_date("2026-10-09T10:44:32Z", None).starts_with("Zulu: 2026-10-09T10:44:32Z    Local: "));
    assert!(describe_cpa_date("tomorrow", None).contains("Not a valid date"));
}

#[test]
fn set_start_date_to_now_updates_label() {
    let now = now_cpa_date();
    assert!(NaiveDateTime::parse_from_str(&now, CPA_DATE_FORMAT).is_ok(), "{now}");
    assert!(now.ends_with('Z') && now.len() == 20);
    assert!(describe_cpa_date(&now, None).contains(&now));
}
