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

    /// The kind `label` names, for a key that carries the label
    /// (`View3dKey::Camera(scope, slot)`).
    pub fn from_label(label: &str) -> Option<Self> {
        [Self::Top, Self::Left, Self::Center, Self::Right, Self::Bottom].into_iter().find(|k| k.label() == label)
    }
}
