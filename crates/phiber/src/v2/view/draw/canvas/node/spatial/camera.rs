//! The camera over the graph.
//!
//! The graph plane is the floor: `x` right and `y` up are `bNode.location`'s
//! own axes, and `z` is the height the solids rise to. The camera orbits a
//! pivot on the floor — yaw about `z`, pitch as elevation above the floor, so
//! pitch 90 looks straight down and IS the flat framing.
//!
//! **Nothing is stored that the flat view already knows.** The pivot is the
//! flat view's centre and the eye's distance is set by its width, so
//! Flat → Spatial lands on the same part of the graph and Spatial → Flat lands
//! back on it; a dolly or a pan in spatial rewrites the flat view rather than
//! a second copy of it (`gesture.rs`). Only the two angles are the spatial
//! view's own (`NodeKey::Orbit`).
//!
//! The distance is the one the 3D viewport opens with: Blender's default 50 mm
//! lens shows `LENS_WIDTH_PER_DIST` of width per unit of distance
//! (`scene/camera.rs`), so `dist = width / that` — the flat view's width fills
//! the canvas at the floor exactly as it does flat, and the relief above the
//! floor gets a 50 mm perspective rather than an invented one.

use super::super::view::BNodeView;
use super::set::{OPENING_PITCH, OPENING_YAW, PITCH_MAX, PITCH_MIN};
use crate::v2::host::BHostType;
use crate::v2::scene::camera::LENS_WIDTH_PER_DIST;
use crate::v2::state::{BState, NodeKey};

/// The two angles the spatial view keeps, in degrees.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BOrbit {
    pub yaw: f32,
    pub pitch: f32,
}

impl Default for BOrbit {
    fn default() -> Self {
        Self { yaw: OPENING_YAW, pitch: OPENING_PITCH }
    }
}

impl BOrbit {
    /// `"yaw, pitch"` — the spelling every pair in the store uses.
    pub fn encode(&self) -> String {
        format!("{}, {}", self.yaw, self.pitch)
    }

    /// A stored orbit back; anything unreadable is the opening orbit.
    pub fn decode(raw: &str) -> Option<Self> {
        let mut parts = raw.split(',').map(|p| p.trim().parse::<f32>().ok());
        let orbit = Self { yaw: parts.next()??, pitch: parts.next()?? };
        (orbit.yaw.is_finite() && orbit.pitch.is_finite()).then_some(orbit.clamped())
    }

    /// The orbit `host` is looking from.
    pub fn of(state: &BState, host: BHostType) -> Self {
        state.get(NodeKey::Orbit(host)).and_then(Self::decode).unwrap_or_default()
    }

    /// Turned by `by` degrees of yaw and of pitch, the pitch held to its limits.
    pub fn turned(self, by: (f32, f32)) -> Self {
        Self { yaw: self.yaw + by.0, pitch: self.pitch + by.1 }.clamped()
    }

    fn clamped(self) -> Self {
        Self { yaw: self.yaw.rem_euclid(360.0), pitch: self.pitch.clamp(PITCH_MIN, PITCH_MAX) }
    }
}

/// The camera for one frame: the flat view's framing, turned by an orbit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BSpatialCamera {
    /// Where the camera looks at, on the floor, in view units.
    pub pivot: (f32, f32),
    /// The eye's distance from the pivot, in view units.
    pub dist: f32,
    pub orbit: BOrbit,
    /// Canvas pixels per view unit at the floor — the flat view's own zoom.
    pub zoom: f32,
    /// Where the pivot lands on the canvas, in pixels — the flat view's own
    /// answer for it, so the two framings agree to the pixel.
    pub centre: (f32, f32),
}

impl BSpatialCamera {
    pub fn of(view: &BNodeView, orbit: BOrbit, region: (f32, f32)) -> Self {
        let pivot = ((view.xmin + view.xmax) / 2.0, (view.ymin + view.ymax) / 2.0);
        Self {
            pivot,
            dist: (view.width() / LENS_WIDTH_PER_DIST).max(1e-3),
            orbit,
            zoom: view.zoom(region.0),
            centre: view.to_region(pivot, region),
        }
    }

    /// The view's three axes in graph space: right, up (on screen) and forward
    /// (from the eye toward the pivot).
    fn axes(&self) -> ([f32; 3], [f32; 3], [f32; 3]) {
        let (sy, cy) = self.orbit.yaw.to_radians().sin_cos();
        let (sp, cp) = self.orbit.pitch.to_radians().sin_cos();
        let right = [cy, sy, 0.0];
        let forward = [-sy * cp, cy * cp, -sp];
        let up = [-sy * sp, cy * sp, cp];
        (right, up, forward)
    }

    /// The eye, in graph space.
    pub fn eye(&self) -> [f32; 3] {
        let (_, _, f) = self.axes();
        [self.pivot.0 - f[0] * self.dist, self.pivot.1 - f[1] * self.dist, -f[2] * self.dist]
    }

    /// A graph-space point on the canvas: `(x px, y px, depth)`, depth growing
    /// away from the eye in view units — or `None` when the point is behind
    /// the eye or too near it to project.
    pub fn project(&self, p: [f32; 3]) -> Option<(f32, f32, f32)> {
        let (r, u, f) = self.axes();
        let d = [p[0] - self.pivot.0, p[1] - self.pivot.1, p[2]];
        let dot = |a: [f32; 3]| a[0] * d[0] + a[1] * d[1] + a[2] * d[2];
        let depth = dot(f) + self.dist;
        if depth <= self.dist * 0.05 {
            return None;
        }
        // At the floor under the pivot the factor is 1, so a flat graph seen
        // from straight above lands exactly where the flat canvas puts it.
        let factor = self.zoom * self.dist / depth;
        let x = self.centre.0 + dot(r) * factor;
        let y = self.centre.1 - dot(u) * factor;
        Some((x, y, depth))
    }

    /// A whole ring projected, or `None` when any point of it is behind the
    /// eye — a face or a tube with one end behind the camera is not drawn
    /// rather than drawn through infinity.
    pub fn project_ring(&self, ring: &[[f32; 3]]) -> Option<Vec<(f32, f32, f32)>> {
        ring.iter().map(|p| self.project(*p)).collect()
    }

    /// Whether a face with outward `normal` through `point` faces the eye.
    pub fn faces_eye(&self, point: [f32; 3], normal: [f32; 3]) -> bool {
        let e = self.eye();
        let to_eye = [e[0] - point[0], e[1] - point[1], e[2] - point[2]];
        normal[0] * to_eye[0] + normal[1] * to_eye[1] + normal[2] * to_eye[2] > 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_view() -> BNodeView {
        BNodeView { xmin: -100.0, ymin: -50.0, xmax: 100.0, ymax: 50.0 }
    }

    #[test]
    fn straight_down_is_the_flat_framing() {
        // Pitch 90 from the flat view's own centre: every floor point lands
        // where `BNodeView::to_region` puts it, which is the whole promise of
        // the toggle landing on the same thing.
        let view = flat_view();
        let region = (400.0, 200.0);
        let camera = BSpatialCamera::of(&view, BOrbit { yaw: 0.0, pitch: 90.0 }, region);
        for at in [(-100.0, 50.0), (0.0, 0.0), (37.0, -12.0), (100.0, -50.0)] {
            let (x, y, _) = camera.project([at.0, at.1, 0.0]).expect("in front of the eye");
            let flat = view.to_region(at, region);
            assert!((x - flat.0).abs() < 1e-3 && (y - flat.1).abs() < 1e-3, "{at:?}: {x},{y} vs {flat:?}");
        }
    }

    #[test]
    fn height_rises_on_screen_and_the_near_edge_is_lower() {
        let camera = BSpatialCamera::of(&flat_view(), BOrbit::default(), (400.0, 200.0));
        let (_, floor, _) = camera.project([0.0, 0.0, 0.0]).unwrap();
        let (_, top, _) = camera.project([0.0, 0.0, 30.0]).unwrap();
        assert!(top < floor, "up is up on screen");
        // Looking north from the south, +y is farther away.
        let (_, _, near) = camera.project([0.0, -40.0, 0.0]).unwrap();
        let (_, _, far) = camera.project([0.0, 40.0, 0.0]).unwrap();
        assert!(far > near);
        assert!(camera.faces_eye([0.0, 0.0, 30.0], [0.0, 0.0, 1.0]), "a top face is seen");
        assert!(!camera.faces_eye([0.0, 0.0, 0.0], [0.0, 0.0, -1.0]), "a floor's underside is not");
    }

    #[test]
    fn an_orbit_round_trips_and_holds_its_pitch() {
        let orbit = BOrbit { yaw: 30.0, pitch: 45.0 };
        assert_eq!(BOrbit::decode(&orbit.encode()), Some(orbit));
        assert_eq!(BOrbit::decode("nonsense"), None);
        let over = orbit.turned((400.0, 90.0));
        assert_eq!(over.pitch, PITCH_MAX, "never past the vertical");
        assert!((over.yaw - 70.0).abs() < 1e-4, "yaw wraps: {}", over.yaw);
        assert_eq!(orbit.turned((0.0, -90.0)).pitch, PITCH_MIN, "never along the floor");
    }
}
