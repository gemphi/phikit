//! Actions — the store writes every primitive dispatches. A `BAction` is a
//! pure value: the path to write and the write semantics. No closures, no
//! callbacks — the view layer stays declarative and state-driven.

use super::control::BControl;
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

/// `PartialEq`/`Eq` are gone with the bool: `BControl` has never had them,
/// nothing ever compared a zone toggle, and `BView` itself derives only
/// `Clone, Debug` — the same call `BMenuItem` got when its derive turned out
/// to be a wrong answer waiting for a caller.
#[derive(Clone, Debug)]
pub struct BZoneToggle {
    pub zone: BZoneType,
    /// The control the toggle IS, rather than a bool beside an action.
    ///
    /// It carried `visible: bool` and a bare `BAction`, so `draw/toggle.rs`
    /// decided accent-against-dim for itself and a zone toggle was the one
    /// "on" in v2 that could never be `active` — outside the vocabulary every
    /// other control answers in (`active`, `muted`, `tooltip`), and so
    /// outside anything that reads it. The state lives on the control now and
    /// the glyph reads it back.
    ///
    /// What the glyph draws is unchanged: the zone's shape says on or off,
    /// and the button keeps its hover fill alone. Whether an `active` zone
    /// toggle should also take the active FILL, as every other control with
    /// `active` does, is a look question and not this change — the state is
    /// in one place now, so it is a one-line answer whenever it is wanted.
    pub control: BControl,
}

impl BView {
    pub fn zone_toggle_group(controls: Vec<BZoneToggle>) -> Self {
        Self::ZoneToggleGroup { controls }
    }
}
