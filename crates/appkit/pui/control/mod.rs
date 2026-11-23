//! A control: a view a hand operates. It holds a value and sends an action
//! when the value changes. `PControl: PView`. One file a control: a button, a
//! toggle, a field; a slider, a stepper and a segment to come.
//!
//! The earlier appkit's `NSControl` was a struct holding a view, an action,
//! a target and a tag. Here the value has a type of its own, and the
//! action is given it.

mod button;
mod field;
mod toggle;

pub use button::PButton;
pub use field::PField;
pub use toggle::PToggle;

use crate::pui::core::{PPaint, PPoint, PView};

/// The theme's tokens a control is drawn with. What they stand for is the
/// theme's to say.
pub const FILL: &str = "control.fill";
pub const INK: &str = "control.ink";
pub const ACCENT: &str = "control.accent";
pub const INSET: &str = "control.inset";
pub const LINE: &str = "control.line";
pub const RADIUS: &str = "control.radius";
/// How wide a field's caret is. A theme without it makes the caret as
/// wide as `LINE`, the line under the field that has the keys; a theme
/// that wants no such line gives `LINE` nothing and the caret this.
pub const CARET: &str = "control.caret";
/// A button's own fill and ink. A theme without them draws a button in
/// `FILL` and `INK`.
pub const PRESS: &str = "button.fill";
pub const LABEL: &str = "button.ink";

/// How high a line of text is, when the theme says.
pub const ROW: &str = "text.line";

/// Where a control's line of text starts inside it: `INSET` in from the
/// left, and halfway between its top and bottom when the theme says how
/// high a line is, `INSET` down when it does not.
pub fn inside(paint: &dyn PPaint, height: f32) -> PPoint {
    let inset = paint.size(INSET);
    PPoint::new(inset, paint.theme().size(ROW).map_or(inset, |line| (height - line) / 2.0))
}

/// What a control runs when a hand changes its value.
pub type PAction<V> = Box<dyn FnMut(&V)>;

/// A view a hand operates: it holds a value and sends an action when the
/// value changes.
pub trait PControl: PView {
    /// What the control holds: a toggle a `bool`, a slider an `f32`, a
    /// field a `String`.
    type Value: Clone + PartialEq;

    fn value(&self) -> Self::Value;

    /// Keep a value without telling anyone: what the app does to show a
    /// value of its own. A hand's change goes through `change`.
    fn store(&mut self, value: Self::Value);

    /// Whether a hand can change it.
    fn enabled(&self) -> bool;

    fn set_enabled(&mut self, enabled: bool);

    /// What to run when a hand changes the value, if anything.
    fn action(&mut self) -> Option<&mut PAction<Self::Value>>;

    /// Say what to run when a hand changes the value.
    fn set_action(&mut self, action: PAction<Self::Value>);

    /// A hand set the value: it is kept and the action is sent. Nothing
    /// happens, and the answer is `false`, when the control is disabled or
    /// already holds that value.
    fn change(&mut self, value: Self::Value) -> bool {
        if !self.enabled() || self.value() == value {
            return false;
        }
        self.store(value.clone());
        if let Some(action) = self.action() {
            action(&value);
        }
        true
    }
}
