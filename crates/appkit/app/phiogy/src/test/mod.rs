//! The tests, apart from the code and in the same folders: `test/system/`
//! tests `system/`. This file holds what the others open things with.

#[cfg(not(target_family = "wasm"))]
mod account;
mod app;
mod beside;
mod board;
mod call;
mod comp;
#[cfg(not(target_family = "wasm"))]
mod exchange;
mod flow;
mod hosts;
mod left;
mod pace;
mod pill;
mod ring;
mod room;
mod shop;
mod part;
mod peer;
#[cfg(not(target_family = "wasm"))]
mod person;
#[cfg(not(target_family = "wasm"))]
mod phiora;
mod picture;
#[cfg(not(target_family = "wasm"))]
mod signin;
mod site;
mod system;
mod tab;
#[cfg(not(target_family = "wasm"))]
mod talk;
#[cfg(not(target_family = "wasm"))]
mod told;
mod tools;
mod viewer;
mod voice;

use std::rc::Rc;

use appkit::pui::PApp;
use appkit::pui::core::{PEvent, PMark, PMode, PPoint, PRect, PSheet, PSize};
use appkit::pui::window::PStage;
use appkit::service::PTable;
use appkit::theme::PLook;
use serde_json::Value;

use crate::board::Board;
use crate::comp::{Hold, Limit, Note, Post, Roster, Signal, Tell};
use crate::entity::Caller;
use crate::system::join;
use crate::{Phiogy, Site, data};

/// A window at a desk, and one in a hand.
pub(crate) const DESK: PSize = PSize { width: 1040.0, height: 720.0 };
pub(crate) const HAND: PSize = PSize { width: 390.0, height: 800.0 };

/// A guest of that number, as the line makes one.
pub(crate) fn guest(number: usize) -> Caller {
    Caller::new(&format!("guest_{number}"), "Guest", None)
}

/// A board with the limits Phiogy ships with.
pub(crate) fn board() -> Board {
    Board::new(Limit::own())
}

/// A board with those limits in place of the shipped ones.
pub(crate) fn bound(change: &str) -> Board {
    Board::new(Limit::read(&format!("{change}\n{}", data::LIMIT), data::NAME))
}

/// The guest of that number opens a page's line in a room.
pub(crate) fn opened(board: &mut Board, key: &str, peer: &str, number: usize) -> Hold {
    join::open(board, key, peer, &guest(number), 0).expect("the line opens").0
}

/// What a page posts: to be in the call or out of it.
pub(crate) fn joining(peer: &str, join: bool) -> Post {
    Post { peer: peer.to_string(), join: Some(join), send: Vec::new() }
}

/// What a page posts: signals for another.
pub(crate) fn sending(peer: &str, to: &str, body: &[Value]) -> Post {
    Post { peer: peer.to_string(), join: None, send: body.iter().map(|body| Signal { to: to.to_string(), body: body.clone() }).collect() }
}

/// A note that tells an opening who is in the call.
pub(crate) fn roster(to: Hold, list: &[(&str, usize)]) -> Note {
    Note { to, tell: Tell::Roster(listed(list)) }
}

/// A roster of those pages, each a guest's.
pub(crate) fn listed(list: &[(&str, usize)]) -> Roster {
    Roster { list: list.iter().map(|(peer, number)| (peer.to_string(), guest(*number))).collect() }
}

/// A small site, written as the web Phiogy writes its own: the files
/// of `test/site/`.
pub(crate) fn site() -> Site {
    let load = |name: &str| match name {
        "data/room.json" => Some(include_str!("site/room.json").to_string()),
        "data/host.json" => Some(include_str!("site/host.json").to_string()),
        "data/shop.json" => Some(include_str!("site/data/shop.json").to_string()),
        _ => None,
    };
    // The file of who is linked to whom is read from disk when asked:
    // it is beside where this site says it was read from.
    let from = concat!(env!("CARGO_MANIFEST_DIR"), "/src/test/site/data/phiogy.json").to_string();
    Site { from, ..Site::read(include_str!("site/phiogy.json"), &load, &PTable::read(data::SITE)) }
}

/// Where the line of a staged Phiogy says it is.
pub(crate) const AT: &str = "localhost:4310";

/// Phiogy on the small site, nothing kept, its line said to be open,
/// its stage laid out for a window of that size, and its look.
pub(crate) fn staged(size: PSize) -> (Phiogy, PStage, Rc<PLook>) {
    let look = Rc::new(Phiogy::look(""));
    let app = Phiogy::holding(Rc::clone(&look), site(), Phiogy::board(), Ok(AT.to_string()), None);
    let mut stage = app.stage();
    stage.lay(size, PMode::of_width(size.width));
    (app, stage, look)
}

/// The names of the parts shown, in order.
pub(crate) fn part(stage: &PStage) -> Vec<&str> {
    stage.slot().iter().map(|slot| slot.part.as_str()).collect()
}

/// A press where a line of text was drawn last.
pub(crate) fn press(stage: &mut PStage, look: &PLook, text: &str) -> bool {
    let at = written(stage, look, text).expect("the text is drawn");
    stage.send(&PEvent::PointerDown(at))
}

/// A press on the bar's icon of that name.
pub(crate) fn icon(stage: &mut PStage, look: &PLook, name: &str) -> bool {
    let at = pictured(stage, look, name).expect("on the bar");
    stage.send(&PEvent::PointerDown(at))
}

/// A press on the row of the bar's menu that reads so: the first such
/// line in the menu, whose rows are drawn before the part it holds.
pub(crate) fn row(stage: &mut PStage, look: &PLook, wanted: &str) -> bool {
    let menu = stage.frame().menu;
    let at = |mark: PMark| match mark {
        PMark::Text { at, text, .. } if text == wanted && menu.contains(at) => Some(at.plus(PPoint::new(2.0, 2.0))),
        _ => None,
    };
    let at = drawn(stage, look).into_iter().find_map(at).expect("a row of the menu");
    stage.send(&PEvent::PointerDown(at))
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

/// A place to press a line of text: a little inside where it was drawn
/// last, which is in the modal when one is open.
pub(crate) fn written(stage: &PStage, look: &PLook, wanted: &str) -> Option<PPoint> {
    let at = |mark: PMark| match mark {
        PMark::Text { at, text, .. } if text == wanted => Some(at.plus(PPoint::new(2.0, 2.0))),
        _ => None,
    };
    drawn(stage, look).into_iter().filter_map(at).next_back()
}

/// Whether a line of text stands on a ground of its own as a figure
/// does (appkit's `figure`): somewhere it is drawn, a light as high as
/// the theme makes a figure was laid just before it, around it.
pub(crate) fn grounded(stage: &PStage, look: &PLook, wanted: &str) -> bool {
    use appkit::theme::PTheme;
    let mark = drawn(stage, look);
    let high = look.size(appkit::pui::core::figure::HIGH).unwrap_or_default();
    mark.windows(2).any(|pair| matches!((&pair[0], &pair[1]), (PMark::Blend { rect, .. }, PMark::Text { at, text, .. }) if text == wanted && rect.size.height == high && at.y >= rect.origin.y && at.y <= rect.origin.y + rect.size.height))
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

/// Where a part is shown.
pub(crate) fn rect(stage: &PStage, part: &str) -> PRect {
    stage.slot().iter().find(|slot| slot.part == part).map(|slot| slot.rect).expect("the part is shown")
}
