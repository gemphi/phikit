//! "Load UI" (#28): the workspace and 3D view a `.blend` restores, reduced
//! to what the viewport consumes. Blender ref: `RegionView3D` — the view
//! looks down `-viewinv[2]` at the pivot `-ofs`, `dist` away; `persp` 0 =
//! ortho, 1 = perspective, 2 = through `Scene.camera`.

use crate::blend::{BlendSceneObject, BlendUiState, RV3D_CAMOB, RV3D_ORTHO};

/// Blender viewport sensor (`DEFAULT_SENSOR_WIDTH`) × initial zoom (2) over
/// the default lens (50 mm): visible width ≈ `VIEW_WIDTH_PER_DIST * dist`
/// (`BKE_camera_params_from_view3d`, `CAMERA_PARAM_ZOOM_INIT_PERSP`).
pub const VIEW_WIDTH_PER_DIST: f32 = 36.0 * 2.0 / 50.0;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SceneView {
    /// Azimuth about world Z, degrees `[0, 360)`; 0 looks along +Y.
    pub yaw_deg: f32,
    /// Elevation, degrees; positive looks down.
    pub pitch_deg: f32,
    pub perspective: bool,
    /// The file was saved looking through the scene camera.
    pub camera_view: bool,
    /// The camera frame's zoom in that view (`RegionView3D.camzoom`), which
    /// only means anything while `camera_view` is set.
    pub camzoom: f32,
    /// World-space point the view orbits (`-RegionView3D.ofs`).
    pub pivot: [f32; 3],
    /// Visible width in Blender units at the pivot.
    pub width: f32,
    /// Perspective spread: visible width per unit of distance from the eye.
    pub fov: f32,
    /// Saved shading: `"wireframe"`, `"solid"`, `"material"`, `"rendered"`, or empty.
    pub shading: &'static str,
    /// The viewport's sidebar (N panel) was open.
    pub sidebar: bool,
    /// X-ray as the view had it, `None` when the file says nothing.
    pub xray: Option<SceneXray>,
    /// Workbench's Solid effects as the view had them.
    pub solid_fx: Option<SceneSolidFx>,
    /// Solid's Lighting, Color and Background as the view had them.
    pub solid_shade: Option<SceneSolidShade>,
    /// The Overlays and Gizmos popovers' switches, by RNA name.
    pub overlay: crate::blend::BlendOverlay,
    /// The view's own render region, outside the camera view
    /// (`View3D.render_border`, `use_render_border`).
    pub border: crate::blend::BlendViewBorder,
}

/// `View3DShading`'s X-ray: Solid's switch and alpha (`show_xray`,
/// `xray_alpha`), Wireframe's (`show_xray_wireframe`, `xray_alpha_wireframe`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SceneXray {
    pub solid: bool,
    pub alpha: f32,
    pub wireframe: bool,
    pub alpha_wireframe: f32,
}

impl SceneXray {
    /// `V3D_SHADING_XRAY` and `V3D_SHADING_XRAY_WIREFRAME` with their alphas.
    pub fn of(sh: &crate::blend::BlendShading) -> Self {
        Self { solid: sh.flag & (1 << 1) != 0, alpha: sh.xray_alpha[0], wireframe: sh.flag & (1 << 8) != 0, alpha_wireframe: sh.xray_alpha[1] }
    }
}

/// `View3DShading`'s Solid effects: the object outline (`show_object_outline`,
/// `object_outline_color`) and Cavity (`show_cavity`, `cavity_type` as
/// `"WORLD"` for SSAO, `"SCREEN"` for Curvature, `"BOTH"`, its four factors).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SceneSolidFx {
    pub outline: bool,
    pub outline_color: [f32; 3],
    pub cavity: bool,
    pub cavity_type: &'static str,
    pub cavity_valley_factor: f32,
    pub cavity_ridge_factor: f32,
    pub curvature_ridge_factor: f32,
    pub curvature_valley_factor: f32,
    /// Shadow (`show_shadows`, `V3D_SHADING_SHADOW`) and its intensity.
    pub shadow: bool,
    pub shadow_intensity: f32,
    /// `show_backface_culling` (`V3D_SHADING_BACKFACE_CULLING`) and
    /// `show_specular_highlight` (`V3D_SHADING_SPECULAR_HIGHLIGHT`).
    pub backface_culling: bool,
    pub specular: bool,
    /// `use_dof` (`V3D_SHADING_DEPTH_OF_FIELD`): Workbench's depth of
    /// field through the camera looked through.
    pub dof: bool,
}

impl SceneSolidFx {
    /// `V3D_SHADING_OBJECT_OUTLINE`, `_CAVITY`, `_SHADOW`, `_BACKFACE_CULLING`,
    /// `_SPECULAR_HIGHLIGHT`, `_DEPTH_OF_FIELD`; the RNA names of
    /// `eView3DShading_CavityType`.
    pub fn of(sh: &crate::blend::BlendShading) -> Self {
        Self {
            outline: sh.flag & 1 != 0,
            outline_color: sh.outline_color,
            cavity: sh.flag & (1 << 5) != 0,
            cavity_type: match sh.cavity_type { 0 => "WORLD", 2 => "BOTH", _ => "SCREEN" },
            cavity_valley_factor: sh.cavity[0],
            cavity_ridge_factor: sh.cavity[1],
            curvature_ridge_factor: sh.cavity[2],
            curvature_valley_factor: sh.cavity[3],
            shadow: sh.flag & (1 << 2) != 0,
            shadow_intensity: sh.shadow_intensity,
            backface_culling: sh.flag & (1 << 10) != 0,
            specular: sh.flag & (1 << 4) != 0,
            dof: sh.flag & (1 << 11) != 0,
        }
    }
}

/// `View3DShading`'s Lighting, Color and Background, by RNA item: `light`
/// (`STUDIO`, `MATCAP`, `FLAT`), `color_type` (`MATERIAL`, `RANDOM`,
/// `SINGLE`, `TEXTURE`, `OBJECT`, `VERTEX`) with `single_color`, and
/// `background_type` (`THEME`, `WORLD`, `VIEWPORT`) with `background_color`.
#[derive(Clone, Debug, PartialEq)]
pub struct SceneSolidShade {
    pub light: &'static str,
    pub color_type: &'static str,
    /// `View3DShading.matcap` (the file's stem) and `V3D_SHADING_MATCAP_FLIP_X`.
    pub matcap: String,
    pub matcap_flip_x: bool,
    pub single_color: [f32; 3],
    pub background_type: &'static str,
    pub background_color: [f32; 3],
}

impl SceneSolidShade {
    /// The RNA items of `eV3DShadingLightingMode`, `eV3DShadingColorType`
    /// and `eV3DShadingBackgroundType`; `V3D_SHADING_MATCAP_FLIP_X`.
    pub fn of(sh: &crate::blend::BlendShading) -> Self {
        Self {
            light: match sh.light { 0 => "FLAT", 2 => "MATCAP", _ => "STUDIO" },
            matcap: sh.matcap.clone(),
            matcap_flip_x: sh.flag & (1 << 6) != 0,
            color_type: match sh.color_type { 1 => "RANDOM", 2 => "SINGLE", 3 => "TEXTURE", 4 => "OBJECT", 5 => "VERTEX", _ => "MATERIAL" },
            single_color: sh.single_color,
            background_type: match sh.background_type { 1 => "WORLD", 2 => "VIEWPORT", _ => "THEME" },
            background_color: sh.background_color,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SceneUi {
    /// Active workspace name ("Layout", "Shading", …); empty when unknown.
    pub workspace: String,
    pub view: Option<SceneView>,
}

impl SceneView {
    /// Yaw / pitch of a view whose back axis (`m[2]`, Blender layout) is `z`.
    pub fn angles(z: [f32; 3]) -> (f32, f32) {
        let len = (z[0] * z[0] + z[1] * z[1] + z[2] * z[2]).sqrt().max(1e-9);
        let yaw = (-z[0]).atan2(-z[1]).to_degrees().rem_euclid(360.0);
        let pitch = (z[2] / len).clamp(-1.0, 1.0).asin().to_degrees();
        (yaw, pitch)
    }
}

/// Reduce the reader's UI state; `objects` resolves `Scene.camera`.
pub(super) fn ui_from_blend(ui: &BlendUiState, objects: &[BlendSceneObject]) -> SceneUi {
    let view = ui.view3d.as_ref().map(|v| {
        let cam = objects.iter().find(|o| ui.camera != 0 && o.block_address == ui.camera);
        let camera_view = v.persp == RV3D_CAMOB && cam.is_some();
        let lens = if v.lens > 0.0 { v.lens } else { 50.0 };
        let mut fov = 72.0 / lens;
        let (z, mut perspective, mut width) = (v.view_inv[2], v.persp != RV3D_ORTHO, v.dist * fov);
        let mut pivot = [-v.ofs[0], -v.ofs[1], -v.ofs[2]];
        let z = match (camera_view, cam) {
            (true, Some(c)) => {
                // Looking through the camera: its position is the eye and its
                // lens the field of view (frame fitted to the viewport width).
                let bz = c.matrix_world[2];
                let n = (bz[0] * bz[0] + bz[1] * bz[1] + bz[2] * bz[2]).sqrt().max(1e-9);
                let eye = c.matrix_world[3];
                if let crate::blend::BlendObjectData::Camera(d) = &c.data {
                    perspective = d.kind != 1;
                    fov = d.sensor_x.max(1e-3) / d.lens.max(1e-3);
                    width = if perspective { v.dist * fov } else { d.ortho_scale };
                }
                pivot = std::array::from_fn(|k| eye[k] - bz[k] / n * v.dist);
                bz
            }
            _ => z,
        };
        let (yaw_deg, pitch_deg) = SceneView::angles([z[0], z[1], z[2]]);
        SceneView {
            yaw_deg,
            pitch_deg,
            perspective,
            camera_view,
            camzoom: v.camzoom,
            pivot,
            width: width.max(1e-3),
            fov,
            shading: match v.shading.ty { 2 => "wireframe", 3 => "solid", 4 => "material", 5 => "rendered", _ => "" },
            sidebar: v.sidebar,
            overlay: v.overlay.clone(),
            border: v.border,
            xray: Some(SceneXray::of(&v.shading)),
            solid_fx: Some(SceneSolidFx::of(&v.shading)),
            solid_shade: Some(SceneSolidShade::of(&v.shading)),
        }
    });
    SceneUi { workspace: ui.workspace.clone(), view }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn front_view_looks_along_plus_y_level() {
        // Numpad 1: back axis −Y.
        let (yaw, pitch) = SceneView::angles([0.0, -1.0, 0.0]);
        assert!(yaw.abs() < 1e-3 && pitch.abs() < 1e-3, "{yaw} {pitch}");
    }

    #[test]
    fn top_view_pitches_straight_down() {
        let (_, pitch) = SceneView::angles([0.0, 0.0, 1.0]);
        assert!((pitch - 90.0).abs() < 1e-3);
    }

    #[test]
    fn right_view_yaws_a_quarter_turn() {
        // Numpad 3 looks along −X: back axis +X.
        let (yaw, _) = SceneView::angles([1.0, 0.0, 0.0]);
        assert!((yaw - 270.0).abs() < 1e-3, "{yaw}");
    }

    #[test]
    fn the_solid_effects_read_their_flag_bits() {
        let sh = crate::blend::BlendShading { flag: (1 << 0) | (1 << 2) | (1 << 10) | (1 << 11), ..Default::default() };
        let fx = SceneSolidFx::of(&sh);
        assert!(fx.outline && fx.shadow && fx.backface_culling && fx.dof && !fx.cavity && !fx.specular);
        assert!(!SceneSolidFx::of(&Default::default()).dof, "Blender's default view has no depth of field");
    }
}
