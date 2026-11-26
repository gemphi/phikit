//! How the figure lays out and draws a model: one view, or four (the
//! main one and, nearly flat, the top, the front and the right, each from
//! a camera of its own); each a picture drawn again only when what it
//! shows changes; the ball in the main view's corner and the tools at its
//! foot.

use appkit::pui::block::PBand;
use appkit::pui::core::ball::ball;
use appkit::pui::core::orbit::{ROOM, around, framed, shot};
use appkit::pui::core::wire::wire;
use appkit::pui::core::{PCamera, PFit, PPaint, PPiece, PPoint, PRect, PSun};
use appkit::pui::state::PState;
use appkit::theme::PTheme;

use super::{FILM, FOUR, Figure, WIRE};
use crate::data;
use crate::library::{Attribute, Item};
use crate::part::{DIM, GAP};

/// The theme's tokens: the ball's side, how wide a side view sees, and
/// the ink of wire.
const BALL: &str = "scene.ball.size";
const SIDE: &str = "scene.side.fov";
const INK: &str = "scene.wire";

/// The side views: their name, the way each looks along, and its words.
const VIEW: [(&str, [f32; 3], &str); 3] = [("top", [0.0, -1.0, -0.001], "view3d.top"), ("front", [0.0, 0.0, -1.0], "view3d.front"), ("right", [-1.0, 0.0, 0.0], "view3d.right")];

impl Figure {
    /// A model, in one view or four, with the ball and the tools.
    pub(super) fn model(&self, paint: &mut dyn PPaint, item: &Item, room: PRect) {
        let (four, wired) = (!self.desk.shelf.get(FOUR).is_empty(), !self.desk.shelf.get(WIRE).is_empty());
        let gap = paint.size(GAP) / 2.0;
        let (half_w, half_h) = ((room.size.width - gap) / 2.0, (room.size.height - gap) / 2.0);
        let pane = |column: f32, row: f32| PRect::new(room.origin.x + column * (half_w + gap), room.origin.y + row * (half_h + gap), half_w, half_h);
        let main = match four {
            true => pane(0.0, 0.0),
            false => room,
        };
        let library = &self.desk.library;
        let dress = Attribute::new(self.desk.choice.clone());
        let mut piece = library.dressed(item, &dress);
        // The part picked, lighter by the table's share.
        let lift: f32 = library.set.get("pick").parse().unwrap_or(0.0);
        let picked = self.desk.shelf.get("ui/part").parse::<usize>().ok();
        picked.and_then(|part| piece.get_mut(part)).into_iter().for_each(|p| p.colour = p.colour.map(|c| c + (1.0 - c) * lift));
        self.main.set(main);
        let sun = library.sun(item, &dress);
        let fov = library.set.get("camera.fov").parse().unwrap_or(30.0);
        let theme = self.desk.shade.as_ref();
        let roomy = theme.size(ROOM).unwrap_or(1.0);
        let parts = library.model(&item.path).part.len();
        let mark = format!("{}|{}|{:?}", item.key, dress.mark(&item.key, parts), picked);
        // The main view: as the hand turned it, from the way a ball chose.
        let aspect = main.size.width / main.size.height.max(1.0);
        let first = match self.way.get() {
            Some(way) => framed(&piece, way, fov, roomy, aspect),
            None => around(&piece, 0.6, fov, roomy, aspect),
        };
        let camera = self.orbit.camera(&first);
        self.seen.set(Some(camera));
        let fit = match four {
            true => PFit::Whole,
            false => PFit::Top,
        };
        let main_key = format!("{mark}|{:?}|{:?}|{wired}", self.orbit.now(), self.way.get());
        self.shoot(FILM, &main_key, &piece, &camera, &sun, main, wired);
        paint.picture(FILM, main, 0.0, fit);
        // The sides, nearly flat, as wire.
        if four {
            let side_fov = theme.size(SIDE).unwrap_or(2.0);
            for (index, (name, way, word)) in VIEW.iter().enumerate() {
                // The top at the right of the main view, the front and
                // the right under them.
                let (column, row) = match index {
                    0 => (1.0, 0.0),
                    1 => (0.0, 1.0),
                    _ => (1.0, 1.0),
                };
                let rect = pane(column, row);
                let camera = framed(&piece, *way, side_fov, roomy, rect.size.width / rect.size.height.max(1.0));
                let film = format!("{FILM}/{name}");
                self.shoot(&film, &mark, &piece, &camera, &sun, rect, true);
                paint.picture(&film, rect, 0.0, PFit::Whole);
                paint.text(PPoint::new(rect.origin.x + gap, rect.origin.y + gap), self.desk.text.get(word), paint.color(DIM));
            }
            paint.text(PPoint::new(main.origin.x + gap, main.origin.y + gap), self.desk.text.get("view3d.main"), paint.color(DIM));
        }
        // The ball in the main view's corner, the tools at its foot.
        let side = paint.size(BALL);
        let corner = PRect::new(main.origin.x + main.size.width - side - gap, main.origin.y + main.size.height - side - gap, side, side);
        *self.ball.borrow_mut() = ball(paint, corner, &camera);
        let row = data::row(data::VIEWPORT);
        let pill: Vec<(&str, &str, bool)> = row.iter().filter_map(|cell| {
            let on = match *cell.first()? {
                "four" => four,
                "wire" => wired,
                _ => false,
            };
            Some((*cell.get(1)?, "", on))
        }).collect();
        let at = PPoint::new(main.origin.x + gap, main.origin.y + main.size.height - pill.high(paint) - gap);
        let rect = pill.draw(paint, at);
        *self.tool.borrow_mut() = rect.into_iter().zip(row.iter().filter_map(|cell| cell.first().map(|name| name.to_string()))).collect();
    }

    /// Draw a picture of the model into the film under a name, shaded or
    /// as wire, where what it shows or its size changed.
    #[allow(clippy::too_many_arguments)]
    fn shoot(&self, film: &str, key: &str, piece: &[PPiece], camera: &PCamera, sun: &PSun, rect: PRect, wired: bool) {
        let size = (rect.size.width.max(0.0) as u32, rect.size.height.max(0.0) as u32);
        let now = (format!("{key}|{wired}"), size);
        if self.drawn.borrow().get(film) == Some(&now) || size.0 == 0 || size.1 == 0 {
            return;
        }
        let theme = self.desk.shade.as_ref();
        let (wide, high, pixel) = match wired {
            true => {
                let c = theme.color(INK).unwrap_or_default();
                let ink = [c.b, c.g, c.r, c.a].map(|v| (v * 255.0) as u8);
                wire(piece, camera, size, ink, theme)
            }
            false => shot(piece, camera, sun, size, theme),
        };
        self.desk.library.film.show(film, wide, high, pixel);
        self.drawn.borrow_mut().insert(film.to_string(), now);
    }
}
