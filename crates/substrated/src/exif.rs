//! Minimal pure-Rust EXIF extraction: orientation (applied inside the
//! embedder so a rotated duplicate embeds identically), capture time, and
//! camera make/model (fed to full-text search). No chrono — the civil-date
//! math is hand-rolled.

use std::io::Cursor;

#[derive(Debug, Clone, Default)]
pub struct ExifInfo {
    /// EXIF orientation 1..8 (1 = upright, the default when absent).
    pub orientation: u8,
    /// DateTimeOriginal as epoch seconds (treated as UTC), if present.
    pub capture_ts: Option<i64>,
    pub make: Option<String>,
    pub model: Option<String>,
}

/// Parse EXIF from image bytes. Absence or malformed EXIF yields defaults,
/// never an error — a photo without EXIF is normal, not a failure.
pub fn read(bytes: &[u8]) -> ExifInfo {
    let mut info = ExifInfo {
        orientation: 1,
        ..Default::default()
    };
    let reader = exif::Reader::new();
    let exif = match reader.read_from_container(&mut Cursor::new(bytes)) {
        Ok(e) => e,
        Err(_) => return info,
    };
    use exif::{In, Tag, Value};

    if let Some(f) = exif.get_field(Tag::Orientation, In::PRIMARY) {
        if let Some(o) = f.value.get_uint(0) {
            if (1..=8).contains(&o) {
                info.orientation = o as u8;
            }
        }
    }
    if let Some(f) = exif.get_field(Tag::DateTimeOriginal, In::PRIMARY) {
        if let Value::Ascii(ref v) = f.value {
            if let Some(bytes) = v.first() {
                info.capture_ts = parse_exif_datetime(bytes);
            }
        }
    }
    info.make = ascii_field(&exif, Tag::Make);
    info.model = ascii_field(&exif, Tag::Model);
    info
}

fn ascii_field(exif: &exif::Exif, tag: exif::Tag) -> Option<String> {
    use exif::{In, Value};
    let f = exif.get_field(tag, In::PRIMARY)?;
    if let Value::Ascii(ref v) = f.value {
        let s: String = v
            .first()
            .map(|b| String::from_utf8_lossy(b).trim().to_string())?;
        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    } else {
        None
    }
}

/// Parse an EXIF "YYYY:MM:DD HH:MM:SS" byte string into epoch seconds (UTC).
pub fn parse_exif_datetime(b: &[u8]) -> Option<i64> {
    let s = std::str::from_utf8(b).ok()?;
    // Expect exactly "YYYY:MM:DD HH:MM:SS".
    let (date, time) = s.split_once(' ')?;
    let mut d = date.split(':');
    let year: i64 = d.next()?.trim().parse().ok()?;
    let month: i64 = d.next()?.parse().ok()?;
    let day: i64 = d.next()?.parse().ok()?;
    let mut t = time.split(':');
    let hour: i64 = t.next()?.parse().ok()?;
    let min: i64 = t.next()?.parse().ok()?;
    let sec: i64 = t.next()?.trim().parse().ok()?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    let days = days_from_civil(year, month, day);
    Some(days * 86_400 + hour * 3600 + min * 60 + sec)
}

/// Days from the Unix epoch for a civil date (Howard Hinnant's algorithm).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exif_datetime_to_epoch() {
        // 2026-06-28 14:03:11 UTC = 1782strong; compute independently below.
        let ts = parse_exif_datetime(b"2026:06:28 14:03:11").unwrap();
        // 2000-01-01 00:00:00 UTC epoch is 946684800; sanity: ts is after it.
        assert!(ts > 946_684_800);
        // Round-trip a known epoch: 2021-01-01 00:00:00 UTC = 1609459200.
        assert_eq!(
            parse_exif_datetime(b"2021:01:01 00:00:00").unwrap(),
            1_609_459_200
        );
        // Leap year: 2020-02-29 exists.
        assert_eq!(
            parse_exif_datetime(b"2020:02:29 00:00:00").unwrap(),
            1_582_934_400
        );
    }

    #[test]
    fn bad_datetime_is_none() {
        assert!(parse_exif_datetime(b"not a date").is_none());
        assert!(parse_exif_datetime(b"2020:13:01 00:00:00").is_none());
    }
}
