//! Timestamps without a date library: unix seconds, and short labels like
//! "just now", "5m", "3h", "Tue", "Oct 3" or "Oct 3, 2025".

pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// A short label for `then`, as seen at `now`. `offset` is the local UTC
/// offset in seconds, used for days and dates.
pub fn label(then: i64, now: i64, offset: i64) -> String {
    let ago = (now - then).max(0);
    if ago < 60 {
        return "just now".into();
    }
    if ago < 3600 {
        return format!("{}m", ago / 60);
    }
    let day = |t: i64| (t + offset).div_euclid(86_400);
    let days = day(now) - day(then);
    if days == 0 {
        return format!("{}h", ago / 3600);
    }
    if days == 1 {
        return "yesterday".into();
    }
    let (y, m, d) = civil(day(then));
    if days < 7 {
        // 1970-01-01 was a Thursday.
        const WEEK: [&str; 7] = ["Thu", "Fri", "Sat", "Sun", "Mon", "Tue", "Wed"];
        return WEEK[day(then).rem_euclid(7) as usize].into();
    }
    const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    let month = MONTHS[(m - 1) as usize];
    if civil(day(now)).0 == y { format!("{month} {d}") } else { format!("{month} {d}, {y}") }
}

/// UTC date and time as `2026-10-16T12:00:00Z`.
pub fn iso(t: i64) -> String {
    let (y, m, d) = civil(t.div_euclid(86_400));
    let s = t.rem_euclid(86_400);
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z", s / 3600, s / 60 % 60, s % 60)
}

/// Year, month (1-12) and day for a count of days since 1970-01-01
/// (Howard Hinnant's civil_from_days).
fn civil(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = yoe + era * 400 + i64::from(m <= 2);
    (y, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    // 2026-10-16 12:00:00 UTC, a Friday.
    const NOW: i64 = 1_792_152_000;

    #[test]
    fn civil_dates() {
        assert_eq!(civil(0), (1970, 1, 1));
        assert_eq!(civil(NOW / 86_400), (2026, 10, 16));
        assert_eq!(civil(-1), (1969, 12, 31));
    }

    #[test]
    fn iso_times() {
        assert_eq!(iso(NOW), "2026-10-16T12:00:00Z");
        assert_eq!(iso(0), "1970-01-01T00:00:00Z");
        assert_eq!(iso(NOW + 3661), "2026-10-16T13:01:01Z");
    }

    #[test]
    fn labels() {
        assert_eq!(label(NOW - 5, NOW, 0), "just now");
        assert_eq!(label(NOW - 300, NOW, 0), "5m");
        assert_eq!(label(NOW - 3 * 3600, NOW, 0), "3h");
        assert_eq!(label(NOW - 86_400, NOW, 0), "yesterday");
        assert_eq!(label(NOW - 3 * 86_400, NOW, 0), "Tue");
        assert_eq!(label(NOW - 13 * 86_400, NOW, 0), "Oct 3");
        assert_eq!(label(NOW - 400 * 86_400, NOW, 0), "Sep 11, 2025");
    }
}
