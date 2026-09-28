//! Generated coverage for resolved-style preservation in derived render contexts.
//!
//! Feature: phiber-resolved-ui-style-settings, Property 6
//! **Validates: Requirements 3.2, 3.3**

use phiber::v2::{
    host::BHostType,
    state::BState,
    theme::{BPaletteName, resolve_ui_style, ui_style_defaults},
    view::draw::{BActionHandler, BRenderCtx, BResizeHandler, ctx::BCorners},
};
use proptest::prelude::*;
use std::rc::Rc;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// Property 6: Derived render contexts preserve style.
    ///
    /// Feature: phiber-resolved-ui-style-settings, Property 6
    /// **Validates: Requirements 3.2, 3.3**
    #[test]
    fn derived_render_contexts_preserve_style_identity_and_value(
        palette_index in 0usize..BPaletteName::ALL.len(),
        host_index in 0usize..BHostType::ALL.len(),
        tl in any::<bool>(),
        tr in any::<bool>(),
        bl in any::<bool>(),
        br in any::<bool>(),
    ) {
        let palette_name = BPaletteName::ALL[palette_index];
        let host = BHostType::ALL[host_index];
        let corners = BCorners { tl, tr, bl, br };
        let state = BState::new();
        let style = resolve_ui_style(ui_style_defaults(palette_name), &state);
        let expected_style = style.clone();
        let handler: BActionHandler = Rc::new(|_, _, _| {});
        let resize: BResizeHandler = Rc::new(|_, _, _, _, _, _, _| {});
        let context = BRenderCtx::new(&style, &state, handler, resize);

        let host_context = context.with_host(host);
        let corner_context = context.with_corners(corners);
        let host_then_corner_context = host_context.with_corners(corners);
        let corner_then_host_context = corner_context.with_host(host);

        for derived in [
            &host_context,
            &corner_context,
            &host_then_corner_context,
            &corner_then_host_context,
        ] {
            prop_assert!(
                std::ptr::eq(context.style, derived.style),
                "derived context changed style identity for host {:?} and corners {:?}",
                host,
                corners,
            );
            prop_assert_eq!(
                derived.style,
                &expected_style,
                "derived context changed style value for host {:?} and corners {:?}",
                host,
                corners,
            );
        }

        prop_assert_eq!(host_context.host, host);
        prop_assert_eq!(corner_context.corners, corners);
        prop_assert_eq!(host_then_corner_context.host, host);
        prop_assert_eq!(host_then_corner_context.corners, corners);
        prop_assert_eq!(corner_then_host_context.host, host);
        prop_assert_eq!(corner_then_host_context.corners, BCorners::NONE);
    }
}
