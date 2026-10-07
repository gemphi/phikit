//! Restriction columns — the visibility toggles at a row's right side.
//!
//! Blender ref: `bl_ui/space_outliner.py` and
//! `editors/space_outliner/`.

use crate::v2::part::BPartType;
use crate::v2::spec::{BSpec, BSpecKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BOutlineToggle {
    Viewport,     // eye  — 👁 disable in viewports
    Render,       // cam  — disable in renders
    SelectLock,   // arrow — disable selection
    Hide,         // eye-slash — hide in viewport
    Holdout,      // holdout in view layer
    IndirectOnly, // indirect-only in view layer
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BOutlineToggleSpec {
    pub kind: BOutlineToggle,
    pub icon: &'static str,
    pub tooltip: &'static str,
    pub state_key: &'static str,
    /// Whether this column is shown without being asked.
    ///
    /// A deliberate divergence from Blender, which shows only the eye
    /// (`show_restrict_column_hide`) and the camera
    /// (`show_restrict_column_render`) by default and leaves the rest opt-in:
    /// phiber shows the whole restriction cluster on every row, so each row
    /// carries its full set of toggles at a glance. Any column can still be
    /// hidden from the Filter popover (`ctx.outliner.col.<flag>` = false), and
    /// the two View-Layer-only columns (holdout, indirect-only) still appear
    /// only in View Layer mode.
    pub default_on: bool,
}

impl BSpec for BOutlineToggleSpec {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Part(BPartType::Custom)
    }
    fn label(&self) -> &'static str {
        self.tooltip
    }
    fn icon(&self) -> &'static str {
        self.icon
    }
}

/// Restriction columns in Blender's SCREEN order (left → right).
///
/// `outliner_draw_restrictbuts` lays these out right-to-left from the region
/// edge — Indirect Only, Holdout, Render, Viewport, Hide, Select — which puts
/// Select nearest the name and Indirect Only at the far right. Holdout and
/// Indirect Only exist only in View Layer mode (`view_layer_only`).
///
/// Icons are chosen from the shipped `assets/icons` set: there is no
/// `arrow`/`holdout`/`indirect` svg, so selectability uses `mouse_pointer`,
/// holdout uses `circle_dashed` and indirect-only uses `lightbulb`.
pub static TOGGLES: &[BOutlineToggleSpec] = &[
    BOutlineToggleSpec {
        kind: BOutlineToggle::SelectLock,
        icon: "mouse_pointer",
        tooltip: "Disable Selection",
        state_key: "hide_select",
        default_on: true,
    },
    BOutlineToggleSpec {
        kind: BOutlineToggle::Hide,
        icon: "eye",
        tooltip: "Hide in Viewport",
        state_key: "hide_get",
        default_on: true,
    },
    BOutlineToggleSpec {
        kind: BOutlineToggle::Viewport,
        icon: "monitor",
        tooltip: "Disable in Viewports",
        state_key: "hide_viewport",
        default_on: true,
    },
    BOutlineToggleSpec {
        kind: BOutlineToggle::Render,
        icon: "camera",
        tooltip: "Disable in Renders",
        state_key: "hide_render",
        default_on: true,
    },
    BOutlineToggleSpec {
        kind: BOutlineToggle::Holdout,
        icon: "circle_dashed",
        tooltip: "Holdout",
        state_key: "holdout",
        default_on: true,
    },
    BOutlineToggleSpec {
        kind: BOutlineToggle::IndirectOnly,
        icon: "lightbulb",
        tooltip: "Indirect Only",
        state_key: "indirect_only",
        default_on: true,
    },
];

impl BOutlineToggle {
    /// Holdout and Indirect Only are View-Layer-only columns.
    pub fn view_layer_only(self) -> bool {
        matches!(self, BOutlineToggle::Holdout | BOutlineToggle::IndirectOnly)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_whole_restriction_cluster_shows_by_default() {
        assert!(
            TOGGLES.iter().all(|t| t.default_on),
            "every restriction column is on by default now, so a row shows its full set"
        );
        assert_eq!(TOGGLES.len(), 6, "select, hide, viewport, render, holdout, indirect-only");
        let view_layer_only = TOGGLES.iter().filter(|t| t.kind.view_layer_only()).count();
        assert_eq!(view_layer_only, 2, "holdout and indirect-only remain View-Layer-only");
    }
}

