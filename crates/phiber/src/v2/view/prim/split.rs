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

    /// A boolean row: its text from the row's left edge and the switch at
    /// its right (`BCtrlKind::Switch`) — the whole row, so every switch in a
    /// panel or popover lines up and no empty label column pushes its text
    /// halfway across. A heading leading a group is its own line above it.
    pub fn check_row(s: &BState, heading: &str, key: BPath, default: bool, label: &str) -> Self {
        let switch = Self::ctrl(BCtrl::read(BCtrlKind::Switch, key, BValue::Bool(default), s).with_label(label));
        match heading.is_empty() {
            true => switch,
            false => Self::column(vec![Self::label(heading), switch]),
        }
    }

    /// A dropdown row over `(value, label)` pairs; its menu is keyed by `rna`.
    pub fn select_row(s: &BState, heading: &str, key: BPath, rna: &'static str, items: &[(&str, &str)], default: &str) -> Self {
        let ctrl = BCtrl::read(BCtrlKind::Select, key, BValue::Enum(default.to_string()), s)
            .with_options(items.iter().map(|(key, label)| BOption { key: key.to_string(), label: label.to_string(), icon: String::new() }))
            .with_menu(ButsKey::Dropdown(rna).path(), s);
        Self::prop_row(heading, ctrl)
    }
}
