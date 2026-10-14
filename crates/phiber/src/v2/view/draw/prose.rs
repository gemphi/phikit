//! Prose: wrapping text for reading, by role (`BView::Prose`).
//!
//! A label is a line of chrome; prose is a paragraph. Every role is a
//! `BFlex` decision — size, ink, the air above a heading, the marker before
//! an item — so the reader's page is set from the theme like everything else.

use super::ctx::BRenderCtx;
use super::flex::{BAlign, BFlex, BFlexed, BInk, BLen, BSpace, BType};
use crate::v2::view::prim::BProse;
use gpui::prelude::*;
use gpui::{AnyElement, SharedString};

/// The mark before a list item.
const BULLET: &str = "•";

enum BProsePart {
    /// The block: a column of one text run, the width of the page.
    Block(BProse),
    /// An item's marker cell.
    Marker,
    /// An item's text, beside its marker.
    ItemText,
}

impl BFlexed for BProsePart {
    fn flex(&self, _c: &BRenderCtx) -> BFlex {
        let text = BFlex::column().width(BLen::Full).min_width(BLen::Zero);
        match self {
            // Every role takes a reading size (`BType::Prose*`), the title
            // the largest and a note the paragraph's own in the dim ink —
            // a note at the caption size was the one unreadable line on the
            // page, and the paragraphs with no size took the element default
            // and outgrew their title.
            Self::Block(BProse::Title) => text.type_size(BType::ProseTitle).ink().pad_bottom(BSpace::GapSm),
            Self::Block(BProse::Heading) => text.type_size(BType::ProseHeading).ink().pad_top(BSpace::GapLg).pad_bottom(BSpace::GapXs),
            Self::Block(BProse::Para) => text.type_size(BType::Prose).ink().pad_bottom(BSpace::GapSm),
            Self::Block(BProse::Term) => text.type_size(BType::Prose).ink().pad_bottom(BSpace::GapXs),
            Self::Block(BProse::Note) => text.ink_tone(BInk::Dim).type_size(BType::Prose).pad_bottom(BSpace::GapSm),
            Self::Block(BProse::Item) => BFlex::row()
                .width(BLen::Full)
                .min_width(BLen::Zero)
                .align(BAlign::Start)
                .gap(BSpace::GapSm)
                .type_size(BType::Prose)
                .ink()
                .pad_bottom(BSpace::GapXs),
            Self::Marker => BFlex::cell().keeps_size().ink_tone(BInk::Dim),
            Self::ItemText => BFlex::cell().fills().min_width(BLen::Zero),
        }
    }
}

pub fn view(role: BProse, text: &str, ctx: &BRenderCtx) -> AnyElement {
    let block = BProsePart::Block(role).flex(ctx).div(ctx);
    let text = SharedString::from(text.to_string());
    match role {
        BProse::Item => block
            .child(BProsePart::Marker.flex(ctx).div(ctx).child(SharedString::from(BULLET)))
            .child(BProsePart::ItemText.flex(ctx).div(ctx).child(text))
            .into_any_element(),
        _ => block.child(text).into_any_element(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::flex::{PROSE_HEADING_SCALE, PROSE_SCALE, PROSE_TITLE_SCALE};
    use crate::v2::theme::{BPaletteName, BResolvedUiStyle, ui_style_defaults};

    #[test]
    fn a_page_reads_title_over_heading_over_paragraph_and_every_line_is_readable() {
        let style = BResolvedUiStyle::from(ui_style_defaults(BPaletteName::DEFAULT));
        let t = &style.typography;
        let (title, heading, para) = (t.header * PROSE_TITLE_SCALE, t.header * PROSE_HEADING_SCALE, t.body * PROSE_SCALE);
        assert!(title > heading && heading > para, "the hierarchy: {title} > {heading} > {para}");
        assert!(para >= 13.0, "a paragraph is readable, not a label: {para}");
        assert!(para > t.caption && para > t.body, "and larger than any chrome text");
        // Every role is sized: none is left to the element default.
        for role in [BProse::Title, BProse::Heading, BProse::Para, BProse::Term, BProse::Note, BProse::Item] {
            let flex = BProsePart::Block(role).flex(&crate::v2::view::draw::ctx::BRenderCtx::new(
                &style,
                &crate::v2::state::BState::new(),
                std::rc::Rc::new(|_, _, _| {}),
                std::rc::Rc::new(|_, _, _, _, _, _, _| {}),
            ));
            assert!(matches!(flex.type_size, Some(BType::Prose | BType::ProseHeading | BType::ProseTitle)), "{role:?} is sized for reading");
        }
    }
}
