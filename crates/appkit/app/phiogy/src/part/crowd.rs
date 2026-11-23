//! People shown together: each one a seat, which the kit draws as a
//! card (`appkit::pui::block::PCard`), all of them as a deck. This file
//! has no drawing: it says who they are, as the app's own file has
//! them, and which card is whose. Used wherever people are shown
//! together: who is in a voice room, a room's members.

use appkit::pui::block::{PCard, PFace};
use appkit::pui::core::PRect;

use super::Desk;

/// One of a crowd: their id among the app's people, nothing for one
/// who is not of them; their name; a line about them; and their
/// picture, as a cell of a sheet, where the app has one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seat {
    pub id: String,
    pub name: String,
    pub said: String,
    pub cell: Option<(String, u32, u32)>,
}

impl PFace for Seat {
    fn cell(&self) -> Option<(&str, u32, u32)> {
        self.cell.as_ref().map(|(sheet, column, index)| (sheet.as_str(), *column, *index))
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl PCard for Seat {
    fn note(&self) -> &str {
        &self.said
    }
}

impl Desk {
    /// The picture of the person of that id, as a cell of a sheet.
    pub fn cell(&self, id: &str) -> Option<(&str, u32, u32)> {
        self.site.face(id).map(|cell| (cell.sheet.as_str(), cell.column, cell.index))
    }

    /// A seat for one of that id and name, with a line about them.
    pub fn seat(&self, id: &str, name: &str, said: &str) -> Seat {
        let cell = self.cell(id).map(|(sheet, column, index)| (sheet.to_string(), column, index));
        Seat { id: id.to_string(), name: name.to_string(), said: said.to_string(), cell }
    }
}

/// The people a list of names stands for, as the app's own file has
/// them: each with their id and how they are.
pub fn named(desk: &Desk, name: &[String]) -> Vec<Seat> {
    let one = |name: &String| {
        let member = desk.site.member.iter().find(|member| member.name == *name);
        desk.seat(member.map_or("", |member| member.id.as_str()), name, member.map_or("", |member| member.state.as_str()))
    };
    name.iter().map(one).collect()
}

/// Whose card each place is: the places a deck was drawn at, with the
/// id of each one who is of the app's people.
pub fn whose(rect: Vec<PRect>, seat: &[Seat]) -> Vec<(PRect, String)> {
    rect.into_iter().zip(seat).filter(|(_, one)| !one.id.is_empty()).map(|(rect, one)| (rect, one.id.clone())).collect()
}
