//! `--press Calls` presses where those words are drawn, as a person
//! would: once the window is laid out, in the order they are given, the
//! last place they are drawn (in the modal, where one is open). Words
//! that are not drawn are passed over. With it a picture of the
//! documents shows a tab or a fold as it is when pressed; `--wheel`
//! then turns over the place pressed last.

use appkit::pui::core::{PEvent, PMark, PPoint, PSheet};
use appkit::pui::window::PStage;

use super::given;

/// What the stage draws now; drawn, its parts know how long they are.
fn drawn(stage: &PStage) -> Vec<PMark> {
    let theme = stage.theme().clone();
    let mut sheet = PSheet::new(theme.as_ref());
    stage.draw(&mut sheet);
    sheet.mark
}

pub(super) fn press(stage: &mut PStage, word: &[String]) -> Option<PPoint> {
    let mut last = None;
    for wanted in given(word, "--press") {
        let at = drawn(stage).into_iter().rev().find_map(|mark| match mark {
            PMark::Text { at, text, .. } if text == wanted => Some(at.plus(PPoint::new(2.0, 2.0))),
            _ => None,
        });
        at.into_iter().for_each(|at| _ = stage.send(&PEvent::PointerDown(at)));
        last = at.or(last);
    }
    // What a press opened is drawn before the wheel turns over it, as a
    // window draws it before a person can: a tab longer than the one
    // before scrolls as far as its own end.
    if last.is_some() {
        drawn(stage);
    }
    last
}
