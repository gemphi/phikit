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
    if let Some(focus) = focus_on(model, cam) {
        camera.focus = focus;
    }
    lit(&mut camera, look);
    // Film ▸ Transparent is a final render's: the viewport keeps its sky.
    camera.film = [u32::from(set.film_transparent), 0, 0, 0];
    let (still_look, samples, denoise) = crate::v2::view::draw::canvas::view3d::rendered::BStillLook::for_engine(model, set, &mut camera, plain);
    Some(BRenderPlan { camera, w, h, samples, denoise, look: still_look })
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
    /// The finished image, BGRA8 with row 0 at the top — what GPUI paints.
    /// Empty until the render settles.
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
}

#[derive(Default)]
pub(crate) struct BJob {
    pub(crate) shot: Mutex<Option<BRenderShot>>,
    pub(crate) running: AtomicBool,
    cancel: AtomicBool,
    /// Bumped whenever the shot changes, so a painter can tell a new frame
    /// from the one it already drew.
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

/// The render in flight or the last one finished, and its sequence number.
pub fn shot() -> (Option<BRenderShot>, u32) {
    let j = job();
    (j.shot.lock().ok().and_then(|s| s.clone()), j.seq.load(Ordering::Relaxed))
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

/// Drop the last render: a new document has no Render Result
/// (`BPhiberApp::render_on_entry` renders the next one opened).
pub fn forget() {
    let j = job();
    if let Ok(mut slot) = j.shot.lock() {
        *slot = None;
    }
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
    if let Ok(mut started) = j.started.lock() {
        *started = Some(std::time::Instant::now());
    }
    // A new frame has no time yet: the bar counts up again until it finishes.
    if let Ok(mut took) = j.took.lock() {
        *took = None;
    }
    let publish = {
        let j = j.clone();
        move |shot: BRenderShot| {
            if let Ok(mut slot) = j.shot.lock() {
                *slot = Some(shot);
            }
            j.seq.fetch_add(1, Ordering::Relaxed);
            crate::v2::view::draw::canvas::view3d::rendered::landed();
        }
    };
    publish(BRenderShot { w: plan.w, h: plan.h, target: plan.samples, ..Default::default() });
    std::thread::spawn(move || {
        use crate::v2::view::draw::canvas::view3d::rendered::still_stepwise;
        let publish = publish;
        // A GPU validation error inside the tracer unwinds this thread; the
        // job must not stay "running" for ever (`BRunning`).
        let _running = super::render_fit::BRunning(j.clone());
        let began = std::time::Instant::now();
        let out = still_stepwise(
            &objects,
            plan.look,
            plan.camera,
            plan.w,
            plan.h,
            plan.samples,
            plan.denoise,
            |taken, still| {
                let done = still.is_some();
                let still = still.unwrap_or(crate::v2::view::draw::canvas::view3d::rendered::BStill {
                    rgba: Vec::new(),
                    display: Vec::new(),
                    linear: Vec::new(),
                    albedo: Vec::new(),
                    normal: Vec::new(),
                    depth: Vec::new(),
                });
                let mut bgra = still.rgba;
                // GPUI paints blue first; phiray hands back RGBA.
                bgra.chunks_exact_mut(4).for_each(|p| p.swap(0, 2));
                publish(BRenderShot {
                    w: plan.w,
                    h: plan.h,
                    bgra,
                    display: still.display,
                    linear: still.linear,
                    albedo: still.albedo,
                    normal: still.normal,
                    depth: still.depth,
                    taken,
                    target: plan.samples,
                    done,
                });
                !j.cancel.load(Ordering::Relaxed)
            },
        );
        if out.is_none() {
            // No adapter, or a scene the GPU declined: the shot is finished
            // without an image rather than leaving the Rendering space
            // waiting for ever.
            publish(BRenderShot { w: plan.w, h: plan.h, target: plan.samples, done: true, ..Default::default() });
        }
        let taken = j.shot.lock().ok().and_then(|s| s.as_ref().map(|s| s.taken)).unwrap_or(0);
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
