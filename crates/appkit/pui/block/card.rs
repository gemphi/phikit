//! Cards: somebody shown large on a ground of their own, with what
//! they are called and a line about them; and a deck, cards that
//! belong together, in rows that fold to the width they are given.
//! One look for every place people are shown together: who is in a
//! room, a room's members, a list of friends.

use super::super::core::light::surface;
use super::super::core::{PLight, PPaint, PPoint, PRect};
use super::{DIM, GAP, INK, LINE, PFace, PText, RADIUS};

/// The theme's tokens: how wide a card is at least, and how large the
/// face on it.
pub const CARD: &str = "card.width";
pub const FACE: &str = "card.face";

/// A card: a face, a name, and a line about them.
pub trait PCard: PFace {
    /// A line about them, under their name.
    fn note(&self) -> &str {
        ""
    }

    /// How high a card is: the face, two lines under it, and room.
    fn high(&self, paint: &dyn PPaint) -> f32 {
        paint.size(FACE) + paint.size(LINE) * 2.0 + paint.size(GAP) * 1.5
    }

    /// Draw it in a place: a soft ground, the face in the middle at
    /// the top, the name and the line under it, each cut to fit.
    fn card(&self, paint: &mut dyn PPaint, rect: PRect) {
        let (gap, line, radius, size) = (paint.size(GAP), paint.size(LINE), paint.size(RADIUS), paint.size(FACE));
        surface(paint, rect, radius, PLight::Soft, "");
        self.face(paint, PRect::new(rect.center().x - size / 2.0, rect.origin.y + gap / 2.0, size, size));
        let under = [(self.name(), paint.color(INK)), (self.note(), paint.color(DIM))];
        under.into_iter().enumerate().for_each(|(index, (said, color))| {
            let said = said.fit(rect.size.width - gap, paint);
            paint.text(PPoint::new(rect.center().x - paint.measure(&said) / 2.0, rect.origin.y + gap + size + index as f32 * line), &said, color);
        });
    }
}

/// Cards that belong together.
pub trait PDeck {
    /// Its cards, in order.
    fn card(&self) -> Vec<&dyn PCard>;

    /// Draw them from a place in rows as wide as `wide`: as many across
    /// as fit, sharing the width evenly. Where each card stands, and
    /// how far down they end.
    fn draw(&self, paint: &mut dyn PPaint, at: PPoint, wide: f32) -> (Vec<PRect>, f32) {
        let (gap, list) = (paint.size(GAP), self.card());
        let least = paint.size(CARD).min(wide).max(1.0);
        let across = ((wide + gap / 2.0) / (least + gap / 2.0)).floor().max(1.0);
        let card = (wide - (across - 1.0) * gap / 2.0) / across;
        let high = list.iter().map(|one| one.high(paint)).fold(0.0, f32::max);
        let place = |index: usize| {
            let (column, row) = ((index % across as usize) as f32, (index / across as usize) as f32);
            PRect::new(at.x + column * (card + gap / 2.0), at.y + row * (high + gap / 2.0), card, high)
        };
        let rect: Vec<PRect> = (0..list.len()).map(place).collect();
        list.iter().zip(&rect).for_each(|(one, rect)| one.card(paint, *rect));
        let rows = list.len().div_ceil(across as usize) as f32;
        (rect, at.y + rows * high + (rows - 1.0).max(0.0) * gap / 2.0)
    }
}

impl<T: PCard> PDeck for [T] {
    fn card(&self) -> Vec<&dyn PCard> {
        self.iter().map(|one| one as &dyn PCard).collect()
    }
}

/// A cell of a sheet or none, a name, and a line: a card.
impl PCard for (Option<(&str, u32, u32)>, &str, &str) {
    fn note(&self) -> &str {
        self.2
    }
}

impl PFace for (Option<(&str, u32, u32)>, &str, &str) {
    fn cell(&self) -> Option<(&str, u32, u32)> {
        self.0
    }

    fn name(&self) -> &str {
        self.1
    }
}
