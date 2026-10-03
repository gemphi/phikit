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
use crate::v2::theme::{BPreferenceControl, BStyleParser, BUiStyleDefaults};
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

    /// The row as the Properties host draws one: a `BCtrl` of the kind this
    /// preference is, and the reset beside it.
    ///
    /// Preferences ARE properties. The model half of this file earns its keep —
    /// `BStyleDescriptor` parses, validates, bounds and formats, which the
    /// Properties controls do not do — but the VIEW half was a second control
    /// family: a number was a text field with a "Decrease" and an "Increase"
    /// button either side of it, where `BCtrlKind::Number` already draws a
    /// field with a step on each side; a choice was one button per option in a
    /// row, where `Select` is a dropdown; a toggle was a hand-built box glyph,
    /// where `Check` is the checkbox. Three widgets reimplemented beside three
    /// that existed, and the copies were the ones without the stepper.
    ///
    /// The bounds come from the descriptor's own parser, so the control cannot
    /// step past what the descriptor would reject.
    pub fn view(self, s: &BState) -> BView {
        BView::row(vec![BView::ctrl(self.editor.ctrl(s)), self.reset.view()])
    }
}

impl BPreferenceEditor {
    /// The control this preference edits, as the rest of the app spells one.
    fn ctrl(self, s: &BState) -> crate::v2::view::prim::BCtrl {
        use crate::v2::state::BValue;
        use crate::v2::view::prim::{BCtrl, BCtrlKind, BOption};
        let row = match self {
            Self::Field(field) => field.row(),
            Self::Selector(selector) => selector.row(),
            Self::Toggle(toggle) => toggle.row(),
        };
        let key = row.key().clone();
        match row.parser() {
            // A hundredth of the range per step, or one whole unit where the
            // value is in units — the same step the hand-built pair took.
            BStyleParser::Number { min, max } => {
                let step = if row.metadata().unit.is_some() { 1.0 } else { (max - min) / 100.0 };
                BCtrl::read(BCtrlKind::Number { min, max, step }, key, BValue::Float(min), s)
            }
            BStyleParser::Bool => BCtrl::read(BCtrlKind::Check, key, BValue::Bool(false), s),
            BStyleParser::ColorHsla => BCtrl::read(BCtrlKind::Color, key, BValue::Text(String::new()), s),
            // Shadow and Font are choices: the descriptor lists them.
            BStyleParser::Shadow | BStyleParser::Font => {
                let options = row.descriptor().choices().iter().map(|choice| BOption {
                    key: choice.value.to_string(),
                    label: choice.label.to_string(),
                    icon: String::new(),
                });
                let first = row.descriptor().choices().first().map(|c| c.value).unwrap_or_default();
                BCtrl::read(BCtrlKind::Select, key, BValue::Enum(first.to_string()), s).with_options(options)
            }
        }
    }
}
