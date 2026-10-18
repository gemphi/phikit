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
    /// The glyph while the restriction is ON.
    ///
    /// Blender draws every restriction toggle as a `ButtonType::IconToggle`
    /// over a pair of consecutive icons and swaps between them: the button
    /// adds 1 to the icon when selected (`interface.cc`, `but->iconadd`), and
    /// the outliner's buttons set `BUT_ICON_REVERSE` so they subtract instead
    /// (`interface_query.cc`) — which, with the `_OFF` icon as the base,
    /// lands on the `_ON` one beside it in `UI_icons.hh`
    /// (`RESTRICT_VIEW_ON`/`_OFF`, `HIDE_ON`/`_OFF`, and the rest). So "off"
    /// is never a dimmed version of "on" there; it is its own glyph, and the
    /// colour is free to say something else.
    pub icon_on: &'static str,
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
///
/// Each carries its restricted twin (`icon_on`). `eye_off` and `circle_off`
/// already shipped; `mouse_pointer_off`, `monitor_off`, `camera_off` and
/// `lightbulb_off` are the base glyph with a strike through it, which is how
/// Blender draws a `RESTRICT_*_ON` beside its `_OFF`.
pub static TOGGLES: &[BOutlineToggleSpec] = &[
    BOutlineToggleSpec {
        kind: BOutlineToggle::SelectLock,
        icon: "mouse_pointer",
        icon_on: "mouse_pointer_off",
        tooltip: "Disable Selection",
        state_key: "hide_select",
        default_on: true,
    },
    BOutlineToggleSpec {
        kind: BOutlineToggle::Hide,
        icon: "eye",
        icon_on: "eye_off",
        tooltip: "Hide in Viewport",
        state_key: "hide_get",
        default_on: true,
    },
    BOutlineToggleSpec {
        kind: BOutlineToggle::Viewport,
        icon: "monitor",
        icon_on: "monitor_off",
        tooltip: "Disable in Viewports",
        state_key: "hide_viewport",
        default_on: true,
    },
    BOutlineToggleSpec {
        kind: BOutlineToggle::Render,
        icon: "camera",
        icon_on: "camera_off",
        tooltip: "Disable in Renders",
        state_key: "hide_render",
        default_on: true,
    },
    BOutlineToggleSpec {
        kind: BOutlineToggle::Holdout,
        icon: "circle_dashed",
        icon_on: "circle_off",
        tooltip: "Holdout",
        state_key: "holdout",
        default_on: true,
    },
    BOutlineToggleSpec {
        kind: BOutlineToggle::IndirectOnly,
        icon: "lightbulb",
        icon_on: "lightbulb_off",
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

    /// Blender's restriction toggles swap glyph rather than dimming one, so
    /// every column needs its own pair — and a pair that is the same glyph
    /// twice is the bug this replaced: a toggle that reads its state and then
    /// draws the same thing either way.
    #[test]
    fn every_column_has_its_own_restricted_glyph() {
        for t in TOGGLES {
            assert!(!t.icon_on.is_empty(), "{:?} has no restricted glyph", t.kind);
            assert_ne!(t.icon, t.icon_on, "{:?} draws the same glyph either way", t.kind);
        }
        let mut on: Vec<&str> = TOGGLES.iter().map(|t| t.icon_on).collect();
        let n = on.len();
        on.sort_unstable();
        on.dedup();
        assert_eq!(on.len(), n, "two columns would be indistinguishable while restricted");
    }
}

