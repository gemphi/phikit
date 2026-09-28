//! Operator options panel (Blender draws `WM_OT_open_mainfile`'s properties
//! in the file browser's tool-props region): **Load UI** for Open.
//! Declarative: each option is a `(label, key, default)` row.

use crate::v2::state::BKey as _;
use crate::v2::state::FileKey;
use super::purpose::BFilePurpose;
use super::purpose::purpose;
use crate::v2::state::{BPath, BState, val::{BValue, V_FALSE}};
use crate::v2::view::BView;
use crate::v2::view::prim::{BCtrl, BCtrlKind};

pub struct BFileOption {
    pub label: &'static str,
    pub tooltip: &'static str,
    pub key: fn() -> BPath,
}

pub const OPEN_OPTIONS: &[BFileOption] = &[BFileOption {
    label: "Load UI",
    tooltip: "Load user interface setup in the .blend file",
    key: || FileKey::LoadUi.path(),
}];

fn options_for(p: BFilePurpose) -> &'static [BFileOption] {
    match p {
        BFilePurpose::Open => OPEN_OPTIONS,
        _ => &[],
    }
}

/// Options default on; only an explicit `"false"` turns one off.
pub fn is_on(s: &BState, key: &BPath) -> bool {
    s.get(key) != Some(V_FALSE)
}

/// Whether opening a file should restore its workspace and 3D view.
pub fn load_ui(s: &BState) -> bool {
    is_on(s, &FileKey::LoadUi.path())
}

fn row(s: &BState, o: &BFileOption) -> BView {
    // A checkbox over the option's key: the same control as a Properties
    // bool, read and written the same way (`prim::ctrl`).
    let key = (o.key)();
    let default = BValue::Bool(true);
    BView::ctrl(BCtrl::read(BCtrlKind::Check, key, default, s).with_label(o.label).with_tooltip(o.tooltip))
}

/// The options panel for the current purpose, if it has any.
pub fn panel(s: &BState) -> Option<BView> {
    let p = purpose(s);
    let opts = options_for(p);
    (!opts.is_empty()).then(|| super::sidebar::panel(s, p.title(), opts.iter().map(|o| row(s, o)).collect()))
}
