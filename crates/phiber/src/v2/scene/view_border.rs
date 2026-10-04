//! The viewport's own render region — Blender's `View3D.render_border`
//! with `V3D_RENDER_BORDER` in `View3D.flag2`: what Ctrl B sets when the
//! view is not looking through the camera (`render_border_exec`'s other
//! branch), the sidebar's View ▸ Render Region switch (`use_render_border`),
//! cleared by Ctrl Alt B (`clear_render_border_exec`), drawn as
//! `drawrenderborder`'s dashed box over the region, and the rectangle the
//! Rendered shading fills (`ED_view3d_calc_render_border`: only an engine
//! that draws the viewport itself — not Workbench). View data, not scene
//! data: no undo step, nothing written back; a file's view seeds it with
//! the rest of Load UI (`app/load_ui.rs`).

use crate::phiphx::blend::BlendViewBorder;
use crate::v2::state::{BState, View3dKey, val::{V_FALSE, V_TRUE}};

/// What the state holds of the view's region.
pub struct BViewBorder;

impl BViewBorder {
    /// The region as the state has it: off over the whole region until a
    /// file or a gesture says otherwise.
    pub fn now(s: &BState) -> BlendViewBorder {
        let dflt = BlendViewBorder::default();
        BlendViewBorder {
            on: s.get(View3dKey::BorderUse) == Some(V_TRUE),
            rect: s.get(View3dKey::BorderRect).and_then(super::render_border::BRenderBorder::parse).unwrap_or(dflt.rect),
        }
    }

    /// A file's view region into the state.
    pub fn seed(s: &mut BState, file: BlendViewBorder) {
        s.set(View3dKey::BorderUse, if file.on { V_TRUE } else { V_FALSE });
        s.remove(View3dKey::BorderRect);
        if file.rect != BlendViewBorder::default().rect {
            s.set(View3dKey::BorderRect, super::render_border::BRenderBorder::encode(file.rect));
        }
    }

    /// `view3d.render_border` outside the camera view: a box in region
    /// pixels (y down) over a `w × h` region as fractions of it, y up,
    /// clamped; `None` for an empty box, which turns the region off.
    pub fn from_region_box(bx: (f32, f32, f32, f32), (w, h): (f32, f32)) -> Option<[f32; 4]> {
        super::render_border::BRenderBorder::from_frame_box(bx, (0.0, 0.0, w.max(1e-3), h.max(1e-3)))
    }

    /// A gesture's box: the region on (`view:…`), or off for an empty one.
    pub fn set(s: &mut BState, rect: Option<[f32; 4]>) {
        match rect {
            Some(rect) => {
                s.set(View3dKey::BorderRect, super::render_border::BRenderBorder::encode(rect));
                s.set(View3dKey::BorderUse, V_TRUE);
            }
            None => s.set(View3dKey::BorderUse, V_FALSE),
        }
    }

    /// `view3d.clear_render_border` outside the camera view: off, the
    /// whole region again.
    pub fn clear(s: &mut BState) {
        s.set(View3dKey::BorderUse, V_FALSE);
        s.remove(View3dKey::BorderRect);
    }

    /// Whether Ctrl B and Ctrl Alt B act on the view's own region: the
    /// viewport the pointer is over is not looking through the camera
    /// (`rv3d->persp != RV3D_CAMOB`). With no viewport at hand — a menu
    /// fired from elsewhere — the scene's region is meant.
    pub fn is_the_view_s(s: &BState) -> bool {
        super::view_slot::BViewSlot::current().is_some_and(|slot| s.get(View3dKey::Camera(slot.scope, slot.slot)) != Some(V_TRUE))
    }

    /// The region the Rendered shading fills, when it is on and `engine`
    /// draws the viewport itself (`view3d_main_region_do_render_draw`:
    /// `view_update` and `view_draw` — EEVEE and Cycles, not Workbench).
    pub fn filled_by(s: &BState, engine: &str) -> Option<[f32; 4]> {
        let now = Self::now(s);
        (now.on && engine != "BLENDER_WORKBENCH").then_some(now.rect)
    }

    /// Those pixels of a `w × h` viewport, when they are not all of it.
    pub fn fills(s: &BState, engine: &str, w: u32, h: u32) -> Option<(u32, u32, u32, u32)> {
        Self::filled_by(s, engine).and_then(|rect| Self::pixels_of(rect, w, h))
    }

    /// A region's pixels of a `w × h` viewport (`BlendViewBorder::pixels`),
    /// `None` when it is the whole viewport or nothing.
    pub fn pixels_of(rect: [f32; 4], w: u32, h: u32) -> Option<(u32, u32, u32, u32)> {
        BlendViewBorder { on: true, rect }.pixels(w, h).filter(|p| *p != (0, 0, w, h))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_box_sets_the_region_an_empty_one_turns_it_off_and_clear_puts_the_whole_back() {
        let mut s = BState::new();
        assert_eq!(BViewBorder::now(&s), BlendViewBorder::default());
        // The right half of a 200 × 100 region, lower half.
        let rect = BViewBorder::from_region_box((100.0, 50.0, 200.0, 100.0), (200.0, 100.0)).unwrap();
        assert_eq!(rect, [0.5, 1.0, 0.0, 0.5]);
        BViewBorder::set(&mut s, Some(rect));
        assert_eq!(BViewBorder::now(&s), BlendViewBorder { on: true, rect });
        assert_eq!(BViewBorder::fills(&s, "CYCLES", 200, 100), Some((100, 50, 100, 50)));
        assert_eq!(BViewBorder::fills(&s, "BLENDER_WORKBENCH", 200, 100), None, "Workbench draws the whole viewport");
        assert_eq!(BViewBorder::from_region_box((10.0, 10.0, 10.0, 80.0), (200.0, 100.0)), None);
        BViewBorder::set(&mut s, None);
        assert_eq!(BViewBorder::now(&s), BlendViewBorder { on: false, rect }, "off keeps the rectangle");
        assert_eq!(BViewBorder::fills(&s, "CYCLES", 200, 100), None);
        BViewBorder::clear(&mut s);
        assert_eq!(BViewBorder::now(&s), BlendViewBorder::default());
        // A file's region, seeded; the whole region on fills nothing extra.
        BViewBorder::seed(&mut s, BlendViewBorder { on: true, rect: [0.0, 1.0, 0.0, 1.0] });
        assert_eq!(BViewBorder::fills(&s, "CYCLES", 200, 100), None);
        BViewBorder::seed(&mut s, BlendViewBorder { on: true, rect: [0.1, 0.9, 0.2, 0.8] });
        assert_eq!(BViewBorder::now(&s), BlendViewBorder { on: true, rect: [0.1, 0.9, 0.2, 0.8] });
    }

    #[test]
    fn without_a_viewport_at_hand_the_scene_s_region_is_meant() {
        super::super::view_slot::BViewSlot::forget_hover();
        assert!(!BViewBorder::is_the_view_s(&BState::new()));
    }
}
