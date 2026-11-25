//! A form's blocks as the kit's groups: what each shows under its
//! heading, the switches at the heading's right (`head.rs`), a line to
//! type (`field.rs`); what a press on each place keeps, and the keys
//! given to the line that has them.

use super::field::{self, PTyped};
use super::head::{self, PHead};
use super::{Block, PChoice, PForm, Spot, drawn};
use crate::pui::core::{PEvent, PPaint, PPoint, PRect};
use crate::pui::layout::PGrouped;

/// The path a press on a `field` block keeps, the block's key its value:
/// the form gives that block the keys. No state is named so.
pub const FOCUS: &str = "<focus>";

/// A block shown, what it holds, and the switches at its heading.
pub type Shown<'a> = (&'a Block, Vec<PChoice>, PHead);

impl PForm {
    /// The kit's groups for some blocks, with what each shows and the
    /// switches at its heading's right. A block whose list is empty is
    /// not shown, but for a line to type.
    pub(super) fn grouped<'a>(&'a self, block: &[&'a Block]) -> (Vec<PGrouped<'a>>, Vec<Shown<'a>>) {
        let one = |b: &&'a Block| {
            let mut head = PHead::of(&b.variant, &b.key, self.source.as_ref());
            if b.head.is_empty() {
                head.tool.clear();
                head.left.clear();
            }
            let list = match b.kind.as_str() {
                "wheel" => crate::pui::block::wheel::colours().into_iter().map(|value| PChoice { value, ..PChoice::default() }).collect(),
                _ => self.source.list(&b.list),
            };
            (*b, list, self.source.get(&b.path), head)
        };
        let shown: Vec<(&Block, Vec<PChoice>, String, PHead)> = block.iter().map(one).filter(|(b, c, _, _)| !c.is_empty() || b.kind == "field").collect();
        let (typing, draft) = (self.typing.borrow().clone(), self.draft.borrow().clone());
        let group = shown.iter().map(|(b, choice, now, head)| {
            let kind = match (b.kind.as_str(), head.layout.as_str()) {
                ("grid", head::LIST) => "rows".to_string(),
                ("grid", head::LARGE) => "large".to_string(),
                (kind, _) => kind.to_string(),
            };
            let (light, choice, now, head) = (head::light(&b.variant), choice.clone(), now.clone(), head.clone());
            let typed = match typing.as_deref() == Some(b.key.as_str()) {
                true => Some(draft.clone()),
                false => None,
            };
            let (words, over) = (self.text.get(&b.head).to_string(), self.over.get());
            let draw = move |paint: &mut dyn PPaint, at: PPoint, wide: f32, right: bool| {
                let words = paint.measure(&words);
                let mut rect = head.draw(paint, at, wide, words);
                if head.folded {
                    return (at.y, rect);
                }
                let (end, more) = match kind.as_str() {
                    "field" => field::draw(&choice, typed.as_deref().unwrap_or_default(), typed.is_some(), paint, at, wide),
                    _ => drawn(&kind, &light, &choice, &now, paint, at, wide, right, over),
                };
                rect.extend(more);
                (end, rect)
            };
            PGrouped { head: self.text.get(&b.head).to_string(), draw: Box::new(draw) }
        }).collect();
        (group, shown.into_iter().map(|(b, c, _, h)| (b, c, h)).collect())
    }

    /// What a press on each place of a block keeps: its switches first,
    /// then its choices; a line to type takes the keys.
    pub(super) fn spots(block: &Block, head: &PHead, choice: &[PChoice], rect: Vec<PRect>) -> Vec<Spot> {
        let count = head.all().count();
        let tool = rect.iter().take(count).zip(head.all()).map(|(r, tool)| (*r, tool.path.clone(), tool.value.clone()));
        let rest = rect.iter().skip(count);
        let content: Vec<Spot> = match block.kind.as_str() {
            "field" => rest.take(1).map(|r| (*r, FOCUS.to_string(), block.key.clone())).collect(),
            _ => rest.zip(choice).map(|(r, c)| (*r, block.path.clone(), c.value.clone())).collect(),
        };
        tool.chain(content).collect()
    }

    /// A key, given to the line that has the keys: what Enter keeps is
    /// kept at that block's path.
    pub(super) fn typed(&self, event: &PEvent) -> bool {
        let Some(key) = self.typing.borrow().clone() else { return false };
        let typed = field::key(&mut self.draft.borrow_mut(), event);
        match typed {
            Some(PTyped::Kept(text)) => {
                let path = self.block.iter().find(|b| b.key == key).map(|b| b.path.clone()).unwrap_or_default();
                if !text.trim().is_empty() {
                    self.source.set(&path, text.trim());
                }
                true
            }
            Some(PTyped::Left) => {
                *self.typing.borrow_mut() = None;
                true
            }
            // A line that says `live` keeps what is typed as it is typed,
            // so what it searches narrows at once.
            Some(PTyped::Edit) => {
                let block = self.block.iter().find(|b| b.key == key);
                if let Some(block) = block.filter(|b| b.variant.split_whitespace().any(|word| word == "live")) {
                    self.source.set(&block.path, self.draft.borrow().trim());
                }
                true
            }
            None => false,
        }
    }
}
