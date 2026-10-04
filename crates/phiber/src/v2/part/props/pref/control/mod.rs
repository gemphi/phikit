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
use crate::v2::theme::{BPreferenceControl, BStyleParser, BStyleValue, BUiStyleDefaults};
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
    /// The effective value, whichever editor holds it.
    fn value(self) -> BStyleValue {
        match self {
            Self::Field(field) => field.value(),
            Self::Selector(selector) => selector.value(),
            Self::Toggle(toggle) => toggle.value(),
        }
    }

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
        // The value `for_row` already resolved — the stored override, or the
        // palette's default for this preference. `BCtrl::read` takes it as the
        // DEFAULT, so a stored value still wins; what it replaces is the
        // placeholder this used to pass.
        //
        // The placeholder was a real bug, not untidiness. An un-overridden
        // colour reached the renderer as an empty string, `BCtrlSwatch::of` read
        // no colour out of it, and the swatch drew nothing — for all 29 colours
        // in Preferences until the user changed one. A number read as its
        // minimum, so every unset slider sat at the far left.
        let effective = self.value();
        match row.parser() {
            // A hundredth of the range per step, or one whole unit where the
            // value is in units — the same step the hand-built pair took.
            BStyleParser::Number { min, max } => {
                let step = if row.metadata().unit.is_some() { 1.0 } else { (max - min) / 100.0 };
                let at = match effective {
                    BStyleValue::Number(value) => value,
                    _ => min,
                };
                BCtrl::read(BCtrlKind::Number { min, max, step }, key, BValue::Float(at), s)
            }
            BStyleParser::Bool => {
                let on = matches!(effective, BStyleValue::Bool(true));
                BCtrl::read(BCtrlKind::Check, key, BValue::Bool(on), s)
            }
            // In the descriptor's own spelling, which is what the swatch reads
            // (`ctrl::color` → `BCtrlSwatch::of`).
            BStyleParser::ColorHsla => {
                let text = row.descriptor().format_value(effective).unwrap_or_default();
                BCtrl::read(BCtrlKind::Color, key, BValue::Text(text), s)
            }
            // Shadow and Font are choices: the descriptor lists them.
            BStyleParser::Shadow | BStyleParser::Font => {
                let options = row.descriptor().choices().iter().map(|choice| BOption {
                    key: choice.value.to_string(),
                    label: choice.label.to_string(),
                    icon: String::new(),
                });
                let chosen = row
                    .descriptor()
                    .format_value(effective)
                    .or_else(|| row.descriptor().choices().first().map(|c| c.value.to_string()))
                    .unwrap_or_default();
                BCtrl::read(BCtrlKind::Select, key, BValue::Enum(chosen), s).with_options(options)
            }
        }
    }
}
