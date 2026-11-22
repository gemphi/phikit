//! The pane of a person's profile that is shown: who they are linked
//! to, each a row that opens them; the spaces they are in; or the
//! pictures they posted, in rows, a press on one opening it in the
//! viewer.

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
                    if let Some(cell) = self.desk.site.face(id) {
                        paint.picture(&cell.sheet, PRect::new(at.x, top, line, line), line / 2.0, PFit::Cell { column: cell.column, index: cell.index });
                    }
                    paint.text(PPoint::new(at.x + line + gap / 2.0, top), &fit(&other.name, wide - line - gap / 2.0, paint), ink);
                    spot.push((PRect::new(at.x, top, wide, line), Act::Open(id.clone())));
                    top += line + gap / 4.0;
                }
                list.is_empty().then(|| none(paint, top));
            }
            "space" => {
                one.space.iter().enumerate().for_each(|(index, space)| paint.text(PPoint::new(at.x, top + index as f32 * line), &fit(space, wide, paint), ink));
                one.space.is_empty().then(|| none(paint, top));
                top += one.space.len() as f32 * line;
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
