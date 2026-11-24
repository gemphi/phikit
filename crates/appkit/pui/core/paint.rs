//! What a view is drawn with: a point, a size, a rectangle, a colour, and
//! the painter a view draws with. `PPaint` is all a view knows of
//! drawing. The painter a screen uses is a sheet (`sheet.rs`), whose marks
//! the screen plays; a test reads the same marks.

use crate::theme::PTheme;

/// A place, in the coordinates of whoever holds it.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PPoint {
    pub x: f32,
    pub y: f32,
}

impl PPoint {
    pub fn new(x: f32, y: f32) -> PPoint {
        PPoint { x, y }
    }

    /// This place moved by `by`.
    pub fn plus(self, by: PPoint) -> PPoint {
        PPoint { x: self.x + by.x, y: self.y + by.y }
    }

    /// This place as seen from `from`.
    pub fn minus(self, from: PPoint) -> PPoint {
        PPoint { x: self.x - from.x, y: self.y - from.y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PSize {
    pub width: f32,
    pub height: f32,
}

/// A rectangle: where a view is, in its parent's coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PRect {
    pub origin: PPoint,
    pub size: PSize,
}

impl PRect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> PRect {
        PRect { origin: PPoint { x, y }, size: PSize { width, height } }
    }

    /// Whether `point` is inside. The left and top edges are inside and the
    /// right and bottom ones are not, so two rectangles side by side never
    /// both hold a point.
    pub fn contains(&self, point: PPoint) -> bool {
        let at = point.minus(self.origin);
        at.x >= 0.0 && at.y >= 0.0 && at.x < self.size.width && at.y < self.size.height
    }

    /// The rectangle `by` in from every edge of this one.
    pub fn inset(self, by: f32) -> PRect {
        PRect::new(self.origin.x + by, self.origin.y + by, self.size.width - by * 2.0, self.size.height - by * 2.0)
    }

    /// Its middle.
    pub fn center(self) -> PPoint {
        PPoint::new(self.origin.x + self.size.width / 2.0, self.origin.y + self.size.height / 2.0)
    }

    /// The same rectangle moved by `by`.
    pub fn moved(&self, by: PPoint) -> PRect {
        PRect { origin: self.origin.plus(by), size: self.size }
    }
}

/// A colour: red, green, blue and how opaque, each from 0 to 1.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl PColor {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> PColor {
        PColor { r, g, b, a }
    }
}

/// How a picture lies in the rectangle it is given.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PFit {
    /// The whole picture, as large as fits and no larger than it is, in
    /// the rectangle's middle. Nothing of it is cut away.
    #[default]
    Whole,
    /// The whole picture as `Whole` has it, but standing at the top of
    /// the rectangle, in the middle across: a model or a look on a
    /// page, which reads from its head down.
    Top,
    /// The rectangle covered, what of the picture does not fit cut away
    /// evenly: a face, a thumb.
    Cover,
    /// One square cell of a sheet of pictures, covering the rectangle:
    /// how many cells the sheet has across, and the cell's number, the
    /// first at its top left.
    Cell { column: u32, index: u32 },
}

impl PFit {
    /// Where a picture of that size lies in a rectangle: what of the
    /// window it may show in, and where the whole picture stands, which
    /// is larger than what shows when some of it is cut away.
    pub fn place(self, rect: PRect, natural: PSize) -> (PRect, PRect) {
        let (wide, high) = (natural.width.max(1.0), natural.height.max(1.0));
        let centred = |scale: f32, middle: PPoint| PRect::new(rect.center().x - middle.x * scale, rect.center().y - middle.y * scale, wide * scale, high * scale);
        let whole = PPoint::new(wide / 2.0, high / 2.0);
        match self {
            PFit::Whole => {
                let lie = centred((rect.size.width / wide).min(rect.size.height / high).min(1.0), whole);
                (lie, lie)
            }
            PFit::Top => {
                let lie = centred((rect.size.width / wide).min(rect.size.height / high).min(1.0), whole);
                let lie = PRect { origin: PPoint::new(lie.origin.x, rect.origin.y), ..lie };
                (lie, lie)
            }
            PFit::Cover => (rect, centred((rect.size.width / wide).max(rect.size.height / high), whole)),
            PFit::Cell { column, index } => {
                let (across, cell) = (column.max(1), wide / column.max(1) as f32);
                let middle = PPoint::new(((index % across) as f32 + 0.5) * cell, ((index / across) as f32 + 0.5) * cell);
                (rect, centred(rect.size.width.max(rect.size.height) / cell, middle))
            }
        }
    }
}

/// What a view draws with, in the window's coordinates.
pub trait PPaint {
    /// The theme in force: where a view gets every colour and size.
    fn theme(&self) -> &dyn PTheme;

    /// The colour a token of the theme stands for; clear for a token the
    /// theme does not have.
    fn color(&self, token: &str) -> PColor {
        self.theme().color(token).unwrap_or_default()
    }

    /// The size a token of the theme stands for; nothing for a token the
    /// theme does not have.
    fn size(&self, token: &str) -> f32 {
        self.theme().size(token).unwrap_or_default()
    }

    /// Fill a rectangle with a colour.
    fn fill(&mut self, rect: PRect, color: PColor);

    /// Fill a rectangle with its corners rounded by `radius`. A painter
    /// that cannot round fills it square.
    fn round(&mut self, rect: PRect, _radius: f32, color: PColor) {
        self.fill(rect, color);
    }

    /// Fill a rounded rectangle with one colour that turns into another
    /// across it. `turn` says where the second lies from the first, in
    /// degrees: 0 is over it, 90 to its right. A painter with one colour
    /// to a fill uses the one that shows more.
    fn blend(&mut self, rect: PRect, radius: f32, from: PColor, to: PColor, _turn: f32) {
        let color = match from.a >= to.a {
            true => from,
            false => to,
        };
        self.round(rect, radius, color);
    }

    /// Lay glass over a rounded rectangle: what was drawn so far shows
    /// through it blurred, as the theme's glass says. A painter that has
    /// no glass, and a theme that has none, draw nothing.
    fn glass(&mut self, _rect: PRect, _radius: f32) {}

    /// Keep what is drawn from now on inside `rect`, until `unclip`. A
    /// painter that cannot clip draws all of it.
    fn clip(&mut self, _rect: PRect) {}

    /// End the last clip.
    fn unclip(&mut self) {}

    /// How wide a line of text is set. A painter with no type to measure
    /// counts the theme's `text.advance` a character.
    fn measure(&self, text: &str) -> f32 {
        text.chars().count() as f32 * self.size("text.advance")
    }

    /// Set a line of text with its top left at `at`.
    fn text(&mut self, at: PPoint, text: &str, color: PColor);

    /// Set a heading: a line in the theme's heading size (`head.size`). A
    /// painter with one size sets it as any text.
    fn head(&mut self, at: PPoint, text: &str, color: PColor) {
        self.text(at, text, color);
    }

    /// How wide a heading is set: a line of text, as much wider as the
    /// heading's size is larger.
    fn measure_head(&self, text: &str) -> f32 {
        let (head, body) = (self.size("head.size"), self.size("text.size"));
        match head > 0.0 && body > 0.0 {
            true => self.measure(text) * head / body,
            false => self.measure(text),
        }
    }

    /// Draw the icon of that name to fit `rect`, tinted with a colour. The
    /// painter finds the picture among the app's assets.
    fn icon(&mut self, name: &str, rect: PRect, color: PColor);

    /// Draw the picture of that name in `rect`, its corners rounded by
    /// `radius`, lying as `fit` says. It is found among the app's
    /// assets by its name, as an icon is, and is in its own colours. A
    /// painter with no pictures, and a name that is none, draw nothing.
    fn picture(&mut self, _name: &str, _rect: PRect, _radius: f32, _fit: PFit) {}

    /// How large the picture of that name is, in its own measure, when
    /// the painter has it. One not read yet, and a painter with no
    /// pictures, say nothing.
    fn natural(&self, _name: &str) -> Option<PSize> {
        None
    }

    /// Say that what was just drawn in that rectangle is pressed for
    /// something, so the pointer is a hand over it. Every block of the
    /// kit that a press acts on says so as it is drawn (`PPress`), so no
    /// part has to list what it can be pressed on. A painter that keeps
    /// nothing forgets it.
    fn press(&mut self, _rect: PRect) {}

    /// Every rectangle said to be pressed so far, each cut to what was in
    /// sight where it was drawn.
    fn pressed(&self) -> Vec<PRect> {
        Vec::new()
    }
}

/// Something of the kit a press acts on: drawn, it tells the painter
/// where it stands (`PPaint::press`), and the pointer is a hand there.
/// The pill, the band's pills, the swatch, the card and a form's every
/// choice are; an app's own drawing says so with `paint.press(rect)`.
pub trait PPress {
    /// Where it stands, said to the painter as pressed.
    fn pressed_at(&self, paint: &mut dyn PPaint) -> PRect;
}

impl PPress for PRect {
    fn pressed_at(&self, paint: &mut dyn PPaint) -> PRect {
        paint.press(*self);
        *self
    }
}

/// What of a rectangle is inside another; nothing when no part is.
pub fn within(rect: PRect, sight: PRect) -> Option<PRect> {
    let (left, top) = (rect.origin.x.max(sight.origin.x), rect.origin.y.max(sight.origin.y));
    let right = (rect.origin.x + rect.size.width).min(sight.origin.x + sight.size.width);
    let bottom = (rect.origin.y + rect.size.height).min(sight.origin.y + sight.size.height);
    (right > left && bottom > top).then(|| PRect::new(left, top, right - left, bottom - top))
}
