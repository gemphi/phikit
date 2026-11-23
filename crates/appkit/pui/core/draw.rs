//! The root of everything on screen: a thing that draws itself.

use super::{PPaint, PPoint};

/// Anything that draws itself. A view is one; so is a mark or a border
/// that takes no events and holds nothing.
pub trait PDraw {
    /// Draw this, and not what it holds, with its top left at `origin` in
    /// the window. Colours come from the painter's theme, never from here.
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint);
}
