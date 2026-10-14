//! A final render: the scene through the file's own camera, at the file's
//! resolution, with what Properties → Render asks for.
//!
//! Blender ref: `BKE_camera_params_*` (`camera.cc`) for the framing, and
//! `RenderData.xsch / ysch / size` for the resolution F12 uses. The viewport
//! is a different camera — an orbit around a pivot — so this does not reuse
//! `rendered::camera`; what they share is the phiray `Camera` they produce.

use crate::phiphx::scene::{SceneCamera, SceneModel};
use crate::v2::scene::BRenderSet;
use crate::v2::view::draw::canvas::view3d::rendered::{BRayLook, lit};
use glam::Mat4;
use phiray::Camera;

/// The pixels the file asks F12 for: `resolution_[x,y]` scaled by
/// `resolution_percent`, never zero. A render takes the Output panel's
/// (`BResolution`), which starts as these.
pub fn render_size(model: &SceneModel) -> (u32, u32) {
    super::render_set::BResolution::default().pixels(model)
}

/// The camera a final render looks through: `Scene.camera` by name, or the
/// only camera when the scene names none.
pub fn active_camera(model: &SceneModel) -> Option<&SceneCamera> {
    let named = model.cameras.iter().find(|c| c.name == model.render.camera);
    named.or_else(|| model.cameras.first())
}

/// A Blender camera object as phiray sees it.
///
/// phiray puts the screen at `u, v ∈ [-0.5, 0.5]`, takes a view-space point to
/// `(u − pan.x) / scale` across and `−(v / aspect − pan.y) / scale` up, and for
/// perspective places the eye `dist` along `back`. So the half-width the frame
/// spans at the eye's distance is `0.5 / scale`, and matching Blender is a
/// matter of making that subtend the same angle as its sensor does over its
/// focal length:
///
/// ```text
/// tan(fov_x / 2) = (sensor_width / 2) / lens     Blender
///                = (0.5 / scale) / dist          phiray
/// ```
///
/// `dist` only sets where the eye sits along a ray that is normalised
/// afterwards, so any positive value works and the scene's own scale is the
/// natural one. Blender fits the sensor to the **larger** image dimension
/// (`sensor_fit = AUTO`), which is why a portrait frame divides by the aspect.
pub fn camera_of(cam: &SceneCamera, w: u32, h: u32) -> Camera {
    let m = Mat4::from_cols_array_2d(&cam.matrix_world);
    // Blender's camera looks down its own −Z, with +X right and +Y up.
    let right = m.x_axis.truncate().normalize_or_zero();
    let up = m.y_axis.truncate().normalize_or_zero();
    let back = m.z_axis.truncate().normalize_or_zero();
    let eye = m.w_axis.truncate();
    let aspect = w.max(1) as f32 / h.max(1) as f32;
    // AUTO sensor fit: the sensor spans the longer side.
    let wide = w >= h;
    let ortho = cam.kind == "ORTHO";
    let dist = 1.0f32;
    let scale = if ortho {
        // `ortho_scale` is the longer side's span in world units.
        let span = cam.ortho_scale.max(1e-6);
        if wide { 1.0 / span } else { 1.0 / (span * aspect) }
    } else {
        let half = (cam.sensor_width.max(1e-6) * 0.5) / cam.focal_length_mm.max(1e-6);
        let s = 0.5 / (dist * half);
        // The kernel's vertical half-extent is `0.5 / (aspect * scale)`, so
        // spanning the sensor over the *height* divides rather than multiplies.
        if wide { s } else { s / aspect }
    };
    Camera {
        right: right.to_array(),
        up: up.to_array(),
        back: back.to_array(),
        // The kernel puts the eye at `pivot + back * (dist / units)`, so the
        // pivot is the camera's own position pulled back along its axis.
        pivot: (eye - back * dist).to_array(),
        units: 1.0,
        persp: u32::from(!ortho),
        aspect,
        scale,
        dist,
        // Depth of field. `focus` is the distance the camera itself names;
        // a camera that focuses on an *object* is resolved in `plan`, where
        // the other objects are known.
        aperture: cam.dof.aperture(cam.focal_length_mm, ortho),
        focus: cam.dof.focus_distance,
        blades: cam.dof.blades,
        blade_spin: cam.dof.rotation,
        aperture_ratio: cam.dof.ratio,
        ..Default::default()
    }
}

/// Where a camera that focuses on an object is actually focused: the
/// object's distance **along the camera's own axis**, not the straight-line
/// distance to it (`blender_camera_focal_distance`, `cycles/blender/camera.cpp`).
///
/// Returns `None` when the camera focuses by distance, or names an object the
/// scene does not have — in which case the camera's own `focus_distance`
/// stands, as it does in Blender.
pub fn focus_on(model: &SceneModel, cam: &SceneCamera) -> Option<f32> {
    let name = &cam.dof.focus_object;
    if name.is_empty() {
        return None;
    }
    let at = object_at(model, name)?;
    let m = Mat4::from_cols_array_2d(&cam.matrix_world);
    let axis = -m.z_axis.truncate().normalize_or_zero();
    Some((glam::Vec3::from(at) - m.w_axis.truncate()).dot(axis).abs())
}

/// Any object's world position by name, whichever list it lives in — a focus
/// target is very often an Empty.
fn object_at(model: &SceneModel, name: &str) -> Option<[f32; 3]> {
    let from = |m: &[[f32; 4]; 4]| [m[3][0], m[3][1], m[3][2]];
    if let Some(o) = model.objects.iter().find(|o| o.name == name) {
        return Some(from(&o.matrix_world));
    }
    if let Some(o) = model.empties.iter().find(|o| o.name == name) {
        return Some(from(&o.matrix_world));
    }
    if let Some(o) = model.lights.iter().find(|o| o.name == name) {
        return Some(from(&o.matrix_world));
    }
    model.cameras.iter().find(|o| o.name == name).map(|o| from(&o.matrix_world))
}

/// Everything a final render needs, gathered from the file and the panel.
#[derive(Clone, Copy, Debug)]
pub struct BRenderPlan {
    pub camera: Camera,
    pub w: u32,
    pub h: u32,
    pub samples: u32,
    pub denoise: bool,
    /// How the frame is drawn: the path tracer for EEVEE and Cycles, or —
    /// `BLENDER_WORKBENCH` — Solid / Wireframe with `Scene.display.shading`,
    /// as Blender's Workbench render engine draws F12 (`workbench_render.cc`).
    pub look: crate::v2::view::draw::canvas::view3d::rendered::BStillLook,
    /// The render region (`Scene.r.border`, `R_BORDER`): the pixels filled
    /// of the `w × h` frame, and whether the result is cropped to them
    /// (`R_CROP`) or the frame around them left black.
    pub region: crate::v2::view::draw::canvas::view3d::rendered::BStillRegion,
}

/// What F12 would render, or `None` when the scene has no camera to look
/// through — Blender refuses the same way ("No camera found in scene").
///
/// `look` carries the world and the lamps, which a final render stands in
/// exactly as the viewport does; only the framing differs.
pub fn plan(model: &SceneModel, set: BRenderSet, look: &BRayLook, plain: gpui::Hsla) -> Option<BRenderPlan> {
    plan_at(model, set, look, plain, None)
}

/// The same plan through the active camera where it stands at `frame`
/// (Blender renders `cfra`; a still scene stands the same at every frame).
pub fn plan_at(model: &SceneModel, set: BRenderSet, look: &BRayLook, plain: gpui::Hsla, frame: Option<f32>) -> Option<BRenderPlan> {
    let (w, h) = set.resolution.pixels(model);
    let cam = super::motion::active_camera_at(model, frame)?;
    let cam = &cam;
    let mut camera = camera_of(cam, w, h);
    let focus = focus_on(model, cam);
    if let Some(focus) = focus {
        camera.focus = focus;
    }
    lit(&mut camera, look);
    // Film ▸ Transparent is a final render's: the viewport keeps its sky.
    camera.film = [u32::from(set.film_transparent), 0, 0, 0];
    let lens = super::BCameraLens::of(cam).focused_at(focus);
    let (still_look, samples, denoise) = crate::v2::view::draw::canvas::view3d::rendered::BStillLook::for_engine(model, set, lens, &mut camera, plain);
    let region = crate::v2::view::draw::canvas::view3d::rendered::BStillRegion { pixels: set.border.pixels(w, h), crop: set.border.crop };
    Some(BRenderPlan { camera, w, h, samples, denoise, look: still_look, region })
}

// ---------------------------------------------------------------------------
// The job itself
// ---------------------------------------------------------------------------

use crate::v2::scene::BObject3D;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

/// A render in flight, or the last one that finished.
#[derive(Clone, Debug, Default)]
pub struct BRenderShot {
    pub w: u32,
    pub h: u32,
    /// The image, BGRA8 with row 0 at the top — what GPUI paints. Empty
    /// until the first sample is read back (`rendered::PEEK_EVERY`), then
    /// the render as it stands, then the finished frame.
    pub bgra: Vec<u8>,
    /// The same image as display-space floats, and the scene-linear radiance
    /// behind it. Both empty until the render settles; a save reads whichever
    /// the file's format asks for (`render_save.rs`).
    pub display: Vec<[f32; 3]>,
    pub linear: Vec<[f32; 3]>,
    /// The extra render passes, for a multi-layer save. Empty until the
    /// render settles, like the floats above.
    pub albedo: Vec<[f32; 3]>,
    pub normal: Vec<[f32; 3]>,
    pub depth: Vec<f32>,
    /// Samples taken so far, and the target they are working towards.
    pub taken: u32,
    pub target: u32,
    pub done: bool,
}

impl BRenderShot {
    /// 0.0 to 1.0, for a progress bar.
    pub fn fraction(&self) -> f32 {
        if self.done {
            return 1.0;
        }
        (self.taken as f32 / self.target.max(1) as f32).clamp(0.0, 1.0)
    }

    pub fn has_image(&self) -> bool {
        !self.bgra.is_empty() && self.w > 0 && self.h > 0
    }

    /// Whether this is a finished frame with its float passes — what a save
    /// writes — rather than a look at one under way (`BStillAt::Peek`), which
    /// has bytes for the screen and nothing else.
    pub fn settled(&self) -> bool {
        self.done && self.has_image() && !self.display.is_empty()
    }

    /// Whether this is a look at a render under way: bytes, no floats.
    pub fn peek(&self) -> bool {
        self.has_image() && !self.done
    }
}

#[derive(Default)]
pub(crate) struct BJob {
    /// The Render Result's render slots (`Image.renderslots`, eight), the
    /// one shown (`render_slot`) and the one the render in flight — or the
    /// last — went into (`last_render_slot`); `scene::render_slot`.
    pub(crate) slots: Mutex<Vec<Option<BRenderShot>>>,
    pub(crate) active: AtomicU32,
    pub(crate) last: AtomicU32,
    pub(crate) running: AtomicBool,
    cancel: AtomicBool,
    /// Set by [`pause`]: the worker finishes the sample in hand and then
    /// waits, holding the GPU idle, until [`resume`] or [`cancel`].
    paused: AtomicBool,
    /// Bumped whenever a slot changes or another is shown, so a painter can
    /// tell a new frame from the one it already drew.
    pub(crate) seq: AtomicU32,
    /// When the render in flight (or the last one) began, for the info bar.
    pub(super) started: Mutex<Option<std::time::Instant>>,
    /// How long the last finished render took — Blender's
    /// `RenderStats.lastframetime`, which `pipeline.cc:1510` sets ONCE as
    /// `BLI_time_now_seconds() - starttime` when the frame ends, and
    /// `make_renderinfo_string` then prints unchanged.
    ///
    /// Without it the info bar read `started.elapsed()` for ever, so the
    /// header said `Finished` while the clock kept running — 36 minutes on a
    /// render that took seconds, because nothing had rendered since.
    pub(super) took: Mutex<Option<std::time::Duration>>,
}

pub(super) fn job() -> &'static Arc<BJob> {
    static JOB: OnceLock<Arc<BJob>> = OnceLock::new();
    JOB.get_or_init(Arc::default)
}

/// The shown slot's render — the one in flight or the last finished there
/// — and the sequence number.
pub fn shot() -> (Option<BRenderShot>, u32) {
    let j = job();
    (j.slot(j.active.load(Ordering::Relaxed)), j.seq.load(Ordering::Relaxed))
}

impl BJob {
    /// Slot `i`'s render, if one landed there.
    pub(crate) fn slot(&self, i: u32) -> Option<BRenderShot> {
        self.slots.lock().ok().and_then(|slots| slots.get(i as usize).cloned().flatten())
    }

    /// `shot` into slot `i`; the sequence moves on so painters see it.
    pub(crate) fn put(&self, i: u32, shot: Option<BRenderShot>) {
        if let Ok(mut slots) = self.slots.lock() {
            let n = super::render_slot::BRenderSlot::COUNT.max(i as usize + 1);
            slots.resize(n, None);
            slots[i as usize] = shot;
        }
        self.seq.fetch_add(1, Ordering::Relaxed);
    }
}

/// Whether a render is under way.
pub fn rendering() -> bool {
    job().running.load(Ordering::Relaxed)
}

/// Ask the running render to stop. It finishes its current sample first, so
/// the flag is a request rather than a kill.
pub fn cancel() {
    job().cancel.store(true, Ordering::Relaxed);
}

/// Whether the last render was stopped by [`cancel`] (an animation render
/// ends there rather than going on to the next frame).
pub fn cancelled() -> bool {
    job().cancel.load(Ordering::Relaxed)
}

/// Hold the running render after its current sample: the image stays up as
/// it stands and the sample count stops, until [`resume`] or [`cancel`].
/// Blender's render job has no pause (`WM_jobs` stop or nothing); a long
/// render that must yield the GPU for a while here can — deliberate.
pub fn pause() {
    job().paused.store(true, Ordering::Relaxed);
}

/// Let a paused render go on.
pub fn resume() {
    job().paused.store(false, Ordering::Relaxed);
}

/// Whether the running render is held (`pause`). False once nothing runs.
pub fn paused() -> bool {
    let j = job();
    j.running.load(Ordering::Relaxed) && j.paused.load(Ordering::Relaxed)
}

/// Flip between held and running: the header's one button.
pub fn toggle_pause() {
    if paused() { resume() } else { pause() }
}

/// How long a held worker sleeps between looks at the flags.
const PAUSE_TICK: std::time::Duration = std::time::Duration::from_millis(50);

/// Put `shot` up as the image shown — a rendered frame played back
/// (`scene::render_play`) — in place of the Render Result, which the
/// player puts back when it stops. Refused while a render is under way.
pub fn show(shot: BRenderShot) -> bool {
    let j = job();
    if j.running.load(Ordering::Relaxed) {
        return false;
    }
    j.put(j.active.load(Ordering::Relaxed), Some(shot));
    true
}

/// Drop the renders: a new document has no Render Result, its slots empty
/// and the first shown (`BPhiberApp::render_on_entry` renders the next one
/// opened).
pub fn forget() {
    let j = job();
    if let Ok(mut slots) = j.slots.lock() {
        slots.clear();
    }
    j.active.store(0, Ordering::Relaxed);
    j.last.store(0, Ordering::Relaxed);
    j.seq.fetch_add(1, Ordering::Relaxed);
}

/// Start a final render of `objects` on a worker thread. Does nothing while
/// one is already running — Blender behaves the same way, F12 during a render
/// does not start a second.
///
/// The image is not returned: it arrives through [`shot`], which a painter
/// polls. That is what lets the Rendering space show progress instead of
/// freezing for the minutes a full-resolution render takes.
pub fn start(objects: Vec<BObject3D>, plan: BRenderPlan) {
    let j = job().clone();
    if j.running.swap(true, Ordering::SeqCst) {
        return;
    }
    j.cancel.store(false, Ordering::Relaxed);
    j.paused.store(false, Ordering::Relaxed);
    if let Ok(mut started) = j.started.lock() {
        *started = Some(std::time::Instant::now());
    }
    // A new frame has no time yet: the bar counts up again until it finishes.
    if let Ok(mut took) = j.took.lock() {
        *took = None;
    }
    // Into the slot shown (`BKE_image_backup_render`: the render goes to
    // `render_slot`, which becomes `last_render_slot`; the slot rendered
    // before keeps its result).
    let into = j.active.load(Ordering::Relaxed);
    j.last.store(into, Ordering::Relaxed);
    let publish = {
        let j = j.clone();
        move |shot: BRenderShot| {
            j.put(into, Some(shot));
            crate::v2::view::draw::canvas::view3d::rendered::landed();
        }
    };
    let (ow, oh) = plan.region.size(plan.w, plan.h);
    publish(BRenderShot { w: ow, h: oh, target: plan.samples, ..Default::default() });
    std::thread::spawn(move || {
        use crate::v2::view::draw::canvas::view3d::rendered::still_stepwise;
        let publish = publish;
        // A GPU validation error inside the tracer unwinds this thread; the
        // job must not stay "running" for ever (`BRunning`).
        let _running = super::render_fit::BRunning(j.clone());
        let began = std::time::Instant::now();
        let (w, h) = plan.region.size(plan.w, plan.h);
        let mut kept: (Vec<u8>, Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<f32>) = Default::default();
        let out = still_stepwise(
            &objects,
            plan.look,
            plan.camera,
            plan.w,
            plan.h,
            plan.samples,
            plan.denoise,
            plan.region,
            // The image KEPT between readbacks: most steps carry only a
            // count (`BStillAt::Counting`), and publishing those with empty
            // bytes wiped the peek published a moment before — the picture
            // flashed on at each readback and off at the next sample, which
            // is a blank canvas to a glance and a flicker to a stare.
            |taken, at| {
                let done = at.settled();
                let kept = &mut kept;
                if let Some(still) = at.image() {
                    let mut bgra = still.rgba;
                    // GPUI paints blue first; phiray hands back RGBA.
                    bgra.chunks_exact_mut(4).for_each(|p| p.swap(0, 2));
                    *kept = (bgra, still.display, still.linear, still.albedo, still.normal, still.depth);
                }
                publish(BRenderShot {
                    w,
                    h,
                    bgra: kept.0.clone(),
                    display: kept.1.clone(),
                    linear: kept.2.clone(),
                    albedo: kept.3.clone(),
                    normal: kept.4.clone(),
                    depth: kept.5.clone(),
                    taken,
                    target: plan.samples,
                    done,
                });
                // Held: the sample in hand is up; wait here, where the GPU
                // is idle, until let go or stopped.
                while j.paused.load(Ordering::Relaxed) && !j.cancel.load(Ordering::Relaxed) {
                    std::thread::sleep(PAUSE_TICK);
                }
                !j.cancel.load(Ordering::Relaxed)
            },
        );
        if out.is_none() {
            // No adapter, or a scene the GPU declined: the shot is finished
            // without an image rather than leaving the Rendering space
            // waiting for ever.
            publish(BRenderShot { w, h, target: plan.samples, done: true, ..Default::default() });
        }
        let taken = j.slot(into).map(|s| s.taken).unwrap_or(0);
        let secs = began.elapsed().as_secs_f64();
        // `lastframetime`, stored once, for the info bar to print from here on.
        // Cancelled counts too: Blender keeps the time the frame took either way.
        if let Ok(mut took) = j.took.lock() {
            *took = Some(began.elapsed());
        }
        if j.cancel.load(Ordering::Relaxed) {
            // The image as far as it got is the Render Result (Blender keeps
            // the tiles a cancelled render finished).
            crate::v2::state::log::push(format!("Render cancelled after {taken} samples"));
            crate::v2::app::journal::BJournal::note(format!("Render cancelled after {taken} samples, {secs:.1}s"));
        } else {
            // The journal's line, from this thread: how long F12 took.
            crate::v2::app::journal::BJournal::note(format!(
                "Render finished: {}×{}, {taken} samples in {secs:.1}s{}",
                plan.w, plan.h,
                if out.is_none() { " — no image (no adapter, or a scene the GPU declined)" } else { "" }
            ));
        }
    });
}

#[cfg(test)]
mod shot_tests {
    use super::*;

    fn bytes(n: usize) -> Vec<u8> {
        vec![255; n * 4]
    }

    #[test]
    fn a_look_at_a_running_render_has_an_image_but_is_not_settled() {
        let peek = BRenderShot { w: 2, h: 2, bgra: bytes(4), taken: 3, target: 20, ..Default::default() };
        assert!(peek.has_image(), "the viewer paints it");
        assert!(peek.peek());
        assert!(!peek.settled(), "a save has no floats to write");
        assert!((peek.fraction() - 0.15).abs() < 1e-6);
    }

    #[test]
    fn a_finished_render_with_its_floats_is_settled() {
        let done = BRenderShot { w: 2, h: 2, bgra: bytes(4), display: vec![[1.0; 3]; 4], linear: vec![[1.0; 3]; 4], taken: 20, target: 20, done: true, ..Default::default() };
        assert!(done.settled() && !done.peek());
        // A frame that ended without ever drawing (no adapter) is done and empty.
        let blank = BRenderShot { w: 2, h: 2, target: 20, done: true, ..Default::default() };
        assert!(!blank.has_image() && !blank.settled() && !blank.peek());
    }

    #[test]
    fn pause_is_read_only_while_a_render_runs() {
        // The flags of a job that is not running: `paused()` stays false
        // however the flag is set, so a stale pause cannot show in a header
        // after the render ends; `start` clears it for the next.
        let j = BJob::default();
        j.paused.store(true, Ordering::Relaxed);
        assert!(!(j.running.load(Ordering::Relaxed) && j.paused.load(Ordering::Relaxed)));
        j.running.store(true, Ordering::Relaxed);
        assert!(j.running.load(Ordering::Relaxed) && j.paused.load(Ordering::Relaxed));
    }
}
