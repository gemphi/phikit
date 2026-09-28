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
/// The app applies `area::size::apply_delta` and re-renders. This is a
/// dedicated seam because one drag writes two fraction keys atomically.
pub type BResizeHandler =
    Rc<dyn Fn(BSpaceType, BZoneType, usize, f32, f32, &mut Window, &mut App) + 'static>;

#[derive(Clone)]
pub struct BRenderCtx<'a> {
    /// The complete immutable style snapshot resolved for this render pass.
    pub style: &'a BResolvedUiStyle,
    pub state: &'a BState,
    pub handler: BActionHandler,
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
    /// True inside chrome that floats over content rather than docking
    /// against it — the viewport's toolbar and sidebar. Floating chrome
    /// connects to nothing, so its controls round on every side and carry a
    /// resting fill to stay legible over the canvas. Docked chrome does
    /// neither: its caps run into the pane they face, and a resting fill
    /// would turn the strip into a row of pills.
    pub floating: bool,
    /// True inside a chrome strip — a header, a tool/path bar or a footer.
    /// A region sets it for a host; a `BPanelRole` strip sets it for a modal,
    /// which composes panels rather than regions. Without it a modal's
    /// controls fell outside the strip treatment and rested on nothing while
    /// the identical control in a host rested on the strip fill.
    pub strip: bool,
    /// True in a slot that hands its whole width to one control — a property
    /// row's value column. A dropdown there is the column: it lines up with
    /// the label opposite it and with every other value in the stack, instead
    /// of hugging its own longest option.
    pub fill: bool,
    /// The area being drawn, set by the area walker: what tells two areas
    /// holding the same host apart (a header's pan, `region::pan`).
    pub area: Option<crate::v2::area::BAreaType>,
}

impl<'a> BRenderCtx<'a> {
    pub fn new(
        style: &'a BResolvedUiStyle,
        state: &'a BState,
        handler: BActionHandler,
        resize: BResizeHandler,
    ) -> Self {
        Self {
            style,
            state,
            handler,
            resize,
            host: BHostType::Empty,
            region: None,
            corners: BCorners::NONE,
            strip: false,
            floating: false,
            fill: false,
            area: None,
        }
    }

    /// Derive a context inside area `area`.
    pub fn in_area(&self, area: crate::v2::area::BAreaType) -> Self {
        Self { area: Some(area), ..self.clone() }
    }

    /// Derive a context for a slot whose single control takes the full width.
    pub fn filling(&self) -> Self {
        Self { fill: true, ..self.clone() }
    }

    pub fn with_host(&self, host: BHostType) -> Self {
        Self {
            style: self.style,
            state: self.state,
            handler: self.handler.clone(),
            resize: self.resize.clone(),
            host,
            region: self.region,
            corners: BCorners::NONE,
            strip: self.strip,
            floating: self.floating,
            fill: self.fill,
            area: self.area,
        }
    }

    /// Derive a context tagged with the enclosing region kind. Set by the
    /// region walker before it walks its pane children.
    pub fn with_region(&self, region: BRegionType) -> Self {
        Self {
            style: self.style,
            state: self.state,
            handler: self.handler.clone(),
            resize: self.resize.clone(),
            host: self.host,
            region: Some(region),
            corners: self.corners,
            strip: self.strip || strip_region(region, self.host),
            floating: self.floating,
            fill: self.fill,
            area: self.area,
        }
    }

    /// A popover's body: it floats over the region its anchor stands in, so
    /// it belongs to none — a dropdown in a header's menu is a free control,
    /// not one of the header's caps (`BCapSide::of`).
    pub fn in_popover(&self) -> Self {
        let mut ctx = self.clone();
        ctx.region = None;
        ctx
    }

    /// Derive a context inside a `BPanelRole` chrome strip, for modal bodies
    /// that compose panels instead of regions.
    pub fn in_strip(&self) -> Self {
        Self { strip: true, ..self.clone() }
    }

    /// Derive a context for chrome that floats over content instead of
    /// docking against it.
    pub fn floating(&self) -> Self {
        Self { strip: true, floating: true, ..self.clone() }
    }

    /// A region's panels drawn inside floating chrome — the N panel beside
    /// the viewport's floating tab strip: docked panels, as Properties'
    /// are, not strip caps.
    pub fn content_of(&self, region: BRegionType) -> Self {
        Self { region: Some(region), corners: BCorners::NONE, strip: false, floating: false, ..self.clone() }
    }

    pub fn with_corners(&self, corners: BCorners) -> Self {
        Self {
            style: self.style,
            state: self.state,
            handler: self.handler.clone(),
            resize: self.resize.clone(),
            host: self.host,
            region: self.region,
            corners,
            strip: self.strip,
            floating: self.floating,
            fill: self.fill,
            area: self.area,
        }
    }
}

/// Regions whose contents follow the chrome-strip treatment.
fn strip_region(region: BRegionType, host: crate::v2::host::BHostType) -> bool {
    match region {
        BRegionType::Header | BRegionType::ToolHeader | BRegionType::Footer | BRegionType::Execute => true,
        // A tool rail is a strip; a tools column of panels (the file
        // browser's source list) is content.
        BRegionType::Tools => !host.tools_hold_panels(),
        _ => false,
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
        let palette_name = BPaletteName::Graphite;
        let style = resolve_ui_style(ui_style_defaults(palette_name), &state);
        let handler: BActionHandler = Rc::new(|_, _, _| {});
        let resize: BResizeHandler = Rc::new(|_, _, _, _, _, _, _| {});
        let context = BRenderCtx::new(&style, &state, handler, resize);
        let host_context = context.with_host(BHostType::View3D);
        let corners = BCorners::row(true, false);
        let corner_context = host_context.with_corners(corners);

        assert!(std::ptr::eq(context.style, host_context.style));
        assert!(std::ptr::eq(host_context.style, corner_context.style));
        assert!(std::ptr::eq(context.state, corner_context.state));
        assert!(Rc::ptr_eq(&context.handler, &corner_context.handler));
        assert!(Rc::ptr_eq(&context.resize, &corner_context.resize));
        assert_eq!(host_context.host, BHostType::View3D);
        assert_eq!(host_context.corners, BCorners::NONE);
        assert_eq!(corner_context.host, BHostType::View3D);
        assert_eq!(corner_context.corners, corners);
    }
}
