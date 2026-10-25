//! The modal level's layout: every container a modal is built from, and the
//! one `BFlexed` impl that answers how each is laid out.

use super::super::ctx::BRenderCtx;
use super::super::flex::{BAlign, BDim, BEdge, BFill, BFlex, BFlexed, BJustify, BLen, BOverflow, BPlace, BRadius, BSpace, BType};
use crate::v2::theme::{BElevation, BStyleRole, BSurfaceTone};

/// Every container a modal is built from. One impl answers how each is laid
/// out, so the modal level is described in one place, the way every other
/// level is.
#[derive(Clone, Copy)]
pub enum BModalPart {
    /// Covers the window; the backdrop and the card are layered inside it.
    Layer,
    /// The dim over the workspace; clicking it closes the modal.
    Backdrop,
    /// Centres the card in the window, inset by the large pad.
    Centerer,
    /// A card modal: half the window wide. `fills` when it is maximised;
    /// `tall` (the window's height) when it raises an editor; otherwise the
    /// height its spec declares.
    Card { fills: bool, tall: bool, height: f32 },
    /// A reading sheet: `share` of the window's width, the whole height.
    Sheet { share: f32 },
    /// The search palette: half the window wide, most of its height, on the
    /// floating chrome's see-through ground, bordered and rounded.
    Palette,
    /// Hangs the palette under the top bar, centred across the window.
    Above,
    /// The top bar's height, kept clear above the palette.
    AboveGap,
    /// The card's content. `flush` for bodies that compose their own
    /// edge-to-edge regions (file browser, render) — every other modal keeps
    /// the large inner gutter so form content doesn't butt against the border.
    /// `bounded` for a sheet: a column that gives its page a bounded height
    /// to scroll in, rather than a box that grows to the page's length.
    CardBody { flush: bool, bounded: bool },
    /// A viewport-filling modal (Welcome, About).
    Fullscreen,
    /// A fullscreen modal restored to a window: most of the window, centred,
    /// with a card's border and corners.
    FullscreenWindow,
    /// A fullscreen modal's content.
    FullscreenBody,
    /// The title bar — a header strip across the top of the modal.
    TitleBar { top_radius: Option<f32> },
    /// The icon and label at the head of the title bar.
    TitleGroup,
    /// The window controls at the end of the title bar.
    TitleControls,
}

impl BFlexed for BModalPart {
    fn flex(&self, _c: &BRenderCtx) -> BFlex {
        match *self {
            Self::Layer => BFlex::cell().place(BPlace::Overlay),
            Self::Backdrop => BFlex::cell().place(BPlace::Overlay).full().fill(BFill::ModalDim),
            // A modal keeps the same distance from the window's edge as an
            // area keeps from the next: one seam. Every gap in the system is
            // that seam or a region's pad.
            Self::Centerer => BFlex::row()
                .place(BPlace::Overlay)
                .align(BAlign::Center)
                .justify(BJustify::Center)
                .pad(BSpace::Seam),
            Self::Card { fills, tall, height } => card(fills, tall, height),
            Self::Sheet { share } => card(false, true, 0.0).width(BLen::Share(share)),
            Self::Palette => BFlex::column()
                .width(BLen::Share(0.5))
                .height(BLen::Share(0.6))
                .min_width(BLen::Dim(BDim::SideZoneDefaultWidth))
                .fill(BFill::Floating)
                .ink()
                .border(BEdge::All)
                .radius(BRadius::Role(BStyleRole::Modal))
                .overflow(BOverflow::Clip)
                .occludes()
                .elevated(BElevation::Modal),
            Self::Above => BFlex::column().place(BPlace::Overlay).align(BAlign::Center).pad(BSpace::Seam),
            Self::AboveGap => BFlex::cell().height(BLen::Dim(BDim::HeaderHeight)).keeps_size(),
            Self::CardBody { flush, bounded } => card_body(flush, bounded),
            Self::Fullscreen => BFlex::column()
                .place(BPlace::Overlay)
                .full()
                .fill(BFill::Tone(BSurfaceTone::Modal))
                .ink()
                .occludes(),
            Self::FullscreenWindow => BFlex::column()
                .width(BLen::Share(0.86))
                .height(BLen::Share(0.86))
                .fill(BFill::Tone(BSurfaceTone::Modal))
                .ink()
                .border(BEdge::All)
                .radius(BRadius::Role(BStyleRole::Modal))
                .overflow(BOverflow::Clip)
                .occludes()
                .elevated(BElevation::Modal),
            Self::FullscreenBody => BFlex::cell()
                .fills()
                .full()
                .min_height(BLen::Zero)
                .overflow(BOverflow::Clip)
                .pad(BSpace::Seam),
            Self::TitleBar { top_radius } => title_bar(top_radius),
            // The icon and the title sit as a header's switcher and menus
            // do: one `gap_xs` apart.
            Self::TitleGroup => BFlex::row().align(BAlign::Center).gap(BSpace::GapXs),
            Self::TitleControls => BFlex::row().align(BAlign::Center).gap(BSpace::GapXs),
        }
    }
}

/// How much of the window's width an editor in a dialog takes.
const EDITOR_SHARE: f32 = 2.0 / 3.0;

fn card(fills: bool, tall: bool, height: f32) -> BFlex {
    let card = BFlex::column()
        .fill(BFill::Tone(BSurfaceTone::Modal))
        .ink()
        .border(BEdge::All)
        .radius(BRadius::Role(BStyleRole::Modal))
        // Its header and footer strips are square; the card is not, so it
        // clips them to its corners. A popover opened inside paints in the
        // window-root overlay and escapes anyway.
        .overflow(BOverflow::Auto)
        .occludes()
        .elevated(BElevation::Modal);
    // An editor in a dialog is two thirds of the window wide and the
    // window's height — the file browser is three columns (sources, files,
    // details), and at half the window its files were two to a row; a form
    // — a question, a name, a short list — takes the middle third, never
    // narrower than a side zone, and its own height.
    if fills {
        card.fills().full()
    } else if tall {
        card.width(BLen::Share(EDITOR_SHARE)).height(BLen::Full)
    } else {
        card.width(BLen::Share(1.0 / 3.0)).min_width(BLen::Dim(BDim::SideZoneDefaultWidth)).height(BLen::Measured(height))
    }
}

fn card_body(flush: bool, bounded: bool) -> BFlex {
    let body = if bounded { BFlex::column() } else { BFlex::cell() };
    let body = body.fills().min_height(BLen::Zero).overflow(BOverflow::Clip);
    // A dialog's body is padded like a region's content, not more.
    if flush { body } else { body.pad(BSpace::PadMd) }
}

/// The title bar is a header strip (`region::mod`'s header): a header's
/// height, side padding and gap, the header surface, and a rule under it,
/// so a dialog's heading lines up with the areas' headers around it.
fn title_bar(top_radius: Option<f32>) -> BFlex {
    let bar = BFlex::row()
        .align(BAlign::Center)
        .justify(BJustify::Between)
        .height(BLen::Dim(BDim::HeaderHeight))
        .pad_x(BSpace::PadMd)
        .gap(BSpace::GapXs)
        .fill(BFill::Tone(BSurfaceTone::Header))
        // No rule under it: the header's own shade is the edge. A border
        // here drew a dark line across the top of every modal.
        .type_size(BType::Header);
    match top_radius {
        Some(r) => bar.radius(BRadius::Top(r)),
        None => bar,
    }
}
