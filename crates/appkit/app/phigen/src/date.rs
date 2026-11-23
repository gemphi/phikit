//! The day a chat began, as a person reads it: "Today", or a month and a
//! day. A moment is seconds since 1970, and days are counted in UTC:
//! there is no time zone here yet, so near midnight "Today" can be a day
//! out. The words are `data/text.tsv`'s.

use appkit::service::PTable;

/// How many seconds a day has.
const DAY: u64 = 86_400;

/// The day a moment falls on, counted from 1970.
pub fn day(moment: u64) -> u64 {
    moment / DAY
}

/// The month and the day of the month a moment falls on. The arithmetic
/// is the civil calendar's, from a count of days (Howard Hinnant's
/// `civil_from_days`).
pub fn month_day(moment: u64) -> (u32, u32) {
    let shifted = day(moment) as i64 + 719_468;
    let of_era = shifted.rem_euclid(146_097);
    let year_of_era = (of_era - of_era / 1_460 + of_era / 36_524 - of_era / 146_096) / 365;
    let of_year = of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * of_year + 2) / 153;
    let day = of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = match shifted_month < 10 {
        true => shifted_month + 3,
        false => shifted_month - 9,
    };
    (month as u32, day as u32)
}

/// When a chat began, as a person reads it at `now`. Nothing for a chat
/// whose beginning is not known.
pub fn label(when: u64, now: u64, text: &PTable) -> String {
    let (month, of_month) = month_day(when);
    match (when == 0, day(when) == day(now)) {
        (true, _) => String::new(),
        (false, true) => text.get("date.today").to_string(),
        (false, false) => format!("{} {of_month}", text.get(&format!("month.{month}"))),
    }
}
