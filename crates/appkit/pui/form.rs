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
//! * `fill`: the groups share out the room the pane has left. Without it
//!   they stand one under another, and the form scrolls when they are
//!   more than the pane holds.
//!
//! A block whose list is empty is not shown. Blocks under no tabs stand
//! after the tabs, each on its own ground under its heading.

mod kind;
mod list;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

pub use kind::{PChoice, drawn};
pub use list::{PListed, PRows};

use crate::pui::block::{PBand, PTabMove};
use crate::pui::core::{PDraw, PEvent, PPaint, PPoint, PRect, PView};
use crate::pui::layout::{PDry, PGrouped, PScroll, SPACE, stack, tabbed};
use crate::pui::state::PState;
use crate::service::PTable;
use crate::theme::PTheme;

/// The theme's tokens for the room round a form, and its heading's line.
pub const GAP: &str = "gap";
pub const HEAD: &str = "head.line";

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
        PForm { frame: PRect::default(), block, source, text, theme, spot: RefCell::default(), motion: PTabMove::default(), scroll: Cell::default(), origin: Cell::default() }
    }

    fn of(&self, kind: &str) -> impl Iterator<Item = &Block> {
        self.block.iter().filter(move |b| b.kind == kind)
    }

    /// The kit's groups for some blocks, with what each keeps when pressed.
    fn grouped<'a>(&'a self, block: &[&'a Block]) -> (Vec<PGrouped<'a>>, Vec<(&'a Block, Vec<PChoice>)>) {
        let shown: Vec<(&Block, Vec<PChoice>, String)> = block.iter().map(|b| (*b, self.source.list(&b.list), self.source.get(&b.path))).filter(|(_, c, _)| !c.is_empty()).collect();
        let group = shown.iter().map(|(b, choice, now)| {
            let (kind, variant, choice, now) = (b.kind.clone(), b.variant.clone(), choice.clone(), now.clone());
            PGrouped { head: self.text.get(&b.head).to_string(), draw: Box::new(move |paint: &mut dyn PPaint, at: PPoint, wide: f32, right: bool| drawn(&kind, &variant, &choice, &now, paint, at, wide, right)) }
        }).collect();
        (group, shown.into_iter().map(|(b, c, _)| (b, c)).collect())
    }

    /// Draw the form from a place: its blocks, down to `foot` where it
    /// fills. Where it ends, and where each can be pressed.
    fn lay(&self, paint: &mut dyn PPaint, origin: PPoint, foot: f32) -> (f32, Vec<Spot>) {
        let gap = paint.size(GAP);
        let (left, wide) = (origin.x + gap, self.frame.size.width - gap * 2.0);
        let mut spot = Vec::new();
        let mut top = origin.y + gap;
        for block in self.of("head") {
            paint.head(PPoint::new(left, top), self.text.get(&block.head), paint.color(kind::INK));
            top += paint.size(HEAD) + gap / 2.0;
        }
        for block in self.of("tools") {
            let choice = self.source.list(&block.list);
            let icon: Vec<(&str, &str, bool)> = choice.iter().map(|c| (c.icon.as_str(), "", false)).collect();
            let rect = match crate::pui::layout::bare(paint) {
                true => icon.draw_end(&mut PDry(&*paint), PPoint::new(left + wide, top)),
                false => icon.draw_end(paint, PPoint::new(left + wide, top)),
            };
            spot.extend(rect.into_iter().zip(&choice).map(|(r, c)| (r, block.path.clone(), c.value.clone())));
            top += icon.high(paint) + gap / 2.0;
        }
        let free: Vec<&Block> = self.block.iter().filter(|b| b.under.is_empty() && !matches!(b.kind.as_str(), "head" | "tools" | "tabs" | "fill")).collect();
        let (mut own, own_shown) = self.grouped(&free);
        let space = paint.theme().size(SPACE).unwrap_or(gap / 2.0);
        let high = match own.is_empty() {
            true => -space,
            false => stack(&mut PDry(&*paint), PPoint::new(left, 0.0), wide, 0.0, true, &mut own).0,
        };
        let mut end = top - space;
        for tabs in self.of("tabs") {
            let (choice, now) = (self.source.list(&tabs.list), self.source.get(&tabs.path));
            let pill: Vec<(&str, &str, bool)> = choice.iter().map(|c| (c.icon.as_str(), c.label.as_str(), c.value == now)).collect();
            self.motion.to(choice.iter().position(|c| c.value == now).unwrap_or_default());
            let under: Vec<&Block> = self.block.iter().filter(|b| b.under == tabs.key).collect();
            let (mut group, shown) = self.grouped(&under);
            let (rect, pressed, bottom) = tabbed(paint, PPoint::new(left, top), wide, foot - high - space, pill.as_slice(), &self.motion, &mut group);
            spot.extend(rect.into_iter().zip(&choice).map(|(r, c)| (r, tabs.path.clone(), c.value.clone())));
            for (rect, (block, choice)) in pressed.into_iter().zip(&shown) {
                spot.extend(rect.into_iter().zip(choice).map(|(r, c)| (r, block.path.clone(), c.value.clone())));
            }
            end = bottom;
        }
        match own.is_empty() {
            true => (end + space, spot),
            false => {
                // Filling, they stand at the foot; else after what is over them.
                let at = match foot > 0.0 && self.of("tabs").next().is_some() {
                    true => foot - high,
                    false => end + space,
                };
                let (last, pressed) = stack(paint, PPoint::new(left, at), wide, foot.max(at + high), true, &mut own);
                for (rect, (block, choice)) in pressed.into_iter().zip(&own_shown) {
                    spot.extend(rect.into_iter().zip(choice).map(|(r, c)| (r, block.path.clone(), c.value.clone())));
                }
                (last, spot)
            }
        }
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
        let scroll = self.scroll.get();
        paint.clip(PRect { origin, size: self.frame.size });
        let (end, spot) = self.lay(paint, PPoint::new(origin.x, origin.y - scroll.at()), foot);
        paint.unclip();
        let mut scroll = scroll;
        scroll.fit(end + scroll.at() - origin.y + gap, self.frame.size.height);
        self.scroll.set(scroll);
        *self.spot.borrow_mut() = spot;
    }
}

impl PView for PForm {
    fn hand(&self) -> Vec<PRect> {
        self.spot.borrow().iter().map(|(rect, ..)| *rect).collect()
    }

    fn frame(&self) -> PRect {
        self.frame
    }

    fn set_frame(&mut self, frame: PRect) {
        self.frame = frame;
    }

    /// The tabs' light moves while it is on its way; what is live is
    /// drawn every frame.
    fn tick(&mut self) -> bool {
        self.motion.tick(self.theme.as_ref()) || self.source.live()
    }

    fn beat(&self) -> Option<u64> {
        self.source.beat()
    }

    /// A press keeps the value of what it pressed at its block's path;
    /// the wheel scrolls it.
    fn respond(&mut self, event: &PEvent) -> bool {
        match *event {
            PEvent::PointerDown(at) => {
                let place = at.plus(self.origin.get());
                let hit = self.spot.borrow().iter().find(|(rect, ..)| rect.contains(place)).map(|(_, path, value)| (path.clone(), value.clone()));
                hit.is_some_and(|(path, value)| {
                    self.source.set(&path, &value);
                    true
                })
            }
            PEvent::Scroll(_, by) => {
                let mut scroll = self.scroll.get();
                let any = scroll.turn(-by.y);
                self.scroll.set(scroll);
                any
            }
            _ => false,
        }
    }
}
