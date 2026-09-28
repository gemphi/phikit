//! B-prefixed Preferences controls backed exclusively by style descriptors.
//!
//! View composition stays buikit-compatible. Parsing, validation, choices,
//! and canonical formatting remain in the shared `BStyleDescriptor` catalog.

mod field;
mod selector;
mod toggle;

pub use field::BPreferenceField;
pub use selector::BPreferenceSelector;
pub use toggle::BPreferenceToggle;

use super::BPreferenceRow;
use crate::v2::state::BState;
use crate::v2::theme::{BPreferenceControl, BUiStyleDefaults};
use crate::v2::view::{BAction, BControl, BView};

#[derive(Clone, Copy)]
pub struct BPreferenceReset {
    row: BPreferenceRow,
    enabled: bool,
}

impl BPreferenceReset {
    pub const fn enabled(self) -> bool {
        self.enabled
    }

    pub fn action(self) -> BAction {
        BAction::Reset(self.row.key().clone())
    }

    pub fn view(self) -> BView {
        BView::Button(
            BControl::new()
                .with_icon("rotate_ccw")
                .with_label("Restore Default")
                .with_action(self.action()),
        )
    }
}

#[derive(Clone, Copy)]
pub enum BPreferenceEditor {
    Field(BPreferenceField),
    Selector(BPreferenceSelector),
    Toggle(BPreferenceToggle),
}

impl BPreferenceEditor {
    pub fn view(self) -> BView {
        match self {
            Self::Field(field) => field.view(),
            Self::Selector(selector) => selector.view(),
            Self::Toggle(toggle) => toggle.view(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct BPreferenceControls {
    pub editor: BPreferenceEditor,
    pub reset: BPreferenceReset,
}

impl BPreferenceControls {
    pub fn for_row(row: BPreferenceRow, state: &BState, defaults: &BUiStyleDefaults) -> Self {
        let value = state
            .get(row.key())
            .and_then(|raw| row.parse_and_validate(raw).ok())
            .unwrap_or_else(|| row.default_value(defaults));
        let editor = match row.metadata().control {
            BPreferenceControl::Number | BPreferenceControl::Color => {
                BPreferenceEditor::Field(BPreferenceField::new(row, value))
            }
            BPreferenceControl::Choice => {
                BPreferenceEditor::Selector(BPreferenceSelector::new(row, value))
            }
            BPreferenceControl::Toggle => {
                BPreferenceEditor::Toggle(BPreferenceToggle::new(row, value))
            }
        };
        Self {
            editor,
            reset: BPreferenceReset {
                row,
                enabled: state.get(row.key()).is_some(),
            },
        }
    }

    pub fn view(self) -> BView {
        let mut list = match self.editor {
            BPreferenceEditor::Field(field) => vec![
                field.view(),
                BView::Button(
                    BControl::new()
                        .with_icon("minus")
                        .with_label("Decrease")
                        .with_action(field.decrease_action()),
                ),
                BView::Button(
                    BControl::new()
                        .with_icon("plus")
                        .with_label("Increase")
                        .with_action(field.increase_action()),
                ),
            ],
            editor => vec![editor.view()],
        };
        list.push(self.reset.view());
        BView::row(list)
    }
}
