//! What a `BAreaType` is called.
//!
//! The level's *content* — its names — lives here; `mod.rs` holds the
//! type and its structure. Every level keeps the same split, so there is
//! one place to look whichever level you are in.

use super::BAreaType;

impl BAreaType {
    pub fn label(self) -> &'static str {
        match self {
            BAreaType::Top => "Top",
            BAreaType::Left => "Left",
            BAreaType::Center => "Center",
            BAreaType::Right => "Right",
            BAreaType::Bottom => "Bottom",
        }
    }
}
