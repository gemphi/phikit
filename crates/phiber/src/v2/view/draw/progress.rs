//! Progress-bar atom — a determinate bar with a leading status label.
//!
//! Blender's file-read job paints a thin progress bar in the status bar with
//! a short label; this atom mirrors that. It resolves all geometry and colour
//! from the render-pass style (no design literals): the track uses the recessed
//! panel tone, the fill uses the accent, and the label uses the dim text tone.

use super::ctx::BRenderCtx;
use super::flex::{BAlign, BDim, BFill, BFlex, BFlexed, BInk, BLen, BOverflow, BRadius, BSpace, BType};
use crate::v2::host::BHostType;
use crate::v2::theme::{BStyleRole, BSurfaceTone};
use gpui::prelude::*;
use gpui::{AnyElement, SharedString};

/// Render a `[====      ] label` progress row. `fraction` is clamped to
/// `0.0..=1.0`. The bar is a fixed-width track whose accent fill grows with
/// `fraction`; `relative(fraction)` lets the fill track the resolved track
/// width without a hardcoded pixel span.
///
/// In the **status bar** the bar comes first and the stage label sits in a
/// fixed-width box after it. A stage label changes on every tick — "Reading
/// file", then "Reading objects 431/2104 Cube.003" — so a label that sized
/// itself dragged the bar sideways with every object read. There the bar
/// holds one position for the whole load and only the text inside the box
/// changes.
///
/// Anywhere else — the Opening-File modal — the row has a width of its own to
/// fill, so the bar spans it and the label sits on its own line beneath,
/// also full width. Nothing is boxed to a borrowed dimension there, and the
/// label still ellipsises rather than reflowing the card as the stage changes.
pub fn view(fraction: f32, label: &str, ctx: &BRenderCtx) -> AnyElement {
    let frac = fraction.clamp(0.0, 1.0);
    let in_statusbar = ctx.host == BHostType::Statusbar;

    let track_id = if in_statusbar { "statusbar-progress-track" } else { "modal-progress-track" };
    let track = BProgressPart::Track { boxed: in_statusbar }
        .flex(ctx)
        .div(ctx)
        .id(track_id)
        .debug_selector(move || track_id.to_string())
        .child(BProgressPart::Fill(frac).flex(ctx).div(ctx));

    let stage = BProgressPart::Stage { boxed: in_statusbar }
        .flex(ctx)
        .div(ctx)
        .whitespace_nowrap()
        .text_ellipsis()
        .child(SharedString::from(label.to_string()));

    BProgressPart::Row { boxed: in_statusbar }
        .flex(ctx)
        .div(ctx)
        .child(track)
        .child(stage)
        .into_any_element()
}

/// The parts of a progress row — one enum for the level, as every other
/// level states its parts (`BCardPart`, `BModalPart`).
///
/// `boxed` is the status bar: there the bar holds one position for the whole
/// load and only the text inside the box changes. A stage label changes on
/// every tick — "Reading file", then "Reading objects 431/2104 Cube.003" —
/// so a label that sized itself dragged the bar sideways with every object
/// read. Anywhere else (the Opening-File modal) the row has a width of its
/// own to fill, so the bar spans it and the label sits beneath.
enum BProgressPart {
    /// The track: a fixed width in the status bar, the row's width elsewhere.
    Track { boxed: bool },
    /// The filled part: a share of the track, so it follows the resolved
    /// track width without a pixel span of its own.
    Fill(f32),
    /// The stage label.
    Stage { boxed: bool },
    /// Bar then label in the status bar; bar over label in a modal.
    Row { boxed: bool },
}

impl BFlexed for BProgressPart {
    fn flex(&self, ctx: &BRenderCtx) -> BFlex {
        match *self {
            Self::Track { boxed } => {
                // A track is a well, not a raised surface: the tone's own
                // colour with no sweep, or the sweep would read as fill that
                // is not there. Its width reuses a resolved medium dimension
                // rather than inventing one.
                let track = BFlex::row()
                    .height(BLen::Measured(ctx.style.typography.body))
                    .radius(BRadius::Role(BStyleRole::Button))
                    .overflow(BOverflow::Clip)
                    .fill(BFill::Flat(BSurfaceTone::Body));
                if boxed {
                    track.keeps_size().width(BLen::Dim(BDim::ChannelsWidth))
                } else {
                    track.width(BLen::Full)
                }
            }
            Self::Fill(fraction) => BFlex::row()
                .height(BLen::Full)
                .width(BLen::Share(fraction))
                .fill(BFill::Ink(BInk::Accent)),
            Self::Stage { boxed } => {
                let stage = BFlex::row().overflow(BOverflow::Clip);
                if boxed {
                    stage.keeps_size().width(BLen::Dim(BDim::PanelWidth))
                } else {
                    stage.width(BLen::Full).min_width(BLen::Zero)
                }
            }
            Self::Row { boxed } => {
                let wrap = |f: BFlex| f.gap(BSpace::GapSm).dim_ink().type_size(BType::Body);
                if boxed {
                    wrap(BFlex::row().align(BAlign::Center).keeps_size())
                } else {
                    wrap(
                        BFlex::column()
                            .align(BAlign::Stretch)
                            .width(BLen::Full)
                            .min_width(BLen::Zero),
                    )
                }
            }
        }
    }
}
