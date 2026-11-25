//! What each kind of a form's block draws from its choices, and the
//! choice itself. Every kind is one of the kit's blocks; a variation is
//! a word the form's table gives it (`strong`, `soft`).
//!
//! * `band`: one of its choices, as pills across the width.
//! * `swatch`: one of its choices, as colours.
//! * `text`: each choice's words, then its line in the dim ink, broken
//!   to the width.
//! * `rows`: each choice a row lit from the left (as its variation says,
//!   the chosen one at full strength): what leads it, its words and its
//!   line, a note at its end, and under the note a bar as long as its
//!   level.
//! * `cards`: each choice a card with its face, its words and its line.
//! * `player`: each choice a card as a speaker's screen stands a song, a
//!   bar as long as its level and its controls along its foot (`player.rs`).
//! * `grid`: each choice a tile, its picture or face large over its words;
//!   `large`, the same in tiles larger. The one under the pointer is lit.
//! * `wheel`: any colour, on the kit's colour wheel; its choices are the
//!   wheel's own (`block::wheel`), and a press keeps one as `#rrggbb`.
//! * `tree`, `facts`, `picture`: a navigator, an item's attributes, and an
//!   item seen large (`tree.rs`), `banner` a picture as wide as the
//!   theme's `form.banner` times its height; `field`, a line to type
//!   (`field.rs`).

use crate::pui::block::{CARD, PBand, PCard, PDeck, PFace, PLead, PRow, PSwatch};
use crate::pui::core::light::label;
use crate::pui::core::{PColor, PLight, PPaint, PPoint, PRect};
use crate::pui::layout::fold;

/// The theme's tokens: how high a row is, how wide a level's bar at its
/// longest and how high, and the inks.
pub const ROW: &str = "form.row";
pub const METER: &str = "form.meter";
pub const BAR: &str = "form.bar";
pub const INK: &str = "text.ink";
pub const DIM: &str = "text.dim";
pub const LINE: &str = "text.line";
pub const GAP: &str = "gap";
pub const RADIUS: &str = "row.radius";

/// One choice of a list: what is kept when it is pressed, its words, a
/// line about it, a note at its end, its icon, its face (a cell of a
/// sheet), its picture, its colour (a swatch's), and its level (0 to 1)
/// where it has one.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PChoice {
    pub value: String,
    pub label: String,
    pub note: String,
    pub end: String,
    pub icon: String,
    pub face: Option<(String, u32, u32)>,
    pub picture: String,
    pub colour: Option<PColor>,
    pub level: Option<f32>,
    /// How deep it stands in a tree: 0 at the top; and whether its line
    /// is open, its own under it, or can be opened.
    pub depth: u32,
    pub fold: Option<bool>,
    /// What can be done with it where it stands (`PDo`).
    pub deed: Vec<PDo>,
}

/// What can be done with a choice where it stands, as Phiegg's pages
/// float round icons over a picture (like, the bag, full screen): what a
/// press keeps it at (the last word of its path, the block's state's
/// first), its icon, and whether it is on, which lights it.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PDo {
    pub word: String,
    pub icon: String,
    pub on: bool,
}

/// The deeds a kind draws, in the order it draws them, each with its
/// choice: a picture's are its first choice's, floating over it.
pub fn deeds<'a>(kind: &str, choice: &'a [PChoice]) -> Vec<(&'a PChoice, &'a PDo)> {
    match kind {
        "picture" => choice.iter().take(1).flat_map(|c| c.deed.iter().map(move |d| (c, d))).collect(),
        // A player card's controls stand along its foot, every card's.
        "player" => choice.iter().flat_map(|c| c.deed.iter().map(move |d| (c, d))).collect(),
        _ => Vec::new(),
    }
}

impl PChoice {
    /// What leads it: its face, its picture, or its icon.
    pub fn lead(&self) -> PLead<'_> {
        match (&self.face, self.picture.is_empty(), self.icon.is_empty()) {
            (Some((sheet, column, index)), ..) => PLead::Face(sheet, *column, *index),
            (None, false, _) => PLead::Picture(&self.picture),
            (None, true, false) => PLead::Icon(&self.icon),
            (None, true, true) => PLead::None,
        }
    }
}

impl PFace for PChoice {
    fn cell(&self) -> Option<(&str, u32, u32)> {
        self.face.as_ref().map(|(sheet, column, index)| (sheet.as_str(), *column, *index))
    }

    fn name(&self) -> &str {
        &self.label
    }
}

impl PCard for PChoice {
    fn note(&self) -> &str {
        &self.note
    }
}

/// How strong a variation's light is.
fn strength(variant: &str) -> PLight {
    let says = |word: &str| variant.split_whitespace().any(|one| one == word);
    match (says("strong"), says("soft")) {
        (true, _) => PLight::Strong,
        (false, true) => PLight::Soft,
        (false, false) => PLight::Rest,
    }
}

/// What a block of a kind draws from a place across a width, lit from
/// the right where told (against the ground it stands on): where it
/// ends, and where each of its choices can be pressed.
#[allow(clippy::too_many_arguments)]
pub fn drawn(kind: &str, variant: &str, choice: &[PChoice], now: &str, paint: &mut dyn PPaint, at: PPoint, wide: f32, right: bool, over: Option<PPoint>) -> (f32, Vec<PRect>) {
    match kind {
        "swatch" => {
            let colour: Vec<(PColor, bool)> = choice.iter().map(|c| (c.colour.unwrap_or_default(), c.value == now)).collect();
            let (rect, end) = colour.as_slice().draw_in(paint, at, wide);
            (end, rect)
        }
        "text" => (text(choice, paint, at, wide), Vec::new()),
        "rows" => rows(variant, choice, now, paint, at, wide, right),
        "cards" => {
            let (rect, end) = choice.draw(paint, at, wide);
            (end, rect)
        }
        "grid" => grid(choice, paint, at, wide, 1.0, over),
        "large" => grid(choice, paint, at, wide, LARGE, over),
        "small" => grid(choice, paint, at, wide, SMALL, over),
        "slides" => slides(choice, now.parse().unwrap_or_default(), paint, at, wide, over),
        "player" => super::player::player(choice, paint, at, wide, over),
        "tree" => super::tree::tree(choice, now, paint, at, wide, over),
        "facts" => super::tree::facts(choice, paint, at, wide),
        "picture" => super::tree::picture(choice, paint, at, wide, variant.split_whitespace().any(|word| word == "banner")),
        // Any colour: the wheel, as wide as the block or a card, whichever
        // is less.
        "wheel" => {
            let (side, ink) = (wide.min(paint.size(CARD) * 1.5), paint.color(INK));
            let rect: Vec<PRect> = crate::pui::block::wheel::wheel(paint, PRect::new(at.x, at.y, side, side), now, ink).into_iter().map(|(rect, _)| rect).collect();
            (at.y + side, rect)
        }
        _ => {
            let pill: Vec<(&str, &str, bool)> = choice.iter().map(|c| (c.icon.as_str(), c.label.as_str(), c.value == now)).collect();
            let rect = pill.draw_full(paint, at, wide, right);
            (at.y + pill.high(paint), rect)
        }
    }
}

/// Words broken to the width: each choice's own, then its line.
fn text(choice: &[PChoice], paint: &mut dyn PPaint, at: PPoint, wide: f32) -> f32 {
    let (line, ink, dim) = (paint.size(LINE), paint.color(INK), paint.color(DIM));
    let said: Vec<(String, PColor)> = choice.iter().flat_map(|c| [(c.label.clone(), ink), (c.note.clone(), dim)]).filter(|(s, _)| !s.is_empty()).collect();
    let mut y = at.y;
    for (said, colour) in said {
        for one in fold(&said, wide, &|t| paint.measure(t)) {
            paint.text(PPoint::new(at.x, y), &one, colour);
            y += line;
        }
    }
    y
}

/// Rows, each lit from the left, a note and a level at the end.
fn rows(variant: &str, choice: &[PChoice], now: &str, paint: &mut dyn PPaint, at: PPoint, wide: f32, right: bool) -> (f32, Vec<PRect>) {
    let (gap, line, radius, dim) = (paint.size(GAP), paint.size(LINE), paint.size(RADIUS), paint.color(DIM));
    let high = paint.theme().size(ROW).unwrap_or(line * 2.0 + gap / 2.0);
    let mut rect = Vec::new();
    for (index, c) in choice.iter().enumerate() {
        let r = PRect::new(at.x, at.y + index as f32 * (high + gap / 4.0), wide, high);
        let light = match c.value == now && !now.is_empty() {
            true => PLight::Strong,
            false => strength(variant),
        };
        label(paint, r, radius, light, right, "");
        let inner = r.inset(gap / 4.0);
        let right = inner.origin.x + inner.size.width - gap / 4.0;
        let note = paint.measure(&c.end);
        (!c.end.is_empty()).then(|| paint.text(PPoint::new(right - note, inner.center().y - line), &c.end, dim));
        let meter = c.level.map_or(0.0, |level| {
            let (most, bar) = (paint.size(METER), paint.size(BAR));
            let long = level.clamp(0.0, 1.0) * most;
            (long >= 1.0).then(|| label(paint, PRect::new(right - most, inner.center().y + (line - bar) / 2.0, long, bar), bar / 2.0, PLight::Strong, false, ""));
            most
        });
        (c.lead(), c.label.as_str(), c.note.as_str()).row(paint, PRect::new(inner.origin.x + gap / 4.0, inner.origin.y, inner.size.width - gap / 2.0, inner.size.height), note.max(meter) + gap / 2.0);
        rect.push(r);
    }
    let end = at.y + choice.len() as f32 * (high + gap / 4.0) - gap / 4.0;
    (end.max(at.y), rect)
}

/// How many times a card's width a large tile is, and a small one.
pub(super) const LARGE: f32 = 1.75;
const SMALL: f32 = 0.6;

/// Slides: one row of large tiles, from the one it begins at, as many
/// as the width holds; the arrows at its heading step them. Every
/// choice has a place, those out of the row none to press.
fn slides(choice: &[PChoice], from: usize, paint: &mut dyn PPaint, at: PPoint, wide: f32, over: Option<PPoint>) -> (f32, Vec<PRect>) {
    let gap = paint.size(GAP);
    let least = (paint.size(CARD) * LARGE).min(wide).max(1.0);
    let across = (((wide + gap / 2.0) / (least + gap / 2.0)).floor() as usize).max(1);
    let from = from.min(choice.len().saturating_sub(1));
    let upto = (from + across).min(choice.len());
    let (end, row) = grid(&choice[from..upto], paint, at, wide, LARGE, over);
    let place = (0..choice.len()).map(|index| match index.checked_sub(from) {
        Some(at) if at < row.len() => row[at],
        _ => PRect::default(),
    }).collect();
    (end, place)
}

/// Tiles in as many columns as a card's width, so many times, allows;
/// the one under the pointer lit the more.
fn grid(choice: &[PChoice], paint: &mut dyn PPaint, at: PPoint, wide: f32, times: f32, over: Option<PPoint>) -> (f32, Vec<PRect>) {
    let (gap, line) = (paint.size(GAP), paint.size(LINE));
    let least = (paint.size(CARD) * times).min(wide).max(1.0);
    let across = ((wide + gap / 2.0) / (least + gap / 2.0)).floor().max(1.0);
    let side = (wide - (across - 1.0) * gap / 2.0) / across;
    let high = side + line * 2.0 + gap;
    let rect: Vec<PRect> = (0..choice.len()).map(|i| PRect::new(at.x + (i % across as usize) as f32 * (side + gap / 2.0), at.y + (i / across as usize) as f32 * (high + gap / 2.0), side, high)).collect();
    for (c, r) in choice.iter().zip(&rect) {
        let light = match over.is_some_and(|over| r.contains(over)) {
            true => PLight::Strong,
            false => PLight::Soft,
        };
        (c.lead(), c.label.as_str(), c.note.as_str()).tile(paint, *r, light);
    }
    let rows = choice.len().div_ceil(across as usize) as f32;
    (at.y + rows * (high + gap / 2.0) - gap / 2.0, rect)
}
