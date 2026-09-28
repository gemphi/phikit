//! `BView::split` — the property-split row's constructor, and the popover
//! rows built on it: one owner for every checkbox and dropdown a header
//! popover lays out (Playback, Time Jump, Auto Keyframing, the Outliner's
//! Filter), so they share one shape.

use crate::v2::state::{BKey as _, BPath, BState, ButsKey, val::BValue};
use crate::v2::view::BView;
use crate::v2::view::prim::{BCtrl, BCtrlKind, BOption};

impl BView {
    /// A property row: label, then the value (`use_property_split`).
    pub fn split(list: Vec<BView>) -> Self {
        Self::Split { list }
    }

    /// `heading` in the label column — blank to sit under the row above —
    /// and `ctrl` in the value column.
    pub fn prop_row(heading: &str, ctrl: BCtrl) -> Self {
        Self::split(vec![Self::label(heading), Self::ctrl(ctrl)])
    }

    /// A checkbox row: the box and its text in the value column.
    pub fn check_row(s: &BState, heading: &str, key: BPath, default: bool, label: &str) -> Self {
        Self::prop_row(heading, BCtrl::read(BCtrlKind::Check, key, BValue::Bool(default), s).with_label(label))
    }

    /// A dropdown row over `(value, label)` pairs; its menu is keyed by `rna`.
    pub fn select_row(s: &BState, heading: &str, key: BPath, rna: &'static str, items: &[(&str, &str)], default: &str) -> Self {
        let ctrl = BCtrl::read(BCtrlKind::Select, key, BValue::Enum(default.to_string()), s)
            .with_options(items.iter().map(|(key, label)| BOption { key: key.to_string(), label: label.to_string(), icon: String::new() }))
            .with_menu(ButsKey::Dropdown(rna).path(), s);
        Self::prop_row(heading, ctrl)
    }
}
