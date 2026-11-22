//! Controls: the small things a part is worked with, each an icon that
//! says what it is called while the pointer is over it. Which controls
//! a part has, the icon of each and the row of `data/text.tsv` that
//! names it are a table: `data/tool.tsv` for the viewer,
//! `data/act.tsv` for a room. A row whose name ends in `.on` is the
//! same control once it is on.

use appkit::pui::core::{PLight, PPaint, PPoint, PRect, light};
use appkit::service::PTable;

use super::{GAP, ICON, INK, LINE, RADIUS};
use crate::data;

/// How the name of a control that is on ends.
pub const ON: &str = ".on";

/// A table of controls.
#[derive(Debug, Clone, Default)]
pub struct Tool {
    /// A name, its icon, the row of the words that names it.
    row: Vec<(String, String, String)>,
}

impl Tool {
    /// The controls of a table.
    pub fn read(table: &str) -> Tool {
        let row = data::row(table).into_iter().filter_map(|cell| Some((cell.first()?.to_string(), cell.get(1)?.to_string(), cell.get(2)?.to_string())));
        Tool { row: row.collect() }
    }

    /// The names of its controls, in the order of their rows.
    pub fn name(&self) -> impl Iterator<Item = &str> {
        self.row.iter().map(|(name, ..)| name.as_str())
    }

    fn row<'a>(&'a self, name: &str, text: &'a PTable) -> Option<(&'a str, &'a str)> {
        self.row.iter().find(|(known, ..)| known == name).map(|(_, icon, word)| (icon.as_str(), text.get(word)))
    }

    /// The icon and the word of a control: of the row for it on, when
    /// it is on and has one. Nothing for one the table leaves out.
    pub fn of<'a>(&'a self, name: &str, on: bool, text: &'a PTable) -> Option<(&'a str, &'a str)> {
        Some(format!("{name}{ON}")).filter(|_| on).and_then(|name| self.row(&name, text)).or_else(|| self.row(name, text))
    }
}

/// Draw a control: its icon in the middle of its place, on a ground of
/// that strength when it has one.
pub fn button(paint: &mut dyn PPaint, rect: PRect, icon: &str, light: Option<PLight>) {
    let (size, ink, radius) = (paint.size(ICON), paint.color(INK), paint.size(RADIUS));
    light.into_iter().for_each(|light| light::label(paint, rect, radius, light, false, ""));
    paint.icon(icon, PRect::new(rect.center().x - size / 2.0, rect.center().y - size / 2.0, size, size), ink);
}

/// Say what a control is called, by it: under one in the upper half of
/// `frame`, over one in the lower, and never outside the frame.
pub fn say(paint: &mut dyn PPaint, at: PRect, word: &str, frame: PRect) {
    if word.is_empty() {
        return;
    }
    let (gap, line, ink, radius) = (paint.size(GAP), paint.size(LINE), paint.color(INK), paint.size(RADIUS));
    let (wide, high) = (paint.measure(word) + gap, line + gap / 2.0);
    let left = (at.center().x - wide / 2.0).clamp(frame.origin.x + gap / 2.0, (frame.origin.x + frame.size.width - gap / 2.0 - wide).max(frame.origin.x));
    let top = match at.center().y < frame.center().y {
        true => at.origin.y + at.size.height + gap / 4.0,
        false => at.origin.y - gap / 4.0 - high,
    };
    light::surface(paint, PRect::new(left, top, wide, high), radius, PLight::Strong, "");
    paint.text(PPoint::new(left + gap / 2.0, top + gap / 4.0), word, ink);
}

/// A number as a person reads it: its thousands apart, as the web
/// writes them.
pub fn grouped(number: u64) -> String {
    let digit = number.to_string();
    let part: Vec<&str> = digit.as_bytes().rchunks(3).rev().filter_map(|part| std::str::from_utf8(part).ok()).collect();
    part.join(",")
}
