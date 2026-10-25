//! Text primitives — labels, value read-outs and search fields.

use crate::v2::view::BView;
use crate::v2::view::prim::BAction;

impl BView {
    pub fn label(text: impl Into<String>) -> Self {
        Self::Label { text: text.into() }
    }
    /// A read-out's caption and figure, kept apart for the two shades.
    pub fn pair(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self::Pair { label: label.into(), value: value.into() }
    }
    /// Rotated-90° effect — column of characters.
    pub fn vlabel(text: impl Into<String>) -> Self {
        Self::VLabel { text: text.into() }
    }
    /// Bordered value read-out for property rows.
    pub fn field(text: impl Into<String>) -> Self {
        Self::Field { text: text.into() }
    }
    /// Text field the user can type into; `editing` draws the caret.
    /// `on_change` must be `BAction::Set(path, _)` naming the key to type into.
    pub fn search_field(
        placeholder: impl Into<String>,
        value: impl Into<String>,
        on_change: BAction,
    ) -> Self {
        Self::SearchField {
            placeholder: placeholder.into(),
            value: value.into(),
            editing: false,
            on_change,
            searches: true,
        }
    }

    /// A filter typed into: the same field, led by the magnifier.
    pub fn filter_field(placeholder: impl Into<String>, value: impl Into<String>, editing: bool, on_change: BAction) -> Self {
        match Self::text_field(placeholder, value, editing, on_change) {
            Self::SearchField { placeholder, value, editing, on_change, .. } => Self::SearchField { placeholder, value, editing, on_change, searches: true },
            other => other,
        }
    }

    /// The same field, told whether it currently holds the caret.
    pub fn text_field(
        placeholder: impl Into<String>,
        value: impl Into<String>,
        editing: bool,
        on_change: BAction,
    ) -> Self {
        Self::SearchField {
            placeholder: placeholder.into(),
            value: value.into(),
            editing,
            on_change,
            searches: false,
        }
    }
}
