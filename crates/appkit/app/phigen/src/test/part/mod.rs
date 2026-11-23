//! Tests of `part`, one file for each part that does something. They
//! work through the stage, as a hand does: a press, a key.

mod ask;
mod find;
mod pick;
mod roster;
mod setting;
mod talk;
mod thread;

use appkit::pui::core::{PMark, PPaint, PRect, PSheet, PSize};

use crate::Phigen;
use crate::part::fit;
use crate::test::{DESK, HAND, drawn, lit, rect, staged};

/// A desk with a screen far wider than the column.
const WIDE: PSize = PSize { width: 1920.0, height: 1080.0 };

/// The left and the right edge of what those rectangles cover together.
fn across(rect: &[PRect]) -> (f32, f32) {
    let left = rect.iter().map(|rect| rect.origin.x).fold(f32::MAX, f32::min);
    (left, rect.iter().map(|rect| rect.origin.x + rect.size.width).fold(f32::MIN, f32::max))
}

#[test]
fn everything_in_the_body_keeps_to_one_column_down_its_middle() {
    for size in [DESK, WIDE, HAND] {
        let (app, mut stage, look) = staged(size);
        let (mark, ask, card, hello) = (drawn(&stage, &look), rect(&stage, "ask"), rect(&stage, "card"), rect(&stage, "hello"));
        // The column is in the middle of the body, 744 wide where there
        // is room and a gap from the sides where there is not.
        let (start, wide) = app.desk().column(ask.size.width);
        let edge = (ask.origin.x + start, ask.origin.x + start + wide);
        assert_eq!((edge.0 + edge.1) / 2.0, stage.frame().body.center().x);
        assert_eq!(wide, 744.0_f32.min(ask.size.width - 32.0));
        // The field begins at its left and the button ends at its right;
        // the cards go from one edge to the other.
        // The field wears the surface, the button the label.
        let field = lit(&mark, ask, 0.0, false);
        let button = lit(&mark, ask, 90.0, false);
        // Thirds of a width do not add up to it to the last bit.
        let near = |got: (f32, f32)| (got.0 - edge.0).abs() < 0.01 && (got.1 - edge.1).abs() < 0.01;
        assert!(near((across(&field).0, across(&button).1)), "{size:?}");
        assert!(near(across(&lit(&mark, card, 0.0, false))), "{size:?}");
        // The heading over the field is in the middle too.
        let sheet = PSheet::new(look.as_ref());
        let head =mark.iter().find_map(|mark| match mark {
            PMark::Head { at, text, .. } => Some(at.x + sheet.measure_head(text) / 2.0),
            _ => None,
        });
        assert!(head.is_some_and(|middle| (middle - hello.center().x).abs() < 0.01), "{head:?}");
        // Once something is said, the turns and the field under them
        // keep the same edges.
        Phigen::key(&mut stage, "hello");
        let (mark, talk, ask) = (drawn(&stage, &look), rect(&stage, "talk"), rect(&stage, "ask"));
        let turn = [lit(&mark, talk, 90.0, false), lit(&mark, talk, 90.0, true)].concat();
        assert!(near(across(&turn)), "{size:?}");
        assert!(near((across(&lit(&mark, ask, 0.0, false)).0, across(&lit(&mark, ask, 90.0, false)).1)));
    }
}

#[test]
fn every_card_s_words_begin_at_the_same_height_and_a_long_line_takes_two() {
    let (_, stage, look) = staged(DESK);
    let at = |wanted: &str| {
        drawn(&stage, &look).into_iter().find_map(|mark| match mark {
            PMark::Text { at, text, .. } if text == wanted => Some(at),
            _ => None,
        })
    };
    let title = ["Ask a question", "See what Phigen can do", "Think something through"].map(|title| at(title).map(|at| at.y));
    assert!(title[0].is_some() && title.iter().all(|top| *top == title[0]), "{title:?}");
    // The first card's line is too long for its card: it is set in two,
    // the second a line under the first and starting where it does.
    let (first, second) = (at("Type anything and take it").expect("drawn"), at("as far as you want.").expect("drawn"));
    assert_eq!((second.x, second.y), (first.x, first.y + 22.0));
}

#[test]
fn a_text_cut_short_ends_with_the_theme_s_mark() {
    let look = Phigen::look("");
    let sheet = PSheet::new(&look);
    // A letter is 7.5 wide where nothing measures it: seven fit in 52.5.
    assert_eq!(fit("one two", 52.5, &sheet), "one two");
    assert_eq!(fit("one two three", 52.5, &sheet), "one...");
    // A look that has no such mark cuts the text and adds nothing.
    let bare = Phigen::look("word\ttext.cut\t\n");
    assert_eq!(fit("one two three", 52.5, &PSheet::new(&bare)), "one two");
}

#[test]
fn in_a_hand_the_cards_are_stacked_and_none_runs_past_its_part() {
    let (_, stage, look) = staged(HAND);
    let part = rect(&stage, "card");
    let card = lit(&drawn(&stage, &look), part, 0.0, false);
    assert_eq!(card.len(), 3);
    // One under the other, a gap apart, the last ending a gap from the
    // foot of the part or sooner.
    assert!(card.windows(2).all(|pair| pair[1].origin.x == pair[0].origin.x && (pair[1].origin.y - pair[0].origin.y - pair[0].size.height - 16.0).abs() < 0.01));
    let foot = card[2].origin.y + card[2].size.height;
    assert!(foot <= part.origin.y + part.size.height - 16.0 + 0.01, "{foot} in {part:?}");
}
