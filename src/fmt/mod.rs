/// Format a process uptime given seconds since start.
///
/// Rules:
/// * < 60s    → "42s"
/// * < 1h     → "40m"
/// * < 1d     → "3h 20m"
/// * >= 1d    → "1d 9h"
pub fn uptime(secs: u64) -> String {
    const MINUTE: u64 = 60;
    const HOUR: u64 = 60 * MINUTE;
    const DAY: u64 = 24 * HOUR;

    if secs < MINUTE {
        format!("{secs}s")
    } else if secs < HOUR {
        format!("{}m", secs / MINUTE)
    } else if secs < DAY {
        format!("{}h {}m", secs / HOUR, (secs % HOUR) / MINUTE)
    } else {
        format!("{}d {}h", secs / DAY, (secs % DAY) / HOUR)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uptime_seconds() {
        assert_eq!(uptime(0), "0s");
        assert_eq!(uptime(42), "42s");
        assert_eq!(uptime(59), "59s");
    }

    #[test]
    fn test_uptime_minutes() {
        assert_eq!(uptime(60), "1m");
        assert_eq!(uptime(2400), "40m");
        assert_eq!(uptime(3599), "59m");
    }

    #[test]
    fn test_uptime_hours() {
        assert_eq!(uptime(3600), "1h 0m");
        assert_eq!(uptime(12000), "3h 20m");
        assert_eq!(uptime(86399), "23h 59m");
    }

    #[test]
    fn test_uptime_days() {
        assert_eq!(uptime(86400), "1d 0h");
        assert_eq!(uptime(118800), "1d 9h");
        assert_eq!(uptime(864000), "10d 0h");
    }
}
