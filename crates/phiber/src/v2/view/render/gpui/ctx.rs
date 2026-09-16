//! `BRenderCtx` — everything walkers need: resolved style, state, handlers,
//! enclosing host identity, and inherited rounded-surface corner ownership.

use crate::v2::host::BHostType;
use crate::v2::region::BRegionType;
use crate::v2::space::BSpaceType;
use crate::v2::state::BState;
use crate::v2::theme::BResolvedUiStyle;
use crate::v2::view::prim::BAction;
use crate::v2::zone::BZoneType;
use gpui::{App, Window};
use std::rc::Rc;

// Preserve the existing context-module path while ownership logic lives in its
// focused, GPUI-independent surface module.
pub use super::surface::BCorners;

pub type BActionHandler = Rc<dyn Fn(&BAction, &mut Window, &mut App) + 'static>;

/// Area-resize drag callback: `(space, zone, boundary_index, dpx, zone_extent)`.
/// The app applies `state::area_resize::apply_delta` and re-renders. This is a
/// dedicated seam because one drag writes two fraction keys atomically.
pub type BResizeHandler =
    Rc<dyn Fn(BSpaceType, BZoneType, usize, f32, f32, &mut Window, &mut App) + 'static>;

#[derive(Clone)]
pub struct BRenderCtx<'a> {
    /// The complete immutable style snapshot resolved for this render pass.
    pub style: &'a BResolvedUiStyle,
    pub s: &'a BState,
    pub h: BActionHandler,
    pub resize: BResizeHandler,
    /// The host being drawn. Descendant walkers consult this for host-scoped
    /// colours and state paths.
    pub host: BHostType,
    /// The enclosing region kind, set by the region walker so descendant
    /// pane / part walkers can tell a header pane (content hugs, laid out
    /// Left / Center / Right) from a content-region pane (Center stretches to
    /// fill). `None` above the region level.
    pub region: Option<BRegionType>,
    /// Corners of the host surface this element is responsible for painting.
    pub corners: BCorners,
}

impl<'a> BRenderCtx<'a> {
    pub fn new(
        style: &'a BResolvedUiStyle,
        s: &'a BState,
        h: BActionHandler,
        resize: BResizeHandler,
    ) -> Self {
        Self {
            style,
            s,
            h,
            resize,
            host: BHostType::Empty,
            region: None,
            corners: BCorners::NONE,
        }
    }

    pub fn with_host(&self, host: BHostType) -> Self {
        Self {
            style: self.style,
            s: self.s,
            h: self.h.clone(),
            resize: self.resize.clone(),
            host,
            region: self.region,
            corners: BCorners::NONE,
        }
    }

    /// Derive a context tagged with the enclosing region kind. Set by the
    /// region walker before it walks its pane children.
    pub fn with_region(&self, region: BRegionType) -> Self {
        Self {
            style: self.style,
            s: self.s,
            h: self.h.clone(),
            resize: self.resize.clone(),
            host: self.host,
            region: Some(region),
            corners: self.corners,
        }
    }

    pub fn with_corners(&self, corners: BCorners) -> Self {
        Self {
            style: self.style,
            s: self.s,
            h: self.h.clone(),
            resize: self.resize.clone(),
            host: self.host,
            region: self.region,
            corners,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BActionHandler, BCorners, BRenderCtx, BResizeHandler};
    use crate::v2::{
        host::BHostType,
        state::BState,
        theme::{BPaletteName, resolve_ui_style, ui_style_defaults},
    };
    use std::rc::Rc;

    #[test]
    fn derived_contexts_preserve_the_resolved_style_and_render_services() {
        let state = BState::new();
        let palette_name = BPaletteName::BlenderDark;
        let style = resolve_ui_style(ui_style_defaults(palette_name), &state);
        let handler: BActionHandler = Rc::new(|_, _, _| {});
        let resize: BResizeHandler = Rc::new(|_, _, _, _, _, _, _| {});
        let context = BRenderCtx::new(&style, &state, handler, resize);
        let host_context = context.with_host(BHostType::View3D);
        let corners = BCorners::row(true, false);
        let corner_context = host_context.with_corners(corners);

        assert!(std::ptr::eq(context.style, host_context.style));
        assert!(std::ptr::eq(host_context.style, corner_context.style));
        assert!(std::ptr::eq(context.s, corner_context.s));
        assert!(Rc::ptr_eq(&context.h, &corner_context.h));
        assert!(Rc::ptr_eq(&context.resize, &corner_context.resize));
        assert_eq!(host_context.host, BHostType::View3D);
        assert_eq!(host_context.corners, BCorners::NONE);
        assert_eq!(corner_context.host, BHostType::View3D);
        assert_eq!(corner_context.corners, corners);
    }
}
