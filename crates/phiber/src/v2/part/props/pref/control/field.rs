use super::super::BPreferenceRow;
use crate::v2::theme::{BStyleParseError, BStyleParser, BStyleValue};
use crate::v2::view::{BAction, BView};

#[derive(Clone, Copy)]
pub struct BPreferenceField {
    row: BPreferenceRow,
    value: BStyleValue,
}

impl BPreferenceField {
    /// The preference this edits, for the control the view builds from it.
    pub(super) const fn row(self) -> BPreferenceRow {
        self.row
    }

    /// The effective value this edits — the override if there is one, the
    /// palette's default if not (`BPreferenceControls::for_row` resolves it).
    pub(super) const fn value(self) -> BStyleValue {
        self.value
    }

    pub(super) fn new(row: BPreferenceRow, value: BStyleValue) -> Self {
        Self { row, value }
    }

    pub fn text(self) -> String {
        let value = self
            .row
            .descriptor()
            .format_value(self.value)
            .expect("descriptor must format its own value");
        match self.row.metadata().unit {
            Some(unit) => format!("{value} {unit}"),
            None => value,
        }
    }

    pub fn commit(self, raw: &str) -> Result<BAction, BStyleParseError> {
        let raw = match self.row.metadata().unit {
            Some(unit) => raw
                .trim()
                .strip_suffix(unit)
                .map(str::trim_end)
                .unwrap_or(raw),
            None => raw,
        };
        let value = self.row.parse_and_validate(raw)?;
        self.action_for(value).ok_or(BStyleParseError::Malformed)
    }

    pub fn decrease_action(self) -> BAction {
        self.step_action(-1.0)
    }

    pub fn increase_action(self) -> BAction {
        self.step_action(1.0)
    }

    fn step_action(self, direction: f32) -> BAction {
        let next = match (self.row.parser(), self.value) {
            (BStyleParser::Number { min, max }, BStyleValue::Number(value)) => {
                let step = if self.row.metadata().unit.is_some() {
                    1.0
                } else {
                    (max - min) / 100.0
                };
                BStyleValue::Number((value + direction * step).clamp(min, max))
            }
            (BStyleParser::ColorHsla, BStyleValue::Color((h, s, l, a))) => {
                BStyleValue::Color((h, s, (l + direction).clamp(0.0, 100.0), a))
            }
            _ => unreachable!("field metadata must describe a number or color"),
        };
        self.action_for(next)
            .expect("stepped field value must pass descriptor validation")
    }

    fn action_for(self, value: BStyleValue) -> Option<BAction> {
        self.row
            .descriptor()
            .format_value(value)
            .map(|encoded| BAction::Set(self.row.key().clone(), encoded))
    }

    pub fn view(self) -> BView {
        BView::field(self.text())
    }
}
