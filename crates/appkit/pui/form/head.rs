//! What a block's heading holds at its right, as the web kit's sections
//! have it (`puikit/src/components/layout/Sections`): a switch of how its
//! content lies (`layout`: grid or list) and a fold (`fold`: open, or only
//! its heading). A block asks for them by words of its variation
//! (`soft fold layout`); what each keeps is a path of the app's `ui`
//! state, the block's key after it, so the app needs nothing more.

use super::PSource;
use crate::pui::core::{PPaint, PPoint, PRect};

/// The paths a fold and a layout are kept at, the block's key after them.
pub const FOLD: &str = "ui/fold/";
pub const LAYOUT: &str = "ui/layout/";
/// What a folded block, and a block laid as a list, keep there.
pub const FOLDED: &str = "1";
pub const LIST: &str = "list";

/// The theme's tokens: an icon's size, a line, a gap, the ink.
const ICON: &str = "icon.size";
const LINE: &str = "text.line";
const GAP: &str = "gap";
const INK: &str = "text.dim";

/// A switch at a heading's right: its icon, and the value a press keeps
/// at its path.
#[derive(Debug, Clone, PartialEq)]
pub struct PTool {
    pub icon: &'static str,
    pub path: String,
    pub value: String,
}

/// How a block stands now: its switches, whether it is folded, whether
/// it lies as a list.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PHead {
    pub tool: Vec<PTool>,
    pub folded: bool,
    pub list: bool,
}

impl PHead {
    /// The switches a variation asks for, as the source keeps them now.
    pub fn of(variant: &str, key: &str, source: &dyn PSource) -> PHead {
        let word: Vec<&str> = variant.split_whitespace().collect();
        let (fold, layout) = (format!("{FOLD}{key}"), format!("{LAYOUT}{key}"));
        let folded = word.contains(&"fold") && source.get(&fold) == FOLDED;
        let list = word.contains(&"layout") && source.get(&layout) == LIST;
        let mut tool = Vec::new();
        if word.contains(&"layout") {
            let (icon, value) = match list {
                true => ("grid", String::new()),
                false => ("list", LIST.to_string()),
            };
            tool.push(PTool { icon, path: layout, value });
        }
        if word.contains(&"fold") {
            let (icon, value) = match folded {
                true => ("chevron-right", String::new()),
                false => ("chevron-down", FOLDED.to_string()),
            };
            tool.push(PTool { icon, path: fold, value });
        }
        PHead { tool, folded, list }
    }

    /// Draw the switches at the right of the heading over a block whose
    /// content begins at `at`, across `wide`: where each stands.
    pub fn draw(&self, paint: &mut dyn PPaint, at: PPoint, wide: f32) -> Vec<PRect> {
        let (icon, line, gap, ink) = (paint.size(ICON).max(1.0), paint.size(LINE), paint.size(GAP), paint.color(INK));
        let top = at.y - line - gap / 2.0 + (line - icon) / 2.0;
        let mut right = at.x + wide;
        let mut rect = Vec::new();
        for tool in &self.tool {
            right -= icon;
            let place = PRect::new(right, top, icon, icon);
            paint.icon(tool.icon, place, ink);
            paint.press(place);
            rect.push(place);
            right -= gap / 2.0;
        }
        rect
    }
}

/// A variation without the words of the heading's switches: the light a
/// block's rows take (`soft`, `strong`).
pub fn light(variant: &str) -> String {
    variant.split_whitespace().filter(|word| !matches!(*word, "fold" | "layout")).collect::<Vec<_>>().join(" ")
}
