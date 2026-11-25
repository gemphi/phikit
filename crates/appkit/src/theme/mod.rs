//! What everything looks like: tokens resolved to an appearance. `PTheme`.
//!
//! A view never names a colour or a size outright. It names a token, such
//! as `control.fill`, and the theme says what that is today. `PLook` is a
//! theme read from a table an app keeps in its `data/`, and `PShade` is
//! several looks of which one is shown: light, or dark.

mod look;
mod shade;

pub use look::PLook;
pub use shade::{PShade, lightness};

use crate::pui::core::PColor;

/// What everything looks like: a token goes in, an appearance comes out.
pub trait PTheme {
    /// The colour a token stands for, or `None` for a token the theme does
    /// not have.
    fn color(&self, token: &str) -> Option<PColor>;

    /// The size a token stands for: a spacing, a radius, a type size.
    fn size(&self, token: &str) -> Option<f32>;

    /// The word a token stands for: the name of a typeface, the path of
    /// its file. A theme of colours and sizes alone has none.
    fn word(&self, _token: &str) -> Option<&str> {
        None
    }

    /// The system's own light or dark, as the window sees it now: a theme
    /// that follows it (a look on Auto) turns to match. `true` when it
    /// turned. A theme that does not follow it ignores it.
    fn system(&self, _dark: bool) -> bool {
        false
    }
}
