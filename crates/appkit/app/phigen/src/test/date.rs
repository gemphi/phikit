use appkit::service::PTable;

use crate::{data, date};

#[test]
fn a_moment_falls_on_a_month_and_a_day() {
    assert_eq!(date::month_day(0), (1, 1));
    // 1970 was not a leap year: its sixtieth day is the first of March.
    assert_eq!(date::month_day(59 * 86_400), (3, 1));
    assert_eq!(date::month_day(1_700_000_000), (11, 14));
    // The twenty-ninth of February 2024, and the day after.
    assert_eq!([date::month_day(1_709_164_800), date::month_day(1_709_164_800 + 86_400)], [(2, 29), (3, 1)]);
    assert_eq!(date::month_day(1_735_689_599), (12, 31));
}

#[test]
fn a_chat_s_day_reads_today_or_its_month_and_day_and_nothing_when_it_is_not_known() {
    let text = PTable::read(data::TEXT);
    let now = 1_700_000_000;
    assert_eq!(date::label(now - 60, now, &text), "Today");
    assert_eq!(date::label(now - 20 * 86_400, now, &text), "Oct 25");
    assert_eq!(date::label(0, now, &text), "");
    // Every month has its word.
    assert!((1..=12).all(|month| !text.get(&format!("month.{month}")).is_empty()));
}
