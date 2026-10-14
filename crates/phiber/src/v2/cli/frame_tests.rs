//! `BFrameSpec` against `parse_int_range_relative_clamp_n`'s own rules.
//!
//! The scene in every case runs 10 ..= 50, so a relative value is
//! distinguishable from an absolute one.

use super::*;

const SCENE: BFrameLimit = BFrameLimit { first: 10, last: 50 };

fn list(text: &str) -> Result<Vec<i32>, BFrameError> {
    BFrameSpec::parse(text, SCENE).map(|spec| spec.list())
}

#[test]
fn a_plain_number_is_that_frame() {
    assert_eq!(list("12"), Ok(vec![12]));
    assert_eq!(list("0"), Ok(vec![0]));
    assert_eq!(list("1048574"), Ok(vec![MAXFRAME]));
}

#[test]
fn plus_counts_from_the_scenes_start() {
    assert_eq!(list("+0"), Ok(vec![10]), "the start itself");
    assert_eq!(list("+5"), Ok(vec![15]));
}

#[test]
fn minus_counts_back_from_the_end_and_is_off_by_one() {
    // `value = (neg - strtol(str + 1)) + 1`, so `-1` is the end itself and
    // not the frame before it. This is the rule most likely to be ported
    // wrong, which is why it has a case of its own.
    assert_eq!(list("-1"), Ok(vec![50]), "`-1` is the last frame");
    assert_eq!(list("-2"), Ok(vec![49]));
    assert_eq!(list("-0"), Ok(vec![51]), "and `-0` is one past it, as Blender's arithmetic gives");
}

#[test]
fn two_dots_make_an_inclusive_range() {
    assert_eq!(list("1..4"), Ok(vec![1, 2, 3, 4]));
    assert_eq!(list("7..7"), Ok(vec![7]));
    assert_eq!(list("+0..+2"), Ok(vec![10, 11, 12]), "both ends may be relative");
    assert_eq!(list("-3..-1"), Ok(vec![48, 49, 50]));
}

#[test]
fn a_reversed_range_renders_nothing_rather_than_failing() {
    // `if ((frame_range_arr[i][0] <= frame_range_arr[i][1]) == 0)` warns
    // and the loop then does not run: not an error, just no frames.
    let spec = BFrameSpec::parse("9..4", SCENE).expect("parses");
    assert!(spec.span[0].reversed(), "the span knows it is backwards");
    assert_eq!(spec.list(), Vec::<i32>::new());
}

#[test]
fn commas_list_items_and_each_may_be_a_range() {
    assert_eq!(list("1,5,9"), Ok(vec![1, 5, 9]));
    assert_eq!(list("1..3,10"), Ok(vec![1, 2, 3, 10]));
    assert_eq!(list("1..2,+0,-1"), Ok(vec![1, 2, 10, 50]), "all three forms together");
    assert_eq!(list("3,3"), Ok(vec![3, 3]), "a repeat is rendered twice, as Blender does");
}

#[test]
fn a_value_is_clamped_to_the_frame_limit_not_rejected() {
    // `CLAMP` after the relative form resolves, so a wild number is the
    // first or last frame rather than an error.
    assert_eq!(list("-99999999"), Ok(vec![MINAFRAME]));
    assert_eq!(list("99999999"), Ok(vec![MAXFRAME]));
}

#[test]
fn an_empty_item_is_incorrect_comma_use() {
    for text in [",", ",1", "1,", "1,,2", ""] {
        assert_eq!(list(text), Err(BFrameError::IncorrectCommaUse), "{text:?}");
    }
    assert_eq!(BFrameError::IncorrectCommaUse.message(), "incorrect comma use");
}

#[test]
fn trailing_or_missing_digits_are_not_a_number() {
    for text in ["x", "1x", "1 2", "+", "-", "1..", "..2", "1...2", "0x10"] {
        assert_eq!(list(text), Err(BFrameError::NotANumber), "{text:?}");
    }
    assert_eq!(BFrameError::NotANumber.message(), "not a number");
}

#[test]
fn a_number_too_big_for_an_int_exceeds_range() {
    // Distinct from the frame limit, which clamps: this is `strtol`
    // overflowing, which Blender reports rather than clamps.
    assert_eq!(list("99999999999999999999"), Err(BFrameError::ExceedsRange));
    assert_eq!(BFrameError::ExceedsRange.message(), "exceeds range");
}

#[test]
fn a_single_frame_is_stored_as_a_range_with_matching_ends() {
    // "single values are evaluated as a range with matching start/end".
    let spec = BFrameSpec::parse("12", SCENE).expect("parses");
    assert_eq!(spec.span, vec![BFrameSpan { first: 12, last: 12 }]);
    assert!(!spec.span[0].reversed());
}
