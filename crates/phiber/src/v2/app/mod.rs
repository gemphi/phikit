//! `BPhiberApp` — the V2 GPUI entity. Owns `BState`, builds the tree from
//! `BSpaceList`, and hands it to `BGpuiRenderer` with an action handler
//! that calls back into `handle_action` to mutate state.

pub mod handle;
pub mod input;
pub mod keymap;
mod op;
pub mod trace;
pub mod undo;
pub mod import;
pub mod load;
mod playback;
pub mod load_ui;
pub mod health;
pub mod journal;
pub mod session;
pub mod collab;
pub mod seed;

use crate::v2::state::BKey as _;
use crate::v2::state::FileKey;
use crate::phiphx::scene::SceneModel;
use crate::v2::space::BSpaceList;
use crate::v2::state::{BState, default_state};
use crate::v2::view::draw::BActionHandler;
use crate::v2::view::draw::space::ZoneDrag;
use crate::v2::view::draw::zone::AreaDrag;
use crate::v2::view::{draw::BGpuiRenderer, layout::BRenderer};
use crate::v2::view::{BView, BViewList};
use gpui::{
    Context, DragMoveEvent, InteractiveElement, IntoElement, ParentElement, Render, SharedString,
    Styled, Window, div,
};
use std::rc::Rc;

pub struct BPhiberApp {
    pub state: BState,
    /// Keyboard focus for the whole window: typing goes to whichever field
    /// holds the caret (`state::edit`).
    pub(crate) focus: Option<gpui::FocusHandle>,
    /// The caret, selection and IME state for the field being typed into, plus
    /// the geometry the text system needs (`input.rs`, `draw::text_input`).
    pub(crate) input: input::BInput,
    /// The most recent successfully-imported scene, keyed by the `.blend`
    /// path it was loaded from (`loaded_path`). The Scene_Source
    /// (`v2::scene::current_scene`) reads this so the Viewport and Outliner
    /// render the loaded objects; it is threaded through the render pass in
    /// task 12.1. The heavy vertex/index buffers live here rather than being
    /// stringified into `BState`, keeping state writes cheap and the settle
    /// logic pure. `None` until the first successful import.
    /// The scene last read, behind an `Arc`: it is handed to the render-pass
    /// scope on every frame, so it must be shared rather than copied.
    pub scene_cache: Option<(String, std::sync::Arc<SceneModel>)>,
    /// A playback ticker is running (`playback.rs`).
    pub(crate) playing_ticker: bool,
    /// The viewport's image is refining and a task waits for its frames
    /// (`ensure_ray_ticker`).
    pub(crate) ray_ticker: bool,
    /// The undo steps (`undo.rs`).
    pub history: self::undo::BHistory,
    /// The document is a fresh startup file with no path of its own yet
    /// (`wm.read_homefile`): Save asks where, and the read is not recorded.
    pub(crate) untitled: bool,
    /// The "Save changes?" dialog was answered: the action it held runs
    /// once without asking again.
    pub(crate) answered_close: bool,
    /// Quit is closing the window: the platform's "may it close?" is yes.
    pub(crate) closing: bool,
    /// The window's title as last set (`op::close::window_title`).
    pub(crate) title: String,
}

impl BPhiberApp {
    pub fn new() -> Self {
        let mut state = default_state();
        Self::seed_asset_browser_path(&mut state);
        Self {
            state,
            focus: None,
            input: input::BInput::default(),
            scene_cache: None,
            playing_ticker: false,
            ray_ticker: false,
            history: Default::default(),
            untitled: false,
            answered_close: false,
            closing: false,
            title: String::new(),
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
                if let Ok(explicit) = std::env::var("PHIBER_ASSETS") {
            if !explicit.is_empty() && std::path::Path::new(&explicit).is_dir() {
                state.set(&FileKey::Path.path(), explicit);
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
                    state.set(&FileKey::Path.path(), candidate.display().to_string());
                    return;
                }
            }
            match cursor.parent() {
                Some(parent) if parent != cursor => cursor = parent,
                _ => return,
            }
        }
    }

    /// Launch-time: read the persisted session (if any), restore its layout,
    /// and reopen the file it named. Called from `app::run`, never from a test
    /// window — the session file is only written by an armed, running app.
    pub(crate) fn restore_session(&mut self, cx: &mut Context<Self>) {
        if let Some(text) = session::BSession::restore_text() {
            if let Some(path) = self.apply_session(&text) {
                self.start_import(path, cx);
            }
            cx.notify();
        }
    }

    /// Apply a session snapshot's durable layout and, if it named a file that
    /// still exists, arm the reopen with Load UI off (so the file's own saved
    /// workspace and view do not overwrite the restored layout) and return the
    /// path to reopen. The reopen itself is the caller's `start_import`; split
    /// out so the resume decision is testable without a background load.
    pub fn apply_session(&mut self, text: &str) -> Option<String> {
        session::apply(&mut self.state, text);
        let path = session::last_file(text)
            .filter(|p| crate::v2::scene::is_blend_path(p) && std::path::Path::new(p).exists())?;
        self.state.set(FileKey::LoadUi, "false");
        self.state.set(&crate::v2::scene::selected_key(), path.clone());
        // The reopen seeds the file's saved frame; stash the resumed playhead
        // so `settle_import` lands on where we actually were (the session
        // outlives the save). The camera survives on its own — Load UI is off.
        if let Some(frame) = self.state.get(crate::v2::state::CtxKey::Frame).map(str::to_string) {
            self.state.set_encoded(session::RESUME_FRAME, frame);
        }
        Some(path)
    }
}

impl Default for BPhiberApp {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for BPhiberApp {
    fn render(&mut self, w: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
        crate::v2::scene::source::set_loaded_at(self.scene_cache.as_ref().map(|(_, m)| std::sync::Arc::clone(m)), &self.state);

        // Record the window's size so the view — which is built from state
        // alone — can lay out for the room it actually has: the tree reads the
        // width, a menu popover reads the height (it may be as tall as its rows
        // need or as the window allows, whichever is less). Only a real change
        // is written, so this never churns the state between frames.
        let size = w.viewport_size();
        for (key, value) in [
            (crate::v2::state::AppKey::WindowWidth, f32::from(size.width).round()),
            (crate::v2::state::AppKey::WindowHeight, f32::from(size.height).round()),
        ] {
            if self.state.get(key) != Some(value.to_string().as_str()) {
                self.state.set(key, value.to_string());
            }
        }

        // The title bar: the document's name, `* ` while it has unsaved
        // changes (`wm_window_title`), set only when it changes.
        let title = op::close::window_title(&self.state);
        if self.title != title {
            w.set_window_title(&title);
            w.set_window_edited(op::close::has_unsaved_changes(&self.state));
            self.title = title;
        }

        // Undo History's menu lists the app's steps: put this pass's copy
        // where the menu, built from the state alone, can read it.
        undo::publish(&self.history);
        crate::v2::part::outliner::key::BHidden::begin_pass(); // the state holds still until drawn
        let (tree, tree_at) = trace::timed(|| BSpaceList::new().items(&self.state));
        let root = BView::Column { list: tree };
        let handler: BActionHandler = Rc::new(cx.listener(|this, action, w, cx| {
            this.handle_action_in(action, w, cx);
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
        let resize: crate::v2::view::draw::BResizeHandler =
            Rc::new(move |space, zone, index, dpx, extent, w, cx| {
                listener(&(space, zone, index, dpx, extent), w, cx);
            });
        // The ticker is armed from the LAST paint's verdict — whether a
        // viewport left its image still on its way (`rendered::refining`) —
        // so it is asked before `drawing` clears that flag for this draw.
        // Asked after, it read a flag just cleared, and the viewport never
        // refreshed on its own: a session's first image waited for the
        // first click, and so did every refined frame.
        self.ensure_ray_ticker(cx);
        crate::v2::view::draw::canvas::view3d::rendered::drawing();
        // The window entity and its focus thread into the renderer so the field
        // that holds the caret can register the text-input handler for the
        // right focus and draw its caret (`draw::text_input`, `input.rs`).
        let focus = self.focus.get_or_insert_with(|| cx.focus_handle()).clone();
        let input_wiring = (cx.entity(), focus.clone());
        let (rendered, draw_at) = trace::timed(|| BGpuiRenderer::new(handler, resize, Some(input_wiring)).render(&root, &self.state));
        crate::v2::part::outliner::key::BHidden::end_pass();
        // A rig still posing the shown frame on its thread: draw again until it lands.
        if crate::v2::scene::pose::BPoser::waiting() {
            w.request_animation_frame();
        }
        health::BHealth::each_frame();
        journal::BJournal::each_frame();
        // Persist the durable session when it changes (a no-op unless the
        // running app armed it, and when only a rapid family moved).
        session::BSession::persist(&self.state);
        // Apply any collaboration traffic — remote edits and presence (a no-op
        // unless a transport is connected); re-render when it changed something.
        if collab::BCollab::drain(&mut self.state) {
            cx.notify();
        }
        // Every frame is counted (`trace::tick`: the journal's summaries);
        // the scene walk for a frame's own line only when it was slow or
        // the stderr stream is on, so a fast frame pays nothing for it.
        trace::tick(tree_at, draw_at);
        if trace::on() || (tree_at + draw_at).as_secs_f32() * 1000.0 >= trace::SLOW_MS {
            let objects = crate::v2::scene::scene_objects(&self.state);
            let (count, _, tris) = crate::v2::scene::scene_metrics(&objects);
            trace::frame(tree_at, draw_at, count, tris, crate::v2::view::draw::canvas::view3d::on_gpu());
        }

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
        // side columns / bottom strip in absolute pixels (zone::size).
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

        // A sidebar edge being dragged: write the width the pointer is at.
        let panel_move = cx.listener(
            move |this,
                  e: &DragMoveEvent<
                crate::v2::view::draw::resize::PanelDrag,
            >,
                  _w: &mut Window,
                  cx: &mut Context<Self>| {
                let drag = e.drag(cx).clone();
                let scaled = drag.scaled_at(e.event.position);
                this.state.set_encoded(&drag.key, format!("{:.0}", scaled.size));
                this.state.set_encoded(&drag.hidden_key, drag.shut_value(scaled));
                cx.notify();
            },
        );

        // The scope stays set between passes. It used to be cleared here,
        // which left every reader that runs *after* `render` returns — the
        // canvas paint closures (the tracer's world and lamps), the action
        // handler (F12's `render_image`, the Properties refresh on select)
        // — looking at no model at all: the render window opened and nothing
        // rendered. The next pass re-sets it from `scene_cache` anyway, so
        // nothing stale can leak: a new import replaces it, `File ▸ New`
        // clears it (`handle.rs`).

        // Typing: the root holds focus and feeds keys to the edited field
        // (Blender's `ui_textedit_*`: one text button is active at a time).
        // The handle was made above for the renderer; keep the window on it so
        // typing and IME always land here.
        w.focus(&focus, cx);
        let keys = cx.listener(|this, e: &gpui::KeyDownEvent, w: &mut Window, cx: &mut Context<Self>| {
            let k = &e.keystroke;
            let stroke = self::keymap::BKeystroke::of(&k.key, k.key_char.as_deref(), k.modifiers.control, k.modifiers.alt, k.modifiers.shift);
            this.handle_key(&k.key, k.key_char.as_deref(), Some((stroke, w)), cx);
        });
        div()
            .id(SharedString::from("phiber-root"))
            .track_focus(&focus)
            .size_full()
            .on_key_down(keys)
            .on_drag_move(drag_move)
            .on_drag_move(zone_move)
            .on_drag_move(panel_move)
            .child(rendered)
    }
}
