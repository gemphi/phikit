//! The 3D pictures: the robot and the greenhouse, made by the kit's
//! raster from their meshes and shown as pictures that move. The light is
//! the robot's table's; the ground, how much larger a picture is made
//! than shown (so no edge is jagged), and how a drag turns it and the
//! wheel brings it nearer are the theme's. A press that does not drag is
//! a press.

use std::cell::Cell;

use appkit::pui::core::{PCamera, PPoint, PRaster, PSolid, PSun};
use appkit::theme::PTheme;
use phiace::bot::botanist::Data;
use phiace::bot::botanist::figure::Piece;

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

/// Three numbers from a row of the robot's table.
pub fn three(data: &Data, key: &str) -> [f32; 3] {
    let v: Vec<f32> = data.game.get(key).split_whitespace().filter_map(|n| n.parse().ok()).collect();
    [0, 1, 2].map(|i| v.get(i).copied().unwrap_or_default())
}

/// The light, from the table.
pub fn sun(data: &Data) -> PSun {
    let n = |key: &str| data.game.num(key);
    PSun { way: three(data, "sun.way"), key: n("sun.key"), sky: three(data, "light.sky"), ground: three(data, "light.ground"), shine: n("sun.shine"), gloss: n("sun.gloss") }
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
pub fn bounds(piece: &[Piece]) -> ([f32; 3], [f32; 3]) {
    let (mut low, mut high) = ([f32::MAX; 3], [f32::MIN; 3]);
    for v in piece.iter().flat_map(|p| p.triangle.iter().flatten()) {
        (0..3).for_each(|i| (low[i], high[i]) = (low[i].min(v[i]), high[i].max(v[i])));
    }
    (low, high)
}

/// A camera looking along a way at some pieces, far enough back that
/// every corner of their box is in the picture (`wide` is its width over
/// its height), with the room the theme says round them. Nearer corners
/// look larger, so it steps back until the farthest out is in.
pub fn framed(piece: &[Piece], way: [f32; 3], fov: f32, room: f32, wide: f32) -> PCamera {
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
pub fn around(piece: &[Piece], turn: f32, fov: f32, room: f32, wide: f32) -> PCamera {
    framed(piece, [-turn.sin(), -0.25, -turn.cos()], fov, room, wide)
}

/// The camera the robot's table gives a view: its way, from its eye to
/// what it looks at.
pub fn way(data: &Data) -> [f32; 3] {
    let (eye, at) = (three(data, "camera.eye"), three(data, "camera.at"));
    [0, 1, 2].map(|i| at[i] - eye[i])
}

/// Draw pieces into a picture.
pub fn draw(raster: &mut PRaster, camera: &PCamera, piece: &[Piece], sun: &PSun) {
    let shape: Vec<PSolid> = piece.iter().map(|p| PSolid { triangle: &p.triangle, normal: &p.normal, colour: p.colour, material: None }).collect();
    raster.draw(camera, &shape, sun);
}

/// Where pieces' shadow falls on the level plane at a height, the light
/// coming the way the sun comes from.
pub fn shadow(piece: &[Piece], from: [f32; 3], level: f32) -> Vec<[[f32; 3]; 3]> {
    let lay = |p: [f32; 3]| {
        let t = (p[1] - level) / from[1].max(f32::EPSILON);
        [p[0] - from[0] * t, level, p[2] - from[2] * t]
    };
    piece.iter().flat_map(|p| p.triangle.iter().map(|t| t.map(lay))).collect()
}

/// How a view is turned, tipped and brought nearer by hand: a drag turns
/// and tips it, the wheel zooms it, within the theme's limits.
#[derive(Default)]
pub struct Orbit {
    turn: Cell<f32>,
    tip: Cell<f32>,
    zoom: Cell<f32>,
    /// Where a press began, and how the view was then; and whether it
    /// has dragged since, so a press that did not is a press.
    held: Cell<Option<(PPoint, f32, f32)>>,
    dragged: Cell<bool>,
}

impl Orbit {
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

    /// Whether a drag is under way.
    pub fn dragging(&self) -> bool {
        self.held.get().is_some() && self.dragged.get()
    }

    /// A turn of the wheel brings it nearer or farther.
    pub fn wheel(&self, by: f32, theme: &dyn PTheme) -> bool {
        let step = theme.size(ZOOM).unwrap_or_default();
        let (near, far) = (theme.size(NEAREST).unwrap_or(1.0).ln(), theme.size(FARTHEST).unwrap_or(1.0).ln());
        let zoom = (self.zoom.get() + by.signum() * step).clamp(far.min(near), near.max(far));
        self.zoom.replace(zoom) != zoom
    }
}
