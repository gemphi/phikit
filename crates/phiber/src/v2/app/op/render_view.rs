//! The Render menu's other three: `render.view_show` (F11) shows the render
//! window with the last result, or puts it away; `render.play_rendered_anim`
//! (Ctrl F11) plays the frames an animation render wrote (`scene::
//! render_play`); `sound.mixdown` (Render Audio…) writes the scene's audio
//! once the file browser names the file (`scene::mixdown`).

use super::super::BPhiberApp;
use crate::v2::part::text::info::{self, BReportLevel};
use crate::v2::scene::mixdown::{BMixdown, BMixdownMade};
use crate::v2::scene::render_job;
use crate::v2::scene::render_play::BRenderedPlayback;
use crate::v2::state::{CtxKey, ModalKey, log, val::{V_EMPTY, V_MODAL_RENDER}};
use gpui::{AppContext as _, Context};

impl BPhiberApp {
    /// Render ▸ Lock Interface (`use_lock_interface`), while a render runs
    /// (`WM_set_locked_interface`): whether `action` is held back — one
    /// that would change the document (`undo::snapshot`), as an operator
    /// without `OPTYPE_LOCK_BYPASS` does not run
    /// (`wm_operator_check_locked_interface`).
    pub fn interface_locked_against(&self, action: &crate::v2::view::prim::BAction) -> bool {
        let locked = self.state.get(crate::v2::state::ScreenKey::LockInterface) == Some(crate::v2::state::val::V_TRUE) && render_job::rendering();
        if !locked {
            return false;
        }
        let mut trial = self.state.clone();
        crate::v2::state::action::apply_action(&mut trial, action);
        super::super::undo::snapshot(&trial) != super::super::undo::snapshot(&self.state)
    }

    /// F11 — `render.view_show`: the render window with whatever the last
    /// render left, no new render; pressed on the open window, it closes.
    pub(super) fn render_view(&mut self) {
        if self.state.get(ModalKey::Open) == Some(V_MODAL_RENDER) {
            self.state.set(ModalKey::Open, V_EMPTY);
        } else {
            self.state.set(ModalKey::Open, V_MODAL_RENDER);
        }
    }

    /// Ctrl F11 — `render.play_rendered_anim`: the rendered frames played
    /// in the render window at the scene's frame rate, round and round
    /// until the window closes; a movie output is handed to the system's
    /// player. The first frame missing is Blender's warning.
    pub(super) fn render_play(&mut self, cx: &mut Context<Self>) {
        let start = crate::v2::state::key::int_or(&self.state, CtxKey::FrameStart, 1);
        let end = crate::v2::state::key::int_or(&self.state, CtxKey::FrameEnd, 250).max(start);
        let fps = crate::v2::state::key::float_or(&self.state, CtxKey::Fps, 24.0);
        let output = crate::v2::scene::output_set(&self.state);
        let blend = self.state.get(&crate::v2::scene::loaded_path_key()).unwrap_or_default().to_string();
        let folder = std::path::Path::new(&blend).parent().map(std::path::Path::to_path_buf).unwrap_or_default();
        let movie = crate::v2::scene::output_set::wants_movie(&output).then(|| {
            let still = crate::v2::scene::output_set::as_still(&output).file_of(&folder);
            let set = crate::v2::scene::output_set::movie_set(&self.state, self.scene_cache.as_ref().map(|(_, m)| m.render.movie).unwrap_or_default());
            set.file_of(&still, start, end)
        });
        let plan = match BRenderedPlayback::plan(&crate::v2::scene::output_set::as_still(&output), &folder, movie, start, end, fps) {
            Ok(plan) => plan,
            Err(why) => {
                info::push(&mut self.state, BReportLevel::Warning, why);
                return;
            }
        };
        let pause = plan.frame_time();
        match plan {
            BRenderedPlayback::Movie(file) => {
                cx.open_url(&format!("file://{}", file.to_string_lossy().replace('\\', "/")));
                log::push(format!("Playing '{}' in the system's player", file.display()));
            }
            BRenderedPlayback::Frames { files, .. } => {
                if render_job::rendering() || !BRenderedPlayback::begin() {
                    log::push("Already playing".to_string());
                    return;
                }
                self.state.set(ModalKey::Open, V_MODAL_RENDER);
                log::push(format!("Playing {} rendered frame(s)", files.len()));
                let (kept, _) = render_job::shot();
                cx.spawn(async move |this, cx| {
                    'play: loop {
                        for file in &files {
                            let open = this.read_with(cx, |app, _| app.state.get(ModalKey::Open) == Some(V_MODAL_RENDER)).unwrap_or(false);
                            if !open || render_job::rendering() {
                                break 'play;
                            }
                            if let Some(shot) = BRenderedPlayback::load(file) {
                                if !render_job::show(shot) {
                                    break 'play;
                                }
                                let _ = this.update(cx, |_, cx| cx.notify());
                            }
                            cx.background_executor().timer(pause).await;
                        }
                    }
                    // The Render Result back where the frames were.
                    if !render_job::rendering() {
                        match kept {
                            Some(shot) => {
                                render_job::show(shot);
                            }
                            None => render_job::forget(),
                        }
                    }
                    BRenderedPlayback::end();
                    let _ = this.update(cx, |_, cx| cx.notify());
                })
                .detach();
            }
        }
    }

    /// The mixdown the browser named: the range's audio — silence, for a
    /// scene without sound — in Blender's default FLAC, or a WAV where
    /// there is no `ffmpeg` to write one.
    /// The encode runs on the pool and reports when it lands: `ffmpeg`
    /// takes as long as the range is long, and this ran on the UI thread
    /// until it moved (`rendering.md` §49).
    pub(super) fn mixdown_to(&mut self, path: &str, cx: &mut Context<Self>) {
        let start = crate::v2::state::key::int_or(&self.state, CtxKey::FrameStart, 1);
        let end = crate::v2::state::key::int_or(&self.state, CtxKey::FrameEnd, 250);
        let fps = crate::v2::state::key::float_or(&self.state, CtxKey::Fps, 24.0);
        let (mix, file) = (BMixdown::of_range(start, end, fps), std::path::PathBuf::from(path));
        cx.spawn(async move |this, cx| {
            let made = cx.background_spawn(async move { mix.write(&file) }).await;
            let _ = this.update(cx, |app, cx| {
                match made {
                    Ok(BMixdownMade::Ffmpeg(file)) => info::push(&mut app.state, BReportLevel::Info, format!("Saved: '{}'", file.display())),
                    Ok(BMixdownMade::Wav(file)) => info::push(&mut app.state, BReportLevel::Warning, format!("ffmpeg is not installed: saved a WAV instead, '{}'", file.display())),
                    Err(why) => info::push(&mut app.state, BReportLevel::Error, format!("Cannot write the mixdown: {why}")),
                }
                cx.notify();
            });
        })
        .detach();
    }
}
