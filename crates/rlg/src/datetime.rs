// datetime.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Minimal ISO 8601 timestamp generation and validation.
//!
//! Uses only `std::time` — no external date crates. Replaces the historical
//! `dtt` dependency (which transitively pulled the unmaintained `paste`
//! crate, RUSTSEC-2024-0436).

use crate::error::{RlgError, RlgResult};
use std::time::{SystemTime, UNIX_EPOCH};

/// Return the current UTC timestamp in RFC 3339 / ISO 8601 form with
/// nanosecond precision: `YYYY-MM-DDTHH:MM:SS.fffffffffZ`.
///
/// Falls back to `1970-01-01T00:00:00.000000000Z` if the system clock
/// is set before `UNIX_EPOCH`.
#[must_use]
pub fn now_iso8601() -> String {
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format_epoch(dur.as_secs(), dur.subsec_nanos())
}

/// Validate `s` as an RFC 3339 / ISO 8601 timestamp.
///
/// Accepts the subset this crate emits: `YYYY-MM-DDTHH:MM:SS[.fff…]Z`
/// or `YYYY-MM-DDTHH:MM:SS[.fff…][+HH:MM|-HH:MM]`. Returns the original
/// input on success.
///
/// # Errors
/// Returns [`RlgError::DateTimeParseError`] when the string does not
/// match the supported grammar or contains out-of-range components.
pub fn parse_iso8601(s: &str) -> RlgResult<String> {
    validate(s).map(|()| s.to_string()).map_err(|why| {
        RlgError::DateTimeParseError(format!("{s:?}: {why}"))
    })
}

fn validate(s: &str) -> Result<(), &'static str> {
    let bytes = s.as_bytes();
    validate_date_time(bytes)?;
    // Optional fractional seconds, then mandatory zone designator.
    let zone = skip_fraction(bytes, 19)?;
    validate_zone(bytes, zone)
}

/// `YYYY-MM-DDTHH:MM:SS`, the fixed-width head every timestamp has.
fn validate_date_time(bytes: &[u8]) -> Result<(), &'static str> {
    if bytes.len() < 20 {
        return Err("too short");
    }
    if !is_ymd(&bytes[..10]) {
        return Err("invalid date");
    }
    if bytes[10] != b'T' {
        return Err("missing 'T' separator");
    }
    if !is_hms(&bytes[11..19]) {
        return Err("invalid time");
    }
    Ok(())
}

/// The index after `.digits` at `i`, or `i` when there is no fraction.
fn skip_fraction(
    bytes: &[u8],
    i: usize,
) -> Result<usize, &'static str> {
    if bytes.get(i) != Some(&b'.') {
        return Ok(i);
    }
    let digits = bytes[i + 1..]
        .iter()
        .take_while(|b| b.is_ascii_digit())
        .count();
    if digits == 0 {
        return Err("empty fractional seconds");
    }
    Ok(i + 1 + digits)
}

/// `Z` or `±HH:MM` at `i`, ending the string.
fn validate_zone(bytes: &[u8], i: usize) -> Result<(), &'static str> {
    match bytes.get(i) {
        Some(&b'Z') if i + 1 == bytes.len() => Ok(()),
        Some(&b'+' | &b'-') if bytes.len() - i == 6 => {
            if is_offset(&bytes[i + 1..]) {
                Ok(())
            } else {
                Err("invalid timezone offset")
            }
        }
        _ => Err("missing timezone designator"),
    }
}

fn is_ymd(b: &[u8]) -> bool {
    b.len() == 10
        && b[..4].iter().all(u8::is_ascii_digit)
        && b[4] == b'-'
        && b[5..7].iter().all(u8::is_ascii_digit)
        && b[7] == b'-'
        && b[8..10].iter().all(u8::is_ascii_digit)
        && in_range(&b[5..7], 1, 12)
        && in_range(&b[8..10], 1, 31)
}

fn is_hms(b: &[u8]) -> bool {
    b.len() == 8
        && b[..2].iter().all(u8::is_ascii_digit)
        && b[2] == b':'
        && b[3..5].iter().all(u8::is_ascii_digit)
        && b[5] == b':'
        && b[6..8].iter().all(u8::is_ascii_digit)
        && in_range(&b[..2], 0, 23)
        && in_range(&b[3..5], 0, 59)
        && in_range(&b[6..8], 0, 60) // leap second tolerated
}

fn is_offset(b: &[u8]) -> bool {
    b.len() == 5
        && b[..2].iter().all(u8::is_ascii_digit)
        && b[2] == b':'
        && b[3..5].iter().all(u8::is_ascii_digit)
        && in_range(&b[..2], 0, 23)
        && in_range(&b[3..5], 0, 59)
}

fn in_range(b: &[u8], lo: u32, hi: u32) -> bool {
    std::str::from_utf8(b)
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .is_some_and(|n| (lo..=hi).contains(&n))
}

/// Convert `seconds` since `UNIX_EPOCH` (+ `nanos`) into an RFC 3339 string.
///
/// Uses Howard Hinnant's `civil_from_days` algorithm — branch-free, no
/// allocation beyond the final `String`.
fn format_epoch(seconds: u64, nanos: u32) -> String {
    let days = i64::try_from(seconds / 86_400).unwrap_or(0);
    let (year, month, day) = civil_from_days(days);
    let sod = seconds % 86_400;
    // Four-digit years and in-range nanoseconds fill a fixed layout;
    // anything else keeps the general formatter's output.
    let Ok(year) = u32::try_from(year) else {
        return format_epoch_general(seconds, nanos);
    };
    if year > 9999 || nanos > 999_999_999 {
        return format_epoch_general(seconds, nanos);
    }
    let mut buf = *b"0000-00-00T00:00:00.000000000Z";
    put_digits(&mut buf[0..4], year);
    put_digits(&mut buf[5..7], month);
    put_digits(&mut buf[8..10], day);
    put_digits(&mut buf[11..13], (sod / 3600) as u32);
    put_digits(&mut buf[14..16], ((sod % 3600) / 60) as u32);
    put_digits(&mut buf[17..19], (sod % 60) as u32);
    put_digits(&mut buf[20..29], nanos);
    String::from_utf8_lossy(&buf).into_owned()
}

/// Write `n` into `dst` as zero-padded decimal, filling it right to left.
fn put_digits(dst: &mut [u8], mut n: u32) {
    for byte in dst.iter_mut().rev() {
        *byte = b'0' + (n % 10) as u8;
        n /= 10;
    }
}

/// The general form, for years past 9999.
fn format_epoch_general(seconds: u64, nanos: u32) -> String {
    let days = i64::try_from(seconds / 86_400).unwrap_or(0);
    let sod = seconds % 86_400;
    let hour = (sod / 3600) as u32;
    let minute = ((sod % 3600) / 60) as u32;
    let second = (sod % 60) as u32;
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{nanos:09}Z"
    )
}

/// Days since 1970-01-01 (UTC) → (year, month [1..=12], day [1..=31]).
///
/// Algorithm from <http://howardhinnant.github.io/date_algorithms.html>.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
const fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year_offset: i64 = if m <= 2 { 1 } else { 0 };
    ((y + year_offset) as i32, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_epoch_known_instants() {
        let cases = [
            (0, 0, "1970-01-01T00:00:00.000000000Z"),
            (951_782_400, 5, "2000-02-29T00:00:00.000000005Z"),
            (
                1_727_740_800,
                123_456_789,
                "2024-10-01T00:00:00.123456789Z",
            ),
            (
                253_402_300_799,
                999_999_999,
                "9999-12-31T23:59:59.999999999Z",
            ),
            // The first five-digit year takes the general path.
            (253_402_300_800, 0, "10000-01-01T00:00:00.000000000Z"),
        ];
        for (secs, nanos, want) in cases {
            assert_eq!(format_epoch(secs, nanos), want, "{secs}");
        }
    }

    #[test]
    fn now_iso8601_shape() {
        let s = now_iso8601();
        assert_eq!(s.len(), 30, "{s}"); // YYYY-MM-DDTHH:MM:SS.fffffffffZ
        assert!(s.ends_with('Z'));
        assert_eq!(&s[4..5], "-");
        assert_eq!(&s[10..11], "T");
        assert!(parse_iso8601(&s).is_ok(), "roundtrip failed for {s}");
    }

    #[test]
    fn parses_canonical_utc() {
        assert!(parse_iso8601("2024-08-29T12:00:00Z").is_ok());
        assert!(
            parse_iso8601("1970-01-01T00:00:00.000000000Z").is_ok()
        );
        assert!(parse_iso8601("2099-12-31T23:59:59.123Z").is_ok());
    }

    #[test]
    fn parses_offset_zones() {
        assert!(parse_iso8601("2024-08-29T12:00:00+02:00").is_ok());
        assert!(parse_iso8601("2024-08-29T12:00:00.5-05:30").is_ok());
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse_iso8601("").is_err());
        assert!(parse_iso8601("not a date").is_err());
        assert!(parse_iso8601("2024-13-29T12:00:00Z").is_err()); // bad month
        assert!(parse_iso8601("2024-08-32T12:00:00Z").is_err()); // bad day
        assert!(parse_iso8601("2024-08-29T24:00:00Z").is_err()); // bad hour
        assert!(parse_iso8601("2024-08-29T12:00:00").is_err()); // missing zone
        assert!(parse_iso8601("2024-08-29 12:00:00Z").is_err()); // missing T
    }

    #[test]
    fn known_epoch_values() {
        assert_eq!(
            format_epoch(0, 0),
            "1970-01-01T00:00:00.000000000Z"
        );
        // 1700000000 = 2023-11-14T22:13:20Z
        assert_eq!(
            format_epoch(1_700_000_000, 0),
            "2023-11-14T22:13:20.000000000Z"
        );
    }

    #[test]
    fn rejects_dangling_fractional_dot() {
        // Hits the `empty fractional seconds` branch.
        let r = parse_iso8601("2024-08-29T12:00:00.Z");
        let msg = r.unwrap_err().to_string();
        assert!(msg.contains("empty fractional seconds"), "got: {msg}");
    }

    #[test]
    fn rejects_invalid_timezone_offset() {
        // Hits the `invalid timezone offset` branch (right length, bad digits).
        let r = parse_iso8601("2024-08-29T12:00:00+99:99");
        let msg = r.unwrap_err().to_string();
        assert!(msg.contains("invalid timezone offset"), "got: {msg}");
    }

    #[test]
    fn rejects_missing_timezone_designator() {
        // Hits the `_` (catch-all) arm in the match — neither Z nor +/-.
        let r = parse_iso8601("2024-08-29T12:00:00X");
        let msg = r.unwrap_err().to_string();
        assert!(
            msg.contains("missing timezone designator"),
            "got: {msg}"
        );
    }

    #[test]
    fn civil_from_days_handles_pre_epoch() {
        // Exercises the negative-`z` branch in civil_from_days.
        // The algorithm anchors at days = -719_468 → 0000-03-01, so
        // -719_469 lands on 0000-02-29 (year 0 is a leap year in
        // proleptic Gregorian). What matters here is hitting the
        // `z < 0` arm in `civil_from_days`, not the exact value.
        let (y, m, d) = civil_from_days(-719_469);
        assert_eq!((y, m, d), (0, 2, 29));

        // Also exercise a deeply negative input so era != 0.
        let (y2, _, _) = civil_from_days(-1_000_000);
        assert!(y2 < 0);
    }
}
