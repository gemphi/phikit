//! What a `BPaneType` is called.
//!
//! The level's *content* — its names — lives here; `mod.rs` holds the
//! type and its structure. Every level keeps the same split, so there is
//! one place to look whichever level you are in.

use super::BPaneType;

impl BPaneType {
    pub fn label(self) -> &'static str {
        match self {
            BPaneType::Left => "Left",
            BPaneType::Center => "Center",
            BPaneType::Right => "Right",
        }
    }
}
