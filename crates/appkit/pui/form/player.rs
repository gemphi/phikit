//! A player card, as a speaker's screen stands a song or an episode (the
//! user, 25 November 2026: "use these cards styles"): on a ground of its
//! own, the picture square across the top, what it is called and a line
//! about it under it, a bar as long as its level (how far it has played),
//! the note at its end under the bar at the right (its time), and what
//! can be done with it in a row of round controls along its foot, those
//! that are on lit. As many cards across as large tiles fit; the one
//! under the pointer lit the more.

use super::PChoice;
use super::kind::{BAR, DIM, GAP, LARGE, LINE};
use crate::pui::block::{CARD, PRow};
use crate::pui::core::light::label;
use crate::pui::core::{PLight, PPaint, PPoint, PRect};

/// The theme's token for an icon's size.
const ICON: &str = "icon.size";

/// Player cards from a place across a width: where they end, and where
/// each can be pressed, the cards in order, then their controls in order.
pub fn player(choice: &[PChoice], paint: &mut dyn PPaint, at: PPoint, wide: f32, over: Option<PPoint>) -> (f32, Vec<PRect>) {
    let (gap, line, bar, icon, dim) = (paint.size(GAP), paint.size(LINE), paint.size(BAR).max(2.0), paint.size(ICON), paint.color(DIM));
    let least = (paint.size(CARD) * LARGE).min(wide).max(1.0);
    let across = ((wide + gap / 2.0) / (least + gap / 2.0)).floor().max(1.0) as usize;
    let side = (wide - (across - 1) as f32 * gap / 2.0) / across as f32;
    let round = icon + gap * 0.75;
    let high = side + line * 2.0 + gap * 1.5 + bar + line + gap / 2.0 + round + gap;
    let mut card = Vec::new();
    let mut control = Vec::new();
    for (index, c) in choice.iter().enumerate() {
        let r = PRect::new(at.x + (index % across) as f32 * (side + gap / 2.0), at.y + (index / across) as f32 * (high + gap / 2.0), side, high);
        let light = match over.is_some_and(|over| r.contains(over)) {
            true => PLight::Strong,
            false => PLight::Soft,
        };
        let (_, under) = (c.lead(), c.label.as_str(), c.note.as_str()).tile(paint, r, light);
        // How far it has played: the track, and as much of it as its level.
        let (left, room) = (r.origin.x + gap / 2.0, side - gap);
        let y = under.y + gap / 4.0;
        label(paint, PRect::new(left, y, room, bar), bar / 2.0, PLight::Soft, false, "");
        let played = room * c.level.unwrap_or_default().clamp(0.0, 1.0);
        (played >= 1.0).then(|| label(paint, PRect::new(left, y, played, bar), bar / 2.0, PLight::Strong, false, ""));
        // Its time, at the right under the bar.
        let end = paint.measure(&c.end);
        paint.text(PPoint::new(left + room - end, y + bar + gap / 4.0), &c.end, dim);
        // Its controls, round, in a row along its foot.
        let count = c.deed.len() as f32;
        let span = count * round + (count - 1.0).max(0.0) * gap / 2.0;
        let top = r.origin.y + high - gap / 2.0 - round;
        for (n, deed) in c.deed.iter().enumerate() {
            let place = PRect::new(r.center().x - span / 2.0 + n as f32 * (round + gap / 2.0), top, round, round);
            let lit = match deed.on {
                true => PLight::Strong,
                false => PLight::Rest,
            };
            label(paint, place, round / 2.0, lit, false, "");
            paint.icon(&deed.icon, PRect::new(place.center().x - icon / 2.0, place.center().y - icon / 2.0, icon, icon), paint.color(super::kind::INK));
            control.push(place);
        }
        card.push(r);
    }
    let rows = choice.len().div_ceil(across) as f32;
    card.extend(control);
    (at.y + rows * (high + gap / 2.0) - gap / 2.0, card)
}
