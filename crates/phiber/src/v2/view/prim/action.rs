//! Actions — the store writes every primitive dispatches. A `BAction` is a
//! pure value: the path to write and the write semantics. No closures, no
//! callbacks — the view layer stays declarative and state-driven.

use crate::v2::state::BPath;
use crate::v2::view::BView;
use crate::v2::zone::BZoneType;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum BAction {
    Set(BPath, String),
    /// `Set` without committing: the value changes and no overlay is dismissed.
    ///
    /// A `Set` is a decision, and a decision takes every open menu and picker
    /// down with it — picking GPU Compute closes the list it was picked from. An
    /// adjustment is not a decision. Stepping a colour's hue, or dragging a
    /// slider or a node across the canvas, happens *inside* something that is
    /// still open and goes on happening, so dismissing on the first write means
    /// it shuts after one nudge and has to be reopened for the next one.
    /// Blender keeps its colour picker up while you move the wheel and closes it
    /// when you click away, which is what the popover's own dismiss action
    /// already does here.
    ///
    /// The value is written exactly as `Set` writes it — same `before_set`, same
    /// `after_set` rules. The only difference is that nothing closes.
    Adjust(BPath, String),
    Toggle(BPath),
    Open(BPath),
    /// Remove an override so subsequent reads use their declared fallback.
    Reset(BPath),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BZoneToggle {
    pub zone: BZoneType,
    pub visible: bool,
    pub action: BAction,
}

impl BView {
    pub fn zone_toggle_group(controls: Vec<BZoneToggle>) -> Self {
        Self::ZoneToggleGroup { controls }
    }
}
