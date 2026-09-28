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
        let v = SceneView { yaw_deg: 90.0, pitch_deg: 45.0, perspective: false, camera_view: false, camzoom: 0.0, pivot: [1.0, 2.0, 3.0], width: 10.0, fov: 1.44, shading: "material", sidebar: true };
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
    fn load_ui_off_keeps_current_layout() {
        let mut s = BState::default();
        s.set(crate::v2::state::FileKey::LoadUi, "false");
        restore_ui(&mut s, &model("Shading", None));
        assert_eq!(s.get(CtxKey::Space), None);
    }
}
