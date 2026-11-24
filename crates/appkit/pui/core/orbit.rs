//! A thing in 3D seen by hand: its pieces framed by a camera far enough
//! back to hold them, and the view turned and tipped by a drag and
//! brought nearer by the wheel, within the theme's limits. The raster
//! (`raster.rs`) draws what this frames. Any app's 3D view stands on it
//! (Phiogy's library; phibot's robot frames its own the same way).

use std::cell::Cell;

use super::{PCamera, PPoint, PRaster, PSolid, PSun};
use crate::theme::PTheme;

/// The theme's tokens: the colour a 3D picture's ground is, how far round
/// a thing the camera stands (a share of its reach), how many times
/// larger a picture is made than shown, how many points a drag goes to
/// turn a view a radian, how much nearer a notch of the wheel brings it,
/// and how near and how far it may come.
pub const GROUND: &str = "scene.ground";
pub const ROOM: &str = "scene.room";
pub const SMOOTH: &str = "scene.smooth";
pub const DRAG: &str = "scene.drag";
pub const ZOOM: &str = "scene.zoom";
pub const NEAREST: &str = "scene.nearest";
pub const FARTHEST: &str = "scene.farthest";

type Tri = [[f32; 3]; 3];

/// A piece of a thing: its triangles where they stand, the way the
/// surface faces at each corner, and its colour (0 to 1).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PPiece {
    pub triangle: Vec<Tri>,
    pub normal: Vec<Tri>,
    pub colour: [f32; 3],
}

impl PPiece {
    /// Triangles in a colour, each corner facing as its triangle does.
    pub fn flat(triangle: Vec<Tri>, colour: [f32; 3]) -> PPiece {
        let normal = triangle.iter().map(|t| [face(t); 3]).collect();
        PPiece { triangle, normal, colour }
    }
}

/// The way a triangle faces.
fn face(t: &Tri) -> [f32; 3] {
    let (a, b) = ([0, 1, 2].map(|i| t[1][i] - t[0][i]), [0, 1, 2].map(|i| t[2][i] - t[0][i]));
    let n = [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
    let long = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(f32::EPSILON);
    n.map(|v| v / long)
}

/// A picture's ground, blue, green, red and solid, from the theme.
pub fn ground(theme: &dyn PTheme) -> [u8; 4] {
    let c = theme.color(GROUND).unwrap_or_default();
    [c.b, c.g, c.r, c.a].map(|v| (v * 255.0) as u8)
}

/// How many times larger than shown a picture is made.
pub fn smooth(theme: &dyn PTheme) -> u32 {
    theme.size(SMOOTH).unwrap_or(1.0).max(1.0) as u32
}

/// The box round some pieces: its lowest and its highest corner.
pub fn bounds(piece: &[PPiece]) -> ([f32; 3], [f32; 3]) {
    let (mut low, mut high) = ([f32::MAX; 3], [f32::MIN; 3]);
    for v in piece.iter().flat_map(|p| p.triangle.iter().flatten()) {
        (0..3).for_each(|i| (low[i], high[i]) = (low[i].min(v[i]), high[i].max(v[i])));
    }
    (low, high)
}

/// A camera looking along a way at some pieces, far enough back that
/// every corner of their box is in the picture (`wide` is its width
/// over its height), with the room the theme says round them.
pub fn framed(piece: &[PPiece], way: [f32; 3], fov: f32, room: f32, wide: f32) -> PCamera {
    let (low, high) = bounds(piece);
    let middle = [0, 1, 2].map(|i| (low[i] + high[i]) / 2.0);
    let reach = (0..3).map(|i| (high[i] - low[i]) / 2.0).fold(0.0, f32::max).max(f32::EPSILON);
    let long = (way[0] * way[0] + way[1] * way[1] + way[2] * way[2]).sqrt().max(f32::EPSILON);
    let corner: Vec<[f32; 3]> = (0..8).map(|n| [0, 1, 2].map(|i| [low[i], high[i]][(n >> i) & 1])).collect();
    let at = |back: f32| PCamera { eye: [0, 1, 2].map(|i| middle[i] - way[i] / long * back), at: middle, up: [0.0, 1.0, 0.0], fov, near: reach * 0.01 };
    let mut back = reach * 2.0 / (fov.to_radians() / 2.0).tan();
    for _ in 0..6 {
        let camera = at(back);
        let out = corner.iter().map(|c| camera.project(*c, wide * 2.0, 2.0)).map(|p| ((p[0] - wide) / wide).abs().max((p[1] - 1.0).abs())).fold(0.0, f32::max);
        back *= (out * room.max(1.0)).max(0.5);
    }
    at(back)
}

/// A camera round some pieces, turned so far about them (radians), a
/// little above them.
pub fn around(piece: &[PPiece], turn: f32, fov: f32, room: f32, wide: f32) -> PCamera {
    framed(piece, [-turn.sin(), -0.25, -turn.cos()], fov, room, wide)
}

/// Pieces drawn into a picture of that size, made larger and averaged
/// down as the theme says: its width, height and points.
pub fn shot(piece: &[PPiece], camera: &PCamera, sun: &PSun, size: (u32, u32), theme: &dyn PTheme) -> (u32, u32, Vec<u8>) {
    let by = smooth(theme);
    let mut raster = PRaster::new(size.0.max(1) * by, size.1.max(1) * by, ground(theme));
    let shape: Vec<PSolid> = piece.iter().map(|p| PSolid { triangle: &p.triangle, normal: &p.normal, colour: p.colour }).collect();
    raster.draw(camera, &shape, sun);
    raster.smooth(by)
}

/// How a view is turned, tipped and brought nearer by hand: a drag turns
/// and tips it, the wheel zooms it, within the theme's limits.
#[derive(Debug, Default)]
pub struct POrbit {
    turn: Cell<f32>,
    tip: Cell<f32>,
    zoom: Cell<f32>,
    /// Where a press began, and how the view was then; and whether it
    /// has dragged since, so a press that did not is a press.
    held: Cell<Option<(PPoint, f32, f32)>>,
    dragged: Cell<bool>,
}

impl POrbit {
    /// The camera as the hand has it, from the one it began with.
    pub fn camera(&self, from: &PCamera) -> PCamera {
        from.orbit(self.turn.get(), self.tip.get(), self.zoom.get().exp())
    }

    /// How it stands now: turned, tipped, zoomed.
    pub fn now(&self) -> [f32; 3] {
        [self.turn.get(), self.tip.get(), self.zoom.get()]
    }

    pub fn down(&self, at: PPoint) {
        self.held.set(Some((at, self.turn.get(), self.tip.get())));
        self.dragged.set(false);
    }

    /// A move while pressed turns and tips it. `true` where it changed.
    pub fn moved(&self, at: PPoint, theme: &dyn PTheme) -> bool {
        let Some((from, turn, tip)) = self.held.get() else { return false };
        let per = theme.size(DRAG).unwrap_or(1.0).max(f32::EPSILON);
        let (dx, dy) = (at.x - from.x, at.y - from.y);
        self.dragged.set(self.dragged.get() || dx.hypot(dy) > per * 0.02);
        self.turn.set(turn - dx / per);
        self.tip.set(tip + dy / per);
        self.dragged.get()
    }

    /// Let go: `true` where it was a press and not a drag.
    pub fn up(&self) -> bool {
        let pressed = self.held.take().is_some() && !self.dragged.get();
        self.dragged.set(false);
        pressed
    }

    /// A turn of the wheel brings it nearer or farther.
    pub fn wheel(&self, by: f32, theme: &dyn PTheme) -> bool {
        let step = theme.size(ZOOM).unwrap_or_default();
        let (near, far) = (theme.size(NEAREST).unwrap_or(1.0).ln(), theme.size(FARTHEST).unwrap_or(1.0).ln());
        let zoom = (self.zoom.get() + by.signum() * step).clamp(far.min(near), near.max(far));
        self.zoom.replace(zoom) != zoom
    }
}
