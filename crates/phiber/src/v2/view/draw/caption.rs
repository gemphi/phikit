//! Window controls — minimise, maximise/restore, close.
//!
//! One declarative table and one button treatment, shared by the OS window
//! caption (the native title bar is hidden so the top bar follows the theme)
//! and by the modal title bar. Both places show the same three buttons, so
//! they are built from the same three rows rather than each drawing its own.
//!
//! On Windows the OS handles the caption's clicks through `WindowControlArea`
//! (non-client hit test, including snap layouts; the close reaches
//! `BPhiberApp::may_close`); other platforms get the same actions from
//! `on_click` — the close as File ▸ Quit, so it asks about unsaved changes.

use super::color::to_hsla;
use super::ctx::BRenderCtx;
use super::atom::icon_path;
use crate::v2::state::BKey as _;
use crate::v2::theme::BStyleRole;
use gpui::prelude::*;
use gpui::{AnyElement, Div, SharedString, Stateful, Window, WindowControlArea, div, px, svg};

/// One window control: what it shows, and what it means to the platform.
pub struct BWindowControl {
    pub slug: &'static str,
    /// Shown while the surface is at its normal size.
    pub icon: &'static str,
    /// Shown while it is maximised (only Max differs).
    pub icon_maximized: &'static str,
    pub area: WindowControlArea,
    act: fn(&mut Window),
}

impl BWindowControl {
    /// The icon for the surface's current state.
    pub fn icon(&self, maximized: bool) -> &'static str {
        if maximized { self.icon_maximized } else { self.icon }
    }

    pub fn closes(&self) -> bool {
        matches!(self.area, WindowControlArea::Close)
    }
}

pub const WINDOW_CONTROLS: &[BWindowControl] = &[
    BWindowControl {
        slug: "min",
        icon: "minus",
        icon_maximized: "minus",
        area: WindowControlArea::Min,
        act: |w| w.minimize_window(),
    },
    BWindowControl {
        slug: "max",
        icon: "maximize_2",
        icon_maximized: "minimize_2",
        area: WindowControlArea::Max,
        act: |w| w.zoom_window(),
    },
    BWindowControl {
        slug: "close",
        icon: "x",
        icon_maximized: "x",
        area: WindowControlArea::Close,
        act: |w| w.remove_window(),
    },
];

/// The shared chrome of a window control: one square button, one icon size,
/// one hover — red for close, the standard hover for the rest. Callers attach
/// their own click behaviour.
pub fn control(id: impl Into<SharedString>, icon: &str, closes: bool, ctx: &BRenderCtx) -> Stateful<Div> {
    let id: SharedString = id.into();
    let hover = to_hsla(if closes {
        ctx.style.colors.axis_x
    } else {
        ctx.style.host_item_hover(ctx.host)
    });
    let dbg = id.clone();
    div()
        .id(id)
        .debug_selector(move || dbg.to_string())
        .cursor_pointer()
        .flex()
        .items_center()
        .justify_center()
        .flex_shrink_0()
        .size(px(ctx.style.dimensions.button_height))
        .rounded(px(ctx.style.radius_for(BStyleRole::Button)))
        .hover(move |s| s.bg(hover))
        .child(
            svg()
                .path(icon_path(icon))
                .size(px(ctx.style.dimensions.icon_size_sm))
                .flex_shrink_0()
                .text_color(to_hsla(ctx.style.colors.text_dim)),
        )
}

pub fn view(leading: bool, ctx: &BRenderCtx) -> AnyElement {
    let h = ctx.style.dimensions.header_height;
    if leading {
        // macOS draws the traffic lights here; keep the menu clear of them.
        return div().w(px(h * 3.0)).h_full().flex_shrink_0().into_any_element();
    }
    let mut row = div()
        .flex()
        .flex_row()
        .items_center()
        .h_full()
        .flex_shrink_0()
        .gap(px(ctx.style.spacing.gap_xs));
    for b in WINDOW_CONTROLS {
        let (act, closes, handler) = (b.act, b.closes(), ctx.handler.clone());
        row = row.child(
            control(format!("caption-{}", b.slug), b.icon(false), b.closes(), ctx)
                .window_control_area(b.area)
                .on_click(move |_, window, app| {
                    if cfg!(target_os = "windows") {
                        return;
                    }
                    if closes {
                        (handler)(&crate::v2::view::prim::BAction::Set(crate::v2::state::OpKey::Quit.path(), String::new()), window, app);
                    } else {
                        act(window);
                    }
                }),
        );
    }
    row.into_any_element()
}
