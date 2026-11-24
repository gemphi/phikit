//! A drop open under its switch at a heading, over everything else of
//! the form: each choice its icon and its words, the one chosen lit, on
//! a ground of the window's own so nothing shows through. A press on a
//! choice keeps it at the drop's path and puts the drop away; a press
//! anywhere else only puts it away (`form.rs`).

use super::head::{DROP, LAYOUTS, PHead};
use super::{PForm, Spot, kind};
use crate::pui::core::light::{label, surface};
use crate::pui::core::{PLight, PPaint, PPoint, PRect};

/// The theme's token for the window's own ground.
const GROUND: &str = "window.fill";

impl PForm {
    /// The headings of the blocks shown, each with where its switches
    /// stand: the first of the places a press on the block keeps, in the
    /// order `PForm::spots` gives them.
    pub(super) fn heads<'a>(&self, spot: &[Spot], shown: &'a [super::group::Shown<'a>]) -> Vec<(&'a PHead, Vec<PRect>)> {
        shown.iter().map(|(_, _, head)| {
            let rect = head.all().map(|tool| spot.iter().find(|(_, path, value)| *path == tool.path && *value == tool.value).map(|(rect, ..)| *rect).unwrap_or_default()).collect();
            (head, rect)
        }).collect()
    }

    /// Draw the drop that is open, if one is, under the switch it opened
    /// from: each switch of the headings shown is given with where it
    /// stands. What a press on each choice keeps.
    pub(super) fn dropped(&self, paint: &mut dyn PPaint, head: &[(&PHead, Vec<PRect>)]) -> Vec<Spot> {
        let open = self.source.get(DROP);
        if open.is_empty() {
            return Vec::new();
        }
        let found = head.iter().flat_map(|(head, rect)| head.all().zip(rect.iter())).find(|(tool, _)| tool.drop.as_ref().is_some_and(|drop| drop.id == open));
        let Some((tool, anchor)) = found else { return Vec::new() };
        let Some(drop) = &tool.drop else { return Vec::new() };
        let choice: Vec<(String, String, String)> = match drop.list.is_empty() {
            true => LAYOUTS.iter().map(|(value, icon, word)| (value.to_string(), icon.to_string(), self.text.get(word).to_string())).collect(),
            false => self.source.list(&drop.list).into_iter().map(|c| (c.value, c.icon, c.label)).collect(),
        };
        // Nothing chosen yet is the first chosen.
        let now = match self.source.get(&drop.path) {
            kept if kept.is_empty() => choice.first().map(|(value, ..)| value.clone()).unwrap_or_default(),
            kept => kept,
        };
        let (gap, line, icon, radius, ink) = (paint.size(kind::GAP), paint.size(kind::LINE), paint.size("icon.size"), paint.size(kind::RADIUS), paint.color(kind::INK));
        let high = line + gap / 2.0;
        let words = choice.iter().map(|(_, _, word)| paint.measure(word)).fold(0.0, f32::max);
        let wide = words + icon + gap * 1.25;
        // Under its switch: from its left at the left of the form, to its
        // right at the right.
        let frame = self.frame.size.width;
        let x = match anchor.origin.x - self.origin.get().x > frame / 2.0 {
            true => anchor.origin.x + anchor.size.width - wide,
            false => anchor.origin.x,
        };
        let ground = PRect::new(x, anchor.origin.y + anchor.size.height + gap / 4.0, wide, choice.len() as f32 * high + gap / 2.0);
        paint.round(ground, radius, paint.color(GROUND));
        surface(paint, ground, radius, PLight::Rest, "");
        choice.iter().enumerate().map(|(index, (value, mark, word))| {
            let row = PRect::new(ground.origin.x + gap / 4.0, ground.origin.y + gap / 4.0 + index as f32 * high, wide - gap / 2.0, high);
            if *value == now {
                label(paint, row, radius, PLight::Strong, false, "");
            }
            let left = row.origin.x + gap / 4.0;
            (!mark.is_empty()).then(|| paint.icon(mark, PRect::new(left, row.center().y - icon / 2.0, icon, icon), ink));
            paint.text(PPoint::new(left + icon + gap / 4.0, row.center().y - line / 2.0), word, ink);
            paint.press(row);
            (row, drop.path.clone(), value.clone())
        }).collect()
    }
}
