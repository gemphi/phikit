//! What the viewport draws over the scene, as its Overlays and Gizmos
//! popovers ask (`VIEW3D_PT_overlay`, `VIEW3D_PT_gizmo_display`).
//!
//! The popovers' rows are the Tool tab's (`props::tool::OVERLAYS`,
//! `::OPTIONS`) and key on `ctx.buts.<rna>`; this is the one place the
//! canvas reads them, so a box in either place is what the viewport does.
//! The master switch hides every overlay, as `V3D_HIDE_OVERLAYS` does —
//! floor, axes, text, cursor, extras, bones, lines, outlines, origins — but
//! not the gizmos, which have a master of their own.
//!
//! A file opens with its own view's switches (`app::load_ui`), since
//! Blender keeps them per 3D view (`View3D.overlay`).

use crate::v2::part::props::prop::{BDefault, BPropSpec};
use crate::v2::part::props::prop_view::value_key;
use crate::v2::part::props::tool::{OPTIONS, OVERLAYS};
use crate::v2::state::BState;
use crate::v2::state::val::{V_FALSE, V_TRUE};

/// The switches the canvas draws by, the masters already applied.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BOverlays {
    pub floor: bool,
    pub axis_x: bool,
    pub axis_y: bool,
    pub axis_z: bool,
    pub text: bool,
    pub stats: bool,
    pub cursor: bool,
    pub extras: bool,
    pub relations: bool,
    pub outline: bool,
    pub bones: bool,
    pub origins: bool,
    pub origins_all: bool,
    /// The orientation gizmo and the view buttons (a gizmo, not an overlay).
    pub navigate: bool,
}

impl BOverlays {
    /// As the popovers stand now.
    pub fn now(s: &BState) -> Self {
        let all = Self::on(s, OVERLAYS, "show_overlays");
        let o = |key: &str| all && Self::on(s, OVERLAYS, key);
        Self {
            floor: o("show_floor"),
            axis_x: o("show_axis_x"),
            axis_y: o("show_axis_y"),
            axis_z: o("show_axis_z"),
            text: o("show_text"),
            stats: o("show_stats"),
            cursor: o("show_cursor"),
            extras: o("show_extras"),
            relations: o("show_relationship_lines"),
            outline: o("show_outline_selected"),
            bones: o("show_bones"),
            origins: o("show_object_origins"),
            origins_all: o("show_object_origins_all"),
            navigate: Self::on(s, OPTIONS, "show_gizmo") && Self::on(s, OPTIONS, "show_gizmo_navigate"),
        }
    }

    /// One box of `table`: what was ticked, else its Blender default.
    fn on(s: &BState, table: &[BPropSpec], key: &str) -> bool {
        let Some(spec) = table.iter().find(|spec| spec.key == key) else { return false };
        match s.get(&value_key(spec)) {
            Some(V_TRUE) => true,
            Some(V_FALSE) => false,
            _ => matches!(spec.default, Some(BDefault::Bool(true))),
        }
    }

    /// Write the file's own switches (`BlendOverlay`, by RNA name) where the
    /// popovers read them; a name no row carries is left out.
    pub fn seed(s: &mut BState, list: &[(&'static str, bool)]) {
        for (name, on) in list {
            let spec = OVERLAYS.iter().chain(OPTIONS).find(|spec| spec.key == *name);
            if let Some(spec) = spec {
                s.set(&value_key(spec), if *on { V_TRUE } else { V_FALSE });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_viewport_draws_what_blender_s_does() {
        let o = BOverlays::now(&BState::new());
        assert!(o.floor && o.axis_x && o.axis_y && !o.axis_z, "the floor and two axes");
        assert!(o.text && !o.stats && o.cursor && o.extras && o.relations && o.outline && o.bones);
        assert!(o.origins && !o.origins_all && o.navigate);
    }

    #[test]
    fn the_master_switch_hides_every_overlay_but_not_the_gizmo() {
        let mut s = BState::new();
        BOverlays::seed(&mut s, &[("show_overlays", false), ("show_stats", true)]);
        let o = BOverlays::now(&s);
        assert!(!(o.floor || o.axis_x || o.text || o.stats || o.cursor || o.extras || o.relations || o.outline || o.bones || o.origins));
        assert!(o.navigate, "a gizmo, not an overlay");
        BOverlays::seed(&mut s, &[("show_overlays", true), ("show_gizmo", false), ("show_bones", false)]);
        let o = BOverlays::now(&s);
        assert!(o.stats && o.extras && !o.bones && !o.navigate);
    }
}
