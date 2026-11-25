//! How the app appears, as two simple choices rather than a pill for
//! every look (the user, 25 November 2026: "dark, light and surface:
//! solid, gradient, glass"). The app's user table names its axes
//! (`axis`, e.g. `mode surface`), each axis's word (`axis.mode`), each
//! value's word (`value.glass`), and where each look stands on them
//! (`place.<look>`, e.g. `dark glass`). A look with no place is not
//! offered. A press on a value turns to the look that has it and keeps
//! the other axes as they are; where none does, the first that has it.
//!
//! Every group of the profile is an accordion: its heading on a ground of
//! its own with a chevron, a press folding or opening it (`fold`).

use crate::pui::core::light::{label, surface};
use crate::pui::core::{PLight, PPaint, PPoint, PRect};
use crate::service::PTable;
use crate::theme::PShade;

use super::super::{GAP, INK, ROW};
use super::{FILL, LIT, ROUND};

/// The user table's rows: the axes, and what comes before an axis's,
/// a value's and a look's place.
pub const AXIS: &str = "axis";
const WORD: &str = "axis.";
const VALUE: &str = "value.";
const PLACE: &str = "place.";
/// What a heading's press is told, before the group's name.
pub const FOLD: &str = "fold ";
/// The user table's row naming the axis Auto stands on and its two
/// values (`auto mode light dark`), its word (`value.auto`), and what a
/// press on Auto keeps.
pub const AUTO: &str = "auto";
pub const AUTO_PRESS: &str = "<auto>";

/// The axis Auto stands on, and its light and dark values.
pub fn auto(user: &PTable) -> Option<(String, String, String)> {
    let row: Vec<String> = user.get(AUTO).split_whitespace().map(str::to_string).collect();
    match row.as_slice() {
        [axis, light, dark] => Some((axis.clone(), light.clone(), dark.clone())),
        _ => None,
    }
}

/// The looks Auto follows the system with: the look with the light value
/// and the one with the dark, each keeping every other axis as the look
/// shown has it.
pub fn pair(user: &PTable, shade: &PShade) -> Option<(String, String)> {
    let (axis, light, dark) = auto(user)?;
    let index = axes(user).iter().position(|one| *one == axis)?;
    let now = place(user, shade.name());
    let with = |value: &str| {
        let looks = shade.list();
        let same = |look: &&str| {
            let there = place(user, look);
            there.get(index).map(String::as_str) == Some(value) && there.iter().enumerate().all(|(at, one)| at == index || now.get(at) == Some(one))
        };
        looks.iter().copied().find(same).or_else(|| looks.iter().copied().find(|look| place(user, look).get(index).map(String::as_str) == Some(value))).map(str::to_string)
    };
    Some((with(&light)?, with(&dark)?))
}

/// The axes the app names, in order.
pub fn axes(user: &PTable) -> Vec<String> {
    user.get(AXIS).split_whitespace().map(str::to_string).collect()
}

/// Where a look stands on the axes.
fn place(user: &PTable, look: &str) -> Vec<String> {
    user.get(&format!("{PLACE}{look}")).split_whitespace().map(str::to_string).collect()
}

/// A word of the user's table, or the name itself where it has none.
fn word<'a>(user: &'a PTable, key: &str, name: &'a str) -> &'a str {
    Some(user.get(key)).filter(|word| !word.is_empty()).unwrap_or(name)
}

/// A group's heading as an accordion's: the word, a chevron at its
/// right that says whether it is open, on a ground of its own. Where it
/// stands; how far down what it holds begins.
pub fn heading(paint: &mut dyn PPaint, at: PPoint, wide: f32, said: &str, open: bool) -> (PRect, f32) {
    let (gap, line, round, ink) = (paint.size(GAP), paint.size(ROW), paint.size(ROUND), paint.color(INK));
    let rect = PRect::new(at.x, at.y, wide, line + gap);
    label(paint, rect, round, PLight::Soft, false, FILL);
    paint.text(PPoint::new(rect.origin.x + gap, rect.center().y - line / 2.0), said, ink);
    let icon = line * 0.8;
    let chevron = match open {
        true => "chevron-down",
        false => "chevron-right",
    };
    paint.icon(chevron, PRect::new(rect.origin.x + wide - gap - icon, rect.center().y - icon / 2.0, icon, icon), ink);
    (rect, rect.origin.y + rect.size.height + gap / 2.0)
}

/// What a group holds, on a ground of its own under its heading.
pub fn body(paint: &mut dyn PPaint, rect: PRect) {
    let round = paint.size(ROUND);
    surface(paint, rect, round, PLight::Rest, FILL);
}

/// One axis's values as pills on a line that folds, the shown look's lit;
/// each with the look a press turns to. How far down they end.
pub fn values(user: &PTable, shade: &PShade, axis: usize, paint: &mut dyn PPaint, at: PPoint, wide: f32, spot: &mut Vec<(PRect, String)>) -> f32 {
    let (gap, line, round, ink) = (paint.size(GAP), paint.size(ROW), paint.size(ROUND), paint.color(INK));
    let looks = shade.list();
    let now = place(user, shade.name());
    let mut seen: Vec<String> = Vec::new();
    looks.iter().filter_map(|look| place(user, look).get(axis).cloned()).for_each(|value| if !seen.contains(&value) { seen.push(value) });
    // In the order the user's table writes their words.
    let order = |value: &String| user.list().iter().position(|(key, _)| *key == format!("{VALUE}{value}")).unwrap_or(usize::MAX);
    seen.sort_by_key(order);
    let (mut x, mut y, mut end) = (at.x, at.y, at.y);
    for value in &seen {
        // The look with this value that keeps every other axis as it is.
        let wanted = |look: &&str| {
            let there = place(user, look);
            there.get(axis) == Some(value) && there.iter().enumerate().all(|(index, one)| index == axis || now.get(index) == Some(one))
        };
        let any = |look: &&str| place(user, look).get(axis) == Some(value);
        let Some(target) = looks.iter().copied().find(wanted).or_else(|| looks.iter().copied().find(any)) else { continue };
        let said = word(user, &format!("{VALUE}{value}"), value);
        let long = paint.measure(said) + gap * 2.0;
        if x + long > at.x + wide && x > at.x {
            (x, y) = (at.x, y + line + gap * 1.5);
        }
        let rect = PRect::new(x, y, long, line + gap);
        let (light, token) = match now.get(axis) == Some(value) {
            true => (PLight::Strong, LIT),
            false => (PLight::Soft, FILL),
        };
        // Held on the group's ground: its light runs against it.
        label(paint, rect, round, light, true, token);
        paint.text(PPoint::new(rect.origin.x + gap, rect.center().y - line / 2.0), said, ink);
        spot.push((rect, target.to_string()));
        x += long + gap / 2.0;
        end = rect.origin.y + rect.size.height;
    }
    // Auto, where this is its axis: at the right end of the row, lit while
    // the look follows the system (the user: "at the end of that row").
    let ours = auto(user).is_some_and(|(name, ..)| axes(user).get(axis) == Some(&name));
    if ours {
        let said = word(user, &format!("{VALUE}{AUTO}"), AUTO);
        let long = paint.measure(said) + gap * 2.0;
        if x + long > at.x + wide && x > at.x {
            y += line + gap * 1.5;
        }
        let rect = PRect::new(at.x + wide - long, y, long, line + gap);
        let (light, token) = match shade.following().is_some() {
            true => (PLight::Strong, LIT),
            false => (PLight::Soft, FILL),
        };
        label(paint, rect, round, light, true, token);
        paint.text(PPoint::new(rect.origin.x + gap, rect.center().y - line / 2.0), said, ink);
        spot.push((rect, AUTO_PRESS.to_string()));
        end = rect.origin.y + rect.size.height;
    }
    end
}

/// The word over an axis's values.
pub fn said<'a>(user: &'a PTable, axis: &'a str) -> &'a str {
    word(user, &format!("{WORD}{axis}"), axis)
}
