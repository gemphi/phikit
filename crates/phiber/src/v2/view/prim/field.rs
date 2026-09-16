//! Text primitives — labels, value read-outs and search fields.

use crate::v2::view::BView;
use crate::v2::view::prim::BAction;

impl BView {
    pub fn label(text: impl Into<String>) -> Self {
        Self::Label { text: text.into() }
    }
    /// Rotated-90° effect — column of characters.
    pub fn vlabel(text: impl Into<String>) -> Self {
        Self::VLabel { text: text.into() }
    }
    /// Bordered value read-out for property rows.
    pub fn field(text: impl Into<String>) -> Self {
        Self::Field { text: text.into() }
    }
    /// Interactive search / text-entry field.
    ///
    /// Displays `value` when non-empty, `placeholder` otherwise (dimmed).
    /// `on_change` is fired on click to clear, giving immediate feedback; it
    /// carries the target `BPath` so the tree re-filters on the next render.
    /// Must be `BAction::Set(path, _)`.
    pub fn search_field(
        placeholder: impl Into<String>,
        value: impl Into<String>,
        on_change: BAction,
    ) -> Self {
        Self::SearchField {
            placeholder: placeholder.into(),
            value: value.into(),
            on_change,
        }
    }
}
