//! The greenhouse in 3D: Apple's scene drawn once for a view from its
//! own meshes, from the sample's camera turned and brought nearer by
//! hand, and the robot drawn over it each frame it moves, with its shadow
//! on the floor. While the view is dragged the greenhouse is drawn
//! coarser, so the drag keeps up with the hand; let go, it is drawn whole.
//! Each picture is made larger than shown and made smaller, so no edge
//! is jagged.

use std::cell::{Cell, RefCell};

use appkit::pui::core::{PCamera, PPoint, PRaster, PSize};
use appkit::theme::PTheme;
use phiace::bot::botanist::figure::{Piece, coarse, figure, moved, scenery};
use phiace::bot::botanist::garden::GREENHOUSE;
use phiace::bot::botanist::look::Look;
use phiecs::comp::Matrix;
use phiecs::comp::tree::matrix;

use super::Desk;
use super::view::{Orbit, bounds, draw, framed, ground, shadow, smooth, sun, way};

/// The picture the greenhouse is drawn into.
pub const FILM: &str = "bot/garden";
/// The theme's tokens: how much room the camera leaves round the
/// greenhouse, as a share of its reach; how many cells across the robot
/// its corners are kept in, seen as small as it is there; and across the
/// greenhouse while it is dragged.
pub const REACH: &str = "garden.reach";
pub const DETAIL: &str = "garden.detail";
pub const MOVING: &str = "garden.moving";
/// How many cells across the robot its shadow is kept in: a shadow is
/// soft, and needs few.
pub const SHADE: &str = "garden.shade";
/// How many cells across the robot it is seen at before all of its
/// triangles are drawn.
pub const WHOLE: &str = "garden.whole";

/// The greenhouse drawn without the robot: for what view, at what size,
/// whether whole, the camera, the picture made larger with its depths,
/// and as shown.
struct Still {
    key: ([f32; 3], (u32, u32), bool),
    camera: PCamera,
    raster: PRaster,
    shown: Vec<u8>,
}

/// The patch of a picture of a size that what lies in a box covers, a
/// point round it to spare: from where, how wide, how high.
fn patch(camera: &PCamera, low: [f32; 3], high: [f32; 3], size: (u32, u32)) -> (u32, u32, u32, u32) {
    let (w, h) = (size.0 as f32, size.1 as f32);
    let corner = (0..8).map(|n| [0, 1, 2].map(|i| [low[i], high[i]][(n >> i) & 1]));
    let (mut most, mut least) = ([f32::MIN; 2], [f32::MAX; 2]);
    for p in corner.map(|v| camera.project(v, w, h)).filter(|p| p[2] > camera.near) {
        (0..2).for_each(|i| (least[i], most[i]) = (least[i].min(p[i]), most[i].max(p[i])));
    }
    let x = (least[0] - 2.0).clamp(0.0, w) as u32;
    let y = (least[1] - 2.0).clamp(0.0, h) as u32;
    (x, y, ((most[0] + 2.0).clamp(0.0, w) as u32).saturating_sub(x), ((most[1] + 2.0).clamp(0.0, h) as u32).saturating_sub(y))
}

/// The robot's pieces, as its look was, where it stood then: all of
/// them, the box round them, and as many as its size on the picture
/// needs (`cells` across it), and its shadow's.
struct Figure {
    look: Look,
    stood: Matrix,
    full: Vec<Piece>,
    low: [f32; 3],
    high: [f32; 3],
    cells: u32,
    piece: Vec<Piece>,
    shade: Vec<Piece>,
}

impl Figure {
    /// Keep as many of its triangles as `cells` across it shows: all of
    /// them from `whole` cells on. Its shadow is soft, and needs a
    /// quarter as many, but no fewer than `least`.
    fn detail(&mut self, cells: u32, whole: u32, least: f32) {
        self.piece = match cells >= whole {
            true => self.full.clone(),
            false => coarse(&self.full, cells as f32),
        };
        self.shade = coarse(&self.full, (cells as f32 / 4.0).max(least));
        self.cells = cells;
    }
}

/// What a picture was drawn for: where the robot stood, the view, the
/// size.
type Shown = (Matrix, [f32; 3], (u32, u32));

#[derive(Default)]
pub struct Scene {
    pub orbit: Orbit,
    /// The greenhouse's pieces, whole and coarse, and the camera that
    /// frames it at a size.
    piece: RefCell<Option<(Vec<Piece>, Vec<Piece>)>>,
    still: RefCell<Option<Still>>,
    figure: RefCell<Option<Figure>>,
    /// What the picture was last drawn for: where the robot stood, the
    /// view and the size.
    shown: Cell<Option<Shown>>,
}

impl Scene {
    /// Draw what changed: the greenhouse for a new view or size, the
    /// robot's pieces for a new look, and the picture where the robot
    /// moved.
    pub fn shoot(&self, desk: &Desk, size: (u32, u32)) {
        let bot = desk.bot.borrow();
        let (Some(root), true) = (bot.world.lexicon.get(GREENHOUSE), size.0 > 0 && size.1 > 0) else { return };
        let (data, look, theme) = (bot.data().clone(), bot.look(), desk.shade.as_ref());
        let (by, light) = (smooth(theme), sun(&data));
        let mut library = desk.library.borrow_mut();
        let mut piece = self.piece.borrow_mut();
        let (whole, rough) = piece.get_or_insert_with(|| {
            let whole = scenery(&bot.world, root, &[bot.robot], &look, &data, &mut library);
            let rough = coarse(&whole, theme.size(MOVING).unwrap_or(f32::MAX));
            (whole, rough)
        });
        let key = (self.orbit.now(), size, !self.orbit.dragging());
        let mut still = self.still.borrow_mut();
        if still.as_ref().map(|s| s.key) != Some(key) {
            let first = framed(whole, way(&data), data.game.num("camera.fov"), theme.size(REACH).unwrap_or(1.0), size.0 as f32 / size.1 as f32);
            let camera = self.orbit.camera(&first);
            let mut raster = PRaster::new(size.0 * by, size.1 * by, ground(theme));
            let drawn = match key.2 {
                true => &*whole,
                false => &*rough,
            };
            draw(&mut raster, &camera, drawn, &light);
            let shown = raster.smooth(by).2;
            *still = Some(Still { key, camera, raster, shown });
            self.shown.set(None);
        }
        let now = matrix(&bot.world, bot.robot);
        let mut kept = self.figure.borrow_mut();
        if kept.as_ref().map(|f| &f.look) != Some(&look) {
            let full = figure(&bot.world, &look, &data, &mut library);
            let (low, high) = bounds(&full);
            *kept = Some(Figure { full, low, high, cells: 0, piece: Vec::new(), shade: Vec::new(), look, stood: now });
            self.shown.set(None);
        }
        let (Some(still), Some(kept)) = (still.as_ref(), kept.as_mut()) else { return };
        if self.shown.get() == Some((now, key.0, size)) {
            return;
        }
        let by_now = now.then(&kept.stood.inverse().unwrap_or_default());
        // As many triangles as the robot's size on the picture shows: a
        // power of two cells for each point it stands high, so a walk
        // does not make it again each step.
        let corner = (0..8).map(|n| [0, 1, 2].map(|i| [kept.low[i], kept.high[i]][(n >> i) & 1])).map(|v| by_now.apply(v));
        let (mut low, mut high) = ([f32::MAX; 3], [f32::MIN; 3]);
        corner.for_each(|v| (0..3).for_each(|i| (low[i], high[i]) = (low[i].min(v[i]), high[i].max(v[i]))));
        let tall = patch(&still.camera, low, high, (size.0 * by, size.1 * by)).3.max(1);
        let (least, whole) = (theme.size(DETAIL).unwrap_or(1.0), theme.size(WHOLE).unwrap_or(f32::MAX) as u32);
        let cells = (tall as f32).max(least).min(whole as f32) as u32;
        let cells = cells.next_power_of_two().min(whole);
        if cells != kept.cells {
            kept.detail(cells, whole, theme.size(SHADE).unwrap_or(1.0));
        }
        // Only the patch the robot and its shadow cover is drawn again,
        // and laid into the greenhouse as shown.
        let piece = moved(&kept.piece, &by_now);
        let level = matrix(&bot.world, root).origin()[1];
        let dark = shadow(&moved(&kept.shade, &by_now), light.way, level);
        let (mut low, mut high) = bounds(&piece);
        for v in dark.iter().flatten() {
            (0..3).for_each(|i| (low[i], high[i]) = (low[i].min(v[i]), high[i].max(v[i])));
        }
        let (x, y, w, h) = patch(&still.camera, low, high, size);
        let mut part = still.raster.cut(x * by, y * by, w * by, h * by);
        part.shadow(&still.camera, &dark, data.game.num("shadow.share"));
        draw(&mut part, &still.camera, &piece, &light);
        let (pw, ph, pixel) = part.smooth(by);
        let mut frame = still.shown.clone();
        let wide = size.0 as usize;
        for row in 0..ph as usize {
            let (to, from) = (((y as usize + row) * wide + x as usize) * 4, row * pw as usize * 4);
            frame[to..to + pw as usize * 4].copy_from_slice(&pixel[from..from + pw as usize * 4]);
        }
        desk.film.show(FILM, size.0, size.1, frame);
        self.shown.set(Some((now, key.0, size)));
    }

    /// Where a point of the world falls on a part of a size, once the
    /// greenhouse is drawn.
    pub fn seen(&self, p: [f32; 3], size: PSize) -> Option<PPoint> {
        let still = self.still.borrow();
        let at = still.as_ref()?.camera.project(p, size.width, size.height);
        (at[2] > 0.0).then(|| PPoint::new(at[0], at[1]))
    }

    /// Where a press on a part of a size meets the greenhouse's floor.
    pub fn floor(&self, desk: &Desk, at: PPoint, size: PSize) -> Option<[f32; 3]> {
        let bot = desk.bot.borrow();
        let level = bot.world.lexicon.get(GREENHOUSE).map_or(0.0, |root| matrix(&bot.world, root).origin()[1]);
        self.still.borrow().as_ref().and_then(|s| s.camera.floor(at.x, at.y, size.width, size.height, level))
    }
}
