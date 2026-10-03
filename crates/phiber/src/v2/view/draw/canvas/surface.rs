//! The opaque base every canvas paints on, and the corners it inherits.

use super::super::ctx::BRenderCtx;
use gpui::{Corners, Div, Pixels, Styled, div, px};

/// The opaque base every canvas paints on.
///
/// A canvas is the surface that reaches the host's corners, so it has to
/// paint them itself: GPUI's content mask is a plain rectangle, so a rounded
/// parent cannot clip a square child for us. Every canvas starts here, which
/// is why the viewport, the node editors and the image editor all stop
/// squaring off the zone they sit in.
pub fn base(ctx: &BRenderCtx) -> Div {
    round(div().flex_1().min_h_0().size_full().relative().overflow_hidden(), ctx)
}

/// The corners this canvas inherits from its host, as a radius per corner.
///
/// A raster needs this as numbers rather than as styling: `paint_image` takes
/// its own `Corners`, and passing `Corners::default()` painted the 3D
/// viewport's frame square over whatever the surface under it had rounded.
/// `overflow_hidden` cannot save it — GPUI's content mask is a plain
/// rectangle — so the pixels have to be rounded where they are drawn.
pub fn corners(ctx: &BRenderCtx) -> Corners<Pixels> {
    use crate::v2::theme::BStyleRole;
    use crate::v2::view::draw::surface::{
        BChildCornerSupport, BSurfaceOpacity, child_surface_plan, compatible_inner_radius,
    };
    let host = ctx.style.radius_for(BStyleRole::Host).min(compatible_inner_radius(
        ctx.style.radius_for(BStyleRole::Area),
        ctx.style.strokes.border,
    ));
    let plan = child_surface_plan(
        ctx.corners,
        BSurfaceOpacity::Opaque,
        BChildCornerSupport::Propagated,
        host,
        0.0,
    );
    if !plan.corners.any() {
        return Corners::default();
    }
    let r = px(ctx.style.radius_for(BStyleRole::Region).min(plan.radius));
    let at = |owned: bool| if owned { r } else { Pixels::ZERO };
    Corners {
        top_left: at(plan.corners.tl),
        top_right: at(plan.corners.tr),
        bottom_left: at(plan.corners.bl),
        bottom_right: at(plan.corners.br),
    }
}

/// Give an element the corners it inherits from its host.
pub fn round<E: Styled>(element: E, ctx: &BRenderCtx) -> E {
    let c = corners(ctx);
    let element = if c.top_left > Pixels::ZERO { element.rounded_tl(c.top_left) } else { element };
    let element = if c.top_right > Pixels::ZERO { element.rounded_tr(c.top_right) } else { element };
    let element = if c.bottom_left > Pixels::ZERO { element.rounded_bl(c.bottom_left) } else { element };
    if c.bottom_right > Pixels::ZERO { element.rounded_br(c.bottom_right) } else { element }
}
