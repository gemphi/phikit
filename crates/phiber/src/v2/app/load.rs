//! `.blend` open: spawn the phiphx reader off the UI thread, forward its
//! real progress into `scene.load_progress` / `scene.load_stage`, honour the
//! Cancel key, and settle the outcome (`import::settle_import`).
//! Blender ref: `WM_file_read` (`wm_files.cc`) — Phiber reads in the
//! background instead of blocking behind a wait cursor.

use std::path::Path;
use crate::phiphx::Phx;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

use super::BPhiberApp;
use super::import::{ImportOutcome, settle_import};
use crate::phiphx::scene::{
    SceneLoadError, SceneModel,
};
use crate::v2::scene::{load_cancel_key, load_progress_key, load_stage_key, loaded_path_key, loading_key};
use gpui::{AppContext, Context};

/// How often the UI thread drains reader progress into state. Blender's job
/// progress redraws on its timer at a similar cadence.
const PROGRESS_POLL_MS: u64 = 33;

/// Messages from the reader thread to the UI poll loop.
enum ImportMsg {
    Progress(f32, String),
    Done(Result<SceneModel, SceneLoadError>),
}

impl BPhiberApp {
    /// Begin importing `path` (caller has already checked `should_import`).
    pub(crate) fn start_import(&mut self, path: String, cx: &mut Context<Self>) {
        // Loading flag + first readout show on this render pass. From
        // here on the *reader* drives the bar: `phiphx` reports real
        // fractions (bytes indexed, objects read, scene built) through
        // an mpsc channel that the poll loop below forwards into state.
        self.state.set(&loading_key(), "true");
        self.state.set(&load_stage_key(), "Reading file");
        self.state.set(&load_progress_key(), "0");
        self.state.remove(&load_cancel_key());

        cx.spawn(async move |this, cx| {
            let (tx, rx) = mpsc::channel::<ImportMsg>();
            let cancel = Arc::new(AtomicBool::new(false));

            // Reader thread. No watchdog: Blender never times out a
            // file read; the user cancels via the progress bar instead.
            {
                let load_path = path.clone();
                let tx = Mutex::new(tx);
                let cancel = cancel.clone();
                cx.background_spawn(async move {
                    let sink = |fraction: f32, stage: &str| -> bool {
                        if let Ok(tx) = tx.lock() {
                            let _ = tx.send(ImportMsg::Progress(fraction, stage.to_string()));
                        }
                        !cancel.load(Ordering::Relaxed)
                    };
                    let result =
                        Phx::open_with(Path::new(&load_path), &sink);
                    if let Ok(tx) = tx.lock() {
                        let _ = tx.send(ImportMsg::Done(result));
                    }
                })
                .detach();
            }

            // UI-side poll loop (~30 Hz): forward the latest progress into
            // state, mirror the Cancel key into the reader's flag, stop
            // when the reader reports Done.
            let outcome = loop {
                let mut latest: Option<(f32, String)> = None;
                let mut done: Option<Result<SceneModel, SceneLoadError>> = None;
                loop {
                    match rx.try_recv() {
                        Ok(ImportMsg::Progress(f, s)) => latest = Some((f, s)),
                        Ok(ImportMsg::Done(r)) => done = Some(r),
                        Err(mpsc::TryRecvError::Empty) => break,
                        Err(mpsc::TryRecvError::Disconnected) => {
                            if done.is_none() {
                                done = Some(Err(SceneLoadError::Malformed(
                                    "reader thread ended without a result".to_string(),
                                )));
                            }
                            break;
                        }
                    }
                }
                let cancel_flag = cancel.clone();
                let _ = this.update(cx, |this, cx| {
                    if let Some((f, stage)) = latest {
                        this.state
                            .set(&load_progress_key(), format!("{:.3}", f.clamp(0.0, 1.0)));
                        this.state.set(&load_stage_key(), stage);
                        cx.notify();
                    }
                    if this.state.get(&load_cancel_key()) == Some("true") {
                        cancel_flag.store(true, Ordering::Relaxed);
                    }
                });
                if let Some(result) = done {
                    break match result {
                        Ok(model) => ImportOutcome::Loaded(model),
                        Err(err) => ImportOutcome::Failed(err),
                    };
                }
                cx.background_executor()
                    .timer(Duration::from_millis(PROGRESS_POLL_MS))
                    .await;
            };

            // Back on the UI thread: settle the scalar scene keys, cache the
            // model, then notify so the Viewport/Outliner re-render from the
            // new Scene_Source.
            this.update(cx, |this, cx| {
                if let Some(model) = settle_import(&mut this.state, outcome) {
                    crate::v2::state::log::push(format!(
                        "Opened {} ({}): {} objects, {} vertices, {} faces in {} ms",
                        model.source_path.as_deref().map_or("scene", |p| p.rsplit(['/', '\\']).next().unwrap_or(p)),
                        model.report.scene,
                        model.report.objects,
                        model.report.verts,
                        model.report.faces,
                        model.report.duration_ms
                    ));
                    let key = this
                        .state
                        .get(&loaded_path_key())
                        .map(str::to_string)
                        .or_else(|| model.source_path.clone())
                        .unwrap_or(path);
                    this.scene_cache = Some((key, std::sync::Arc::new(model)));
                    super::op::close::tag_saved(&mut this.state);
                    // A startup file is untitled: no path to save over, and
                    // none in the recent list.
                    if this.untitled {
                        this.untitled = false;
                        this.state.remove(&loaded_path_key());
                    }
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }
}
