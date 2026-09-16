//! Style descriptor catalog — one row per resolved UI-style field.
//!
//! Each declaration owns one fixed state key, parser, validation bounds,
//! default reader, resolved-style writer, and UI-neutral preference metadata.
//!
//! Split for the 200-line file cap:
//!   • Types + macros live here (parent).
//!   • `parser.rs`  — `impl BStyleParser`, `impl BStyleDescriptor`, `parse_hsla`.
//!   • `table.rs`   — the `UI_STYLE_DESCRIPTORS` catalog (compact form).
//!   • `tests.rs`   — catalog invariants + parser rejection tests.
//!
//! The macros below are visible to `mod table;` because `macro_rules!` items
//! defined in a parent file are usable from every child module declared after
//! the macro definition.

use super::{BColor, BResolvedUiStyle, BShadowStyle, BUiStyleDefaults};
use crate::v2::state::BPath;

// ── Value + parser + preference type surface ──────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BStyleValue {
    Number(f32),
    Bool(bool),
    Color(BColor),
    Shadow(BShadowStyle),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BStyleParser {
    Number { min: f32, max: f32 },
    Bool,
    ColorHsla,
    Shadow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BStyleParseError {
    Malformed,
    NonFinite,
    OutOfBounds,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BPreferenceControl {
    Number,
    Toggle,
    Color,
    Choice,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BStyleChoice {
    pub value: &'static str,
    pub label: &'static str,
}

pub(crate) const SHADOW_CHOICES: &[BStyleChoice] = &[
    BStyleChoice { value: "none", label: "None" },
    BStyleChoice { value: "extra_large", label: "Extra Large" },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BPreferenceMetadata {
    pub section: &'static str,
    pub label: &'static str,
    pub control: BPreferenceControl,
    pub unit: Option<&'static str>,
}

pub struct BStyleDescriptor {
    pub key: &'static BPath,
    /// Stable field identity used by declarations, tests, and preference rows.
    pub target: &'static str,
    pub parser: BStyleParser,
    pub preference: BPreferenceMetadata,
    default: fn(&BUiStyleDefaults) -> BStyleValue,
    apply: fn(&mut BResolvedUiStyle, BStyleValue) -> bool,
    read: fn(&BResolvedUiStyle) -> BStyleValue,
}

// ── Row-builder macros ────────────────────────────────────────────────────
//
// Defined here (in the parent) so they are visible from `mod table;` below —
// the `macro_rules!` scoping rule is: macros defined before a `mod X;`
// declaration are usable inside that submodule.

macro_rules! number {
    ($key:ident, $domain:ident.$field:ident, $section:literal, $label:literal,
     $min:expr, $max:expr, $unit:expr) => {
        BStyleDescriptor {
            key: &key::$key,
            target: concat!(stringify!($domain), ".", stringify!($field)),
            parser: BStyleParser::Number { min: $min, max: $max },
            preference: BPreferenceMetadata {
                section: $section,
                label: $label,
                control: BPreferenceControl::Number,
                unit: $unit,
            },
            default: |d| BStyleValue::Number(d.$domain.$field),
            apply: |s, v| match v {
                BStyleValue::Number(v) => { s.$domain.$field = v; true }
                _ => false,
            },
            read: |s| BStyleValue::Number(s.$domain.$field),
        }
    };
}

macro_rules! color {
    ($key:ident, $field:ident, $label:literal) => {
        BStyleDescriptor {
            key: &key::$key,
            target: concat!("colors.", stringify!($field)),
            parser: BStyleParser::ColorHsla,
            preference: BPreferenceMetadata {
                section: "Colors",
                label: $label,
                control: BPreferenceControl::Color,
                unit: None,
            },
            default: |d| BStyleValue::Color(d.colors.$field),
            apply: |s, v| match v {
                BStyleValue::Color(v) => { s.colors.$field = v; true }
                _ => false,
            },
            read: |s| BStyleValue::Color(s.colors.$field),
        }
    };
}

macro_rules! toggle {
    ($key:ident, $domain:ident.$field:ident, $section:literal, $label:literal) => {
        BStyleDescriptor {
            key: &key::$key,
            target: concat!(stringify!($domain), ".", stringify!($field)),
            parser: BStyleParser::Bool,
            preference: BPreferenceMetadata {
                section: $section,
                label: $label,
                control: BPreferenceControl::Toggle,
                unit: None,
            },
            default: |d| BStyleValue::Bool(d.$domain.$field),
            apply: |s, v| match v {
                BStyleValue::Bool(v) => { s.$domain.$field = v; true }
                _ => false,
            },
            read: |s| BStyleValue::Bool(s.$domain.$field),
        }
    };
}

macro_rules! shadow {
    ($key:ident, $domain:ident.$field:ident, $section:literal, $label:literal) => {
        BStyleDescriptor {
            key: &key::$key,
            target: concat!(stringify!($domain), ".", stringify!($field)),
            parser: BStyleParser::Shadow,
            preference: BPreferenceMetadata {
                section: $section,
                label: $label,
                control: BPreferenceControl::Choice,
                unit: None,
            },
            default: |d| BStyleValue::Shadow(d.$domain.$field),
            apply: |s, v| match v {
                BStyleValue::Shadow(v) => { s.$domain.$field = v; true }
                _ => false,
            },
            read: |s| BStyleValue::Shadow(s.$domain.$field),
        }
    };
}

// Child modules — declared AFTER the macros so they can use them.
mod parser;
mod table;
#[cfg(test)]
mod tests;

pub use table::UI_STYLE_DESCRIPTORS;
