//! Focused tree family: indent, guide, disclosure, and row renderers.
//!
//! A tree is a table with indentation: a row's depth is a run of indent
//! units before its arrow slot, which every row keeps (a leaf's is blank,
//! as Blender reserves it), and the hierarchy lines that run down those
//! units are drawn once behind the whole column (`guide::guides_backdrop`),
//! so they reach the column's foot and no row draws a line of its own.

use super::atom;
use super::ctx::BRenderCtx;
use super::flex::{BAlign, BDim, BFill, BFlex, BFlexed, BJustify, BLen, BOverflow, BPlace, BRadius, BSpace, BType};
use super::walk;
use crate::v2::theme::BStyleRole;
use crate::v2::view::BView;
use crate::v2::view::prim::{BAction, BRowShade};
use gpui::prelude::*;
use gpui::AnyElement;

mod disclosure;
use self::disclosure::BTreeDisclosure;

mod drag;
mod guide;
pub use self::guide::guides_backdrop;

mod window;
pub use self::window::BRowWindow;

/// The parts of a tree row — one enum for the family, as every other level
/// and family states its parts (`BCardPart`, `BMenuPart`, `BRailPart`).
pub(super) enum BTreePart {
    /// The run of indent units before the arrow slot: a row's depth, drawn.
    Indent(u16),
    /// The row itself. `shade` is what it paints at rest and `selected` is
    /// whether the pointer may repaint it — a selected row keeps its fill.
    Row { shade: BRowShade, selected: bool },
    /// A one-unit slot in the row: the type icon's, and the disclosure
    /// triangle's (`disclosure`). One shape, so the arrow column and the
    /// icon column line up down the whole tree by construction rather than
    /// by two places agreeing on a number.
    IconSlot,
    /// The name and whatever the row lists after it: takes the slack, clips.
    Main,
    /// The controls at the row's end (the restriction toggles).
    Trail,
}

impl BFlexed for BTreePart {
    fn flex(&self, ctx: &BRenderCtx) -> BFlex {
        let unit = ctx.style.dimensions.indent_unit;
        match *self {
            Self::Indent(depth) => {
                BFlex::cell().width(BLen::Measured(unit * f32::from(depth))).keeps_size()
            }
            // A tree row takes the same fills as every other selectable
            // control in the app — the row shade when banded or selected,
            // the shared item hover otherwise. Palette-only `row_alt` /
            // `row_selected` entries let the outliner drift to shades no
            // other region used.
            //
            // At rest it is its editor's body (`host_body_surface`), the
            // surface the list sits on, so the Outliner's content and the
            // region round it are one ground — lifted rows read as strips on
            // a darker parent.
            Self::Row { shade, selected } => {
                let row = BFlex::row()
                    .width(BLen::Full)
                    .height(BLen::Dim(BDim::RowHeight))
                    // A row keeps its height: when the host is short the list
                    // scrolls (`column`) rather than squashing every row.
                    .keeps_size()
                    .align(BAlign::Center)
                    .place(BPlace::Anchor)
                    .radius(BRadius::Role(BStyleRole::MenuItem))
                    .pad_right(BSpace::PadMd)
                    .overflow(BOverflow::Clip)
                    .ink()
                    .type_size(BType::Body)
                    // Asked of the shade, not listed here. The list said
                    // `Selected | Active`, so `Alt` — the band — fell into the
                    // catch-all and painted the body: the Outliner computed a
                    // banded shade for every other row and the renderer threw
                    // it away, which is why its rows had no background at all
                    // but the selected one. The doc two lines up has said
                    // "banded or selected" the whole time.
                    //
                    // `BRowShade::fill` already knows which shades have a
                    // fill; a second list of them here could only drift from
                    // it, and did.
                    .fill(match shade.fill(ctx.style) {
                        Some(_) => BFill::Row(shade),
                        None => BFill::HostBody,
                    });
                if selected { row } else { row.hovered(BFill::RowHover(shade)) }
            }
            Self::IconSlot => BFlex::row()
                .width(BLen::Measured(unit))
                .height(BLen::Full)
                .keeps_size()
                .align(BAlign::Center)
                .justify(BJustify::Center),
            Self::Main => BFlex::row()
                .fills()
                .min_width(BLen::Zero)
                .align(BAlign::Center)
                .gap(BSpace::GapXs)
                .overflow(BOverflow::Clip),
            Self::Trail => {
                BFlex::row().keeps_size().align(BAlign::Center).gap(BSpace::GapXs)
            }
        }
    }
}

pub struct BTreeIndent(pub u16);

pub struct BTreeRow<'a> {
    pub depth: u16,
    pub expanded: Option<bool>,
    pub toggle: Option<&'a BAction>,
    /// Between the disclosure and the icon (a collection's checkbox).
    pub lead: &'a [BView],
    pub icon: &'a str,
    pub list: &'a [BView],
    pub trail: &'a [BView],
    pub shade: BRowShade,
    /// Dragged and dropped on (`drag.rs`).
    pub drag: Option<&'a crate::v2::view::prim::BTreeDrag>,
}

impl BTreeIndent {
    pub fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        BTreePart::Indent(self.0).flex(ctx).div(ctx).into_any_element()
    }
}

impl BTreeRow<'_> {
    pub fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        let selected = matches!(self.shade, BRowShade::Selected | BRowShade::Active);
        let mut element = BTreePart::Row { shade: self.shade, selected }.flex(ctx).div(ctx);
        element = element.child(BTreeIndent(self.depth).render(ctx));
        element = element.child(
            BTreeDisclosure {
                expanded: self.expanded,
                toggle: self.toggle,
            }
            .render(ctx),
        );
        for child in self.lead {
            element = element.child(walk(child, ctx));
        }
        element = element
            .child(BTreePart::IconSlot.flex(ctx).div(ctx).child(atom::icon(self.icon, ctx)));

        // The row is the control. Blender selects an outliner item by clicking
        // anywhere on its row and lights the whole row on hover; a labelled
        // button as the name cell gave the name its own hover fill inside the
        // row's, so only the text looked hovered. The name's action lifts to
        // the row and the name draws as plain text.
        let (select, cell_list) = match self.list.split_first() {
            Some((BView::Button(ctl), rest)) if ctl.icon.is_none() && ctl.action.is_some() => {
                (ctl.action.clone(), std::iter::once(BView::label(ctl.label.clone().unwrap_or_default())).chain(rest.iter().cloned()).collect::<Vec<_>>())
            }
            _ => (None, self.list.to_vec()),
        };
        let mut main = BTreePart::Main.flex(ctx).div(ctx);
        for child in &cell_list {
            main = main.child(walk(child, ctx));
        }
        element = element.child(main);

        if !self.trail.is_empty() {
            let mut trail = BTreePart::Trail.flex(ctx).div(ctx);
            for child in self.trail {
                trail = trail.child(walk(child, ctx));
            }
            element = element.child(trail);
        }
        let id = match (&select, self.drag) {
            (_, Some(drag)) => format!("outliner-row-{}", drag.payload),
            (Some(action), None) => super::button::element_id("tree-row", None, action),
            (None, None) => return element.into_any_element(),
        };
        let mut element = element.id(gpui::SharedString::from(id));
        if let Some(action) = select {
            let handler = ctx.handler.clone();
            element = element.cursor_pointer().on_click(move |_event, window, app| {
                (std::rc::Rc::clone(&handler))(&action, window, app);
            });
        }
        match self.drag {
            Some(drag) => self::drag::BRowDrag::wire(element, drag, self.icon, ctx).into_any_element(),
            None => element.into_any_element(),
        }
    }
}
