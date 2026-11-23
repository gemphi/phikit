//! The tests, apart from the code and in the same folders: `test/part/`
//! tests `part/`. This file holds what the others open Phigen with.

mod app;
mod choice;
mod date;
mod fact;
mod flow;
mod history;
mod keep;
mod model;
mod part;
mod reply;

use std::rc::Rc;

use appkit::pui::PApp;
use appkit::pui::core::{PMark, PMode, PPoint, PRect, PSheet, PSize};
use appkit::pui::window::PStage;
use appkit::theme::PLook;

use crate::{Phigen, Reply, Source, Turn, Whole};

/// A window at a desk, and one in a hand.
pub(crate) const DESK: PSize = PSize { width: 1040.0, height: 720.0 };
pub(crate) const HAND: PSize = PSize { width: 390.0, height: 800.0 };

/// Phigen, its stage laid out for a window of that size, and its look.
pub(crate) fn staged(size: PSize) -> (Phigen, PStage, Rc<PLook>) {
    let look = Rc::new(Phigen::look(""));
    let app = Phigen::new(Rc::clone(&look));
    let mut stage = app.stage();
    stage.lay(size, PMode::of_width(size.width));
    (app, stage, look)
}

/// Somebody who answers under one of several names: what was said, after
/// the name that answers. Only a name it has can be chosen.
pub(crate) struct Named {
    name: String,
    known: Vec<String>,
}

impl Named {
    /// One who answers under the first of those names.
    pub(crate) fn of(known: &[&str]) -> Named {
        Named { name: known.first().copied().unwrap_or_default().to_string(), known: known.iter().map(|name| name.to_string()).collect() }
    }
}

impl Reply for Named {
    fn reply(&mut self, said: &str, _past: &[Turn]) -> Box<dyn Source> {
        Box::new(Whole(format!("{}: {said}", self.name)))
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn list(&mut self) -> Vec<String> {
        self.known.clone()
    }

    fn choose(&mut self, name: &str) -> bool {
        let known = self.known.iter().any(|known| known == name);
        if known {
            self.name = name.to_string();
        }
        known
    }
}

/// Phigen answered by one of those models, its stage laid out for a
/// window of that size, and its look.
pub(crate) fn voiced_by(known: &[&str], size: PSize) -> (Phigen, PStage, Rc<PLook>) {
    let look = Rc::new(Phigen::look(""));
    let app = Phigen::answered(Rc::clone(&look), Box::new(Named::of(known)));
    let mut stage = app.stage();
    stage.lay(size, PMode::of_width(size.width));
    (app, stage, look)
}

/// Phigen answered by `alpha`, with `beta` to choose instead.
pub(crate) fn voiced(size: PSize) -> (Phigen, PStage, Rc<PLook>) {
    voiced_by(&["alpha", "beta"], size)
}

/// A place to press a line of text: a little inside where it was drawn
/// last, which is in the modal when one is open.
pub(crate) fn written(stage: &PStage, look: &PLook, wanted: &str) -> Option<PPoint> {
    let at = |mark: PMark| match mark {
        PMark::Text { at, text, .. } if text == wanted => Some(at.plus(PPoint::new(2.0, 2.0))),
        _ => None,
    };
    drawn(stage, look).into_iter().filter_map(at).next_back()
}

/// The middle of the bar's icon of that name: the first one drawn, since
/// the bar is drawn before its menu and before the parts.
pub(crate) fn pictured(stage: &PStage, look: &PLook, wanted: &str) -> Option<PPoint> {
    let at = |mark: PMark| match mark {
        PMark::Icon { name, rect, .. } if name == wanted => Some(rect.center()),
        _ => None,
    };
    drawn(stage, look).into_iter().find_map(at)
}

/// A place to press the row of the bar's menu that reads so: the first
/// such line in the menu, whose rows are drawn before the chats in it.
pub(crate) fn listed(stage: &PStage, look: &PLook, wanted: &str) -> Option<PPoint> {
    let menu = stage.frame().menu;
    let at = |mark: PMark| match mark {
        PMark::Text { at, text, .. } if text == wanted && menu.contains(at) => Some(at.plus(PPoint::new(2.0, 2.0))),
        _ => None,
    };
    drawn(stage, look).into_iter().find_map(at)
}

/// Everything the stage draws.
pub(crate) fn drawn(stage: &PStage, look: &PLook) -> Vec<PMark> {
    let mut sheet = PSheet::new(look);
    stage.draw(&mut sheet);
    sheet.mark
}

/// Every line of text the stage draws, in order.
pub(crate) fn shown(stage: &PStage, look: &PLook) -> Vec<String> {
    let text = |mark: PMark| match mark {
        PMark::Text { text, .. } | PMark::Head { text, .. } => Some(text),
        _ => None,
    };
    drawn(stage, look).into_iter().filter_map(text).collect()
}

/// The rectangles that begin inside `within` and wear the light a given
/// way: fading up, as a surface does (`turn` 0), or across, as a label
/// does (90); and from their far side, as what sits at the right does
/// (`end`), or from their near one.
pub(crate) fn lit(mark: &[PMark], within: PRect, turn: f32, end: bool) -> Vec<PRect> {
    let rect = |mark: &PMark| match mark {
        PMark::Blend { rect, from, to, turn: way, .. } if *way == turn && (from.a < to.a) == end && within.contains(rect.origin) => Some(*rect),
        _ => None,
    };
    mark.iter().filter_map(rect).collect()
}

/// Where a part is shown.
pub(crate) fn rect(stage: &PStage, part: &str) -> PRect {
    stage.slot().iter().find(|slot| slot.part == part).map(|slot| slot.rect).expect("the part is shown")
}
