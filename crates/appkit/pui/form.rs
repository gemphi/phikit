//! A form: a part made only of the kit's blocks, laid out by a table in
//! the app's data, its content and its values the app's, asked for by
//! name. No app draws a form of its own: it names its blocks and answers
//! what a list holds and what a path keeps.
//!
//! A row of the table is a block: the form it belongs to, its kind, its
//! key, its heading (a word of the app's text, `-` for none), the list
//! its content comes from, the path its value is kept at, and the tabs it
//! stands under (`-` for none). The kinds:
//!
//! * `tools`: icons at the top right; a press keeps its choice's value
//!   at the path, as a deed (Reset, Randomize).
//! * `tabs`: a choice of which, standing over the blocks under it, on one
//!   ground (`layout::tabbed`), its light moving to the one chosen.
//! * `band`: one of its choices, as pills across the width.
//! * `swatch`: one of its choices, as colours.
//!
//! A block whose list is empty is not shown. The blocks under no tabs
//! stand at the foot, each on its own ground.

use std::cell::RefCell;
use std::rc::Rc;

use crate::pui::block::{PBand, PSwatch, PTabMove};
use crate::pui::core::{PColor, PDraw, PEvent, PPaint, PPoint, PRect, PView};
use crate::pui::layout::{PDry, PGrouped, SPACE, stack, tabbed};
use crate::pui::state::PState;
use crate::service::PTable;
use crate::theme::PTheme;

/// The theme's token for the room round a form.
pub const GAP: &str = "gap";

/// One choice of a list: what is kept when it is chosen, its words, its
/// icon, its colour (a swatch's).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PChoice {
    pub value: String,
    pub label: String,
    pub icon: String,
    pub colour: Option<PColor>,
}

/// Where a form's content and values come from: the app's lists by
/// name, and its state by path.
pub trait PSource: PState {
    /// The choices a list holds now.
    fn list(&self, name: &str) -> Vec<PChoice>;
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
            Block { kind: at(1), key: at(2), head: at(3), list: at(4), path: at(5), under: at(6) }
        }).collect();
        PForm { frame: PRect::default(), block, source, text, theme, spot: RefCell::default(), motion: PTabMove::default() }
    }

    fn of(&self, kind: &str) -> impl Iterator<Item = &Block> {
        self.block.iter().filter(move |b| b.kind == kind)
    }

    /// A block's choices, and which is chosen.
    fn choices(&self, block: &Block) -> (Vec<PChoice>, String) {
        (self.source.list(&block.list), self.source.get(&block.path))
    }

    /// The kit's groups for some blocks, with what each keeps when pressed.
    fn grouped<'a>(&'a self, block: &[&'a Block]) -> (Vec<PGrouped<'a>>, Vec<(&'a Block, Vec<PChoice>)>) {
        let shown: Vec<(&Block, Vec<PChoice>, String)> = block.iter().map(|b| (*b, self.choices(b))).map(|(b, (c, now))| (b, c, now)).filter(|(_, c, _)| !c.is_empty()).collect();
        let group = shown.iter().map(|(b, choice, now)| {
            let (kind, choice, now) = (b.kind.clone(), choice.clone(), now.clone());
            PGrouped { head: self.text.get(&b.head).to_string(), draw: Box::new(move |paint: &mut dyn PPaint, at: PPoint, wide: f32| drawn(&kind, &choice, &now, paint, at, wide)) }
        }).collect();
        (group, shown.into_iter().map(|(b, c, _)| (b, c)).collect())
    }
}

/// What a block draws: pills across the width, or colours.
fn drawn(kind: &str, choice: &[PChoice], now: &str, paint: &mut dyn PPaint, at: PPoint, wide: f32) -> (f32, Vec<PRect>) {
    match kind {
        "swatch" => {
            let colour: Vec<(PColor, bool)> = choice.iter().map(|c| (c.colour.unwrap_or_default(), c.value == now)).collect();
            let (rect, end) = colour.as_slice().draw_in(paint, at, wide);
            (end, rect)
        }
        _ => {
            let pill: Vec<(&str, &str, bool)> = choice.iter().map(|c| (c.icon.as_str(), c.label.as_str(), c.value == now)).collect();
            let rect = pill.draw_full(paint, at, wide);
            (at.y + pill.high(paint), rect)
        }
    }
}

impl PDraw for PForm {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        let gap = paint.size(GAP);
        let (left, wide, foot) = (origin.x + gap, self.frame.size.width - gap * 2.0, origin.y + self.frame.size.height - gap);
        let mut spot = Vec::new();
        // The tools at the top right.
        let mut top = origin.y + gap;
        for block in self.of("tools") {
            let (choice, _) = self.choices(block);
            let icon: Vec<(&str, &str, bool)> = choice.iter().map(|c| (c.icon.as_str(), "", false)).collect();
            let rect = icon.draw_end(paint, PPoint::new(left + wide, top));
            spot.extend(rect.into_iter().zip(&choice).map(|(r, c)| (r, block.path.clone(), c.value.clone())));
            top += icon.high(paint) + gap / 2.0;
        }
        // The blocks under no tabs, at the foot; the tabs over theirs,
        // down to them.
        let free: Vec<&Block> = self.block.iter().filter(|b| b.under.is_empty() && matches!(b.kind.as_str(), "band" | "swatch")).collect();
        let (mut own, own_shown) = self.grouped(&free);
        let space = paint.theme().size(SPACE).unwrap_or(gap / 2.0);
        let high = match own.is_empty() {
            true => -space,
            false => stack(&mut PDry(&*paint), PPoint::new(left, 0.0), wide, 0.0, &mut own).0,
        };
        for tabs in self.of("tabs") {
            let (choice, now) = self.choices(tabs);
            let pill: Vec<(&str, &str, bool)> = choice.iter().map(|c| (c.icon.as_str(), c.label.as_str(), c.value == now)).collect();
            self.motion.to(choice.iter().position(|c| c.value == now).unwrap_or_default());
            let under: Vec<&Block> = self.block.iter().filter(|b| b.under == tabs.key).collect();
            let (mut group, shown) = self.grouped(&under);
            let (rect, pressed) = tabbed(paint, PPoint::new(left, top), wide, foot - high - space, pill.as_slice(), &self.motion, &mut group);
            spot.extend(rect.into_iter().zip(&choice).map(|(r, c)| (r, tabs.path.clone(), c.value.clone())));
            for (rect, (block, choice)) in pressed.into_iter().zip(&shown) {
                spot.extend(rect.into_iter().zip(choice).map(|(r, c)| (r, block.path.clone(), c.value.clone())));
            }
        }
        if !own.is_empty() {
            let (_, pressed) = stack(paint, PPoint::new(left, foot - high), wide, foot, &mut own);
            for (rect, (block, choice)) in pressed.into_iter().zip(&own_shown) {
                spot.extend(rect.into_iter().zip(choice).map(|(r, c)| (r, block.path.clone(), c.value.clone())));
            }
        }
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

    /// The tabs' light moves while it is on its way.
    fn tick(&mut self) -> bool {
        self.motion.tick(self.theme.as_ref())
    }

    /// A press keeps the value of what it pressed at its block's path.
    fn respond(&mut self, event: &PEvent) -> bool {
        let PEvent::PointerDown(at) = *event else { return false };
        let place = at.plus(self.frame.origin);
        let hit = self.spot.borrow().iter().find(|(rect, ..)| rect.contains(place)).map(|(_, path, value)| (path.clone(), value.clone()));
        hit.is_some_and(|(path, value)| {
            self.source.set(&path, &value);
            true
        })
    }
}
