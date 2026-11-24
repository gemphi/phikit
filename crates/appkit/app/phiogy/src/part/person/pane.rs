//! The pane of a person's profile that is shown: who they are linked
//! to, each a row that opens them; the spaces they are in; or the
//! pictures they posted, in rows, a press on one opening it in the
//! viewer.

use appkit::pui::block::PFace;
use appkit::pui::core::{PFit, PPaint, PPoint, PRect};

use super::{ACROSS, Act, PANE, Person};
use crate::part::{DIM, GAP, INK, LINE, RADIUS, Shot, fit};
use crate::site::Member;

impl Person {
    /// The body of the pane shown, from `at` down; how far it goes.
    pub(super) fn body(&self, paint: &mut dyn PPaint, at: PPoint, wide: f32, one: &Member, shot: &[Shot], spot: &mut Vec<(PRect, Act)>) -> f32 {
        let (gap, line, ink, dim) = (paint.size(GAP), paint.size(LINE), paint.color(INK), paint.color(DIM));
        let mut top = at.y;
        let none = |paint: &mut dyn PPaint, top: f32| paint.text(PPoint::new(at.x, top), self.desk.text.get("person.none"), dim);
        match PANE[self.pane.get().min(PANE.len() - 1)] {
            "linked" => {
                let list = self.linked(&one.id);
                for id in &list {
                    let Some(other) = self.desk.site.member(id) else { continue };
                    // Each a row: their face on its ground, and their name.
                    let cell = self.desk.site.face(id).map(|cell| (cell.sheet.as_str(), cell.column, cell.index));
                    (cell, other.name.as_str()).face(paint, PRect::new(at.x + gap / 8.0, top + gap / 8.0, line, line));
                    paint.text(PPoint::new(at.x + line + gap * 0.75, top + gap / 8.0), &fit(&other.name, wide - line - gap, paint), ink);
                    spot.push((PRect::new(at.x, top, wide, line + gap / 4.0), Act::Open(id.clone())));
                    top += line + gap / 2.0;
                }
                list.is_empty().then(|| none(paint, top));
            }
            "space" => {
                // Each a row: the space's mark, its name, how many are in
                // it; a press goes to its first room.
                for space in &one.space {
                    let venue = self.desk.site.venue.iter().find(|venue| venue.label == *space);
                    let mark = venue.and_then(|venue| venue.mark.as_ref()).map(|cell| (cell.sheet.as_str(), cell.column, cell.index));
                    (mark, space.as_str()).face(paint, PRect::new(at.x + gap / 8.0, top + gap / 8.0, line, line));
                    let count = venue.filter(|venue| venue.count > 0).map(|venue| crate::part::tool::grouped(venue.count)).unwrap_or_default();
                    let room = wide - line - gap - paint.measure(&count);
                    paint.text(PPoint::new(at.x + line + gap * 0.75, top + gap / 8.0), &fit(space, room, paint), ink);
                    paint.text(PPoint::new(at.x + wide - paint.measure(&count), top + gap / 8.0), &count, dim);
                    if let Some(first) = venue.and_then(|venue| venue.plate.first()) {
                        spot.push((PRect::new(at.x, top, wide, line + gap / 4.0), Act::Go(first.id.clone())));
                    }
                    top += line + gap / 2.0;
                }
                one.space.is_empty().then(|| none(paint, top));
            }
            _ => {
                let across = paint.size(ACROSS).max(1.0);
                let side = (wide - (across - 1.0) * gap / 2.0) / across;
                for (index, one) in shot.iter().enumerate() {
                    let rect = PRect::new(at.x + (index % across as usize) as f32 * (side + gap / 2.0), top + (index / across as usize) as f32 * (side + gap / 2.0), side, side);
                    paint.picture(&one.key, rect, paint.size(RADIUS), PFit::Cover);
                    spot.push((rect, Act::Shot(index)));
                }
                shot.is_empty().then(|| none(paint, top));
                top += shot.len().div_ceil(across as usize) as f32 * (side + gap / 2.0);
            }
        }
        top + line
    }
}
