//! Projects and strokes the overlay extras (camera frustums, light gizmos,
//! empties, rigs' bones) produced by `scene::extras`, with the same world → view mapping
//! the mesh painter uses (Blender Z-up → projector (x, up, z), MESH_SCALE).

use crate::v2::scene::extra::{BExtraKind, scene_extras_now};
use crate::v2::scene::{MESH_SCALE, project3_full, source::with_loaded};
use crate::v2::state::BState;
use gpui::{Bounds, Hsla, PathBuilder, Pixels, Window, point, px};

/// Stroke colours per kind (Blender theme: camera / light / empty wires),
/// and the selection's — a selected one in the meshes' outline accent, the
/// active one a step lighter, as Blender draws `TH_SELECT` / `TH_ACTIVE`.
#[derive(Clone, Copy)]
pub(super) struct BExtraPalette {
    pub camera: Hsla,
    pub light: Hsla,
    pub empty: Hsla,
    pub selected: Hsla,
    pub active: Hsla,
    /// A child's line to its parent: the wire colour, as Blender's
    /// `colors.wire`; a constraint's line: the Z axis's (`grid_axis_z`).
    pub relation: Hsla,
    pub constraint: Hsla,
}

impl BExtraPalette {
    pub fn from_style(ctx: &crate::v2::view::draw::ctx::BRenderCtx, meshes: &super::BMeshPalette) -> Self {
        use crate::v2::view::draw::color::to_hsla;
        Self {
            camera: to_hsla(ctx.style.colors.text_dim),
            light: to_hsla(ctx.style.colors.accent_soft),
            empty: to_hsla(ctx.style.colors.text_dim),
            selected: meshes.accent,
            active: meshes.accent_active,
            relation: to_hsla(ctx.style.colors.text_dim),
            constraint: to_hsla(ctx.style.colors.axis_z),
        }
    }
}

#[derive(Clone)]
pub(super) struct BProjectedLine {
    pub a: (f32, f32),
    pub b: (f32, f32),
    pub color: Hsla,
    /// Stippled, as Blender draws relationship lines.
    pub dash: bool,
}

/// Project each object's lines in the colour its state gives it. What an
/// object is called now, whether it is hidden and whether it is selected
/// are asked once per object, not once per line — a rig draws a few
/// thousand. `hidden` is the frame's collection-hidden set, built once.
pub(super) fn project(
    s: &BState,
    yaw: f32,
    pitch: f32,
    pan: (f32, f32),
    zoom: f32,
    palette: BExtraPalette,
) -> Vec<BProjectedLine> {
    let to_screen = |p: [f32; 3]| {
        let (x, y, _) = project3_full(p[0] * MESH_SCALE, p[2] * MESH_SCALE, p[1] * MESH_SCALE, yaw, pitch, pan, zoom);
        (x, y)
    };
    let hidden = crate::v2::part::outliner::key::BHidden::now(s);
    let (selection, active) = (crate::v2::scene::select::BSelection::now(s), s.get(crate::v2::state::CtxKey::Object).unwrap_or(""));
    let edit = crate::v2::scene::edit::BSceneEdit::now(s);
    let shown = crate::v2::scene::BOverlays::now(s);
    with_loaded(|m| {
        let Some(model) = m else { return Vec::new() };
        let lines = scene_extras_now(s, model);
        let mut out = Vec::with_capacity(lines.len());
        for run in lines.chunk_by(|a, b| a.kind == b.kind && a.name_index == b.name_index) {
            let kind = run[0].kind;
            if !BExtraKind::shown(kind, shown) {
                continue;
            }
            // Cameras, lights, empties and rigs hide with their collection too.
            let Some(file) = run[0].file_name(model).filter(|f| !hidden.hides(s, f)) else { continue };
            // A line to a hidden parent is not drawn (`OB_VISIBLE_SELF`).
            let parent = |of: usize| model.motion.motion.list.get(of).map(|m| m.name.as_str());
            if let BExtraKind::Parent { of } = kind && parent(of).is_none_or(|p| hidden.hides(s, p)) {
                continue;
            }
            let name = edit.renamed.iter().find(|(_, old)| old == file).map_or(file, |(new, _)| new.as_str());
            let color = match kind {
                BExtraKind::Parent { .. } => palette.relation,
                BExtraKind::Constraint => palette.constraint,
                _ if name == active && selection.contains(name) => palette.active,
                _ if selection.contains(name) => palette.selected,
                BExtraKind::Camera => palette.camera,
                BExtraKind::Light => palette.light,
                // A rig's bones in the object's wire colour (`const_color`).
                BExtraKind::Empty | BExtraKind::Bone => palette.empty,
            };
            let dash = matches!(kind, BExtraKind::Parent { .. } | BExtraKind::Constraint);
            out.extend(run.iter().map(|l| BProjectedLine { a: to_screen(l.a), b: to_screen(l.b), color, dash }));
        }
        out
    })
}

/// A relationship line's dash and gap, in pixels (`overlay_extra_wire`'s
/// stipple).
const DASH: f32 = 6.0;

/// Stroke `lines`, each run of one colour as one path: a rig draws a few
/// thousand segments, and a path each was most of what they cost.
pub(super) fn paint(bounds: Bounds<Pixels>, lines: &[BProjectedLine], width: f32, window: &mut Window) {
    let w = f32::from(bounds.size.width);
    let h = f32::from(bounds.size.height);
    let at = |p: (f32, f32)| {
        point(
            bounds.origin.x + px((0.5 + p.0) * w),
            bounds.origin.y + px((0.5 + p.1) * h),
        )
    };
    for run in lines.chunk_by(|a, b| a.color == b.color && a.dash == b.dash) {
        let mut builder = PathBuilder::stroke(px(width));
        if run[0].dash {
            builder = builder.dash_array(&[px(DASH), px(DASH)]);
        }
        for line in run {
            builder.move_to(at(line.a));
            builder.line_to(at(line.b));
        }
        if let Ok(path) = builder.build() {
            window.paint_path(path, run[0].color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phiphx::blend::BlendFile;
    use crate::phiphx::scene::SceneModel;
    use crate::v2::state::{BKey as _, CtxKey};

    #[test]
    fn a_selected_light_draws_in_the_selection_ink_and_the_active_one_lighter() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/parented.blend");
        let scene = BlendFile::parse_bytes(&std::fs::read(&path).unwrap()).unwrap().extract_scene().unwrap();
        crate::v2::scene::source::set_loaded_scene(Some(SceneModel::from_blend_owned(scene, &path.to_string_lossy())));
        use crate::v2::view::draw::color::to_hsla;
        let style = crate::v2::theme::ui_style_defaults(crate::v2::theme::BPaletteName::DEFAULT);
        let c = &style.colors;
        let palette = BExtraPalette { camera: to_hsla(c.text_dim), light: to_hsla(c.accent_soft), empty: to_hsla(c.text), selected: to_hsla(c.accent), active: to_hsla(c.bg_panel), relation: to_hsla(c.border), constraint: to_hsla(c.axis_z) };
        let colours = |s: &BState| project(s, 0.0, 30.0, (0.0, 0.0), 1.0, palette).into_iter().map(|l| l.color).collect::<Vec<_>>();
        let mut s = BState::new();
        crate::v2::scene::select::BSelection { list: vec!["Camera".into(), "Light".into()] }.write(&mut s);
        s.set(CtxKey::Object.path(), "Light");
        let drawn = colours(&s);
        assert!(drawn.contains(&palette.active) && drawn.contains(&palette.selected), "the Light active, the Camera selected");
        assert!(!drawn.contains(&palette.light) && !drawn.contains(&palette.camera));
        crate::v2::scene::select::BSelection::default().write(&mut s);
        let drawn = colours(&s);
        assert!(drawn.contains(&palette.light) && drawn.contains(&palette.camera) && !drawn.contains(&palette.selected));
        crate::v2::scene::source::set_loaded_scene(None);
    }

    #[test]
    fn a_rig_s_bones_draw_in_its_wire_colour_and_in_the_selection_s_when_picked() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../phiphx/tests/fixtures/armature_display.blend");
        let scene = BlendFile::parse_bytes(&std::fs::read(&path).unwrap()).unwrap().extract_scene().unwrap();
        crate::v2::scene::source::set_loaded_scene(Some(SceneModel::from_blend_owned(scene, &path.to_string_lossy())));
        let ink = |r: u8| Hsla { h: 0.0, s: 0.0, l: f32::from(r) / 10.0, a: 1.0 };
        let palette = BExtraPalette { camera: ink(1), light: ink(2), empty: ink(3), selected: ink(4), active: ink(5), relation: ink(6), constraint: ink(7) };
        let count = |s: &BState, c: Hsla| project(s, 0.0, 30.0, (0.0, 0.0), 1.0, palette).iter().filter(|l| l.color == c).count();
        let mut s = BState::new();
        assert_eq!(count(&s, palette.empty), 2 * 12 + 3 * 16 + 12 + 1, "Rig's and Stick's bones in the wire colour");
        crate::v2::scene::select::BSelection { list: vec!["Rig".into(), "Stick".into()] }.write(&mut s);
        s.set(CtxKey::Object.path(), "Stick");
        assert_eq!((count(&s, palette.selected), count(&s, palette.active), count(&s, palette.empty)), (2 * 12 + 3 * 16 + 12, 1, 0));
        crate::v2::scene::source::set_loaded_scene(None);
    }
}
