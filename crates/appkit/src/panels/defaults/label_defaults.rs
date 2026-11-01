//! Text label defaults for different UI contexts.
//!
//! Provides factory methods for creating pre-styled text labels
//! (headers, secondary, tertiary, values, row labels/values).

use crate::prelude::*;
use crate::theme::{self, keys};

/// Text label defaults for different UI contexts.
pub struct LabelDefaults;

impl LabelDefaults {
    /// Header text (panel titles, section headers)
    pub fn header(text: &str) -> NSTextField {
        let mut lbl = NSTextField::label(text);
        lbl.set_font_size(NSLayoutConstants::FONT_SIZE_LARGE);
        lbl.set_text_color(theme::color(keys::CLR_TEXT_LABEL));
        lbl
    }

    /// Secondary text (subsection labels)
    pub fn secondary(text: &str) -> NSTextField {
        let mut lbl = NSTextField::label(text);
        lbl.set_font_size(NSLayoutConstants::FONT_SIZE_SMALL);
        lbl.set_text_color(theme::color(keys::CLR_TEXT_SECONDARY));
        lbl
    }

    /// Tertiary text (placeholder, disabled)
    pub fn tertiary(text: &str) -> NSTextField {
        let mut lbl = NSTextField::label(text);
        lbl.set_font_size(NSLayoutConstants::FONT_SIZE_SMALL);
        lbl.set_text_color(theme::color(keys::CLR_TEXT_TERTIARY));
        lbl
    }

    /// Value text (numbers, names)
    pub fn value(text: &str) -> NSTextField {
        let mut lbl = NSTextField::label(text);
        lbl.set_font_size(NSLayoutConstants::FONT_SIZE_LABEL);
        lbl.set_text_color(theme::color(keys::CLR_TEXT_LABEL));
        lbl
    }

    /// Row label (property names in inspector rows)
    pub fn row_label(text: &str) -> NSTextField {
        let mut lbl = NSTextField::label(text);
        lbl.set_font_size(NSLayoutConstants::FONT_SIZE_SMALL);
        lbl.set_text_color(theme::color(keys::CLR_TEXT_SECONDARY));
        lbl
    }

    /// Row value (property values in inspector rows)
    pub fn row_value(text: &str) -> NSTextField {
        let mut lbl = NSTextField::label(text);
        lbl.set_font_size(NSLayoutConstants::FONT_SIZE_SMALL);
        lbl.set_text_color(theme::color(keys::CLR_TEXT_LABEL));
        lbl
    }
}
