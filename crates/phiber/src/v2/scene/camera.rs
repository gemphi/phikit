//! The viewport camera (#44) — Blender's `RegionView3D` orbit model.
//!
//! The view looks at `pivot` from the direction of its back axis
//! `b = (−sin yaw·cos pitch, −cos yaw·cos pitch, sin pitch)` (Blender world,
//! Z-up), with right `r = (cos yaw, −sin yaw, 0)` and up `u = b × r`, so
//! yaw 0 / pitch 0 is Blender's Front view (numpad 1) and pitch 90 is Top.
//! Orthographic scales by `zoom`; perspective places the eye `dist` behind
//! the pivot, where `dist` is the distance at which Blender's default lens
//! (36 mm sensor × zoom 2 / 50 mm) shows the same width as the ortho view.
//!
//! Inputs keep the projector's historic (x, up, z) = (X, Z, Y)·MESH_SCALE
//! convention; outputs are (screen x, screen y, depth) with screen units
//! relative to the canvas width on both axes (y is scaled by the aspect so
//! the painter's `y × height` stays square) and depth growing away from
//! the viewer (painters draw larger depths first).

use std::cell::Cell;

/// Screen units per scaled world unit at zoom 1.
pub const SCREEN: f32 = 0.42;
/// Blender viewport: visible width ≈ `LENS_WIDTH_PER_DIST × dist`.
pub const LENS_WIDTH_PER_DIST: f32 = 36.0 * 2.0 / 50.0;

/// `View3D.lens`: the viewport's focal length in mm — the area's own, else
/// the shared one, else Blender's 50.
pub fn view_lens(s: &crate::v2::state::BState) -> f32 {
    super::view_slot::BViewSlot::get(s, crate::v2::state::View3dKey::Lens)
        .and_then(|v| v.parse::<f32>().ok())
        .filter(|l| *l >= 1.0)
        .unwrap_or(DEFAULT_LENS)
}

/// Blender's default viewport lens, mm.
pub const DEFAULT_LENS: f32 = 50.0;

/// Visible width per unit of eye distance for a lens (`2 × 36 mm / lens`,
/// the sensor Blender's viewport assumes).
pub fn fov_of_lens(lens: f32) -> f32 {
    36.0 * 2.0 / lens.max(1.0)
}

/// The lens that shows `fov` — the inverse of [`fov_of_lens`].
pub fn lens_of_fov(fov: f32) -> f32 {
    36.0 * 2.0 / fov.max(1e-3)
}

/// Per-frame view settings the canvas scopes around projection.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BViewSetup {
    pub perspective: bool,
    /// Orbit centre in scaled projector units (x, up, z).
    pub pivot: (f32, f32, f32),
    /// Canvas width / height.
    pub aspect: f32,
    /// Visible width per unit of eye distance (Blender default lens: 1.44).
    pub fov: f32,
}

impl Default for BViewSetup {
    fn default() -> Self {
        Self { perspective: false, pivot: (0.0, 0.0, 0.0), aspect: 1.0, fov: LENS_WIDTH_PER_DIST }
    }
}

/// The canvas size assumed before the first paint reports one.
const FIRST_SIZE: (f32, f32) = (1280.0, 720.0);

thread_local! {
    static SETUP: Cell<BViewSetup> = Cell::new(BViewSetup::default());
    static LAST_ASPECT: Cell<f32> = const { Cell::new(FIRST_SIZE.0 / FIRST_SIZE.1) };
    static LAST_SIZE: Cell<(f32, f32)> = const { Cell::new(FIRST_SIZE) };
}

/// Phiber zoom showing `width` Blender units across the viewport.
pub fn zoom_for(width: f32) -> f32 {
    (1.0 / (super::MESH_SCALE * SCREEN * width.max(1e-3))).clamp(super::ZOOM_MIN, super::ZOOM_MAX)
}

/// Run `f` with `setup` as the active view.
pub fn with_view<R>(setup: BViewSetup, f: impl FnOnce() -> R) -> R {
    let prev = SETUP.with(|c| c.replace(setup));
    let out = f();
    SETUP.with(|c| c.set(prev));
    out
}

/// Aspect of the last painted canvas (projection runs before layout).
pub fn last_aspect() -> f32 {
    LAST_ASPECT.with(Cell::get)
}

/// Record the painted aspect; true when it changed (caller refreshes).
pub fn note_aspect(aspect: f32) -> bool {
    let a = if aspect.is_finite() && aspect > 0.0 { aspect } else { return false };
    LAST_ASPECT.with(|c| (c.replace(a) - a).abs() > 1e-3)
}

/// Record the painted size — the camera frame is fitted to it — and its
/// aspect; true when either changed (caller refreshes).
pub fn note_size(width: f32, height: f32) -> bool {
    if !(width.is_finite() && height.is_finite() && width > 0.0 && height > 0.0) {
        return false;
    }
    let moved = LAST_SIZE.with(|c| c.replace((width, height)) != (width, height));
    note_aspect(width / height) || moved
}

/// Size of the last painted canvas, in pixels.
pub fn last_size() -> (f32, f32) {
    LAST_SIZE.with(Cell::get)
}

/// The pivot stored under `View3dKey::Pivot.path()` (Blender units), origin if unset.
pub fn view_pivot(s: &crate::v2::state::BState) -> [f32; 3] {
    let raw = super::view_slot::BViewSlot::get(s, crate::v2::state::View3dKey::Pivot).unwrap_or("");
    let v: Vec<f32> = raw.split(',').filter_map(|p| p.trim().parse().ok()).collect();
    if v.len() == 3 { [v[0], v[1], v[2]] } else { [0.0; 3] }
}

/// The view setup the canvas uses this frame.
pub fn setup_from_state(s: &crate::v2::state::BState) -> BViewSetup {
    let p = view_pivot(s);
    let ms = super::MESH_SCALE;
    BViewSetup {
        perspective: super::is_perspective(s),
        pivot: (p[0] * ms, p[2] * ms, p[1] * ms),
        aspect: last_aspect(),
        fov: fov_of_lens(view_lens(s)),
    }
}

/// Rotate a projector-space direction into view space (right, up, back).
pub fn to_view(v: (f32, f32, f32), yaw_deg: f32, pitch_deg: f32) -> (f32, f32, f32) {
    let (sy, cy) = yaw_deg.to_radians().sin_cos();
    let (sp, cp) = pitch_deg.to_radians().sin_cos();
    let (x, y, z) = (v.0, v.2, v.1); // projector (x, up, z) → Blender (X, Y, Z)
    let right = x * cy - y * sy;
    let up = x * sp * sy + y * sp * cy + z * cp;
    let back = -x * sy * cp - y * cy * cp + z * sp;
    (right, up, back)
}

/// Project a projector-space point with the active [`BViewSetup`].
pub fn view_project(x: f32, up: f32, z: f32, yaw: f32, pitch: f32, pan: (f32, f32), zoom: f32) -> (f32, f32, f32) {
    let s = SETUP.with(Cell::get);
    let d = (x - s.pivot.0, up - s.pivot.1, z - s.pivot.2);
    let (vx, vy, vz) = to_view(d, yaw, pitch);
    let scale = SCREEN * zoom;
    let (factor, depth) = if s.perspective {
        let dist = 1.0 / (scale * s.fov);
        let ahead = (dist - vz).max(dist * 0.05);
        (dist / ahead, ahead)
    } else {
        (1.0, -vz)
    };
    let sx = vx * scale * factor + pan.0;
    let sy = (-vy * scale * factor + pan.1) * s.aspect;
    (sx, sy, depth)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn front_view_maps_x_right_and_z_up() {
        let (x, y, _) = view_project(1.0, 0.0, 0.0, 0.0, 0.0, (0.0, 0.0), 1.0);
        assert!((x - SCREEN).abs() < 1e-6 && y.abs() < 1e-6);
        let (x, y, _) = view_project(0.0, 1.0, 0.0, 0.0, 0.0, (0.0, 0.0), 1.0);
        assert!(x.abs() < 1e-6 && (y + SCREEN).abs() < 1e-6, "up is up on screen");
    }

    #[test]
    fn front_view_depth_grows_along_plus_y() {
        // Front view looks along +Y: farther points have larger Y.
        let (_, _, near) = view_project(0.0, 0.0, -1.0, 0.0, 0.0, (0.0, 0.0), 1.0);
        let (_, _, far) = view_project(0.0, 0.0, 1.0, 0.0, 0.0, (0.0, 0.0), 1.0);
        assert!(far > near);
    }

    #[test]
    fn top_view_shows_plus_y_upward() {
        let (_, y, _) = view_project(0.0, 0.0, 1.0, 0.0, 90.0, (0.0, 0.0), 1.0);
        assert!(y < 0.0, "+Y is toward the top of the screen in Top view");
    }

    #[test]
    fn perspective_shrinks_far_points_and_keeps_pivot_scale() {
        let setup = BViewSetup { perspective: true, ..Default::default() };
        with_view(setup, || {
            let (at_pivot, _, _) = view_project(1.0, 0.0, 0.0, 0.0, 0.0, (0.0, 0.0), 1.0);
            let (far, _, _) = view_project(1.0, 0.0, 1.0, 0.0, 0.0, (0.0, 0.0), 1.0);
            assert!((at_pivot - SCREEN).abs() < 1e-5);
            assert!(far < at_pivot);
        });
    }

    #[test]
    fn aspect_keeps_pixels_square() {
        let setup = BViewSetup { aspect: 2.0, ..Default::default() };
        with_view(setup, || {
            let (_, y, _) = view_project(0.0, 1.0, 0.0, 0.0, 0.0, (0.0, 0.0), 1.0);
            assert!((y + 2.0 * SCREEN).abs() < 1e-6);
        });
    }
}
