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
use crate::v2::theme::{BStyleRole, BSurfaceTone};
use crate::v2::view::BView;
use crate::v2::view::prim::{BCard, BCardKind, BColumn};
use gpui::prelude::*;
use gpui::{AnyElement, SharedString};

enum BCardPart {
    /// The card: backdrop and outline, per kind.
    Frame(BCardKind),
    /// The heading strip; `open` decides which corners it rounds.
    Head { kind: BCardKind, open: bool },
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
    Body { headless: bool },
    /// One row; bare text gets a row's own inset so it lines up with rows
    /// that carry theirs.
    Row { text: bool },
    /// The strip under the body — a dialog's actions.
    Foot,
}

fn corner(ctx: &BRenderCtx) -> f32 {
    ctx.style.radius_for(BStyleRole::Card)
}

impl BFlexed for BCardPart {
    fn flex(&self, ctx: &BRenderCtx) -> BFlex {
        match self {
            Self::Frame(kind) => {
                let column = BFlex::column().width(BLen::Full);
                match kind {
                    // No clipping: a dropdown opened from a row has to escape
                    // the card (`file_browser_accordion_refinement`).
                    BCardKind::Panel => column
                        .radius(BRadius::Role(BStyleRole::Card))
                        .border(BEdge::All)
                        .fill(BFill::Tone(panel_tone(ctx))),
                    // A sub-panel is a branch of its panel's tree: indented
                    // under its parent's rows with a guide down its left
                    // edge, as an Outliner child row is, on the panel's own
                    // shade (`BSurfaceTone::Nested` is the card's).
                    BCardKind::Sub { depth } => column
                        .margin_left(BSpace::PadSm)
                        .border(BEdge::Guide)
                        .radius(BRadius::Role(BStyleRole::Card))
                        .fill(BFill::Tone(BSurfaceTone::Nested(*depth))),
                    BCardKind::Bare | BCardKind::Strip(_) | BCardKind::Column(_) => column,
                }
            }
            Self::Head { kind, open } => {
                let strip = BFlex::row()
                    .width(BLen::Full)
                    .height(BLen::Dim(super::flex::BDim::CardHead))
                    .align(BAlign::Center)
                    .gap(BSpace::GapSm)
                    .pad_x(BSpace::PadMd)
                    .type_size(BType::Body)
                    .ink();
                let rounded = if *open { BRadius::Top(corner(ctx)) } else { BRadius::Role(BStyleRole::Card) };
                match kind {
                    // A panel's heading lifts a step off its card
                    // (`heading_surface`). Blender paints only a top-level
                    // header (`panel_draw_aligned_backdrop`); the approved
                    // mock-ups lift a sub-panel's too, off its own well.
                    BCardKind::Panel => strip.fill(BFill::Heading(BSurfaceTone::Card)).radius(rounded),
                    BCardKind::Sub { depth } => {
                        strip.fill(BFill::Heading(BSurfaceTone::Nested(*depth))).radius(rounded)
                    }
                    BCardKind::Bare | BCardKind::Strip(_) | BCardKind::Column(_) => strip,
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
            Self::Body { headless } => {
                let column = BFlex::column().width(BLen::Full).pad_x(BSpace::CardInset).gap(BSpace::GapXs);
                if *headless { column } else { column.pad_y(BSpace::PadXs) }
            }
            // A row is the card-head module tall at least, its content
            // centred, so a button in it has room over and under it.
            Self::Row { text } => {
                let row = BFlex::row().width(BLen::Full).min_height(BLen::Dim(super::flex::BDim::CardHead)).align(BAlign::Center);
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
/// `#303030` region).
fn panel_tone(_ctx: &BRenderCtx) -> BSurfaceTone {
    BSurfaceTone::Card
}

/// The name a card's selectors are built from: its fold key, its title, or
/// — for a headless card — the first words of its first row.
fn slot(card: &BCard) -> String {
    let words = match (&card.fold, card.title()) {
        (Some(fold), _) => return fold.key.encode().replace('.', "-"),
        (None, Some(title)) if !title.is_empty() => title.to_string(),
        _ => card.body.first().map(first_words).unwrap_or_default(),
    };
    if words.is_empty() { "card".to_string() } else { words.to_lowercase().replace(' ', "-") }
}

fn first_words(v: &BView) -> String {
    match v {
        BView::Label { text } => text.clone(),
        BView::Button(ctl) => ctl.label.clone().unwrap_or_default(),
        BView::Icon { name } => name.clone(),
        BView::Row { list } | BView::Column { list } | BView::Split { list } => {
            list.first().map(first_words).unwrap_or_default()
        }
        // A region column is named by what it holds, as its panel role was.
        BView::Card(card) if matches!(card.kind, BCardKind::Column(_)) => {
            card.body.first().map(first_words).unwrap_or_default()
        }
        BView::ListRow { list, .. } => list.first().map(first_words).unwrap_or_default(),
        _ => String::new(),
    }
}

fn tagged(element: gpui::Div, id: String) -> gpui::Stateful<gpui::Div> {
    let selector = id.clone();
    element.id(SharedString::from(id)).debug_selector(move || selector.clone())
}

fn head(card: &BCard, name: &str, ctx: &BRenderCtx) -> Option<AnyElement> {
    let head = card.head.as_ref()?;
    let open = card.is_open();
    let mut strip = tagged(BCardPart::Head { kind: card.kind, open }.flex(ctx).div(ctx), format!("card-head-{name}"));
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
    let mut frame = tagged(BCardPart::Frame(card.kind).flex(ctx).div(ctx), format!("card-{name}"));
    if let Some(head) = head(card, &name, ctx) {
        frame = frame.child(head);
    }
    if card.is_open() && !card.body.is_empty() {
        let mut body = tagged(BCardPart::Body { headless }.flex(ctx).div(ctx), format!("card-content-{name}"));
        for (n, row) in card.body.iter().enumerate() {
            let text = matches!(row, BView::Label { .. });
            body = body.child(
                tagged(BCardPart::Row { text }.flex(ctx).div(ctx), format!("card-row-{name}-{n}")).child(walk(row, ctx)),
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

