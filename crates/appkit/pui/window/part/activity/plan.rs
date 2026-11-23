//! Where everything on the activity bar is, for one rectangle of it.

use std::rc::Rc;

use super::super::menu::{PItem, PSeat, ROW};
use super::{PActivity, PCaption};
use crate::pui::core::PRect;
use crate::theme::PTheme;

/// The rows an app adds to its bar while it runs, asked again whenever
/// the window is laid out and once a frame: the places a person is in,
/// each with its picture. They follow the bar's top rows.
pub type PRail = Rc<dyn Fn() -> Vec<PItem>>;

/// Where everything on the bar is.
pub(super) struct Plan<'a> {
    pub head: PRect,
    pub item: Vec<(PRect, &'a PItem)>,
    pub user: PRect,
    pub lying: bool,
}

/// Rows one under another from `start`, each in a cell like `cell`.
fn stand(list: Vec<&PItem>, cell: PRect, start: f32) -> Vec<(PRect, &PItem)> {
    list.into_iter().enumerate().map(|(index, item)| (PRect::new(cell.origin.x, start + index as f32 * cell.size.height, cell.size.width, cell.size.height), item)).collect()
}

impl PActivity {
    /// Whether the mark and the button that puts a menu away are the
    /// bar's own: they are the caption's in a theme that has one.
    pub(super) fn headed(theme: &dyn PTheme) -> bool {
        PCaption::high(theme) <= 0.0
    }

    /// Roll the live rows by so many, a standing bar's: never past the
    /// last that leaves the bar full. `true` when that moved them.
    pub fn roll(&mut self, by: isize, bar: PRect, theme: &dyn PTheme) -> bool {
        let shown = self.plan(bar, theme).item.iter().filter(|(_, item)| self.live.iter().any(|live| std::ptr::eq(live, *item))).count();
        let most = (self.live.len() - self.skip.min(self.live.len())).saturating_sub(shown) + self.skip;
        let next = (self.skip as isize + by).clamp(0, most as isize) as usize;
        std::mem::replace(&mut self.skip, next) != next
    }

    /// Where everything is in `bar`. Lying, the rows are squares from the
    /// left in the order of their seats, and the user's is at the right.
    /// Standing, the mark has a square at the head and the user one at
    /// the foot; the top rows follow the mark, and after them the rows
    /// the app adds while it runs, as many as there is room for; the
    /// bottom ones end over the user, and the middle ones stand round
    /// the bar's own middle while there is room. A row with no deed and
    /// no menu is left out. Under a caption there is no mark, and the
    /// top rows start as far from the bar's top as they stand from its
    /// sides.
    pub(super) fn plan(&self, bar: PRect, theme: &dyn PTheme) -> Plan<'_> {
        let (left, top, wide, tall) = (bar.origin.x, bar.origin.y, bar.size.width, bar.size.height);
        let seated = |seat: PSeat| self.item.iter().filter(move |item| item.seat == seat && (item.deed.is_some() || !item.group.is_empty()));
        match wide > tall {
            true => {
                let order = seated(PSeat::Top).chain(seated(PSeat::Center)).chain(seated(PSeat::Bottom));
                let item = order.enumerate().map(|(index, item)| (PRect::new(left + index as f32 * tall, top, tall, tall), item)).collect();
                Plan { head: PRect::default(), item, user: PRect::new(left + wide - tall, top, tall, tall), lying: true }
            }
            false => {
                let row = theme.size(ROW).unwrap_or(wide);
                let head = match PActivity::headed(theme) {
                    true => wide,
                    false => ((wide - row) / 2.0).max(0.0),
                };
                let high = |seat: PSeat| seated(seat).count() as f32 * row;
                let (under, foot) = (top + head, top + tall - wide - high(PSeat::Bottom));
                // The live rows have the room the others leave, from the
                // one the wheel rolled to.
                let room = ((foot - high(PSeat::Center) - under) / row.max(1.0)).floor().max(0.0) as usize;
                let lead: Vec<&PItem> = seated(PSeat::Top).chain(self.live.iter().skip(self.skip).take(room.saturating_sub(seated(PSeat::Top).count()))).collect();
                let middle = (bar.center().y - high(PSeat::Center) / 2.0).min(foot - high(PSeat::Center)).max(under + lead.len() as f32 * row);
                let cell = PRect::new(left, 0.0, wide, row);
                let item = [stand(lead, cell, under), stand(seated(PSeat::Center).collect(), cell, middle), stand(seated(PSeat::Bottom).collect(), cell, foot)].concat();
                Plan { head: PRect::new(left, top, wide, head), item, user: PRect::new(left, top + tall - wide, wide, wide), lying: false }
            }
        }
    }
}
