use super::super::BPreferenceRow;
use crate::v2::theme::BStyleValue;
use crate::v2::view::{BAction, BControl, BView};

#[derive(Clone, Copy)]
pub struct BPreferenceToggle {
    row: BPreferenceRow,
    checked: bool,
}

impl BPreferenceToggle {
    /// The preference this edits, for the control the view builds from it.
    pub(super) const fn row(self) -> BPreferenceRow {
        self.row
    }

    /// The effective value this edits, back in the descriptor's own terms.
    pub(super) const fn value(self) -> BStyleValue {
        BStyleValue::Bool(self.checked)
    }

    pub(super) fn new(row: BPreferenceRow, value: BStyleValue) -> Self {
        let BStyleValue::Bool(checked) = value else {
            unreachable!("toggle metadata must describe a boolean value")
        };
        Self { row, checked }
    }

    pub const fn checked(self) -> bool {
        self.checked
    }

    pub fn action(self) -> BAction {
        BAction::Set(self.row.key().clone(), (!self.checked).to_string())
    }

    pub fn view(self) -> BView {
        let icon = if self.checked {
            "check_square"
        } else {
            "square"
        };
        BView::Button(
            BControl::new()
                .with_icon(icon)
                .with_label(self.row.metadata().label)
                .with_action(self.action()),
        )
    }
}
