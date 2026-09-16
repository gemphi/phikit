use super::super::BPreferenceRow;
use crate::v2::theme::BStyleValue;
use crate::v2::view::{BAction, BView};

#[derive(Clone, Copy)]
pub struct BPreferenceToggle {
    row: BPreferenceRow,
    checked: bool,
}

impl BPreferenceToggle {
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
        BView::button_action(icon, self.row.metadata().label, self.action())
    }
}
