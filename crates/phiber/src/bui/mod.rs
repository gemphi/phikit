//! Phiber UI — view layer for the editor.
//!
//! Structure matches the store hierarchy 1-to-1 (single tree under views/):
//!   views/space/   — SpaceView (store/spaces/)   — composes zones, triggers only
//!   views/zone/    — ZoneView (store/zones/)     — loops areas, matches container
//!   views/area/    — AreaView (store/areas/)     — resolves active host
//!   views/host/    — HostView (store/hosts/)     — loops regions
//!   views/region/  — RegionView (store/regions/) — loops panes / flattens parts
//!   views/pane/    — PaneView (store/panes/)     — loops parts
//!   views/part/    — PartView dispatcher (store/parts/)
//!   part/          — concrete part views (the actual rendering)
//!   core/          — atomic rendering widgets
//!   modal.rs       — ModalView (store/modals/)

pub mod bview;
pub mod bview_debug;
pub mod card;
pub mod core;
pub mod icon;
pub mod palette;
pub mod render;
pub mod state;
pub mod theme;
pub mod view;

pub mod modal;
pub mod part;

// Unified view tree — one BViewBuilder trait behind every level.
pub mod views;

pub use bview::{ActionHandler, BAction, BView, CardShade};
pub use state::{BKey, BState};
pub use theme::BPalette;

pub use modal::ModalView;
pub use views::{
    AreaView, BViewBuilder, HostView, PaneView, PartView, RegionView, SpaceView, ZoneView,
};

use gpui::{App, Context, IntoElement, Render, Window};
use std::rc::Rc;

impl Render for BView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let p = BPalette::blender_dark();
        let noop: ActionHandler = Rc::new(|_a: &BAction, _w: &mut Window, _cx: &mut App| {});
        self.into_elm(&p, noop)
    }
}
