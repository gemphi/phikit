//! People as cards: each a ground with their face large on it, their
//! name, and a line about them. One part for every place people are
//! shown together: who is in a voice room, a room's members. A row that
//! folds; a press on a card opens the person.

use appkit::pui::core::{PLight, PPaint, PPoint, PRect, light};

use super::call::face;
use super::{DIM, Desk, GAP, INK, LINE, RADIUS, fit};

/// The theme's tokens: how wide a person's card is, and how large the
/// face on it.
pub const CARD: &str = "voice.card";
pub const FACE: &str = "voice.face";

/// One of a crowd: their id among the app's people, nothing for one
/// who is not of them; their name; and a line about them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seat {
    pub id: String,
    pub name: String,
    pub said: String,
}

/// The people a list of names stands for, as the app's own file has
/// them: each with their id and how they are.
pub fn named(desk: &Desk, name: &[String]) -> Vec<Seat> {
    let one = |name: &String| {
        let member = desk.site.member.iter().find(|member| member.name == *name);
        Seat { id: member.map(|member| member.id.clone()).unwrap_or_default(), name: name.clone(), said: member.map(|member| member.state.clone()).unwrap_or_default() }
    };
    name.iter().map(one).collect()
}

/// Draw people as cards from `at`, in rows as wide as `wide`. Where the
/// card of each one of the app's people is, and how far down they end.
pub fn cards(desk: &Desk, paint: &mut dyn PPaint, at: PPoint, wide: f32, seat: &[Seat]) -> (Vec<(PRect, String)>, f32) {
    let (gap, line, radius, ink, dim) = (paint.size(GAP), paint.size(LINE), paint.size(RADIUS), paint.color(INK), paint.color(DIM));
    let (card, size) = (paint.size(CARD).min(wide), paint.size(FACE));
    // As many across as fit, sharing the width evenly.
    let across = ((wide + gap / 2.0) / (card + gap / 2.0)).floor().max(1.0);
    let card = (wide - (across - 1.0) * gap / 2.0) / across;
    let high = size + line * 2.0 + gap * 1.5;
    let mut spot = Vec::new();
    for (index, one) in seat.iter().enumerate() {
        let (column, row) = ((index % across as usize) as f32, (index / across as usize) as f32);
        let rect = PRect::new(at.x + column * (card + gap / 2.0), at.y + row * (high + gap / 2.0), card, high);
        light::surface(paint, rect, radius, PLight::Soft, "");
        face(desk, paint, PRect::new(rect.center().x - size / 2.0, rect.origin.y + gap / 2.0, size, size), &one.id, &one.name);
        for (index, (said, color)) in [(&one.name, ink), (&one.said, dim)].into_iter().enumerate() {
            let said = fit(said, card - gap, paint);
            paint.text(PPoint::new(rect.center().x - paint.measure(&said) / 2.0, rect.origin.y + gap + size + index as f32 * line), &said, color);
        }
        spot.extend(Some((rect, one.id.clone())).filter(|_| !one.id.is_empty()));
    }
    let rows = seat.len().div_ceil(across as usize) as f32;
    (spot, at.y + rows * high + (rows - 1.0).max(0.0) * gap / 2.0)
}
