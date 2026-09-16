//! `BPhiberApp` — the V2 GPUI entity. Owns `BState`, builds the tree from
//! `BSpaceList`, and hands it to `BGpuiRenderer` with an action handler
//! that calls back into `handle_action` to mutate state.

pub mod handle;
pub mod import;

use crate::phiphx::services::scene::SceneModel;
use crate::v2::space::BSpaceList;
use crate::v2::state::{BState, default_state};
use crate::v2::view::render::gpui::BActionHandler;
use crate::v2::view::render::gpui::space::ZoneDrag;
use crate::v2::view::render::gpui::zone::AreaDrag;
use crate::v2::view::render::{BGpuiRenderer, BRenderer};
use crate::v2::view::{BView, BViewList};
use gpui::{
    Context, DragMoveEvent, InteractiveElement, IntoElement, ParentElement, Render, SharedString,
    Styled, Window, div,
};
use std::rc::Rc;

pub struct BPhiberApp {
    pub state: BState,
    /// The most recent successfully-imported scene, keyed by the `.blend`
    /// path it was loaded from (`loaded_path`). The Scene_Source
    /// (`v2::scene::current_scene`) reads this so the Viewport and Outliner
    /// render the loaded objects; it is threaded through the render pass in
    /// task 12.1. The heavy vertex/index buffers live here rather than being
    /// stringified into `BState`, keeping state writes cheap and the settle
    /// logic pure. `None` until the first successful import.
    pub scene_cache: Option<(String, SceneModel)>,
}

impl BPhiberApp {
    pub fn new() -> Self {
        let mut state = default_state();
        Self::seed_asset_browser_path(&mut state);
        Self {
            state,
            scene_cache: None,
        }
    }

    /// Boot-time seed: point the File_Browser at whichever example-asset
    /// directory we can resolve at launch, so a fresh user sees blend files
    /// immediately without having to navigate from `$HOME`.
    ///
    /// Resolution order (first hit wins):
    ///   1. `PHIBER_ASSETS` env var (explicit user override for demos/tests).
    ///   2. `<workspace>/crates/phiber/assets/CW-Fashion` if it exists —
    ///      the checked-in production `.blend` fixture set.
    ///   3. `<workspace>/crates/phiber/assets` — top-level assets folder.
    ///   4. Fall through: leave the key unset so `current_dir` falls back to
    ///      `$HOME`/CWD as before.
    fn seed_asset_browser_path(state: &mut BState) {
        use crate::v2::part::custom::file::keys::k_path;
        if let Ok(explicit) = std::env::var("PHIBER_ASSETS") {
            if !explicit.is_empty() && std::path::Path::new(&explicit).is_dir() {
                state.set(&k_path(), explicit);
                return;
            }
        }
        let Ok(cwd) = std::env::current_dir() else {
            return;
        };
        // Walk upward looking for the phiace workspace root (has a Cargo.toml
        // whose sibling is `crates/phiber/assets`). This lets `cargo run` work
        // whether invoked from the workspace root or a nested directory.
        let mut cursor = cwd.as_path();
        loop {
            for candidate in [
                cursor.join("crates/phiber/assets/CW-Fashion"),
                cursor.join("crates/phiber/assets"),
                cursor.join("assets/CW-Fashion"),
                cursor.join("assets"),
            ] {
                if candidate.is_dir() {
                    state.set(&k_path(), candidate.display().to_string());
                    return;
                }
            }
            match cursor.parent() {
                Some(parent) if parent != cursor => cursor = parent,
                _ => return,
            }
        }
    }
}

impl Default for BPhiberApp {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for BPhiberApp {
    fn render(&mut self, _w: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Scene_Source scope (task 12.1): thread the cached loaded `SceneModel`
        // (keyed by `loaded_path`) into the render-pass-scoped context BEFORE
        // building the tree, so both consumers derive from the same value in
        // one pass. The Outliner reads it during `BSpaceList::items(&state)`
        // (the tree build below) and the Viewport reads it during the renderer
        // walk (`BGpuiRenderer::render`), both of which run synchronously in
        // this method; the scope is cleared at the bottom before returning so
        // it never leaks into the next pass. A new import updates `scene_cache`
        // and calls `cx.notify()`, so the next render re-reads the fresh value
        // and re-renders both consumers (Requirement 6.5).
        crate::v2::scene::set_loaded_scene(self.scene_cache.as_ref().map(|(_, m)| m.clone()));

        let tree = BSpaceList::new().items(&self.state);
        let root = BView::Column { children: tree };
        let handler: BActionHandler = Rc::new(cx.listener(|this, action, _w, cx| {
            this.handle_action(action, cx);
        }));
        // Resize seam: a tuple-event listener adapted to the multi-arg
        // `BResizeHandler` signature the renderer's drag handles call.
        let listener = cx.listener(
            |this,
             args: &(
                crate::v2::space::BSpaceType,
                crate::v2::zone::BZoneType,
                usize,
                f32,
                f32,
            ),
             _w,
             cx| {
                let (space, zone, index, dpx, extent) = *args;
                this.handle_resize(space, zone, index, dpx, extent, cx);
            },
        );
        let resize: crate::v2::view::render::gpui::BResizeHandler =
            Rc::new(move |space, zone, index, dpx, extent, w, cx| {
                listener(&(space, zone, index, dpx, extent), w, cx);
            });
        let rendered = BGpuiRenderer::new(handler, resize).render(&root, &self.state);

        // Root-level area-resize dispatch. A `BResizeHandle` only *starts*
        // the drag; the running pointer-move dispatch lives here so the
        // divider tracks the cursor for the whole gesture even after the
        // pointer leaves the thin handle strip (mirrors Zed's dock resize:
        // `DraggedDock` payload + workspace-level `on_drag_move`).
        let drag_move = cx.listener(
            move |this, e: &DragMoveEvent<AreaDrag>, w: &mut Window, cx: &mut Context<Self>| {
                let drag = e.drag(cx).clone();
                // Zone extent along the variable axis. Prefer the zone's true
                // laid-out size (recorded during prepaint) so the seam tracks
                // the cursor 1:1; fall back to the window dimension only until
                // the first paint has measured the zone (DEFECT 1).
                let ws = w.viewport_size();
                let fallback = if drag.vertical {
                    f32::from(ws.width)
                } else {
                    f32::from(ws.height)
                };
                let extent = drag.extent_or(fallback);
                let total = drag.total(e.event.position);
                this.handle_resize_drag(
                    drag.space,
                    drag.zone,
                    drag.index,
                    drag.anchor_i,
                    drag.anchor_j,
                    total,
                    extent,
                    drag.min_extent,
                    cx,
                );
            },
        );

        // Zone-level seam dispatch — the outer companion to the area
        // resize above. Space-rule: seams around the Center band resize the
        // side columns / bottom strip in absolute pixels (state::zone_size).
        let zone_move = cx.listener(
            move |this, e: &DragMoveEvent<ZoneDrag>, w: &mut Window, cx: &mut Context<Self>| {
                let drag = e.drag(cx).clone();
                let ws = w.viewport_size();
                let extent = if drag.is_column() {
                    f32::from(ws.width)
                } else {
                    f32::from(ws.height)
                };
                let anchor = drag.anchor_size();
                if anchor <= 0.0 {
                    return;
                } // not laid out yet
                let total = drag.total(e.event.position);
                this.handle_zone_drag(
                    drag.space,
                    drag.zone,
                    anchor,
                    total,
                    drag.sign,
                    extent,
                    drag.min_extent,
                    cx,
                );
            },
        );

        // Both consumers have now read the scoped model (the Outliner during
        // `BSpaceList::items` above, the Viewport during `BGpuiRenderer::render`
        // which produced `rendered`). Clear the scope so it does not leak into
        // the next render pass.
        crate::v2::scene::clear_loaded_scene();

        div()
            .id(SharedString::from("phiber-root"))
            .size_full()
            .on_drag_move(drag_move)
            .on_drag_move(zone_move)
            .child(rendered)
    }
}
