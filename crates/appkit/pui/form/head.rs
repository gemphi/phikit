//! What a block's heading holds, as the web kit's sections have it
//! (`puikit/src/components/layout/Sections`) and Finder's and Reality
//! Composer Pro's browsers do: beside its words, how its content lies
//! (`layout`: a drop of three, a grid, large tiles, a list, `list` too
//! where it begins as one); at its
//! right, drops that choose from a list (`drop=icon:list:path`, a filter,
//! an order), for a block that lays out a switch that widens it to the
//! whole form and back (`ui/expand/<key>`), and a fold (`fold`: open, or
//! only its heading). A block
//! asks for them by words of its variation; what a layout and a fold
//! keep is a path of the app's `ui` state, the block's key after it, so
//! the app needs nothing more. Which drop is open is kept at `ui/drop`.

use super::PSource;
use crate::pui::core::{PPaint, PPoint, PRect};

/// The paths a fold and a layout are kept at, the block's key after them,
/// and the one that says which drop is open.
pub const FOLD: &str = "ui/fold/";
/// The path a block widened to the whole form is kept at, its key after.
pub const EXPAND: &str = "ui/expand/";
pub const LAYOUT: &str = "ui/layout/";
pub const DROP: &str = "ui/drop";
/// What a folded block, and a block laid as a list or as large tiles,
/// keep there.
pub const FOLDED: &str = "1";
pub const LIST: &str = "list";
pub const LARGE: &str = "large";
/// The three ways a block can lie: what each keeps, its icon, and the
/// word of the app's text that names it.
pub const LAYOUTS: [(&str, &str, &str); 3] = [("grid", "grid", "layout.grid"), (LARGE, "square", "layout.large"), (LIST, "list", "layout.list")];

/// The theme's tokens: an icon's size, a line, a gap, the ink.
const ICON: &str = "icon.size";
const LINE: &str = "text.line";
const GAP: &str = "gap";
const INK: &str = "text.dim";

/// A switch at a heading: its icon, the value a press keeps at its path,
/// and, for a drop, its name and the list and path it chooses for.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PTool {
    pub icon: String,
    pub path: String,
    pub value: String,
    pub drop: Option<PDrop>,
}

/// A drop: which it is (`key/name`), the list it shows and the path its
/// choice is kept at; for a layout, no list but the three.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PDrop {
    pub id: String,
    pub list: String,
    pub path: String,
}

/// How a block stands now: the switch beside its words, those at its
/// right, whether it is folded, and how it lies.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PHead {
    pub left: Vec<PTool>,
    pub tool: Vec<PTool>,
    pub folded: bool,
    pub layout: String,
}

/// A drop's switch: a press opens it, and another puts it away.
fn drop(id: String, icon: &str, list: &str, path: String, open: &str) -> PTool {
    let value = match open == id {
        true => String::new(),
        false => id.clone(),
    };
    PTool { icon: icon.to_string(), path: DROP.to_string(), value, drop: Some(PDrop { id, list: list.to_string(), path }) }
}

impl PHead {
    /// The switches a variation asks for, as the source keeps them now.
    pub fn of(variant: &str, key: &str, source: &dyn PSource) -> PHead {
        let word: Vec<&str> = variant.split_whitespace().collect();
        let (fold, layout, open) = (format!("{FOLD}{key}"), format!("{LAYOUT}{key}"), source.get(DROP));
        let folded = word.contains(&"fold") && source.get(&fold) == FOLDED;
        // How it lies: as the person chose; until they do, a list where
        // the variation says `list` (rooms in a narrow column), else a grid.
        let now = match (word.contains(&"layout"), word.contains(&LIST)) {
            (true, said) => match (source.get(&layout), said) {
                (kept, _) if !kept.is_empty() => kept,
                (_, true) => LIST.to_string(),
                (_, false) => String::new(),
            },
            (false, _) => String::new(),
        };
        let mut head = PHead { folded, layout: now.clone(), ..PHead::default() };
        if word.contains(&"layout") {
            let icon = LAYOUTS.iter().find(|(value, ..)| *value == now).map_or("grid", |(_, icon, _)| icon);
            head.left.push(drop(format!("{key}/layout"), icon, "", layout, &open));
        }
        for spec in word.iter().filter_map(|word| word.strip_prefix("drop=")) {
            let mut part = spec.splitn(3, ':');
            let (Some(icon), Some(list), Some(path)) = (part.next(), part.next(), part.next()) else { continue };
            head.tool.push(drop(format!("{key}/{icon}"), icon, list, path.to_string(), &open));
        }
        // Every block that lays out (a grid) can take the whole form, and
        // give it back: as Figma's sections expand.
        if word.contains(&"layout") {
            let path = format!("{EXPAND}{key}");
            let (icon, value) = match source.get(&path) == FOLDED {
                true => ("x", String::new()),
                false => ("maximize", FOLDED.to_string()),
            };
            head.tool.push(PTool { icon: icon.to_string(), path, value, drop: None });
        }
        if word.contains(&"fold") {
            let (icon, value) = match folded {
                true => ("chevron-right", String::new()),
                false => ("chevron-down", FOLDED.to_string()),
            };
            head.tool.push(PTool { icon: icon.to_string(), path: fold, value, drop: None });
        }
        head
    }

    /// Every switch, those beside the words first: in the order `draw`
    /// gives their places.
    pub fn all(&self) -> impl Iterator<Item = &PTool> {
        self.left.iter().chain(&self.tool)
    }

    /// Draw the switches of the heading over a block whose content
    /// begins at `at`, across `wide`, its words `words` wide: where each
    /// stands, those beside the words first. A drop's switch has a small
    /// arrow after its icon.
    pub fn draw(&self, paint: &mut dyn PPaint, at: PPoint, wide: f32, words: f32) -> Vec<PRect> {
        let (icon, line, gap, ink) = (paint.size(ICON).max(1.0), paint.size(LINE), paint.size(GAP), paint.color(INK));
        let top = at.y - line - gap / 2.0 + (line - icon) / 2.0;
        let small = icon * 0.6;
        let one = |tool: &PTool, x: f32, paint: &mut dyn PPaint| {
            let wide = match tool.drop.is_some() {
                true => icon + small,
                false => icon,
            };
            paint.icon(&tool.icon, PRect::new(x, top, icon, icon), ink);
            if wide > icon {
                paint.icon("chevron-down", PRect::new(x + icon, top + (icon - small) / 2.0, small, small), ink);
            }
            let place = PRect::new(x, top, wide, icon);
            paint.press(place);
            place
        };
        let mut rect = Vec::new();
        let mut left = at.x + words + gap / 2.0;
        for tool in &self.left {
            let place = one(tool, left, paint);
            left += place.size.width + gap / 2.0;
            rect.push(place);
        }
        // Those at the right, in their order, the last at its end.
        let span = |tool: &PTool| match tool.drop.is_some() {
            true => icon + small,
            false => icon,
        };
        let all: f32 = self.tool.iter().map(|tool| span(tool) + gap / 2.0).sum();
        let mut x = at.x + wide - (all - gap / 2.0).max(0.0);
        for tool in &self.tool {
            rect.push(one(tool, x, paint));
            x += span(tool) + gap / 2.0;
        }
        rect
    }
}

/// A variation without the words of the heading's switches: the light a
/// block's rows take (`soft`, `strong`).
pub fn light(variant: &str) -> String {
    variant.split_whitespace().filter(|word| !matches!(*word, "fold" | "layout" | LIST) && !word.starts_with("drop=")).collect::<Vec<_>>().join(" ")
}
