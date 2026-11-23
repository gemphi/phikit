//! A menu: what a row of the activity bar shows beside it, in a second
//! column. Its rows stand in groups, and every group is a ground of its
//! own: a card lit against the bar, with the row that is shown lit
//! against the card in turn. It is puikit's shell
//! (`src/components/navigation/AppShell`): groups at the top, in the
//! middle and at the foot, the middle kept in the middle whenever there
//! is room.
//!
//! A group may hold the app's side part instead of rows: a list of chats,
//! of files. That group stands in the middle and takes the room the
//! others leave.

use super::{DIM, GAP, ICON, INK as TEXT, ROW as LINE};
use crate::pui::core::light::label;
use crate::pui::core::{PLight, PPaint, PPoint, PRect};
use crate::pui::window::{PDeed, PEntry, PWindow};
use crate::theme::PTheme;

/// The theme's tokens a menu shares with the bar: how high a row is, the
/// ink of an icon, the fill behind the row that is shown, and how round
/// a group is.
pub const ROW: &str = "activity.row";
pub const INK: &str = "activity.ink";
pub const LIT: &str = "activity.active";
pub const ROUND: &str = "activity.radius";
/// What a table calls a row that heads its group, and one that has the
/// group hold the app's side part.
pub const HEAD: &str = "head";
pub const SIDE: &str = "side";

/// Where something stands in a column: from its top, in its middle, or
/// at its foot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PSeat {
    #[default]
    Top,
    Center,
    Bottom,
}

impl PSeat {
    /// The seat a table names: `center`, `bottom`, and the top otherwise.
    pub fn named(word: &str) -> PSeat {
        match word {
            "center" => PSeat::Center,
            "bottom" => PSeat::Bottom,
            _ => PSeat::Top,
        }
    }
}

/// One row, of the bar or of a menu: what it is, what a press on it does,
/// where it stands, and what it shows beside the bar.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PItem {
    pub entry: PEntry,
    /// Nothing for a row that only shows its menu.
    pub deed: Option<PDeed>,
    pub seat: PSeat,
    /// Its menu: its children, in groups.
    pub group: Vec<PGroup>,
    /// A picture in its icon's place, where it has one: the sheet it is
    /// on, how many pictures the sheet has in a row, and which it is.
    pub face: Option<(String, u32, u32)>,
    /// Whether the app says it is the one shown.
    pub on: bool,
}

/// A group of a menu: rows that belong together, on one ground.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PGroup {
    pub slug: String,
    pub seat: PSeat,
    /// Its heading, or nothing.
    pub head: String,
    pub item: Vec<PItem>,
    /// Whether it holds the app's side part in place of rows.
    pub side: bool,
}

/// A group as it is laid out: its ground, its heading's row, its rows.
pub struct PLaid<'a> {
    pub group: &'a PGroup,
    pub rect: PRect,
    pub row: Vec<(PRect, &'a PItem)>,
}

impl PItem {
    /// Take a row of a menu's table into this row's menu: the seat and
    /// the name of its group, then its slug, label and icon, and the
    /// kind and the target of its deed. A `head` row is the group's
    /// heading and a `side` row has the group hold the app's side part;
    /// any other row that names no deed is skipped.
    pub fn hold(&mut self, cell: &[&str]) {
        let word = |index: usize| cell.get(index).copied().unwrap_or_default();
        let deed = PDeed::named(word(5), word(6));
        if deed.is_none() && word(5) != HEAD {
            return;
        }
        let index = self.group.iter().position(|group| group.slug == word(1)).unwrap_or_else(|| {
            self.group.push(PGroup { slug: word(1).to_string(), seat: PSeat::named(word(0)), ..PGroup::default() });
            self.group.len() - 1
        });
        let group = &mut self.group[index];
        match word(5) {
            HEAD => group.head = word(3).to_string(),
            SIDE => group.side = true,
            _ => group.item.push(PItem { entry: PEntry::new(word(2), word(3), word(4)), deed, ..PItem::default() }),
        }
    }
}

/// How high a group is: half a gap round its rows, a row for each and one
/// for its heading. One that holds the side part has no height of its
/// own.
fn high(group: &PGroup, row: f32, pad: f32) -> f32 {
    match group.side {
        true => 0.0,
        false => pad * 2.0 + (group.item.len() + usize::from(!group.head.is_empty())) as f32 * row,
    }
}

/// Lay the groups of a menu out in `room`. The ones at the top stand from
/// its top and the ones at the foot end at its foot, each half a gap from
/// the next and from the sides. The ones in the middle stand round the
/// room's own middle while there is room for that, and under the top
/// ones otherwise; the one that holds the side part takes what is left.
pub fn lay<'a>(group: &'a [PGroup], room: PRect, theme: &dyn PTheme) -> Vec<PLaid<'a>> {
    let (row, pad) = (theme.size(ROW).unwrap_or_default(), theme.size(GAP).unwrap_or_default() / 2.0);
    let seated = |seat: PSeat| group.iter().filter(move |group| group.seat == seat);
    let tall = |seat: PSeat| seated(seat).map(|group| high(group, row, pad) + pad).sum::<f32>();
    let (top, foot) = (room.origin.y + tall(PSeat::Top), room.origin.y + room.size.height - tall(PSeat::Bottom));
    // What the middle may take, and what its groups with rows do take.
    let (free, fixed) = ((foot - pad - top).max(0.0), (tall(PSeat::Center) - pad).max(0.0));
    let spare = (free - fixed).max(0.0);
    let middle = match seated(PSeat::Center).any(|group| group.side) {
        true => top,
        false => (room.center().y - fixed / 2.0).min(foot - pad - fixed).max(top),
    };
    let mut at = [(PSeat::Top, room.origin.y), (PSeat::Center, middle), (PSeat::Bottom, foot)];
    let (left, wide) = (room.origin.x + pad, room.size.width - pad * 2.0);
    let mut out = Vec::new();
    for one in group {
        let slot = at.iter_mut().find(|(seat, _)| *seat == one.seat).expect("every seat is in the list");
        let height = match one.side {
            true => spare,
            false => high(one, row, pad),
        };
        let rect = PRect::new(left, slot.1, wide, height);
        slot.1 += height + pad;
        let first = rect.origin.y + pad + usize::from(!one.head.is_empty()) as f32 * row;
        let line = |index: usize| PRect::new(left + pad, first + index as f32 * row, wide - pad * 2.0, row);
        out.push(PLaid { group: one, rect, row: one.item.iter().enumerate().map(|(index, item)| (line(index), item)).collect() });
    }
    out
}

/// Where a press on a menu does something: its rows.
pub fn spot(group: &[PGroup], room: PRect, theme: &dyn PTheme) -> Vec<(PRect, PDeed)> {
    let row = lay(group, room, theme).into_iter().flat_map(|laid| laid.row);
    row.filter_map(|(rect, item)| Some((rect, item.deed.clone()?))).collect()
}

/// The room a menu leaves for the app's side part: inside the group that
/// holds it, half a gap from its head and its foot. None in a menu with
/// no such group.
pub fn room(group: &[PGroup], room: PRect, theme: &dyn PTheme) -> PRect {
    let pad = theme.size(GAP).unwrap_or_default() / 2.0;
    let held = lay(group, room, theme).into_iter().find(|laid| laid.group.side && laid.rect.size.height > pad * 2.0);
    held.map(|laid| PRect::new(laid.rect.origin.x, laid.rect.origin.y + pad, laid.rect.size.width, laid.rect.size.height - pad * 2.0)).unwrap_or_default()
}

/// Draw a menu. Every group stands on a ground of its own, lit from the
/// right, against the bar's icons beside it, which are lit from the
/// left; the row that is shown is lit the other way, against its group,
/// and so, more softly, is the row the pointer is over; a heading is
/// dimmer than a row.
pub fn draw(group: &[PGroup], room: PRect, window: &PWindow, over: Option<PPoint>, paint: &mut dyn PPaint) {
    let (ink, text, dim) = (paint.color(INK), paint.color(TEXT), paint.color(DIM));
    let (icon, round, line, row) = (paint.size(ICON), paint.size(ROUND), paint.size(LINE), paint.size(ROW));
    let pad = paint.size(GAP) / 2.0;
    for PLaid { group, rect, row: list } in lay(group, room, paint.theme()).into_iter().filter(|laid| laid.rect.size.height > 0.0) {
        label(paint, rect, round, PLight::Rest, true, "");
        if !group.head.is_empty() {
            paint.text(PPoint::new(rect.origin.x + pad * 2.0, rect.origin.y + pad + (row - line) / 2.0), &group.head, dim);
        }
        for (rect, item) in list {
            let on = item.deed.as_ref().is_some_and(|deed| window.shows(deed));
            match (on, over.is_some_and(|at| rect.contains(at)) && item.deed.is_some()) {
                (true, _) => label(paint, rect, round / 2.0, PLight::Rest, false, LIT),
                (false, true) => label(paint, rect, round / 2.0, PLight::Soft, false, ""),
                (false, false) => {}
            }
            let middle = rect.center().y;
            paint.icon(&item.entry.icon, PRect::new(rect.origin.x + pad, middle - icon / 2.0, icon, icon), ink);
            paint.text(PPoint::new(rect.origin.x + pad * 2.0 + icon, middle - line / 2.0), &item.entry.label, text);
        }
    }
}
