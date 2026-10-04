//! "Load UI" (#28) — after a successful open, restore the workspace tab and
//! the 3D view the file was saved with (`WM_OT_open_mainfile.load_ui`).
//! Blender's view is mapped onto Phiber's orbit model: yaw / pitch from the
//! view's back axis, zoom from the visible width, pan so the saved pivot
//! sits at the viewport centre.

use crate::v2::state::{CtxKey, View3dKey};
use crate::phiphx::scene::{SceneModel, SceneView};
use crate::v2::part::file::option::load_ui;
use crate::v2::scene::shading::{BShading, shading_key};
use crate::v2::space::BSpaceType;
use crate::v2::state::{BState};

pub use crate::v2::scene::camera::zoom_for;

/// Workspace tab for a Blender workspace name ("UV Editing" / "UvEditing").
pub fn space_for(name: &str) -> Option<BSpaceType> {
    BSpaceType::from_slug(name).or_else(|| {
        BSpaceType::ALL
            .iter()
            .copied()
            .find(|s| s.label().eq_ignore_ascii_case(name) || s.slug().eq_ignore_ascii_case(&name.replace(' ', "")))
    })
}

/// Write the view keys for `v`.
pub fn apply_view(state: &mut BState, v: &SceneView) {
    let zoom = zoom_for(v.width);
    let p = v.pivot;
    // The file's view is every viewport's: the shared keys, and each area's
    // own view dropped so it reads them (`scene::view_slot`).
    crate::v2::scene::view_slot::BViewSlot::reset(state);
    state.set(View3dKey::Yaw, format!("{:.3}", v.yaw_deg));
    state.set(View3dKey::Pitch, format!("{:.3}", v.pitch_deg.clamp(-90.0, 90.0)));
    state.set(View3dKey::Zoom, format!("{zoom:.4}"));
    state.set(View3dKey::Perspective, v.perspective.to_string());
    // The camera orbits the saved pivot; screen pan starts centred.
    state.set(View3dKey::Pivot, format!("{:.5},{:.5},{:.5}", p[0], p[1], p[2]));
    state.set(View3dKey::Lens, format!("{:.3}", crate::v2::scene::camera::lens_of_fov(v.fov)));
    state.set(View3dKey::PanX, "0");
    state.set(View3dKey::PanY, "0");
    if BShading::from_slug(v.shading).is_some() {
        state.set(&shading_key(), v.shading.to_string());
    }
    // X-ray as the view had it (`View3DShading.flag`, its two alphas).
    if let Some(x) = v.xray {
        let on = |b: bool| if b { crate::v2::state::val::V_TRUE } else { crate::v2::state::val::V_FALSE };
        state.set(View3dKey::Xray, on(x.solid));
        state.set(View3dKey::XrayAlpha, x.alpha.to_string());
        state.set(View3dKey::XrayWireframe, on(x.wireframe));
        state.set(View3dKey::XrayAlphaWireframe, x.alpha_wireframe.to_string());
    }
    // Workbench's outline and Cavity as the view had them (`View3DShading`).
    if let Some(fx) = &v.solid_fx {
        crate::v2::scene::BSolidFx::seed(state, fx);
    }
    if let Some(sh) = &v.solid_shade {
        crate::v2::scene::BSolidShade::seed(state, sh);
    }
    // The Overlays and Gizmos popovers as the view had them (`View3D.overlay`).
    crate::v2::scene::BOverlays::seed(state, &v.overlay.list);
    // The view's own render region (`View3D.render_border`).
    crate::v2::scene::view_border::BViewBorder::seed(state, v.border);
    // The sidebar as the file left it (`SpaceView3D.show_region_ui`).
    let shown = crate::v2::part::sidebar::show::BSidebarShown::view3d_key();
    state.set(&shown, if v.sidebar { crate::v2::state::val::V_TRUE } else { crate::v2::state::val::V_FALSE });
}

pub fn restore_ui(state: &mut BState, model: &SceneModel) {
    if !load_ui(state) {
        return;
    }
    if let Some(space) = space_for(&model.ui.workspace) {
        state.set(CtxKey::Space, space.slug().to_string());
    }
    if let Some(v) = &model.ui.view {
        apply_view(state, v);
    }
    // Cavity ▸ World's and Shadow's settings are the scene's
    // (`Scene.display`), not the view's.
    crate::v2::scene::BSolidFx::seed_ssao(state, &model.render.ssao);
    crate::v2::scene::BSolidFx::seed_shadow(state, &model.render.shadow);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phiphx::scene::SceneUi;
    use crate::v2::scene::{is_perspective, view_pan, view_pitch, view_yaw, view_zoom};

    fn model(ws: &str, view: Option<SceneView>) -> SceneModel {
        SceneModel { ui: SceneUi { workspace: ws.into(), view }, ..Default::default() }
    }

    #[test]
    fn restores_workspace_and_view() {
        let v = SceneView { yaw_deg: 90.0, pitch_deg: 45.0, perspective: false, camera_view: false, camzoom: 0.0, pivot: [1.0, 2.0, 3.0], width: 10.0, fov: 1.44, shading: "material", sidebar: true, xray: None, solid_fx: None, solid_shade: None, overlay: Default::default(), border: Default::default() };
        let mut s = BState::default();
        restore_ui(&mut s, &model("UV Editing", Some(v)));
        assert_eq!(s.get(CtxKey::Space), Some("UvEditing"));
        assert_eq!((view_yaw(&s), view_pitch(&s)), (90.0, 45.0));
        assert!(!is_perspective(&s));
        assert_eq!(s.get(&shading_key()), Some("material"));
        assert!((view_zoom(&s) - zoom_for(10.0)).abs() < 1e-3);
        assert_eq!(view_pan(&s), (0.0, 0.0));
        assert_eq!(crate::v2::scene::camera::view_pivot(&s), [1.0, 2.0, 3.0]);
        // The lens behind a 1.44 spread is Blender's 50 mm, and the sidebar
        // opens as the file left it.
        assert!((crate::v2::scene::camera::view_lens(&s) - 50.0).abs() < 1e-2);
        assert!(crate::v2::part::sidebar::show::BSidebarShown(crate::v2::host::BHostType::View3D).on(&s));
    }

    #[test]
    fn a_file_opens_with_its_view_s_overlays() {
        // `truth/blender_overlays.py` set each switch away from its default.
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../phiphx/tests/fixtures/overlays.blend");
        let scene = crate::phiphx::blend::BlendFile::parse_bytes(&std::fs::read(&path).unwrap()).unwrap().extract_scene().unwrap();
        let model = SceneModel::from_blend_owned(scene, &path.to_string_lossy());
        let mut s = BState::default();
        apply_view(&mut s, model.ui.view.as_ref().expect("the Layout view"));
        let o = crate::v2::scene::BOverlays::now(&s);
        assert!(!o.floor && o.axis_z && !o.text && o.stats && !o.bones && !o.outline && o.origins_all && !o.navigate, "{o:?}");
        assert!(o.cursor && o.extras && o.relations && o.origins && o.axis_x);
        // Workbench's Solid effects come with the view too (`View3DShading`):
        // this file keeps the startup's — the outline on, Cavity off.
        let fx = crate::v2::scene::BSolidFx::now(&s);
        assert!(fx.outline && !fx.cavity && fx.cavity_type == "SCREEN", "{fx:?}");
        assert_eq!(s.get(crate::v2::state::View3dKey::Shade("show_object_outline")).as_deref(), Some("true"), "seeded, not the factory default");
    }

    #[test]
    fn load_ui_off_keeps_current_layout() {
        let mut s = BState::default();
        s.set(crate::v2::state::FileKey::LoadUi, "false");
        restore_ui(&mut s, &model("Shading", None));
        assert_eq!(s.get(CtxKey::Space), None);
    }
}
