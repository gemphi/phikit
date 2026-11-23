//! The panel an open modal is shown in, over a veil, with a title and two
//! buttons: a square in the middle of the window, a drawer at its right
//! edge, or a sheet at its foot, as the modal's shape says. One button
//! makes it as large as the window and what it was again, as any window
//! can be made larger; the other, a press on the veil, or the Escape key
//! closes it.
//!
//! A bare panel has no head: no title and neither button. All of it is
//! its content's, which has its own way out; Escape closes it still.

use super::{PDeed, PShape};
use crate::pui::core::light::pane;
use crate::pui::core::{PPaint, PPoint, PRect, PSize};
use crate::theme::PTheme;

/// The theme's tokens for the panel: how much of the window's shorter side
/// the square takes, the veil behind it, its fill, how round it is, and
/// how high its head is.
pub const SHARE: &str = "modal.share";
/// How much of it the square takes in a hand, where a window is small. A
/// theme without it gives a hand the same share as a desk.
pub const HAND: &str = "modal.hand";
/// How much of the window's whole space the square fills, where the theme
/// says: a half makes a square as large as half the window. The share
/// above is then the most of the shorter side it may take.
pub const AREA: &str = "modal.area";
pub const VEIL: &str = "modal.veil";
pub const FILL: &str = "modal.fill";
pub const ROUND: &str = "modal.radius";
pub const HEAD: &str = "modal.head";
/// How far a drawer and a sheet stay from the window's edges, how wide a
/// drawer is, how much of the window's height a sheet takes, and the
/// most a sheet is wide. Neither is ever wider than the window leaves.
pub const MARGIN: &str = "modal.margin";
pub const DRAWER: &str = "drawer.width";
pub const SHEET: &str = "sheet.share";
pub const SPAN: &str = "sheet.width";

/// Where an open modal is, in the window.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PPanel {
    /// The whole window, which the veil covers.
    pub whole: PRect,
    /// The panel itself.
    pub rect: PRect,
    /// Under its head: where the modal's content goes.
    pub body: PRect,
    /// The two buttons at the right of its head.
    pub expand: PRect,
    pub close: PRect,
    /// Whether it is as large as the window.
    pub wide: bool,
    /// Whether it has no head: its body is all of it.
    pub bare: bool,
}

impl PPanel {
    /// The same panel `by` lower: one laid in the room a caption leaves.
    pub fn lower(self, by: f32) -> PPanel {
        let down = |rect: PRect| rect.moved(PPoint::new(0.0, by));
        PPanel { whole: down(self.whole), rect: down(self.rect), body: down(self.body), expand: down(self.expand), close: down(self.close), ..self }
    }

    /// The same panel with no head, when `bare`: its body is all of it,
    /// and its two buttons are nowhere.
    pub fn bared(self, bare: bool) -> PPanel {
        match bare {
            true => PPanel { body: self.rect, expand: PRect::default(), close: PRect::default(), bare, ..self },
            false => self,
        }
    }

    /// The panel in a window of that size, in that shape, or the whole
    /// window when `wide`. A square is in the middle, and in a hand takes
    /// the share the theme gives a hand. A drawer stands at the right
    /// edge and a sheet lies at the foot, in the middle of its width,
    /// each a margin clear of the window's edges.
    pub fn of(size: PSize, shape: PShape, wide: bool, hand: bool, theme: &dyn PTheme) -> PPanel {
        let whole = PRect { origin: PPoint::default(), size };
        let share = theme.size(HAND).filter(|_| hand).or(theme.size(SHARE)).unwrap_or(1.0);
        let most = size.width.min(size.height) * share;
        let side = match theme.size(AREA) {
            Some(area) => (size.width * size.height * area).max(0.0).sqrt().min(most),
            None => most,
        };
        let margin = theme.size(MARGIN).unwrap_or_default();
        let (room, high) = ((size.width - margin * 2.0).max(0.0), (size.height - margin * 2.0).max(0.0));
        let across = |token: &str| theme.size(token).unwrap_or(room).min(room);
        let rect = match (wide, shape) {
            (true, _) | (false, PShape::Whole) => whole,
            (false, PShape::Square) => PRect::new((size.width - side) / 2.0, (size.height - side) / 2.0, side, side),
            (false, PShape::Drawer) => PRect::new(size.width - margin - across(DRAWER), margin, across(DRAWER), high),
            (false, PShape::Sheet) => {
                let tall = (size.height * theme.size(SHEET).unwrap_or(0.5)).min(high);
                PRect::new((size.width - across(SPAN)) / 2.0, size.height - margin - tall, across(SPAN), tall)
            }
        };
        let head = theme.size(HEAD).unwrap_or_default();
        let button = |nth: f32| PRect::new(rect.origin.x + rect.size.width - nth * head, rect.origin.y, head, head);
        let body = PRect::new(rect.origin.x, rect.origin.y + head, rect.size.width, rect.size.height - head);
        PPanel { whole, rect, body, expand: button(2.0), close: button(1.0), wide, bare: false }
    }

    /// What a press at `at` does. `None` for a press on the panel that is
    /// on neither button: it is the content's.
    pub fn deed(&self, at: PPoint) -> Option<PDeed> {
        match (self.close.contains(at), self.expand.contains(at), self.rect.contains(at)) {
            (true, _, _) => Some(PDeed::Close),
            (false, true, _) => Some(PDeed::Expand),
            (false, false, true) => None,
            (false, false, false) => Some(PDeed::Close),
        }
    }

    /// The same panel while it slides, `out` of its way still to come
    /// (1 is out of sight, 0 in its place), and how far that moves it:
    /// a drawer along the window's width, from its right edge, and any
    /// other along its height, from its foot. It slows as it arrives.
    pub fn slid(self, out: f32) -> (PPanel, PPoint) {
        let left = out.clamp(0.0, 1.0).powi(2);
        let drawer = self.rect.size.height > self.rect.size.width && self.rect.center().x > self.whole.center().x;
        let by = match drawer {
            true => PPoint::new((self.whole.origin.x + self.whole.size.width - self.rect.origin.x) * left, 0.0),
            false => PPoint::new(0.0, (self.whole.origin.y + self.whole.size.height - self.rect.origin.y) * left),
        };
        let moved = |rect: PRect| rect.moved(by);
        (PPanel { rect: moved(self.rect), body: moved(self.body), expand: moved(self.expand), close: moved(self.close), ..self }, by)
    }

    /// Draw the veil, the panel, and its title and its buttons where it
    /// has a head.
    pub fn draw(&self, title: &str, paint: &mut dyn PPaint) {
        self.shown(title, 1.0, paint);
    }

    /// Draw it with its veil only so far there: 1 is all of it.
    pub fn shown(&self, title: &str, there: f32, paint: &mut dyn PPaint) {
        let (veil, ink) = (paint.color(VEIL), paint.color("text.ink"));
        let veil = crate::pui::core::PColor { a: veil.a * there.clamp(0.0, 1.0), ..veil };
        let (gap, line, icon) = (paint.size("gap"), paint.size("text.line"), paint.size("icon.size"));
        let round = match self.wide {
            true => 0.0,
            false => paint.size(ROUND),
        };
        let head = self.close.size.height;
        paint.fill(self.whole, veil);
        // A panel floats: it is a pane, glass where the theme has glass.
        pane(paint, self.rect, round, FILL);
        if self.bare {
            return;
        }
        paint.text(self.rect.origin.plus(PPoint::new(gap, (head - line) / 2.0)), title, ink);
        paint.icon("maximize", self.expand.inset((head - icon) / 2.0), ink);
        paint.icon("x", self.close.inset((head - icon) / 2.0), ink);
    }
}
