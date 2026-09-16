//! HUD set — floating overlay above the viewport canvas.
//! Blender ref: `RGN_TYPE_HUD`; hosts the "Adjust Last Operation" pop-up.

use crate::v2::part::BPartType;
use crate::v2::spec::{BSpec, BSpecKind};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BHudSlot {
    pub label: &'static str,
}

impl BSpec for BHudSlot {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Part(BPartType::HUD)
    }
    fn label(&self) -> &'static str {
        self.label
    }
}

pub static SLOTS: &[BHudSlot] = &[
    BHudSlot {
        label: "Adjust Last Operation",
    },
    BHudSlot { label: "Redo" },
];
