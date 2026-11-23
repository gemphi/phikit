//! The caption: the row at the window's very top, where a system puts
//! its title bar. It carries the app's own mark over the activity bar,
//! on a ground as large as the ground of every icon of the bar under
//! it, and the app's name while a menu stands beside the bar. The mark
//! is also the pin: while the pointer is over it, it shows the pin, and
//! a press on it puts the bar's menu away and brings it back. It is
//! puikit's shell (`src/components/navigation/AppShell/Brand.tsx`): the
//! header's first cell, exactly as wide as the sidebar under it.
//!
//! Where the window has no bar of the system's (`bare`), the caption is
//! what the window is moved by, and its right end carries the window's
//! three buttons. A theme that gives the caption no height has none, and
//! the mark and the pin are the activity bar's own.

use super::activity::square;
use super::menu::{INK, LIT, ROUND, ROW};
use super::{GAP, ICON, INK as TEXT, PActivity, ROW as LINE};
use crate::pui::core::light::label;
use crate::pui::core::{PLight, PPaint, PPoint, PRect};
use crate::pui::window::{PDeed, PFrame, PWindow};
use crate::theme::PTheme;

/// The theme's token for how high the caption is.
pub const HIGH: &str = "caption.height";
/// The pin's two icons, puikit's: a pill in a rounded square, wide while
/// the menu is shown.
pub const OPEN: &str = "pin-open";
pub const SHUT: &str = "pin-shut";

/// What a place on the caption is to the system's window: where it is
/// moved by, and its three buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PGrip {
    Drag,
    Min,
    Max,
    Close,
}

impl PGrip {
    /// The window's buttons, from its right end, each with its icon.
    const KNOB: [(PGrip, &'static str); 3] = [(PGrip::Close, "x"), (PGrip::Max, "square"), (PGrip::Min, "minus")];
}

/// The caption of a window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PCaption {
    /// Whether the window has no bar of the system's, so that the
    /// caption is what moves it and carries its buttons.
    pub bare: bool,
    /// Whether the pointer is over the mark, which then shows the pin.
    pub over: bool,
}

/// Where everything on the caption is.
struct Plan {
    mark: PRect,
    /// Where the app's name starts, while a menu stands beside the bar.
    name: Option<f32>,
    knob: Vec<(PRect, PGrip)>,
}

impl PCaption {
    /// How high a theme has the caption: nothing in one without it.
    pub fn high(theme: &dyn PTheme) -> f32 {
        theme.size(HIGH).unwrap_or_default()
    }

    /// Where everything is. The mark is over the bar: as wide as the bar
    /// where it stands, a square where it lies. The buttons are squares
    /// from the right end.
    fn plan(&self, frame: &PFrame, theme: &dyn PTheme) -> Plan {
        let (row, bar, menu) = (frame.caption, frame.activity, frame.menu);
        let (top, high) = (row.origin.y, row.size.height);
        let lying = bar.size.width > bar.size.height;
        let wide = match lying {
            true => high,
            false => bar.size.width,
        };
        let mark = PRect::new(bar.origin.x, top, wide, high);
        let beside = !lying && menu.size.width > 0.0;
        let right = row.origin.x + row.size.width;
        let knob = PGrip::KNOB.iter().enumerate().map(|(nth, (grip, _))| (PRect::new(right - (nth + 1) as f32 * high, top, high, high), *grip));
        Plan { mark, name: beside.then(|| menu.origin.x + theme.size(GAP).unwrap_or_default()), knob: knob.filter(|_| self.bare).collect() }
    }

    /// What the pin does: at a desk it puts the bar's menu away and
    /// brings it back, in a hand it draws it out and puts it away.
    fn pin(frame: &PFrame) -> PDeed {
        match frame.activity.size.width > frame.activity.size.height {
            true => PDeed::Side,
            false => PDeed::Fold,
        }
    }

    /// Whether the mark is the pin: it is while a row of the bar has a
    /// menu to put away.
    fn pinned(activity: &PActivity) -> bool {
        activity.item.iter().any(|item| !item.group.is_empty())
    }

    /// Where the mark is, in a caption that has one: nothing otherwise.
    pub fn mark(&self, frame: &PFrame, activity: &PActivity, theme: &dyn PTheme) -> PRect {
        match frame.caption.size.height > 0.0 && !activity.is_empty() {
            true => self.plan(frame, theme).mark,
            false => PRect::default(),
        }
    }

    /// Where a press does something, and what: the mark, which does
    /// what the pin does while a row of the bar has a menu, and
    /// otherwise what the app's data has it do. An app with no bar has
    /// none.
    pub fn spot(&self, frame: &PFrame, activity: &PActivity, theme: &dyn PTheme) -> Vec<(PRect, PDeed)> {
        let pin = PCaption::pin(frame);
        let deed = match PCaption::pinned(activity) {
            true => pin,
            false => activity.home.clone().unwrap_or(pin),
        };
        let mark = self.mark(frame, activity, theme);
        Some((mark, deed)).filter(|_| mark.size.width > 0.0).into_iter().collect()
    }

    /// What the system's window is told of the caption, where it is
    /// bare: its buttons, and between everything that is pressed, where
    /// it is moved by.
    pub fn grip(&self, frame: &PFrame, activity: &PActivity, theme: &dyn PTheme) -> Vec<(PRect, PGrip)> {
        let (row, mut out) = (frame.caption, self.plan(frame, theme).knob);
        let mut taken: Vec<PRect> = self.spot(frame, activity, theme).into_iter().map(|(rect, _)| rect).chain(out.iter().map(|(rect, _)| *rect)).collect();
        taken.sort_by(|one, other| one.origin.x.total_cmp(&other.origin.x));
        taken.push(PRect::new(row.origin.x + row.size.width, row.origin.y, 0.0, 0.0));
        let mut left = row.origin.x;
        for rect in taken.iter().filter(|_| self.bare) {
            let free = PRect::new(left, row.origin.y, rect.origin.x - left, row.size.height);
            out.extend((free.size.width > 0.0).then_some((free, PGrip::Drag)));
            left = left.max(rect.origin.x + rect.size.width);
        }
        out
    }

    /// Draw the caption: the mark on a ground as the bar's icons are,
    /// the pin in its place while the pointer is over it, the name, and
    /// the window's buttons. An app with no bar has its name alone.
    pub fn draw(&self, frame: &PFrame, activity: &PActivity, window: &PWindow, paint: &mut dyn PPaint) {
        let plan = self.plan(frame, paint.theme());
        if frame.caption.size.height <= 0.0 {
            return;
        }
        let (ink, text) = (paint.color(INK), paint.color(TEXT));
        let (icon, round, line, gap) = (paint.size(ICON), paint.size(ROUND), paint.size(LINE), paint.size(GAP));
        let chip = (paint.size(ROW) - gap / 4.0).min(plan.mark.size.height);
        let name = match activity.logo.label.is_empty() {
            true => window.title.as_str(),
            false => activity.logo.label.as_str(),
        };
        let middle = plan.mark.center();
        let lying = frame.activity.size.width > frame.activity.size.height;
        // The pin shows which way a press goes: wide while a menu is shown.
        let pin = match activity.shown(window, lying) {
            Some(_) => OPEN,
            None => SHUT,
        };
        let over = self.over && PCaption::pinned(activity);
        let ground = square(plan.mark, chip);
        match (activity.is_empty(), over, activity.logo.icon.is_empty()) {
            (true, ..) => paint.text(PPoint::new(frame.caption.origin.x + gap, middle.y - line / 2.0), name, text),
            (false, true, _) => {
                label(paint, ground, round, PLight::Strong, false, LIT);
                paint.icon(pin, square(plan.mark, icon), ink);
            }
            (false, false, false) => {
                label(paint, ground, round, PLight::Soft, false, "");
                paint.icon(&activity.logo.icon, square(plan.mark, icon), ink);
            }
            (false, false, true) => {
                let letter: String = name.chars().take(1).collect();
                label(paint, ground, round, PLight::Strong, false, LIT);
                paint.text(PPoint::new(middle.x - paint.measure(&letter) / 2.0, middle.y - line / 2.0), &letter, text);
            }
        }
        for at in plan.name.into_iter().filter(|_| !activity.is_empty()) {
            paint.text(PPoint::new(at, middle.y - line / 2.0), name, text);
        }
        for ((rect, _), (_, picture)) in plan.knob.iter().zip(PGrip::KNOB) {
            paint.icon(picture, square(*rect, icon * 4.0 / 5.0), ink);
        }
    }
}
