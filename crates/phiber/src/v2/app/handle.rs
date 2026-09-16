//! GPUI action dispatch. State transitions are pure and shared with tests;
//! this layer is responsible only for notifying GPUI after a transition.

use std::path::Path;
use std::time::Duration;

use super::BPhiberApp;
use super::import::{ImportOutcome, settle_import, should_import};
use crate::phiphx::services::scene::load_blend_scene;
use crate::v2::scene::{
    LoadStage, load_progress_key, load_stage_key, loaded_path_key, loading_key, selected_key,
};
use crate::v2::space::BSpaceType;
use crate::v2::state::{
    action::apply_action,
    area_resize::{apply_delta, apply_drag_with_min_extent, normalize},
    zone_size::apply_zone_drag_with_min_extent,
};
use crate::v2::view::prim::BAction;
use crate::v2::zone::BZoneType;
use gpui::{AppContext, Context, FutureExt};

/// How long an in-flight `.blend` import may run before the watchdog fires and
/// the import is settled as a timeout (Requirement 8.4). Thirty seconds per the
/// design (§Components section 4).
const IMPORT_WATCHDOG: Duration = Duration::from_secs(30);

impl BPhiberApp {
    pub fn handle_action(&mut self, action: &BAction, cx: &mut Context<Self>) {
        // 1. Apply the pure state transition (unchanged behavior).
        apply_action(&mut self.state, action);

        // 2. Import spawn/settle shell around the pure core. If the just-applied
        //    action moved `file.selected` to a `.blend` path differing from the
        //    path the Scene_Source currently reflects (`scene.loaded_path`), we
        //    kick off exactly one import off the UI thread.
        let selected = self.state.get(&selected_key());
        let prev = self.state.get(&loaded_path_key());
        if should_import(prev, selected) {
            // Own the selected path before we borrow `self` mutably / move into
            // the async closure. `should_import` guarantees `selected` is Some.
            let path = selected
                .expect("should_import implies a selection")
                .to_string();

            // Loading-in-progress flag shows immediately (Requirement 1.3, 8.1),
            // plus the first staged progress readout (Blender WM_progress). The
            // opaque background parse can't call back mid-flight, so the UI
            // thread seeds the "Reading file" stage here and the settle path
            // advances to Done; the statusbar bar animates between them.
            self.state.set(&loading_key(), "true");
            self.state
                .set(&load_stage_key(), LoadStage::Reading.label());
            self.state
                .set(&load_progress_key(), LoadStage::Reading.progress().to_string());

            // Spawn the import. `cx.spawn` hands us a weak handle to this entity
            // plus an `AsyncApp` that can be held across await points. The parse
            // itself (blocking file IO + SDNA walk) runs on the background
            // executor so the UI thread never blocks (Requirement 8.3).
            cx.spawn(async move |this, cx| {
                let load_path = path.clone();
                let load =
                    cx.background_spawn(async move { load_blend_scene(Path::new(&load_path)) });

                // Arm the 30-second import watchdog (Requirement 8.4). `with_timeout`
                // races the load against an `executor.timer(30s)` internally and
                // resolves to whichever settles first: `Ok(load_result)` if the
                // import wins, or `Err(Timeout)` if the watchdog fires. Whichever
                // loses is simply dropped, so there is exactly one settle — no
                // guard against a late clobber is needed.
                let outcome = match load
                    .with_timeout(IMPORT_WATCHDOG, cx.background_executor())
                    .await
                {
                    // Watchdog fired before the import settled -> timeout error
                    // and clear loading (Requirement 8.4, 8.5).
                    Err(_timeout) => ImportOutcome::TimedOut,
                    // Import settled first: map its Result into the outcome.
                    Ok(Ok(model)) => ImportOutcome::Loaded(model),
                    Ok(Err(err)) => ImportOutcome::Failed(err),
                };

                // Back on the UI thread: settle the scalar scene keys, cache the
                // model, then notify so the Viewport/Outliner re-render from the
                // new Scene_Source.
                this.update(cx, |this, cx| {
                    if let Some(model) = settle_import(&mut this.state, outcome) {
                        // Cache keyed by the reflected loaded_path (settle sets
                        // it from the model's source_path when present).
                        let key = this
                            .state
                            .get(&loaded_path_key())
                            .map(str::to_string)
                            .or_else(|| model.source_path.clone())
                            .unwrap_or(path);
                        this.scene_cache = Some((key, model));
                    }
                    cx.notify();
                })
                .ok();
            })
            .detach();
        }

        // 3. Always notify after the synchronous part so the loading flag (and
        //    any other state change) shows on this render pass (as before).
        cx.notify();
    }

    /// Apply an incremental area-resize delta (legacy seam kept for the
    /// `BResizeHandler` callback) and notify only after a valid change.
    pub fn handle_resize(
        &mut self,
        space: BSpaceType,
        zone: BZoneType,
        index: usize,
        dpx: f32,
        extent: f32,
        cx: &mut Context<Self>,
    ) {
        if apply_delta(&mut self.state, space, zone, index, dpx, extent).is_ok() {
            normalize(&mut self.state, space, zone);
            cx.notify();
        }
    }

    /// Apply an *absolute* area-resize drag from a fixed anchor. The
    /// interactive divider uses this so the resize is idempotent in the total
    /// pixel delta — re-dispatching the same move converges to the same
    /// fractions regardless of how many move events gpui emits.
    #[allow(clippy::too_many_arguments)]
    pub fn handle_resize_drag(
        &mut self,
        space: BSpaceType,
        zone: BZoneType,
        index: usize,
        anchor_i: f32,
        anchor_j: f32,
        total_dpx: f32,
        extent: f32,
        min_extent: f32,
        cx: &mut Context<Self>,
    ) {
        if apply_drag_with_min_extent(
            &mut self.state,
            space,
            zone,
            index,
            anchor_i,
            anchor_j,
            total_dpx,
            extent,
            min_extent,
        )
        .is_ok()
        {
            normalize(&mut self.state, space, zone);
            cx.notify();
        }
    }

    /// Apply an absolute zone-seam drag (whole Left/Right column width or
    /// Bottom strip height). `anchor` is the zone size at drag start,
    /// `total` the cursor travel since then, `sign` the direction mapping.
    pub fn handle_zone_drag(
        &mut self,
        space: BSpaceType,
        zone: BZoneType,
        anchor: f32,
        total: f32,
        sign: f32,
        extent: f32,
        min_extent: f32,
        cx: &mut Context<Self>,
    ) {
        if apply_zone_drag_with_min_extent(
            &mut self.state,
            space,
            zone,
            anchor,
            total,
            sign,
            extent,
            min_extent,
        ) {
            cx.notify();
        }
    }
}
