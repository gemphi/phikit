//! Execute set — action buttons in the File/Preferences Execute region.
//! Blender ref: `RGN_TYPE_EXECUTE` (space_file, space_userpref).

use crate::v2::host::BHostType;
use crate::v2::part::BPartType;
use crate::v2::spec::{BSpec, BSpecKind};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BExecuteAction {
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
}

const fn a(label: &'static str, primary: bool) -> BExecuteAction {
    BExecuteAction { label, primary }
}

static FILE: &[BExecuteAction] = &[a("Cancel", false), a("Open File", true)];
static ASSET: &[BExecuteAction] = &[a("Cancel", false), a("Import", true)];
static PREFS: &[BExecuteAction] = &[a("Save Preferences", true)];

pub fn actions_for(host: BHostType) -> &'static [BExecuteAction] {
    match host {
        BHostType::File => FILE,
        BHostType::Asset => ASSET,
        BHostType::Prefs => PREFS,
        _ => &[],
    }
}
