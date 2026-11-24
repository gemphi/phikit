//! The tabs of a person's profile and the tab that is shown, as
//! Phiegg's profile has them (`data/profile.tsv`): their pictures in a
//! grid, each opening in the viewer; the clubs they hold, the spaces
//! they are in and who they are linked to as the kit's cards, in rows
//! that fold to the column's width. No list of thin rows.

use appkit::pui::block::PDeck;
use appkit::pui::core::{PFit, PPaint, PPoint, PRect};

use super::{ACROSS, Act, Person};
use crate::data;
use crate::part::{DIM, GAP, LINE, RADIUS, Shot};
use crate::site::Member;

/// A tab: what it shows, and its word (the table's first column names
/// the row for whoever reads the table).
pub(super) struct Pane {
    pub show: String,
    pub word: String,
}

/// The tabs, in the table's order.
pub(super) fn panes() -> Vec<Pane> {
    data::row(data::PROFILE).into_iter().filter(|row| row.len() >= 3).map(|row| Pane { show: row[1].to_string(), word: row[2].to_string() }).collect()
}

impl Person {
    /// How much a tab holds for somebody.
    pub(super) fn holds(&self, pane: &Pane, one: &Member, shot: &[Shot]) -> usize {
        match pane.show.as_str() {
            "picture" => shot.len(),
            "club" => one.club.len(),
            "space" => one.space.len(),
            "linked" => one.link as usize,
            _ => 0,
        }
    }

    /// The body of the tab shown, from `at` down; how far it goes.
    pub(super) fn body(&self, paint: &mut dyn PPaint, at: PPoint, wide: f32, one: &Member, shot: &[Shot], pane: &Pane, spot: &mut Vec<(PRect, Act)>) -> f32 {
        let (gap, line, dim) = (paint.size(GAP), paint.size(LINE), paint.color(DIM));
        let text = &self.desk.text;
        let count = |word: &str, n: u64| text.get(word).replace("{n}", &crate::part::tool::grouped(n));
        let end = match pane.show.as_str() {
            "picture" => {
                // Square pictures, as many across as the theme says.
                let across = paint.size(ACROSS).max(1.0);
                let side = (wide - (across - 1.0) * gap / 2.0) / across;
                for (index, one) in shot.iter().enumerate() {
                    let rect = PRect::new(at.x + (index % across as usize) as f32 * (side + gap / 2.0), at.y + (index / across as usize) as f32 * (side + gap / 2.0), side, side);
                    paint.picture(&one.key, rect, paint.size(RADIUS), PFit::Cover);
                    spot.push((rect, Act::Shot(index)));
                }
                at.y + shot.len().div_ceil(across as usize) as f32 * (side + gap / 2.0)
            }
            "club" => {
                let note: Vec<String> = one.club.iter().map(|club| count("person.holder", club.holder)).collect();
                let card: Vec<(Option<(&str, u32, u32)>, &str, &str)> = one.club.iter().zip(&note).map(|(club, note)| (Some((club.mark.sheet.as_str(), club.mark.column, club.mark.index)), club.name.as_str(), note.as_str())).collect();
                card.as_slice().draw(paint, at, wide).1
            }
            "space" => {
                let venue: Vec<_> = one.space.iter().map(|space| self.desk.site.venue.iter().find(|venue| venue.label == *space)).collect();
                let note: Vec<String> = venue.iter().map(|venue| venue.filter(|venue| venue.count > 0).map(|venue| count("person.inside", venue.count)).unwrap_or_default()).collect();
                let card: Vec<(Option<(&str, u32, u32)>, &str, &str)> = one.space.iter().zip(&venue).zip(&note)
                    .map(|((space, venue), note)| (venue.and_then(|venue| venue.mark.as_ref()).map(|cell| (cell.sheet.as_str(), cell.column, cell.index)), space.as_str(), note.as_str())).collect();
                let (rect, end) = card.as_slice().draw(paint, at, wide);
                let first = venue.iter().map(|venue| venue.and_then(|venue| venue.plate.first()).map(|plate| plate.id.clone()));
                spot.extend(rect.into_iter().zip(first).filter_map(|(rect, room)| Some((rect, Act::Go(room?)))));
                end
            }
            "linked" => {
                let list = self.linked(&one.id);
                let other: Vec<&Member> = list.iter().filter_map(|id| self.desk.site.member(id)).collect();
                let handle: Vec<String> = other.iter().map(|other| format!("@{}", other.handle)).collect();
                let card: Vec<(Option<(&str, u32, u32)>, &str, &str)> = other.iter().zip(&handle)
                    .map(|(other, handle)| (self.desk.site.face(&other.id).map(|cell| (cell.sheet.as_str(), cell.column, cell.index)), other.name.as_str(), handle.as_str())).collect();
                let (rect, end) = card.as_slice().draw(paint, at, wide);
                spot.extend(rect.into_iter().zip(other.iter().map(|other| Act::Open(other.id.clone()))));
                end
            }
            _ => at.y,
        };
        if end <= at.y {
            paint.text(at, text.get("person.none"), dim);
            return at.y + line * 2.0;
        }
        end + line
    }
}
