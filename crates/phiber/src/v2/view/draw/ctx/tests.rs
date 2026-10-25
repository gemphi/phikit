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
