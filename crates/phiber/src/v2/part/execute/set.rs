//! Execute set — action buttons in the File/Preferences Execute region.
//! Blender ref: `RGN_TYPE_EXECUTE` (space_file, space_userpref).

use crate::v2::host::BHostType;
use crate::v2::part::BPartType;
use crate::v2::spec::{BSpec, BSpecKind};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BExecuteAction {
    pub icon: &'static str,
    pub label: &'static str,
    pub primary: bool,
}

impl BSpec for BExecuteAction {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Part(BPartType::Execute)
    }
    fn label(&self) -> &'static str {
        self.label
    }
    fn icon(&self) -> &'static str {
        self.icon
    }
}

const fn a(icon: &'static str, label: &'static str, primary: bool) -> BExecuteAction {
    BExecuteAction { icon, label, primary }
}

static FILE: &[BExecuteAction] =
    &[a("x", "Cancel", false), a("folder_open", "Open File", true)];
static ASSET: &[BExecuteAction] = &[a("x", "Cancel", false), a("import", "Import", true)];
static PREFS: &[BExecuteAction] = &[a("file_tick", "Save Preferences", true)];

pub fn actions_for(host: BHostType) -> &'static [BExecuteAction] {
    match host {
        BHostType::File => FILE,
        BHostType::Asset => ASSET,
        BHostType::Prefs => PREFS,
        _ => &[],
    }
}
