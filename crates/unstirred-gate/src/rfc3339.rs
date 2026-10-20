#![doc = "The one timestamp a gate puts on the wire: a challenge's expiry, as RFC 3339 in UTC \
(PRD section 3.3). The store counts in unix seconds, so this is the conversion, written out \
rather than taken from a date library the rest of the build has no use for."]

const SECONDS_PER_DAY: i64 = 86_400;

/// `1970-01-01T00:00:00Z` for 0, and civil UTC for every other second the store can hold.
pub fn from_unix_seconds(seconds: i64) -> String {
    let days = seconds.div_euclid(SECONDS_PER_DAY);
    let within_day = seconds.rem_euclid(SECONDS_PER_DAY);
    let (year, month, day) = civil_from_days(days);
    let (hour, minute, second) = (within_day / 3600, (within_day % 3600) / 60, within_day % 60);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

/// Howard Hinnant's `civil_from_days`: days since the unix epoch to a civil year, month and day
/// in the proleptic Gregorian calendar.
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era = (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    (if month <= 2 { year + 1 } else { year }, month, day)
}

#[cfg(test)]
mod tests {
    use super::from_unix_seconds;

    #[test]
    fn known_instants_render_as_rfc3339_in_utc() {
        assert_eq!(from_unix_seconds(0), "1970-01-01T00:00:00Z");
        assert_eq!(from_unix_seconds(1), "1970-01-01T00:00:01Z");
        assert_eq!(from_unix_seconds(1_700_000_000), "2023-11-14T22:13:20Z");
        assert_eq!(from_unix_seconds(1_767_225_600), "2026-01-01T00:00:00Z");
        assert_eq!(from_unix_seconds(951_782_400), "2000-02-29T00:00:00Z");
        assert_eq!(from_unix_seconds(-1), "1969-12-31T23:59:59Z");
    }

    #[test]
    fn every_day_of_four_centuries_round_trips_through_the_calendar() {
        let mut expected = (1970, 1, 1);
        for day in 0..146_097 {
            let rendered = from_unix_seconds(day * 86_400);
            let (year, month, this_day) = expected;
            assert_eq!(rendered, format!("{year:04}-{month:02}-{this_day:02}T00:00:00Z"));
            expected = next_day(expected);
        }
    }

    fn next_day((year, month, day): (i64, i64, i64)) -> (i64, i64, i64) {
        if day < days_in(year, month) {
            (year, month, day + 1)
        } else if month < 12 {
            (year, month + 1, 1)
        } else {
            (year + 1, 1, 1)
        }
    }

    fn days_in(year: i64, month: i64) -> i64 {
        match month {
            2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
            2 => 28,
            4 | 6 | 9 | 11 => 30,
            _ => 31,
        }
    }
}
