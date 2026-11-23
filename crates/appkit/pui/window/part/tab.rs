//! The tabs: one for each space of the window, the one shown filled. A
//! press on a tab shows its space. Every app has the same tabs; its
//! spaces are its data.

use super::{PBar, ROW};
use crate::pui::core::light::{ink as lit_on, label, pane};
use crate::pui::core::{PLight, PPaint, PPoint, PRect};
use crate::pui::window::{PDeed, PLevel, PWindow};
use crate::theme::PTheme;

/// The theme's tokens for the tabs: the fill of their bar, the ink of a
/// tab, the fill and ink of the one shown, how wide and how round a tab
/// is, and the room between them.
pub const FILL: &str = "tab.fill";
pub const INK: &str = "tab.ink";
pub const LIT: &str = "tab.active";
pub const LIT_INK: &str = "tab.active.ink";
pub const WIDTH: &str = "tab.width";
pub const ROUND: &str = "tab.radius";
pub const GAP: &str = "tab.gap";

/// The tabs of a window's spaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PTab;

impl PTab {
    /// The rectangle of each tab, from the left. A tab is as wide as the
    /// theme says, and narrower when the bar has no room for that.
    fn pill(&self, bar: PRect, window: &PWindow, theme: &dyn PTheme) -> Vec<PRect> {
        let (gap, count) = (theme.size(GAP).unwrap_or_default(), window.space.len().max(1) as f32);
        let wide = theme.size(WIDTH).unwrap_or_default().min((bar.size.width - gap) / count - gap);
        let pill = |index: usize| PRect::new(bar.origin.x + gap + index as f32 * (wide + gap), bar.origin.y + gap, wide, bar.size.height - gap * 2.0);
        (0..window.space.len()).map(pill).collect()
    }
}

impl PBar for PTab {
    fn spot(&self, bar: PRect, window: &PWindow, theme: &dyn PTheme) -> Vec<(PRect, PDeed)> {
        let pill = self.pill(bar, window, theme);
        pill.into_iter().zip(&window.space).map(|(rect, space)| (rect, PDeed::Show(space.slug().to_string()))).collect()
    }

    fn draw(&self, bar: PRect, window: &PWindow, paint: &mut dyn PPaint) {
        let pill = self.pill(bar, window, paint.theme());
        let (ink, lit_ink) = (paint.color(INK), lit_on(paint, LIT_INK));
        let (round, line) = (paint.size(ROUND), paint.size(ROW));
        pane(paint, bar, 0.0, FILL);
        for (rect, space) in pill.into_iter().zip(&window.space) {
            let shown = window.shows(&PDeed::Show(space.slug().to_string()));
            let color = match shown {
                true => {
                    label(paint, rect, round, PLight::Strong, false, LIT);
                    lit_ink
                }
                false => ink,
            };
            let (middle, wide) = (rect.center(), paint.measure(space.label()));
            paint.text(PPoint::new(middle.x - wide / 2.0, middle.y - line / 2.0), space.label(), color);
        }
    }
}
