//! The render region — Blender's `Scene.r.border` with `R_BORDER` and
//! `R_CROP`: Properties → Output → Format's Render Region and Crop to
//! Render Region, set by a box dragged in the camera view
//! (`view3d.render_border`, Ctrl B) or over the Image editor's picture
//! (`image.render_border`), cleared by Ctrl Alt B
//! (`view3d.clear_render_border`, `image.clear_render_border`), and the
//! rectangle F12 fills (`render_init_from_main`'s `disprect`). The two
//! switches are the Output panel's keys; the rectangle is one key over
//! the file (`ctx.scene.render_border`, `xmin,xmax,ymin,ymax`), stepped by
//! Undo with the other scene edits and written back on save.

use crate::phiphx::blend::BlendRenderBorder;
use crate::v2::state::{BKey as _, BPath, BState, ButsKey, ImageKey, OpKey, SceneKey, View3dKey, log, rule::BRule, val::{BValue, BValueKind, V_TRUE}};

/// What the state holds over the file's region.
pub struct BRenderBorder;

impl BRenderBorder {
    pub const USE: &'static str = "use_border";
    pub const CROP: &'static str = "use_crop_to_border";

    /// The region as it stands: the panel's switches over `file`'s, the
    /// rectangle the gesture set over the file's.
    pub fn now(s: &BState, file: BlendRenderBorder) -> BlendRenderBorder {
        let switch = |key: &'static str, dflt: bool| match s.value(&ButsKey::Value(key).path(), BValueKind::Bool) {
            Some(BValue::Bool(b)) => b,
            _ => dflt,
        };
        let rect = s.get(SceneKey::RenderBorder).and_then(Self::parse).unwrap_or(file.rect);
        BlendRenderBorder { on: switch(Self::USE, file.on), crop: switch(Self::CROP, file.crop), rect }
    }

    /// A file's region into the state (`app::seed`): the switches and, when
    /// it is not the whole frame, the rectangle.
    pub fn seed(s: &mut BState, file: BlendRenderBorder) {
        s.set(ButsKey::Value(Self::USE), BValue::Bool(file.on).encode());
        s.set(ButsKey::Value(Self::CROP), BValue::Bool(file.crop).encode());
        s.remove(SceneKey::RenderBorder);
        if file.rect != BlendRenderBorder::default().rect {
            s.set(SceneKey::RenderBorder, Self::encode(file.rect));
        }
    }

    pub fn encode(rect: [f32; 4]) -> String {
        rect.iter().map(|v| format!("{v}")).collect::<Vec<_>>().join(",")
    }

    pub fn parse(raw: &str) -> Option<[f32; 4]> {
        let v: Vec<f32> = raw.split(',').filter_map(|p| p.parse().ok()).collect();
        (v.len() == 4).then(|| [v[0], v[1], v[2], v[3]])
    }

    /// `view3d.render_border` in the camera view: a box in region pixels (y
    /// down) over the camera frame `(left, top, right, bottom)` as fractions
    /// of the frame, y up, clamped; `None` for an empty box, which Blender
    /// turns border rendering off with.
    pub fn from_frame_box(bx: (f32, f32, f32, f32), frame: (f32, f32, f32, f32)) -> Option<[f32; 4]> {
        let (fw, fh) = ((frame.2 - frame.0).max(1e-3), (frame.3 - frame.1).max(1e-3));
        let x = |px: f32| ((px - frame.0) / fw).clamp(0.0, 1.0);
        let y = |px: f32| (1.0 - (px - frame.1) / fh).clamp(0.0, 1.0);
        let rect = [x(bx.0.min(bx.2)), x(bx.0.max(bx.2)), y(bx.1.max(bx.3)), y(bx.1.min(bx.3))];
        (rect[0] != rect[1] && rect[2] != rect[3]).then_some(rect)
    }

    /// `image.render_border`: a box as fractions of the picture shown (x
    /// right, y down), under `current` — inside the cropped region when the
    /// picture is one — snapped to the `w × h` frame's pixels; `None` when
    /// it covers the whole frame or no pixel, which turns the region off.
    pub fn from_image_box(bx: (f32, f32, f32, f32), current: BlendRenderBorder, w: u32, h: u32) -> Option<[f32; 4]> {
        let mut rect = [bx.0.min(bx.2), bx.0.max(bx.2), 1.0 - bx.1.max(bx.3), 1.0 - bx.1.min(bx.3)];
        if current.on && current.crop {
            let c = current.rect;
            rect = [c[0] + rect[0] * (c[1] - c[0]), c[0] + rect[1] * (c[1] - c[0]), c[2] + rect[2] * (c[3] - c[2]), c[2] + rect[3] * (c[3] - c[2])];
        }
        let rect = rect.map(|v| v.clamp(0.0, 1.0));
        if (rect[0] <= 0.0 && rect[1] >= 1.0 && rect[2] <= 0.0 && rect[3] >= 1.0) || rect[0] == rect[1] || rect[2] == rect[3] {
            return None;
        }
        let (fw, fh) = (w.max(1) as f32, h.max(1) as f32);
        Some([(rect[0] * fw).floor() / fw, (rect[1] * fw).ceil() / fw, (rect[2] * fh).floor() / fh, (rect[3] * fh).ceil() / fh])
    }
}

/// `ctx.op.render_border`: `arm` (the viewport's Ctrl B), `arm_image` (the
/// Image editor's), `set:xmin,xmax,ymin,ymax` (a gesture's box in the
/// camera view: the region on), `off` (an empty box: the region off, the
/// rectangle kept), `view:…` and `view_off` (the same outside the camera
/// view: the viewport's own region, `BViewBorder`) and `clear` (Ctrl Alt
/// B: off, the whole frame again — the viewport's own when the viewport at
/// hand is not looking through the camera, as `clear_render_border_exec`
/// picks; `clear_scene` is the Image editor's, always the scene's).
pub struct BRenderBorderRule;

impl BRule for BRenderBorderRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == OpKey::RenderBorder.path()
    }
    fn after(&self, path: &BPath, value: &str, s: &mut BState) {
        s.remove(path);
        match value {
            "arm" => s.set(View3dKey::RenderBorder, V_TRUE),
            "arm_image" => s.set(ImageKey::BorderArm, V_TRUE),
            "off" => s.set(ButsKey::Value(BRenderBorder::USE), BValue::Bool(false).encode()),
            "view_off" => super::view_border::BViewBorder::set(s, None),
            "clear" if super::view_border::BViewBorder::is_the_view_s(s) => super::view_border::BViewBorder::clear(s),
            "clear" | "clear_scene" => {
                s.set(ButsKey::Value(BRenderBorder::USE), BValue::Bool(false).encode());
                s.remove(SceneKey::RenderBorder);
                log::push("Render region cleared".to_string());
            }
            _ if value.starts_with("view:") => super::view_border::BViewBorder::set(s, value.strip_prefix("view:").and_then(BRenderBorder::parse)),
            _ => {
                let Some(rect) = value.strip_prefix("set:").and_then(BRenderBorder::parse) else { return };
                s.set(SceneKey::RenderBorder, BRenderBorder::encode(rect));
                s.set(ButsKey::Value(BRenderBorder::USE), BValue::Bool(true).encode());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_box_over_the_camera_frame_becomes_fractions_y_up() {
        // The frame 100 px wide and 50 high from (10, 20); a box over its
        // right half, upper quarter.
        let rect = BRenderBorder::from_frame_box((60.0, 20.0, 110.0, 32.5), (10.0, 20.0, 110.0, 70.0)).unwrap();
        assert!((rect[0] - 0.5).abs() < 1e-6 && (rect[1] - 1.0).abs() < 1e-6 && (rect[2] - 0.75).abs() < 1e-6 && (rect[3] - 1.0).abs() < 1e-6, "{rect:?}");
        assert_eq!(BRenderBorder::from_frame_box((60.0, 20.0, 60.0, 40.0), (10.0, 20.0, 110.0, 70.0)), None, "no width: off");
        // Past the frame: clamped.
        assert_eq!(BRenderBorder::from_frame_box((-50.0, -50.0, 500.0, 500.0), (10.0, 20.0, 110.0, 70.0)), Some([0.0, 1.0, 0.0, 1.0]));
    }

    #[test]
    fn a_box_over_the_picture_snaps_to_pixels_and_the_whole_frame_turns_it_off() {
        let off = BlendRenderBorder::default();
        // The top-left quarter-ish of a 200 × 100 frame: snapped outward.
        let rect = BRenderBorder::from_image_box((0.0, 0.0, 0.503, 0.496), off, 200, 100).unwrap();
        assert_eq!(rect, [0.0, 0.505, 0.5, 1.0]);
        assert_eq!(BRenderBorder::from_image_box((0.0, 0.0, 1.0, 1.0), off, 200, 100), None, "the whole frame");
        // Under a cropped region, the box is inside it.
        let cropped = BlendRenderBorder { on: true, crop: true, rect: [0.5, 1.0, 0.0, 0.5] };
        let inner = BRenderBorder::from_image_box((0.0, 0.0, 0.5, 1.0), cropped, 200, 100).unwrap();
        assert_eq!(inner, [0.5, 0.75, 0.0, 0.5]);
    }

    #[test]
    fn the_rule_sets_clears_and_arms() {
        let mut s = BState::new();
        let r = BRenderBorderRule;
        r.after(&OpKey::RenderBorder.path(), "set:0.25,0.75,0.5,1", &mut s);
        let now = BRenderBorder::now(&s, BlendRenderBorder::default());
        assert_eq!(now, BlendRenderBorder { on: true, crop: false, rect: [0.25, 0.75, 0.5, 1.0] });
        r.after(&OpKey::RenderBorder.path(), "off", &mut s);
        assert_eq!(BRenderBorder::now(&s, BlendRenderBorder::default()), BlendRenderBorder { on: false, crop: false, rect: [0.25, 0.75, 0.5, 1.0] }, "off keeps the rectangle");
        r.after(&OpKey::RenderBorder.path(), "clear", &mut s);
        assert_eq!(BRenderBorder::now(&s, BlendRenderBorder::default()), BlendRenderBorder::default());
        r.after(&OpKey::RenderBorder.path(), "arm", &mut s);
        assert_eq!(s.get(View3dKey::RenderBorder), Some(V_TRUE));
        // Outside the camera view the box is the viewport's own region, and
        // the scene's is left alone.
        r.after(&OpKey::RenderBorder.path(), "view:0.5,1,0,0.5", &mut s);
        assert_eq!(super::super::view_border::BViewBorder::now(&s).rect, [0.5, 1.0, 0.0, 0.5]);
        assert_eq!(BRenderBorder::now(&s, BlendRenderBorder::default()), BlendRenderBorder::default());
        r.after(&OpKey::RenderBorder.path(), "view_off", &mut s);
        assert!(!super::super::view_border::BViewBorder::now(&s).on);
        // The file's region, seeded, is what `now` reads until a gesture.
        let file = BlendRenderBorder { on: true, crop: true, rect: [0.1, 0.9, 0.2, 0.8] };
        BRenderBorder::seed(&mut s, file);
        assert_eq!(BRenderBorder::now(&s, BlendRenderBorder::default()), file);
    }
}
