//! The opaque base every canvas paints on, and the corners it inherits.

use super::super::ctx::BRenderCtx;
use gpui::{Div, Styled, div, px};

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

/// Give an element the corners it inherits from its host.
pub fn round<E: Styled>(element: E, ctx: &BRenderCtx) -> E {
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
        return element;
    }
    let r = px(ctx.style.radius_for(BStyleRole::Region).min(plan.radius));
    let element = if plan.corners.tl { element.rounded_tl(r) } else { element };
    let element = if plan.corners.tr { element.rounded_tr(r) } else { element };
    let element = if plan.corners.bl { element.rounded_bl(r) } else { element };
    if plan.corners.br { element.rounded_br(r) } else { element }
}
