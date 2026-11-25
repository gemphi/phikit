//! A form: a part made only of the kit's blocks, laid out by a table in
//! the app's data, its content and its values the app's, asked for by
//! name. No app draws a part of its own: it names its blocks and answers
//! what a list holds and what a path keeps.
//!
//! A row of the table is a block: the form it belongs to, its kind, its
//! key, its heading (a word of the app's text, `-` for none), the list
//! its content comes from, the path a press keeps its value at, the tabs
//! it stands under (`-` for none), and its variation (`-` for none).
//! Beside the kinds every block can be (`form/kind.rs`):
//!
//! * `head`: the form's heading, at its top.
//! * `tools`: icons at the top right, each keeping its value at the path.
//! * `tabs`: a choice of which, standing over the blocks under it.
//! * a block whose variation says `bar` stands in the form's toolbar
//!   under its heading (`form/bar.rs`): crumbs, tools, bands, a layout
//!   switch, on lines of their own with no heading or ground.
//! * `fill`: the groups share out the room the pane has left. Without it
//!   they stand one under another, and the form scrolls when they are
//!   more than the pane holds.
//! * a block whose variation says `side` stands in a column at the
//!   form's left, as a record's picture does on the web's pages, held in
//!   view while the rest scrolls beside it; in a form narrower than the
//!   theme's `form.split` the side stands at the top, under the bar.
//! * `head` with a list: its first choice's words are the heading, its
//!   line under them in the dim ink (a record's name and what it is).
//!
//! A block whose list is empty is not shown. Blocks under no tabs stand
//! after the tabs, each on its own ground under its heading.

mod bar;
mod drop;
mod field;
mod group;
mod head;
mod kind;
mod list;
mod tree;
mod view;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

pub use field::PTyped;
pub use head::{DROP, FOLD, FOLDED, LARGE, LAYOUT, LAYOUTS, LIST, PDrop, PHead, PTool};
pub use kind::{PChoice, PDo, drawn};
pub use list::{PListed, PRows};

use bar::side;
use crate::pui::block::{PBand, PTabMove};
use crate::pui::core::{PDraw, PPaint, PPoint, PRect};
use crate::pui::layout::{PDry, PScroll, SPACE, stack, tabbed};
use crate::pui::state::PState;
use crate::service::PTable;
use crate::theme::PTheme;

/// The theme's tokens for the room round a form, and its heading's line.
pub const GAP: &str = "gap";
pub const HEAD: &str = "head.line";
/// The theme's tokens for the side column: its share of the width, and
/// the width under which it stands at the top instead.
pub const SIDE: &str = "form.side";
/// The theme's token for how many tabs stand in their row at most.
pub const TABS: &str = "tabs.most";
pub const SPLIT: &str = "form.split";

/// Where a form's content and values come from: the app's lists by
/// name, and its state by path.
pub trait PSource: PState {
    /// The choices a list holds now.
    fn list(&self, name: &str) -> Vec<PChoice>;

    /// Whether what it shows changes by itself every frame (a call).
    fn live(&self) -> bool {
        false
    }

    /// What it watches that may change at any time, as a number that
    /// changes when it does; nothing for nothing.
    fn beat(&self) -> Option<u64> {
        None
    }
}

/// One block of a form, as its table gives it.
#[derive(Debug, Clone, PartialEq)]
struct Block {
    kind: String,
    key: String,
    head: String,
    list: String,
    path: String,
    under: String,
    variant: String,
}

/// What a press on a form keeps: a value at a path.
type Spot = (PRect, String, String);


pub struct PForm {
    frame: PRect,
    block: Vec<Block>,
    source: Rc<dyn PSource>,
    text: Rc<PTable>,
    theme: Rc<dyn PTheme>,
    spot: RefCell<Vec<Spot>>,
    motion: PTabMove,
    scroll: Cell<PScroll>,
    origin: Cell<PPoint>,
    /// The `field` block that has the keys, and what is typed in it.
    typing: RefCell<Option<String>>,
    draft: RefCell<String>,
    /// Where the pointer is, in the window: what is under it is lit.
    over: Cell<Option<PPoint>>,
}

impl PForm {
    /// The form of that name in a table, its lists and values from a
    /// source, its headings in the app's words, moving as its theme says.
    pub fn new(name: &str, table: &str, source: Rc<dyn PSource>, text: Rc<PTable>, theme: Rc<dyn PTheme>) -> PForm {
        let cell = |line: &str| line.split('\t').map(|c| match c {
            "-" => String::new(),
            other => other.to_string(),
        }).collect::<Vec<String>>();
        let block = table.lines().filter(|line| !line.starts_with('#') && !line.trim().is_empty()).map(cell).filter(|c| c.first().is_some_and(|f| f == name)).map(|c| {
            let at = |i: usize| c.get(i).cloned().unwrap_or_default();
            Block { kind: at(1), key: at(2), head: at(3), list: at(4), path: at(5), under: at(6), variant: at(7) }
        }).collect();
        PForm { frame: PRect::default(), block, source, text, theme, spot: RefCell::default(), motion: PTabMove::default(), scroll: Cell::default(), origin: Cell::default(), typing: RefCell::default(), draft: RefCell::default(), over: Cell::default() }
    }

    fn of(&self, kind: &str) -> impl Iterator<Item = &Block> {
        self.block.iter().filter(move |b| b.kind == kind)
    }

    /// A row of tabs: as many around the one on as fit across the width,
    /// up to the theme's `tabs.most`, an arrow at each end of the row
    /// where there are more, stepping to the tab before or after. Each
    /// pill with the value a press keeps.
    fn row<'a>(&self, paint: &dyn PPaint, choice: &'a [PChoice], now: &str, wide: f32) -> (Vec<(&'a str, &'a str, bool)>, Vec<String>) {
        let at = choice.iter().position(|c| c.value == now).unwrap_or_default();
        let most = paint.theme().size(TABS).unwrap_or(4.0).max(1.0) as usize;
        let value = |index: usize| choice.get(index).map(|c| c.value.clone()).unwrap_or_default();
        // With their icons where at least three (or all) fit so; else
        // their words alone, so as many stand as can.
        let build = |count: usize, icons: bool| {
            let start = at.saturating_sub(count / 2).min(choice.len().saturating_sub(count));
            let more = count < choice.len();
            let mut pill: Vec<(&'a str, &'a str, bool)> = Vec::new();
            let mut kept: Vec<String> = Vec::new();
            if more {
                pill.push(("chevron-left", "", false));
                kept.push(value(at.saturating_sub(1)));
            }
            for c in choice.iter().skip(start).take(count) {
                let icon = match icons {
                    true => c.icon.as_str(),
                    false => "",
                };
                pill.push((icon, c.label.as_str(), c.value == now));
                kept.push(c.value.clone());
            }
            if more {
                pill.push(("chevron-right", "", false));
                kept.push(value((at + 1).min(choice.len().saturating_sub(1))));
            }
            (pill, kept)
        };
        let fits = |count: usize, icons: bool| build(count, icons).0.as_slice().span(paint) <= wide;
        let count = |icons: bool| (1..=most.min(choice.len()).max(1)).rev().find(|count| fits(*count, icons)).unwrap_or(1);
        let least = choice.len().clamp(1, 3);
        match count(true) >= least {
            true => build(count(true), true),
            false => build(count(false), false),
        }
    }

    /// The block widened to the whole form, where one is.
    fn expanded(&self) -> Option<&Block> {
        self.block.iter().find(|b| b.variant.split_whitespace().any(|word| word == "layout") && self.source.get(&format!("{}{}", head::EXPAND, b.key)) == head::FOLDED)
    }

    /// The blocks of the side column.
    fn sided(&self) -> Vec<&Block> {
        self.block.iter().filter(|b| side(b)).collect()
    }

    /// Draw some blocks one under another from a place across a width,
    /// each on its own ground: where they end, and what a press keeps.
    fn column(&self, paint: &mut dyn PPaint, at: PPoint, wide: f32, block: &[&Block]) -> (f32, Vec<Spot>) {
        let (mut group, shown) = self.grouped(block);
        let (end, pressed) = stack(paint, at, wide, 0.0, true, &mut group);
        let spot = pressed.into_iter().zip(&shown).flat_map(|(rect, (block, choice, head))| PForm::spots(block, head, choice, rect)).collect();
        (end, spot)
    }

    /// Draw the form from a place across a width: its blocks, down to
    /// `foot` where it fills; the side's too unless it stands `apart`.
    /// Where it ends, and where each can be pressed.
    fn lay(&self, paint: &mut dyn PPaint, origin: PPoint, width: f32, foot: f32, apart: bool) -> (f32, Vec<Spot>) {
        let gap = paint.size(GAP);
        let (left, wide) = (origin.x + gap, width - gap * 2.0);
        let mut spot = Vec::new();
        let mut top = origin.y + gap;
        for block in self.of("head") {
            // A heading of a list: its first choice's words, its line under.
            let first = Some(&block.list).filter(|list| !list.is_empty()).and_then(|list| self.source.list(list).into_iter().next());
            let (said, line) = match &first {
                Some(first) => (first.label.as_str(), first.note.as_str()),
                None => (self.text.get(&block.head), ""),
            };
            paint.head(PPoint::new(left, top), said, paint.color(kind::INK));
            top += paint.size(HEAD) + gap / 2.0;
            if !line.is_empty() {
                let line = tree::fitted(line, wide, paint);
                paint.text(PPoint::new(left, top - gap / 4.0), &line, paint.color(kind::DIM));
                top += paint.size(kind::LINE) + gap / 4.0;
            }
        }
        let (below, barred) = self.bar(paint, left, top, wide);
        spot.extend(barred);
        top = below;
        // A block widened: it alone, under the heading and the bar.
        if let Some(block) = self.expanded() {
            let (mut group, shown) = self.grouped(&[block]);
            let (end, pressed) = stack(paint, PPoint::new(left, top), wide, 0.0, true, &mut group);
            for (rect, (block, choice, head)) in pressed.into_iter().zip(&shown) {
                spot.extend(PForm::spots(block, head, choice, rect));
            }
            let drop = self.dropped(paint, &self.heads(&spot, &shown));
            spot.splice(0..0, drop);
            spot.iter().filter(|(_, path, _)| !path.is_empty()).for_each(|(rect, _, _)| paint.press(*rect));
            return (end, spot);
        }
        // The side, where it stands at the top.
        let sided = self.sided();
        if !apart && !sided.is_empty() {
            let (end, pressed) = self.column(paint, PPoint::new(left, top), wide, &sided);
            spot.extend(pressed);
            top = end + paint.theme().size(SPACE).unwrap_or(gap / 2.0);
        }
        for block in self.of("tools").filter(|b| !bar::barred(b)) {
            let choice = self.source.list(&block.list);
            let icon: Vec<(&str, &str, bool)> = choice.iter().map(|c| (c.icon.as_str(), "", false)).collect();
            let rect = match crate::pui::layout::bare(paint) {
                true => icon.draw_end(&mut PDry(&*paint), PPoint::new(left + wide, top)),
                false => icon.draw_end(paint, PPoint::new(left + wide, top)),
            };
            spot.extend(rect.into_iter().zip(&choice).map(|(r, c)| (r, block.path.clone(), c.value.clone())));
            top += icon.high(paint) + gap / 2.0;
        }
        let free: Vec<&Block> = self.block.iter().filter(|b| b.under.is_empty() && !bar::barred(b) && !side(b) && !matches!(b.kind.as_str(), "head" | "tools" | "tabs" | "fill")).collect();
        let (mut own, own_shown) = self.grouped(&free);
        let space = paint.theme().size(SPACE).unwrap_or(gap / 2.0);
        let high = match own.is_empty() {
            true => -space,
            false => stack(&mut PDry(&*paint), PPoint::new(left, 0.0), wide, 0.0, true, &mut own).0,
        };
        let mut end = top - space;
        for tabs in self.of("tabs") {
            let (choice, now) = (self.source.list(&tabs.list), self.source.get(&tabs.path));
            // One row of tabs, centred, as many as fit up to the theme's
            // most; arrows at its ends step to the tab before and after.
            let (pill, value) = self.row(paint, &choice, &now, wide);
            self.motion.to(value.iter().zip(&pill).position(|(v, p)| *v == now && !p.1.is_empty()).unwrap_or_default());
            let under: Vec<&Block> = self.block.iter().filter(|b| b.under == tabs.key).collect();
            let (mut group, shown) = self.grouped(&under);
            let (rect, pressed, bottom) = tabbed(paint, PPoint::new(left, top), wide, foot - high - space, pill.as_slice(), &self.motion, &mut group);
            spot.extend(rect.into_iter().zip(value).map(|(r, value)| (r, tabs.path.clone(), value)));
            for (rect, (block, choice, head)) in pressed.into_iter().zip(&shown) {
                spot.extend(PForm::spots(block, head, choice, rect));
            }
            // A drop over the groups of these tabs.
            let drop = self.dropped(paint, &self.heads(&spot, &shown));
            spot.splice(0..0, drop);
            end = bottom;
        }
        let (last, spot) = match own.is_empty() {
            true => (end + space, spot),
            false => {
                // Filling, they stand at the foot; else after what is over them.
                let at = match foot > 0.0 && self.of("tabs").next().is_some() {
                    true => foot - high,
                    false => end + space,
                };
                let (last, pressed) = stack(paint, PPoint::new(left, at), wide, foot.max(at + high), true, &mut own);
                let mut spot = spot;
                for (rect, (block, choice, head)) in pressed.into_iter().zip(&own_shown) {
                    spot.extend(PForm::spots(block, head, choice, rect));
                }
                // The drop that is open is over everything, and pressed first.
                let drop = self.dropped(paint, &self.heads(&spot, &own_shown));
                spot.splice(0..0, drop);
                (last, spot)
            }
        };
        // Every choice that keeps a value is pressed for something.
        spot.iter().filter(|(_, path, _)| !path.is_empty()).for_each(|(rect, _, _)| paint.press(*rect));
        (last, spot)
    }
}

impl PDraw for PForm {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        let gap = paint.size(GAP);
        self.origin.set(origin);
        let fills = self.of("fill").next().is_some();
        let foot = match fills {
            true => origin.y + self.frame.size.height - gap,
            false => 0.0,
        };
        let (scroll, width, high) = (self.scroll.get(), self.frame.size.width, self.frame.size.height);
        // Wide, the side stands in a column at the left.
        let sided = self.sided();
        let column = match !sided.is_empty() && self.expanded().is_none() && width >= paint.theme().size(SPLIT).unwrap_or(f32::MAX) {
            true => (width * paint.theme().size(SIDE).unwrap_or(0.5)).floor(),
            false => 0.0,
        };
        paint.clip(PRect { origin, size: self.frame.size });
        let (mut spot, mut held) = (Vec::new(), 0.0);
        if column > 0.0 {
            // Held in view; one taller than the pane scrolls only until
            // its end is in sight.
            let left = origin.x + gap;
            let tall = self.column(&mut PDry(&*paint), PPoint::new(left, 0.0), column - gap, &sided).0 + gap * 2.0;
            let lift = scroll.at().min((tall - high).max(0.0));
            spot = self.column(paint, PPoint::new(left, origin.y + gap - lift), column - gap, &sided).1;
            held = tall;
        }
        let (end, rest) = self.lay(paint, PPoint::new(origin.x + column, origin.y - scroll.at()), width - column, foot, column > 0.0);
        // What the rest keeps is pressed first: its open drop is over all.
        spot.splice(0..0, rest);
        paint.unclip();
        let mut scroll = scroll;
        scroll.fit((end + scroll.at() - origin.y + gap).max(held), high);
        self.scroll.set(scroll);
        *self.spot.borrow_mut() = spot;
    }
}
