use super::super::BPreferenceRow;
use crate::v2::theme::{BStyleChoice, BStyleParseError, BStyleValue};
use crate::v2::view::{BAction, BControl, BView};

#[derive(Clone, Copy)]
pub struct BPreferenceSelector {
    row: BPreferenceRow,
    value: BStyleValue,
}

impl BPreferenceSelector {
    /// The preference this edits, for the control the view builds from it.
    pub(super) const fn row(self) -> BPreferenceRow {
        self.row
    }

    /// The effective value this edits (`BPreferenceControls::for_row`).
    pub(super) const fn value(self) -> BStyleValue {
        self.value
    }

    pub(super) fn new(row: BPreferenceRow, value: BStyleValue) -> Self {
        Self { row, value }
    }

    pub fn choices(self) -> &'static [BStyleChoice] {
        self.row.descriptor().choices()
    }

    pub fn selected(self) -> String {
        self.row
            .descriptor()
            .format_value(self.value)
            .expect("descriptor must format its own choice")
    }

    pub fn select(self, value: &str) -> Result<BAction, BStyleParseError> {
        if !self.choices().iter().any(|choice| choice.value == value) {
            return Err(BStyleParseError::Malformed);
        }
        let parsed = self.row.parse_and_validate(value)?;
        let encoded = self
            .row
            .descriptor()
            .format_value(parsed)
            .ok_or(BStyleParseError::Malformed)?;
        Ok(BAction::Set(self.row.key().clone(), encoded))
    }

    pub fn view(self) -> BView {
        BView::row(
            self.choices()
                .iter()
                .map(|choice| {
                    BView::Button(
                        BControl::new()
                            .with_label(choice.label)
                            .with_action(
                                self.select(choice.value)
                                    .expect("declared choice must pass descriptor validation"),
                            ),
                    )
                })
                .collect(),
        )
    }
}
