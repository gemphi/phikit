//! The one renderer for `BView::Card` — Blender's panel drawing
//! (`draw_aligned_panel` → `panel_draw_aligned_backdrop` +
//! `panel_draw_aligned_widgets`, `interface_panel.cc`).
//!
//! The header is one strip, `PNL_HEADER` tall: the fold triangle at its left
//! (`ICON_RIGHTARROW` closed, `ICON_DOWNARROW_HLT` open), the header's own
//! controls (`draw_header`, e.g. Motion Blur's checkbox), the title, then,
//! pushed right, the preset controls (`draw_header_preset`) and the grip
//! (`ICON_GRIP`, half alpha).
//! A closed panel is its header alone, rounded on every corner; an open one
//! rounds the header's top and the backdrop's bottom. The whole header,
//! except what it holds, is the fold's click target, as Blender's is.
//!
//! Header gestures (`handle_panel_header`): a click folds; Ctrl-click opens
//! this panel and closes the rest of its region (`panels_collapse_all`); a
//! press dragged across headers flips the first and sets every header it
//! crosses to the same state (`panel_drag_collapse`).
//!
//! Every container here is a `BFlex` — this file writes no GPUI styling
//! (`tests/style_budget.rs` starts it at zero).

use super::flex::{BAlign, BEdge, BFill, BFlex, BFlexed, BJustify, BLen, BOverflow, BRadius, BSpace, BType};
use super::ctx::BRenderCtx;
use super::walk;
use super::slot::slot;
use crate::v2::theme::{BStyleRole, BSurfaceTone};
use crate::v2::view::BView;
use crate::v2::view::prim::{BCard, BCardKind, BColumn};
use gpui::prelude::*;
use gpui::{AnyElement, SharedString};

enum BCardPart {
    /// The card: backdrop and outline, per kind. `open` is whether a
    /// FOLDING card is unfolded — a bare accordion takes a border while its
    /// rows are showing, so where its contents start and stop is visible
    /// against whatever it sits on, and gives it back when it folds away.
    /// Without it a column of open plain accordions runs together.
    Frame { kind: BCardKind, open: bool },
    /// The heading strip; `open` decides which corners it rounds, `rule`
    /// whether a line runs under it — an unfolded heading over its rows.
    Head { kind: BCardKind, open: bool, rule: bool },
    /// A fixed-size slot in the header: the triangle, the grip.
    Glyph { dimmed: bool },
    /// The title, which takes the header's free space and clips.
    Title,
    /// A run of header controls (`draw_header` / `draw_header_preset`).
    Controls,
    /// The rows under the header.
    /// The rows under the heading — or, in a headless card, the whole card:
    /// its rows then start where a heading would, so a headless card and a
    /// headed one share one row grid with the rail beside them.
    /// `list`: every row a menu row, which is its own height — they stack
    /// with no gap, as a menu's rows do.
    Body { headless: bool, fold: Option<bool>, list: bool },
    /// One row; bare text gets a row's own inset so it lines up with rows
    /// that carry theirs.
    /// `own`: a menu row, which is its own module and needs no floor.
    Row { text: bool, own: bool },
    /// The strip under the body — a dialog's actions.
    Foot,
}

fn corner(ctx: &BRenderCtx) -> f32 {
    ctx.style.radius_for(BStyleRole::Card)
}

impl BFlexed for BCardPart {
    fn flex(&self, ctx: &BRenderCtx) -> BFlex {
        match self {
            Self::Frame { kind, open } => {
                let column = BFlex::column().width(BLen::Full);
                match kind {
                    // No clipping: a dropdown opened from a row has to escape
                    // the card (`file_browser_accordion_refinement`).
                    // A group is a ground, not a box: its fill marks it out, with no
                    // border round it (menus, panels and areas alike).
                    BCardKind::Panel => column.radius(BRadius::Role(BStyleRole::Card)).fill(panel_fill(ctx)),
                    // A sub-panel is its parent's card again, one shade in:
                    // no indent and no guide down its left edge. Both marked
                    // it out as a different kind of thing, when the whole
                    // point is that a nested accordion is an accordion — it
                    // sits in its parent's body on that body's inset, so the
                    // space around it is even on all four sides.
                    // Indented one gutter under its parent's rows, its
                    // heading a shade in, behaving as the parent's does: a
                    // child accordion reads as its parent's child.
                    // No margin of its own: its parent's body already sets it
                    // in under the parent's title (`BSpace::FoldBody`).
                    BCardKind::Sub { depth } => BFlex::column()
                        .fills()
                        .radius(BRadius::Role(BStyleRole::Card))
                        .fill(BFill::Tone(BSurfaceTone::Nested(*depth))),
                    // A source list's section: no surface, a divider under it.
                    BCardKind::Section => column.border(BEdge::Bottom),
                    // A plain accordion with its rows showing takes a
                    // border, so the open one reads apart from its
                    // neighbours; closed, it is a row of text again. The
                    // Properties editor's transparent rows are bare cards
                    // that do not fold, so `open` is false there and they
                    // keep their outer border alone.
                    // Open, a plain accordion is a group: it takes a ground, not a
                    // border.
                    BCardKind::Bare if *open => column.radius(BRadius::Role(BStyleRole::Card)).fill(BFill::Tone(BSurfaceTone::Card)),
                    BCardKind::Bare | BCardKind::Strip(_) | BCardKind::Column(_) => column,
                }
            }
            Self::Head { kind, open, rule } => {
                // A nested heading pads `PadSm`, so its fold triangle lands
                // exactly under its parent's title, the next step of the tree.
                let pad = if matches!(kind, BCardKind::Sub { .. }) { BSpace::PadSm } else { BSpace::PadMd };
                let strip = BFlex::row()
                    .width(BLen::Full)
                    .height(BLen::Dim(super::flex::BDim::CardHead))
                    .align(BAlign::Center)
                    .gap(BSpace::GapSm)
                    .pad_x(pad)
                    .type_size(BType::Body)
                    .ink();
                // A section's own divider is under its rows, not its heading.
                let strip = if *rule && *kind != BCardKind::Section { strip.border(BEdge::Bottom) } else { strip };
                let rounded = if *open { BRadius::Top(corner(ctx)) } else { BRadius::Role(BStyleRole::Card) };
                match kind {
                    // A panel's heading lifts a step off its card
                    // (`heading_surface`). Blender paints only a top-level
                    // header (`panel_draw_aligned_backdrop`); the approved
                    // mock-ups lift a sub-panel's too, off its own well.
                    BCardKind::Panel => {
                        strip.fill(BFill::Heading(BSurfaceTone::Card)).radius(rounded).overflow(BOverflow::Clip)
                    }
                    BCardKind::Sub { depth } => {
                        strip
                            .fill(BFill::Heading(BSurfaceTone::Nested(*depth)))
                            .radius(rounded)
                            .overflow(BOverflow::Clip)
                    }
                    BCardKind::Bare | BCardKind::Section | BCardKind::Strip(_) | BCardKind::Column(_) => strip,
                }
            }
            Self::Glyph { dimmed } => {
                let cell = BFlex::row().align(BAlign::Center).justify(BJustify::Center).keeps_size();
                if *dimmed { cell.dimmed() } else { cell }
            }
            Self::Title => BFlex::row()
                .fills()
                .min_width(BLen::Zero)
                .align(BAlign::Center)
                .overflow(BOverflow::Clip),
            Self::Controls => BFlex::row().align(BAlign::Center).gap(BSpace::GapXs).keeps_size(),
            Self::Body { headless, fold, list } => {
                let column = BFlex::column().width(BLen::Full).pad_x(BSpace::CardInset);
                let column = if *list { column.gap(BSpace::None) } else { column.gap(BSpace::GapXs) };
                // A folding card's children are its tree: they stand under
                // its title, one step in, at every depth.
                let column = match fold {
                    Some(sub) => column.pad_left(BSpace::FoldBody { sub: *sub }),
                    None => column,
                };
                if *headless { column } else { column.pad_y(BSpace::PadXs) }
            }
            // A row is the card-head module tall at least, its content
            // centred, so a button in it has room over and under it.
            Self::Row { text, own } => {
                let row = BFlex::row().width(BLen::Full).align(BAlign::Center);
                let row = if *own { row } else { row.min_height(BLen::Dim(super::flex::BDim::CardHead)) };
                if *text { row.pad_x(BSpace::PadSm) } else { row }
            }
            Self::Foot => BFlex::row()
                .width(BLen::Full)
                .align(BAlign::Center)
                .justify(BJustify::End)
                .gap(BSpace::GapSm)
                .pad_x(BSpace::PadMd)
                .pad_y(BSpace::PadSm),
        }
    }
}


/// A panel's body surface (`TH_PANEL_BACK`): a step over the region it sits
/// in, in Properties as everywhere (Blender's `#3D3D3D` panel on its
/// `#303030` region) — and how that step is painted, with the theme's sweep
/// or flat.
///
/// Flat when the panel IS the region: the activity bar's secondary strip,
/// which fills its pane edge to edge with none of the pane showing round it.
/// A sweep reads as a raised card BECAUSE there is a surface under it to be
/// raised from; where none shows, the sweep's top stop is not a highlight but
/// a second shade running down the panel's own edge. And measured, that stop
/// was the exact shade the rail's entry tabs beside it are painted in, so the
/// strip read as more rail than panel. Flat, it is the card shade throughout:
/// the same shade as the content it carries.
///
/// One function, not a tone and a fill and a predicate. `card.rs` sits on its
/// line cap (`renderer_decomposition`), so a rule that arrives here has to pay
/// for itself — this one replaces the `panel_tone` that only ever answered
/// `Card`.
fn panel_fill(ctx: &BRenderCtx) -> BFill {
    // The activity bar's secondary column takes the floating bar's material
    // — the shade the viewport's toolbar and sidebar bars wear — so every
    // column standing beside a rail is one surface, not a card shade here
    // and a lighter bar there.
    if ctx.host == crate::v2::host::BHostType::Activity && ctx.region == Some(crate::v2::region::BRegionType::Center) { BFill::Floating } else { BFill::Tone(BSurfaceTone::Card) }
}

fn tagged(element: gpui::Div, id: String) -> gpui::Stateful<gpui::Div> {
    let selector = id.clone();
    element.id(SharedString::from(id)).debug_selector(move || selector)
}

fn head(card: &BCard, name: &str, ctx: &BRenderCtx) -> Option<AnyElement> {
    let head = card.head.as_ref()?;
    let open = card.is_open();
    let rule = card.fold.as_ref().is_some_and(|f| f.open) && !card.body.is_empty();
    let mut strip = tagged(BCardPart::Head { kind: card.kind, open, rule }.flex(ctx).div(ctx), format!("card-head-{name}"));
    if card.fold.is_some() {
        let triangle = if open { "triangle_down" } else { "triangle_right" };
        strip = strip.child(tagged(
            BCardPart::Glyph { dimmed: false }.flex(ctx).div(ctx),
            format!("card-fold-{name}"),
        )
        .child(walk(&BView::icon(triangle), ctx)));
    }
    if let Some(icon) = &head.icon {
        strip = strip.child(BCardPart::Glyph { dimmed: false }.flex(ctx).div(ctx).child(walk(&BView::icon(icon.clone()), ctx)));
    }
    // `draw_header` lays its controls out before the label: Blender offsets
    // the title by what the header drew (`panel->labelofs`).
    if !head.controls.is_empty() {
        strip = strip.child(controls(&head.controls, ctx));
    }
    strip = strip.child(
        tagged(BCardPart::Title.flex(ctx).div(ctx), format!("card-title-{name}"))
            .child(SharedString::from(head.title.clone())),
    );
    if !head.preset.is_empty() {
        strip = strip.child(controls(&head.preset, ctx));
    }
    // The grip is a top-level panel's, in a stack it could be reordered in.
    if card.kind == BCardKind::Panel && card.fold.is_some() {
        strip = strip.child(tagged(
            BCardPart::Glyph { dimmed: true }.flex(ctx).div(ctx),
            format!("card-grip-{name}"),
        )
        .child(walk(&BView::icon("grip"), ctx)));
    }
    if let Some(fold) = &card.fold {
        strip = super::fold::gestures(strip, fold, ctx);
    }
    Some(strip.into_any_element())
}

/// Header controls take their own clicks: the header's fold is not fired
/// through them (Blender's header buttons sit over the header's hit area).
fn controls(list: &[BView], ctx: &BRenderCtx) -> AnyElement {
    let run = BCardPart::Controls.flex(ctx).div(ctx).occlude();
    list.iter().fold(run, |run, view| run.child(walk(view, ctx))).into_any_element()
}

/// The focused renderer for one card.
pub struct BCardView<'a>(pub &'a BCard);

pub fn view(card: &BCard, ctx: &BRenderCtx) -> AnyElement {
    match card.kind {
        BCardKind::Strip(kind) => return super::bar::BBar(kind, card).render(ctx),
        // A region column: the body scrolls its list, the sidebar holds its
        // list at a width the user drags.
        BCardKind::Column(BColumn::Body) => return super::body::BBody(&card.body).render(ctx),
        BCardKind::Column(BColumn::Sidebar) => return super::sidebar::BSidebar(&card.body).render(ctx),
        _ => {}
    }
    BCardView(card).render(ctx)
}

impl BCardView<'_> {
    pub fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        draw(self.0, ctx)
    }
}

fn draw(card: &BCard, ctx: &BRenderCtx) -> AnyElement {
    let name = slot(card);
    let headless = card.head.is_none();
    let mut frame = tagged(BCardPart::Frame { kind: card.kind, open: card.fold.as_ref().is_some_and(|f| f.open) }.flex(ctx).div(ctx), format!("card-{name}"));
    if let Some(head) = head(card, &name, ctx) {
        frame = frame.child(head);
    }
    if card.is_open() && !card.body.is_empty() {
        let fold = (card.fold.is_some() && !headless).then_some(matches!(card.kind, BCardKind::Sub { .. }));
        let list = card.body.iter().all(|row| matches!(row, BView::MenuItem(_)));
        let mut body = tagged(BCardPart::Body { headless, fold, list }.flex(ctx).div(ctx), format!("card-content-{name}"));
        let listed = BRenderCtx { listed: card.kind == BCardKind::Section, ..ctx.clone() };
        for (n, row) in card.body.iter().enumerate() {
            // Every read-out that is only words — a label, a `name: value`,
            // a paragraph — takes the row's inset; a control carries its own.
            // Only `Label` did, so the details column's pairs ran to the
            // card's edge while its heading and preview stood in from it.
            let text = matches!(row, BView::Label { .. } | BView::Pair { .. } | BView::Prose { .. });
            // A menu row and a read-out are their own height: a stack of
            // them reads as a list, not rungs a button's height apart.
            let own = matches!(row, BView::MenuItem(_) | BView::Pair { .. });
            body = body.child(
                tagged(BCardPart::Row { text, own }.flex(ctx).div(ctx), format!("card-row-{name}-{n}")).child(walk(row, &listed)),
            );
        }
        frame = frame.child(body);
    }
    if card.is_open() && !card.foot.is_empty() {
        let mut foot = tagged(BCardPart::Foot.flex(ctx).div(ctx), format!("card-foot-{name}"));
        for v in &card.foot {
            foot = foot.child(walk(v, ctx));
        }
        frame = frame.child(foot);
    }
    frame.into_any_element()
}


