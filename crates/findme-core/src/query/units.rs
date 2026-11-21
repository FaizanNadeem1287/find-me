//! Size units and relative date parsers for query operators

use crate::record::FileFlags;

/// Parses a size filter value string (e.g. `>10mb`, `<500kb`, `100kb..5mb`, `empty`)
/// returning `(min_bytes, max_bytes)`
pub fn parse_size_filter(s: &str) -> Option<(Option<u64>, Option<u64>)> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    if s.eq_ignore_ascii_case("empty") {
        return Some((Some(0), Some(0)));
    }

    // Range: min..max
    if let Some((left, right)) = s.split_once("..") {
        let min = if left.is_empty() {
            None
        } else {
            Some(parse_size_bytes(left)?)
        };
        let max = if right.is_empty() {
            None
        } else {
            Some(parse_size_bytes(right)?)
        };
        return Some((min, max));
    }

    // Comparison operators
    if let Some(rest) = s.strip_prefix(">=") {
        let val = parse_size_bytes(rest)?;
        Some((Some(val), None))
    } else if let Some(rest) = s.strip_prefix('>') {
        let val = parse_size_bytes(rest)?;
        Some((val.checked_add(1), None))
    } else if let Some(rest) = s.strip_prefix("<=") {
        let val = parse_size_bytes(rest)?;
        Some((None, Some(val)))
    } else if let Some(rest) = s.strip_prefix('<') {
        let val = parse_size_bytes(rest)?;
        let max = if val > 0 { val - 1 } else { 0 };
        Some((None, Some(max)))
    } else if let Some(rest) = s.strip_prefix('=') {
        let val = parse_size_bytes(rest)?;
        Some((Some(val), Some(val)))
    } else {
        // Exact size
        let val = parse_size_bytes(s)?;
        Some((Some(val), Some(val)))
    }
}

/// Converts a size string with optional binary unit into exact byte count
pub fn parse_size_bytes(s: &str) -> Option<u64> {
    let s = s.trim().to_ascii_lowercase();
    if s.is_empty() {
        return None;
    }

    if s == "empty" {
        return Some(0);
    }

    let split_pos = s.find(|c: char| c.is_alphabetic()).unwrap_or(s.len());
    let (num_part, unit_part) = s.split_at(split_pos);
    let num: f64 = num_part.trim().parse().ok()?;

    let multiplier: f64 = match unit_part.trim() {
        "tb" | "t" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        "gb" | "g" => 1024.0 * 1024.0 * 1024.0,
        "mb" | "m" => 1024.0 * 1024.0,
        "kb" | "k" => 1024.0,
        "b" | "bytes" | "" => 1.0,
        _ => return None,
    };

    Some((num * multiplier) as u64)
}

/// Parses a date modified filter string relative to `now_secs` epoch timestamp
/// returning `(min_epoch_secs, max_epoch_secs)`
pub fn parse_date_filter(s: &str, now_secs: u64) -> Option<(u64, u64)> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    let s_lower = s.to_ascii_lowercase();
    const SECS_PER_DAY: u64 = 86400;

    // Start of current day (UTC-aligned calculation)
    let current_day_start = (now_secs / SECS_PER_DAY) * SECS_PER_DAY;

    match s_lower.as_str() {
        "today" => Some((
            current_day_start,
            now_secs.max(current_day_start + SECS_PER_DAY),
        )),
        "yesterday" => {
            let yest_start = current_day_start.saturating_sub(SECS_PER_DAY);
            Some((yest_start, current_day_start.saturating_sub(1)))
        }
        "lastweek" | "last7days" => {
            let week_start = now_secs.saturating_sub(7 * SECS_PER_DAY);
            Some((week_start, now_secs))
        }
        "thismonth" => {
            // Approximate 30 days window
            let month_start = now_secs.saturating_sub(30 * SECS_PER_DAY);
            Some((month_start, now_secs))
        }
        _ => {
            // Range: YYYY-MM-DD..YYYY-MM-DD
            if let Some((left, right)) = s.split_once("..") {
                let min = parse_iso_date(left)?.0;
                let max = parse_iso_date(right)?.1;
                return Some((min, max));
            }

            // Comparison: >YYYY-MM-DD or <YYYY-MM-DD
            if let Some(rest) = s.strip_prefix(">=") {
                let (start, _) = parse_iso_date(rest)?;
                Some((start, u64::MAX))
            } else if let Some(rest) = s.strip_prefix('>') {
                let (_, end) = parse_iso_date(rest)?;
                Some((end.saturating_add(1), u64::MAX))
            } else if let Some(rest) = s.strip_prefix("<=") {
                let (_, end) = parse_iso_date(rest)?;
                Some((0, end))
            } else if let Some(rest) = s.strip_prefix('<') {
                let (start, _) = parse_iso_date(rest)?;
                Some((0, start.saturating_sub(1)))
            } else {
                // Exact single date YYYY-MM-DD
                parse_iso_date(s)
            }
        }
    }
}

/// Parses an ISO date string `YYYY-MM-DD` into `(start_of_day_secs, end_of_day_secs)`
pub fn parse_iso_date(s: &str) -> Option<(u64, u64)> {
    let parts: Vec<&str> = s.trim().split('-').collect();
    if parts.len() != 3 {
        return None;
    }

    let year: i32 = parts[0].parse().ok()?;
    let month: u32 = parts[1].parse().ok()?;
    let day: u32 = parts[2].parse().ok()?;

    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }

    let days = ymd_to_days(year, month, day);
    let start_secs = (days as u64) * 86400;
    let end_secs = start_secs + 86399;
    Some((start_secs, end_secs))
}

/// Converts (year, month, day) to days since 1970-01-01
fn ymd_to_days(year: i32, month: u32, day: u32) -> i64 {
    let (y, m) = if month <= 2 {
        (year as i64 - 1, month as i64 + 9)
    } else {
        (year as i64, month as i64 - 3)
    };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * m + 2) / 5 + day as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

/// Parses attribute flag letters into `(required_flags, forbidden_flags)`
pub fn parse_attrib_filter(s: &str) -> (FileFlags, FileFlags) {
    let mut required = FileFlags::NONE;
    let mut forbidden = FileFlags::NONE;
    let mut negate = false;

    for c in s.chars() {
        match c {
            '!' | '-' => negate = true,
            '+' => negate = false,
            'd' | 'D' => {
                if negate {
                    forbidden |= FileFlags::IS_DIR;
                } else {
                    required |= FileFlags::IS_DIR;
                }
                negate = false;
            }
            'h' | 'H' => {
                if negate {
                    forbidden |= FileFlags::IS_HIDDEN;
                } else {
                    required |= FileFlags::IS_HIDDEN;
                }
                negate = false;
            }
            'x' | 'X' => {
                if negate {
                    forbidden |= FileFlags::IS_EXECUTABLE;
                } else {
                    required |= FileFlags::IS_EXECUTABLE;
                }
                negate = false;
            }
            'l' | 'L' | 's' | 'S' => {
                if negate {
                    forbidden |= FileFlags::IS_SYMLINK;
                } else {
                    required |= FileFlags::IS_SYMLINK;
                }
                negate = false;
            }
            'r' | 'R' => {
                if negate {
                    forbidden |= FileFlags::IS_READONLY;
                } else {
                    required |= FileFlags::IS_READONLY;
                }
                negate = false;
            }
            _ => {}
        }
    }

    (required, forbidden)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_parsing() {
        assert_eq!(parse_size_bytes("empty"), Some(0));
        assert_eq!(parse_size_bytes("500"), Some(500));
        assert_eq!(parse_size_bytes("500b"), Some(500));
        assert_eq!(parse_size_bytes("1kb"), Some(1024));
        assert_eq!(parse_size_bytes("1.5mb"), Some(1572864));
        assert_eq!(parse_size_bytes("2gb"), Some(2147483648));
    }

    #[test]
    fn test_size_filter_operators() {
        assert_eq!(parse_size_filter("empty"), Some((Some(0), Some(0))));
        assert_eq!(parse_size_filter(">10mb"), Some((Some(10485761), None)));
        assert_eq!(parse_size_filter("<=500kb"), Some((None, Some(512000))));
        assert_eq!(
            parse_size_filter("100kb..5mb"),
            Some((Some(102400), Some(5242880)))
        );
    }

    #[test]
    fn test_date_filter_relative() {
        let now = 1763740000;
        assert!(parse_date_filter("today", now).is_some());
        assert!(parse_date_filter("yesterday", now).is_some());
        assert!(parse_date_filter("lastweek", now).is_some());
    }

    #[test]
    fn test_iso_date_parsing() {
        let (start, end) = parse_iso_date("2026-11-20").expect("iso date failed");
        assert_eq!(end - start, 86399);
    }

    #[test]
    fn test_attrib_filter() {
        let (req, forb) = parse_attrib_filter("d");
        assert!(req.is_dir());
        assert!(!forb.is_dir());

        let (req2, forb2) = parse_attrib_filter("!h");
        assert!(!req2.is_hidden());
        assert!(forb2.is_hidden());
    }
}
