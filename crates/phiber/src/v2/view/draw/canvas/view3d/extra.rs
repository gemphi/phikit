//! Projects and strokes the overlay extras (camera frustums, light gizmos,
//! empties) produced by `scene::extras`, with the same world → view mapping
//! the mesh painter uses (Blender Z-up → projector (x, up, z), MESH_SCALE).

use crate::v2::scene::extra::{BExtraKind, BExtraLine, scene_extras_now};
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
        }
    }
}

#[derive(Clone)]
pub(super) struct BProjectedLine {
    pub a: (f32, f32),
    pub b: (f32, f32),
    pub color: Hsla,
}

/// Is this extra drawn? `hidden` is the frame's collection-hidden set,
/// built ONCE by the caller. Building it in here — a walk of the whole
/// collection tree — ran once per line, for every line, every frame.
fn visible(s: &BState, hidden: &crate::v2::part::outliner::key::BHidden, line: &BExtraLine) -> bool {
    // Cameras, lights and empties hide with their collection too.
    with_loaded(|m| m.and_then(|m| line.file_name(m)).is_some_and(|n| !hidden.hides(s, n)))
}

pub(super) fn project(
    s: &BState,
    yaw: f32,
    pitch: f32,
    pan: (f32, f32),
    zoom: f32,
    palette: BExtraPalette,
) -> Vec<BProjectedLine> {
    let lines = with_loaded(|m| m.map(|m| scene_extras_now(s, m)).unwrap_or_default());
    let to_screen = |p: [f32; 3]| {
        let (x, y, _) = project3_full(
            p[0] * MESH_SCALE,
            p[2] * MESH_SCALE,
            p[1] * MESH_SCALE,
            yaw,
            pitch,
            pan,
            zoom,
        );
        (x, y)
    };
    let hidden = crate::v2::part::outliner::key::BHidden::now(s);
    let (selection, active) = (crate::v2::scene::select::BSelection::now(s), s.get(crate::v2::state::CtxKey::Object).unwrap_or(""));
    let edit = crate::v2::scene::edit::BSceneEdit::now(s);
    let name_now = |l: &BExtraLine| {
        let file = with_loaded(|m| m.and_then(|m| l.file_name(m)).map(str::to_string)).unwrap_or_default();
        edit.renamed.iter().find(|(_, old)| *old == file).map_or(file.clone(), |(new, _)| new.clone())
    };
    lines
        .iter()
        .filter(|l| visible(s, &hidden, l))
        .map(|l| {
            let name = name_now(l);
            let color = match l.kind {
                _ if name == active && selection.contains(&name) => palette.active,
                _ if selection.contains(&name) => palette.selected,
                BExtraKind::Camera => palette.camera,
                BExtraKind::Light => palette.light,
                BExtraKind::Empty => palette.empty,
            };
            BProjectedLine { a: to_screen(l.a), b: to_screen(l.b), color }
        })
        .collect()
}

pub(super) fn paint(bounds: Bounds<Pixels>, lines: &[BProjectedLine], width: f32, window: &mut Window) {
    let w = f32::from(bounds.size.width);
    let h = f32::from(bounds.size.height);
    let at = |p: (f32, f32)| {
        point(
            bounds.origin.x + px((0.5 + p.0) * w),
            bounds.origin.y + px((0.5 + p.1) * h),
        )
    };
    for line in lines {
        let mut builder = PathBuilder::stroke(px(width));
        builder.move_to(at(line.a));
        builder.line_to(at(line.b));
        if let Ok(path) = builder.build() {
            window.paint_path(path, line.color);
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
        let palette = BExtraPalette { camera: to_hsla(c.text_dim), light: to_hsla(c.accent_soft), empty: to_hsla(c.text), selected: to_hsla(c.accent), active: to_hsla(c.bg_panel) };
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
}
