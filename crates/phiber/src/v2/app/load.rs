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
use crate::v2::scene::{LoadStage, load_cancel_key, load_progress_key, load_stage_key, loaded_path_key, loading_key};
use gpui::{AppContext, Context};

/// How often the UI thread drains reader progress into state. Blender's job
/// progress redraws on its timer at a similar cadence.
const PROGRESS_POLL_MS: u64 = 33;

/// The reader's result: the base scene, and the modifier work still to finish
/// (progressive open), or a load error.
type ImportResult = Result<(SceneModel, Option<crate::phiphx::scene::DeferredModifiers>), SceneLoadError>;

/// Messages from the reader thread to the UI poll loop.
enum ImportMsg {
    Progress(f32, String),
    Done(ImportResult),
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
            // Kept for the deferred-modifier swap below (the import consumes
            // `path` when it settles).
            let loaded = path.clone();
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
                    // Progressive: the base scene comes back now; its modifier
                    // work is finished off this thread and swapped in after.
                    let result = Phx::open_progressive(Path::new(&load_path), &sink);
                    if let Ok(tx) = tx.lock() {
                        let _ = tx.send(ImportMsg::Done(result));
                    }
                })
                .detach();
            }

            // UI-side poll loop (~30 Hz): forward the latest progress into
            // state, mirror the Cancel key into the reader's flag, stop
            // when the reader reports Done.
            //
            // The coarse phase a tick falls in drives the loading panel's
            // running log: one timed line per phase, pushed when the phase
            // changes — never the per-tick detail that would flood it.
            let mut last_phase: Option<LoadStage> = None;
            let (outcome, deferred) = loop {
                let mut latest: Option<(f32, String)> = None;
                let mut done: Option<ImportResult> = None;
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
                        // Each stage once, with its time, in the journal.
                        if this.state.get(&load_stage_key()) != Some(stage.as_str()) {
                            super::journal::BJournal::note(format!("Reading: {stage}"));
                        }
                        // A phase line in the action log when the phase turns
                        // over — the loading panel reads these back, each with
                        // the time since the one before it.
                        let phase = LoadStage::for_fraction(f);
                        if last_phase != Some(phase) {
                            crate::v2::state::log::push(phase.label());
                            last_phase = Some(phase);
                        }
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
                        Ok((model, deferred)) => (ImportOutcome::Loaded(model), deferred),
                        Err(err) => (ImportOutcome::Failed(err), None),
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
                    // Its large meshes' pick trees and outline points, on a
                    // thread, before the first click or orbit asks for them.
                    if let Some((_, model)) = &this.scene_cache {
                        crate::v2::scene::warm::BWarm::of(model).start();
                    }
                    super::op::close::tag_saved(&mut this.state);
                    // Another document: its Render Result is not the old one's.
                    crate::v2::scene::render_job::forget();
                    crate::v2::scene::image_list::BImageList::forget();
                    this.render_on_entry();
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

            // Progressive open, stage two: the modifiers were captured, not
            // applied. Finish them off the UI thread and swap the evaluated
            // meshes into the cached scene — a heavy subdivision or array no
            // longer holds the first paint of the base meshes.
            if let Some(deferred) = deferred.filter(|d| !d.is_empty()) {
                let swaps = cx.background_spawn(async move { deferred.finish() }).await;
                this.update(cx, |this, cx| {
                    let Some((key, model)) = this.scene_cache.take() else { return };
                    // Only swap when the same file is still shown (the user may
                    // have opened another while this evaluated).
                    if key != loaded {
                        this.scene_cache = Some((key, model));
                        return;
                    }
                    let mut scene = std::sync::Arc::try_unwrap(model).unwrap_or_else(|a| (*a).clone());
                    scene.apply_deferred(swaps); // refreshes report.verts/tris to the evaluated meshes
                    // The read summary counted the base meshes; now that the
                    // modifiers have landed, report the evaluated geometry the
                    // viewport and status bar actually show.
                    crate::v2::state::log::push(format!(
                        "Modifiers evaluated: {} vertices, {} tris",
                        scene.report.verts, scene.report.tris
                    ));
                    let scene = std::sync::Arc::new(scene);
                    // Rebuild pick trees and outline points for the new meshes.
                    crate::v2::scene::warm::BWarm::of(&scene).start();
                    this.scene_cache = Some((key, scene));
                    this.render_on_entry();
                    cx.notify();
                })
                .ok();
            }
        })
        .detach();
    }
}
