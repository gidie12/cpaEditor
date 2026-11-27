//! CPA dates (xsd:dateTime): the description in Zulu time and local time below the date fields, and the current
//! moment in CPA format. Port of `helper_classes/cpaDates.py`.
//!
//! Python reads the date with `datetime.fromisoformat` (Python 3.11), which accepts much more than RFC 3339:
//! basic notation (`20261009T104432Z`), ISO week dates, any single character as separator, hours without minutes,
//! a comma for fractions and offsets with seconds. [`parse_iso_datetime`] follows the C implementation of CPython
//! 3.11 step by step, so the same texts are accepted and rejected.

use chrono::{DateTime, Datelike, Duration, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};

use crate::CPA_DATE_FORMAT;

/// Message for a date that `fromisoformat` does not accept.
pub const INVALID_DATE_MESSAGE: &str = "Not a valid date, expected for example 2026-10-09T10:44:32Z";
/// Message for a date without timezone.
pub const NO_TIMEZONE_MESSAGE: &str = "Date has no timezone: add Z for Zulu time (UTC)";

/// A moment in CPA format: `2026-10-09T10:44:32Z`.
pub fn format_cpa_date(moment: DateTime<Utc>) -> String {
    moment.format(CPA_DATE_FORMAT).to_string()
}

/// The current date and time (UTC) in CPA format.
pub fn now_cpa_date() -> String {
    format_cpa_date(Utc::now())
}

/// Describes a CPA date in Zulu time (UTC) and in local time, for example
/// `Zulu: 2026-10-09T10:44:32Z    Local: 2026-10-09 12:44:32 CEST (UTC+02:00)`.
///
/// `local` is the timezone for the local time; `None` takes the timezone set on this computer. A fixed offset is
/// named like Python's `datetime.timezone(timedelta(...))` names it (`UTC+02:00`, `UTC` for offset zero); the
/// computer's timezone gets its own abbreviation (`CEST`) on Unix.
///
/// Returns an empty string for an empty value, otherwise the reason why the date cannot be described.
pub fn describe_cpa_date(value: &str, local: Option<FixedOffset>) -> String {
    let value = value.trim();
    if value.is_empty() {
        return String::new();
    }
    let Some(parsed) = parse_iso_datetime(value) else {
        return INVALID_DATE_MESSAGE.to_string();
    };
    let Some(offset) = parsed.offset else {
        return NO_TIMEZONE_MESSAGE.to_string();
    };
    let Some(moment) = offset.from_local_datetime(&parsed.naive).single() else {
        return INVALID_DATE_MESSAGE.to_string();
    };
    let utc = moment.with_timezone(&Utc);
    // Python's datetime stops at year 1 and 9999; the conversion raises outside that range
    if !(1..=9999).contains(&utc.year()) {
        return INVALID_DATE_MESSAGE.to_string();
    }
    let (local_offset, name) = match local {
        Some(offset) => (offset, fixed_offset_name(offset)),
        None => system_local_zone(utc),
    };
    let local_time = utc.with_timezone(&local_offset);
    if !(1..=9999).contains(&local_time.year()) {
        return INVALID_DATE_MESSAGE.to_string();
    }
    let offset_text = python_offset(local_offset);
    format!(
        "Zulu: {}    Local: {} {} (UTC{}:{})",
        utc.format(CPA_DATE_FORMAT),
        local_time.format("%Y-%m-%d %H:%M:%S"),
        name,
        &offset_text[..3],
        &offset_text[3..]
    )
}

/// The result of [`parse_iso_datetime`]: the date and time as written plus the offset, if any.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IsoDateTime {
    pub naive: NaiveDateTime,
    pub offset: Option<FixedOffset>,
}

/// Parses a date like Python 3.11 `datetime.fromisoformat`; None where Python raises ValueError.
///
/// Offsets are kept to the second; sub-second parts of offsets are dropped (chrono has no such offsets).
pub fn parse_iso_datetime(text: &str) -> Option<IsoDateTime> {
    let bytes = text.as_bytes();
    let length = bytes.len();
    // CPython reads the C string, which ends with a NUL byte
    let at = |index: usize| -> u8 { bytes.get(index).copied().unwrap_or(0) };
    let separator = find_separator(bytes, &at)?;
    let date = parse_date(separator, &at)?;
    let mut time = (0u32, 0u32, 0u32, 0u32);
    let mut offset: Option<(i64, i64)> = None;
    if length > separator {
        let start = separator + 1;
        let (parsed_time, parsed_offset) = parse_time(&at, start, length)?;
        time = parsed_time;
        offset = parsed_offset;
    }
    let (hour, minute, second, microsecond) = time;
    let naive_time = NaiveTime::from_hms_micro_opt(hour, minute, second, microsecond)?;
    let naive = NaiveDateTime::new(date, naive_time);
    let offset = match offset {
        None => None,
        Some((seconds, microseconds)) => {
            let total = Duration::seconds(seconds) + Duration::microseconds(microseconds);
            if total.num_microseconds()?.abs() >= 86_400_000_000 {
                return None;
            }
            Some(FixedOffset::east_opt(total.num_seconds() as i32)?)
        }
    };
    Some(IsoDateTime { naive, offset })
}

fn is_digit(byte: u8) -> bool {
    byte.is_ascii_digit()
}

/// `_find_isoformat_datetime_separator`: where the date ends.
fn find_separator(bytes: &[u8], at: &dyn Fn(usize) -> u8) -> Option<usize> {
    let length = bytes.len();
    if length == 7 {
        return Some(7);
    }
    if at(4) == b'-' {
        if at(5) == b'W' {
            if length < 8 {
                return None;
            }
            if length > 8 && at(8) == b'-' {
                if length == 9 {
                    return None;
                }
                if length > 10 && is_digit(at(10)) {
                    return Some(8);
                }
                return Some(10);
            }
            return Some(8);
        }
        return Some(10);
    }
    if at(4) == b'W' {
        let mut index = 7;
        while index < length && is_digit(at(index)) {
            index += 1;
        }
        if index < 9 {
            return Some(index);
        }
        return Some(if index % 2 == 0 { 7 } else { 8 });
    }
    Some(8)
}

fn parse_digits(at: &dyn Fn(usize) -> u8, position: &mut usize, count: usize) -> Option<u32> {
    let mut value = 0u32;
    for _ in 0..count {
        let byte = at(*position);
        if !is_digit(byte) {
            return None;
        }
        value = value * 10 + u32::from(byte - b'0');
        *position += 1;
    }
    Some(value)
}

/// `parse_isoformat_date`: YYYY-MM-DD, YYYYMMDD and the ISO week dates.
fn parse_date(length: usize, at: &dyn Fn(usize) -> u8) -> Option<NaiveDate> {
    let mut p = 0usize;
    let year = parse_digits(at, &mut p, 4)?;
    let uses_separator = at(p) == b'-';
    if uses_separator {
        p += 1;
    }
    if at(p) == b'W' {
        p += 1;
        let week = parse_digits(at, &mut p, 2)?;
        let day = if p < length {
            if uses_separator {
                let separator = at(p);
                p += 1;
                if separator != b'-' {
                    return None;
                }
            }
            parse_digits(at, &mut p, 1)?
        } else {
            1
        };
        if year == 0 || !(1..=7).contains(&day) {
            return None;
        }
        let weekday = chrono::Weekday::try_from((day - 1) as u8).ok()?;
        return NaiveDate::from_isoywd_opt(year as i32, week, weekday);
    }
    let month = parse_digits(at, &mut p, 2)?;
    if uses_separator {
        let separator = at(p);
        p += 1;
        if separator != b'-' {
            return None;
        }
    }
    let day = parse_digits(at, &mut p, 2)?;
    if year == 0 {
        return None;
    }
    NaiveDate::from_ymd_opt(year as i32, month, day)
}

/// `parse_isoformat_time`: the time with an optional offset. Returns ((hour, minute, second, microsecond),
/// Some((offset seconds, offset microseconds)) when there is an offset).
type ParsedTime = ((u32, u32, u32, u32), Option<(i64, i64)>);

fn parse_time(at: &dyn Fn(usize) -> u8, start: usize, end: usize) -> Option<ParsedTime> {
    let mut tz_position = start;
    loop {
        let byte = at(tz_position);
        if byte == b'Z' || byte == b'+' || byte == b'-' {
            break;
        }
        tz_position += 1;
        if tz_position >= end {
            break;
        }
    }
    let (time, more) = parse_hh_mm_ss_ff(at, start, tz_position)?;
    if tz_position >= end {
        return if more { None } else { Some((time, None)) };
    }
    if at(tz_position) == b'Z' {
        return if at(tz_position + 1) != 0 { None } else { Some((time, Some((0, 0)))) };
    }
    let sign: i64 = if at(tz_position) == b'-' { -1 } else { 1 };
    let ((hours, minutes, seconds, microseconds), more) = parse_hh_mm_ss_ff(at, tz_position + 1, end)?;
    if more {
        return None;
    }
    let offset = sign * (i64::from(hours) * 3600 + i64::from(minutes) * 60 + i64::from(seconds));
    Some((time, Some((offset, sign * i64::from(microseconds)))))
}

/// `parse_hh_mm_ss_ff`: HH[:?MM[:?SS[{.,}ffffff]]]; the flag tells whether text follows.
fn parse_hh_mm_ss_ff(at: &dyn Fn(usize) -> u8, start: usize, end: usize) -> Option<((u32, u32, u32, u32), bool)> {
    let mut p = start;
    let mut values = [0u32; 3];
    let mut has_separator = true;
    for index in 0..3 {
        values[index] = parse_digits(at, &mut p, 2)?;
        let c = at(p);
        p += 1;
        if index == 0 {
            has_separator = c == b':';
        }
        if p >= end {
            return Some(((values[0], values[1], values[2], 0), c != 0));
        } else if has_separator && c == b':' {
            continue;
        } else if c == b'.' || c == b',' {
            break;
        } else if !has_separator {
            p -= 1;
        } else {
            return None;
        }
    }
    let remaining = end - p;
    let to_parse = remaining.min(6);
    let mut microsecond = parse_digits(at, &mut p, to_parse)?;
    if to_parse < 6 {
        microsecond *= 10u32.pow((6 - to_parse) as u32);
    }
    while is_digit(at(p)) {
        p += 1;
    }
    Some(((values[0], values[1], values[2], microsecond), at(p) != 0))
}

/// Python's `strftime('%z')`: +HHMM, with seconds when the offset has them.
fn python_offset(offset: FixedOffset) -> String {
    let total = offset.local_minus_utc();
    let sign = if total < 0 { '-' } else { '+' };
    let total = total.abs();
    let (hours, minutes, seconds) = (total / 3600, total / 60 % 60, total % 60);
    if seconds != 0 {
        format!("{sign}{hours:02}{minutes:02}{seconds:02}")
    } else {
        format!("{sign}{hours:02}{minutes:02}")
    }
}

/// The name Python gives `datetime.timezone(timedelta(...))`: `UTC` for zero, otherwise `UTC+02:00`.
pub fn fixed_offset_name(offset: FixedOffset) -> String {
    let total = offset.local_minus_utc();
    if total == 0 {
        return "UTC".to_string();
    }
    let sign = if total < 0 { '-' } else { '+' };
    let total = total.abs();
    let (hours, minutes, seconds) = (total / 3600, total / 60 % 60, total % 60);
    if seconds != 0 {
        format!("UTC{sign}{hours:02}:{minutes:02}:{seconds:02}")
    } else {
        format!("UTC{sign}{hours:02}:{minutes:02}")
    }
}

/// Offset and abbreviation of the computer's timezone at a moment, like Python's `astimezone()`, which reads them
/// from the C library (`localtime`).
#[cfg(unix)]
fn system_local_zone(moment: DateTime<Utc>) -> (FixedOffset, String) {
    use std::ffi::{c_char, c_int, c_long, CStr};

    #[repr(C)]
    struct Tm {
        tm_sec: c_int,
        tm_min: c_int,
        tm_hour: c_int,
        tm_mday: c_int,
        tm_mon: c_int,
        tm_year: c_int,
        tm_wday: c_int,
        tm_yday: c_int,
        tm_isdst: c_int,
        tm_gmtoff: c_long,
        tm_zone: *const c_char,
    }
    extern "C" {
        fn localtime_r(time: *const i64, result: *mut Tm) -> *mut Tm;
    }
    let timestamp: i64 = moment.timestamp();
    let mut tm = Tm {
        tm_sec: 0,
        tm_min: 0,
        tm_hour: 0,
        tm_mday: 0,
        tm_mon: 0,
        tm_year: 0,
        tm_wday: 0,
        tm_yday: 0,
        tm_isdst: 0,
        tm_gmtoff: 0,
        tm_zone: std::ptr::null(),
    };
    // SAFETY: localtime_r only writes into the struct we pass; tm_zone points to static storage of the C library
    let result = unsafe { localtime_r(&timestamp, &mut tm) };
    if result.is_null() {
        return fallback_local_zone(moment);
    }
    let Some(offset) = FixedOffset::east_opt(tm.tm_gmtoff as i32) else {
        return fallback_local_zone(moment);
    };
    let name = if tm.tm_zone.is_null() {
        fixed_offset_name(offset)
    } else {
        // SAFETY: a non-null tm_zone is a NUL-terminated string of the C library
        unsafe { CStr::from_ptr(tm.tm_zone) }.to_string_lossy().into_owned()
    };
    (offset, name)
}

#[cfg(not(unix))]
fn system_local_zone(moment: DateTime<Utc>) -> (FixedOffset, String) {
    fallback_local_zone(moment)
}

/// The offset of chrono's local timezone, named like a fixed offset.
fn fallback_local_zone(moment: DateTime<Utc>) -> (FixedOffset, String) {
    let offset = *moment.with_timezone(&chrono::Local).offset();
    (offset, fixed_offset_name(offset))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_cpa_date_is_zulu() {
        let moment = Utc.with_ymd_and_hms(2026, 10, 9, 10, 44, 32).unwrap();
        assert_eq!(format_cpa_date(moment), "2026-10-09T10:44:32Z");
    }

    #[test]
    fn parses_basic_and_week_notation() {
        let expected = NaiveDate::from_ymd_opt(2026, 10, 9).unwrap().and_hms_opt(10, 0, 0).unwrap();
        assert_eq!(parse_iso_datetime("2026-W41-5T10:00Z").unwrap().naive, expected);
        assert_eq!(parse_iso_datetime("20261009T10Z").unwrap().naive, expected);
        assert!(parse_iso_datetime("2026-10-09Z").is_none());
    }
}
