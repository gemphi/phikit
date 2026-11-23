//! Where a 3D picture is seen from (`PCamera`), worked out once for a
//! picture (`Lens`), and the sums on points in space the raster and the
//! light share.

/// Where a picture is seen from: an eye, what it looks at, which way is
/// up, how wide it sees (degrees, top to bottom), and how near it sees.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PCamera {
    pub eye: [f32; 3],
    pub at: [f32; 3],
    pub up: [f32; 3],
    pub fov: f32,
    pub near: f32,
}

/// A camera worked out once for a picture: its eye, its ways (ahead, to
/// the right, up), and how far a point one ahead is from the middle at
/// the picture's edge.
#[derive(Clone, Copy)]
pub(super) struct Lens {
    pub eye: [f32; 3],
    pub forward: [f32; 3],
    pub right: [f32; 3],
    pub up: [f32; 3],
    pub focal: f32,
}

impl Lens {
    pub fn project(&self, p: [f32; 3], width: f32, height: f32) -> [f32; 3] {
        let d = sub(p, self.eye);
        let (x, y, z) = (dot(d, self.right), dot(d, self.up), dot(d, self.forward));
        let scale = self.focal / z * height / 2.0;
        [x * scale + width / 2.0, height / 2.0 - y * scale, z]
    }
}

impl PCamera {
    pub(super) fn lens(&self) -> Lens {
        let forward = unit(sub(self.at, self.eye));
        let right = unit(cross(forward, self.up));
        Lens { eye: self.eye, forward, right, up: cross(right, forward), focal: 1.0 / (self.fov.to_radians() / 2.0).tan() }
    }

    /// Where a point falls on a picture of a size: across, down, and how
    /// far ahead it is (behind the eye where that is below `near`).
    pub fn project(&self, p: [f32; 3], width: f32, height: f32) -> [f32; 3] {
        self.lens().project(p, width, height)
    }

    /// Where the ray through a point of the picture meets the level
    /// plane at a height, if it does, ahead of the eye.
    pub fn floor(&self, across: f32, down: f32, width: f32, height: f32, level: f32) -> Option<[f32; 3]> {
        let Lens { forward, right, up, focal, .. } = self.lens();
        let (x, y) = ((across - width / 2.0) / (height / 2.0) / focal, (height / 2.0 - down) / (height / 2.0) / focal);
        let way = [0, 1, 2].map(|i| forward[i] + right[i] * x + up[i] * y);
        let t = (level - self.eye[1]) / way[1];
        (t.is_finite() && t > 0.0).then(|| [0, 1, 2].map(|i| self.eye[i] + way[i] * t))
    }

    /// The camera moved round what it looks at: turned about the upright
    /// by `turn` and tipped by `tip` (radians, up is more), and nearer by
    /// `near` times (more than one is nearer). How the 3D views are turned
    /// and zoomed.
    pub fn orbit(&self, turn: f32, tip: f32, near: f32) -> PCamera {
        let back = sub(self.eye, self.at);
        let far = dot(back, back).sqrt();
        let flat = (back[0] * back[0] + back[2] * back[2]).sqrt();
        let (yaw, pitch) = (back[0].atan2(back[2]) + turn, (back[1].atan2(flat) + tip).clamp(-1.5, 1.5));
        let far = far / near.max(f32::EPSILON);
        let eye = [self.at[0] + far * pitch.cos() * yaw.sin(), self.at[1] + far * pitch.sin(), self.at[2] + far * pitch.cos() * yaw.cos()];
        PCamera { eye, near: self.near / near.max(f32::EPSILON).max(1.0), ..*self }
    }
}

pub(super) fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub(super) fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

pub(super) fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub(super) fn unit(a: [f32; 3]) -> [f32; 3] {
    let l = dot(a, a).sqrt().max(f32::EPSILON);
    a.map(|v| v / l)
}
