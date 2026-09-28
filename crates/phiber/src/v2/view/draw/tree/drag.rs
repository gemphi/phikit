//! A tree row as a drag source and a drop target — the Outliner's drag and
//! drop (`outliner_dragdrop.cc`): press on a row and move, and a ghost of
//! it (its icon and name) follows the pointer while the row under the
//! pointer lights as a place to drop; letting go there writes
//! `<dragged>><target>` (`BTreeDrag::drop`), led by `&` when Ctrl is held,
//! which the Outliner's rule turns into a move, a link, a parent or a
//! nesting.

use super::super::color::to_hsla;
use super::super::ctx::BRenderCtx;
use super::super::flex::{BAlign, BEdge, BFill, BFlex, BLen, BRadius, BSpace, BType};
use crate::v2::theme::{BStyleRole, BSurfaceTone};
use crate::v2::view::prim::{BAction, BTreeDrag};
use gpui::prelude::*;
use gpui::{Context, Div, IntoElement, Render, SharedString, Stateful, StyleRefinement, Window, div, svg};
use std::rc::Rc;

/// What is carried: the dragged row's payload.
#[derive(Clone, Debug)]
pub(super) struct BRowDrag(pub String);

/// The ghost under the pointer — the row's icon and name on a lifted chip,
/// as Blender draws a drag beside the cursor (`wm_drag_draw_default`). The
/// row itself stays where it is. gpui draws the ghost with no render
/// context, so it carries its style as `BFlex` resolved it when the drag
/// was wired.
#[derive(Clone)]
struct BGhost {
    chip: StyleRefinement,
    glyph: StyleRefinement,
    icon: SharedString,
    label: SharedString,
}

impl BGhost {
    /// The chip: the floating surface the action log's drawer sits on.
    const CHIP: BFlex = BFlex::row()
        .align(BAlign::Center)
        .gap(BSpace::GapXs)
        .pad_x(BSpace::PadSm)
        .pad_y(BSpace::PadXs)
        .fill(BFill::Tone(BSurfaceTone::Modal))
        .border(BEdge::All)
        .radius(BRadius::Role(BStyleRole::Popover))
        .lifted()
        .ink()
        .type_size(BType::Body);

    fn of(label: &str, icon: &str, ctx: &BRenderCtx) -> Self {
        let side = BLen::Measured(ctx.style.dimensions.icon_size);
        let mut chip = Self::CHIP.styled(div(), ctx);
        let mut glyph = BFlex::cell().width(side).height(side).keeps_size().ink().styled(svg(), ctx);
        Self {
            chip: chip.style().clone(),
            glyph: glyph.style().clone(),
            icon: super::super::atom::icon_path(icon).into(),
            label: label.to_string().into(),
        }
    }
}

impl Render for BGhost {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let (mut chip, mut glyph) = (div(), svg().path(self.icon.clone()));
        *chip.style() = self.chip.clone();
        *glyph.style() = self.glyph.clone();
        chip.debug_selector(|| "outliner-drag-ghost".into()).child(glyph).child(self.label.clone())
    }
}

impl BRowDrag {
    /// `row` (showing `icon`) dragged as `drag.payload`, and dropped on.
    pub fn wire(row: Stateful<Div>, drag: &BTreeDrag, icon: &str, ctx: &BRenderCtx) -> Stateful<Div> {
        let payload = drag.payload.clone();
        let (target, here) = (drag.payload.clone(), drag.payload.clone());
        let key = drag.drop.clone();
        let handler = ctx.handler.clone();
        let lit = to_hsla(ctx.style.colors.row_selected);
        let debug = format!("outliner-row-{payload}");
        let ghost = Rc::new(BGhost::of(&drag.label, icon, ctx));
        row.debug_selector(move || debug.clone())
            .on_drag(BRowDrag(payload), move |_, _, _, cx| cx.new(|_| (*ghost).clone()))
            .drag_over::<BRowDrag>(move |style, dragged: &BRowDrag, _, _| if dragged.0 == target { style } else { style.bg(lit) })
            .on_drop::<BRowDrag>(move |dragged: &BRowDrag, window, app| {
                // A row let go on itself is no drop; Ctrl links rather than moves.
                if dragged.0 != here {
                    let ctrl = if window.modifiers().control { "&" } else { "" };
                    (Rc::clone(&handler))(&BAction::Set(key.clone(), format!("{ctrl}{}>{here}", dragged.0)), window, app);
                }
            })
    }
}
