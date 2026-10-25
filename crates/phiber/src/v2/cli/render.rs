//! One headless render pass, as a type — the loop that
//! `examples/animation_render.rs` has run since batch 53, lifted into the
//! library so the command line and the examples share it instead of each
//! carrying a copy.
//!
//! Nothing about the scene is changed by rendering it: the camera, the
//! lamps, the world and the view transform are the file's, because the
//! point is what the file looks like. The switches under
//! [`BLookOverride`] exist for *measurement* — isolating where the light
//! comes from when comparing against Blender — and are off unless asked
//! for.

use crate::v2::app::seed::{seed_output_props, seed_render_props, seed_view_layer};
use crate::v2::scene::{BObject3D, plan_at, render_set, source::set_loaded_shared};
use crate::v2::state::{BState, CtxKey};
use crate::v2::view::draw::canvas::view3d::rendered::{BMatStand, BRayLook, environment, still, view_words};

use super::set::BRenderQuality;

pub use super::cost::{BFrameCost, BLookOverride};

/// A render to perform: which file, which frames, and where the frames go.
#[derive(Clone, Debug)]
pub struct BHeadlessRender {
    pub file: String,
    pub out: String,
    /// The frames, already resolved — `BCli::frame_list` for the command
    /// line, a `start ..= end` walk for the examples.
    pub frame: Vec<f32>,
    /// `None` takes [`BRenderQuality::SAMPLE_FLOOR`], so a render is the
    /// quality one unless it was asked to be otherwise.
    pub samples: Option<u32>,
    pub draft: bool,
    /// `--camera`: render through this camera object instead of the
    /// scene's active one (there is no Blender flag for this; it saves a
    /// headless probe from having to edit the file).
    pub camera: Option<String>,
    pub look: BLookOverride,
}

impl BHeadlessRender {
    /// The frames `start ..= end` by `step`, the way Ctrl F12 takes them —
    /// an `end` below `start` means the file's own end, and an `end` equal
    /// to `start` is one frame and not "unset".
    pub fn range(start: f32, end: f32, step: f32, file_end: f32) -> Vec<f32> {
        let end = if end >= start { end } else { file_end };
        let step = if step > 0.0 { step } else { 1.0 };
        let mut out = Vec::new();
        let mut frame = start;
        while frame <= end + 1e-3 {
            out.push(frame);
            frame += step;
        }
        out
    }

    /// The samples every frame takes, as the command line takes them.
    pub fn sample_count(&self) -> u32 {
        BRenderQuality::samples(self.samples, self.draft)
    }

    /// Where frame `frame` is written, by Blender's own rule
    /// (`BLI_path_frame`, `BKE_image_path_from_imformat`): the last
    /// component of `-o` is a **name prefix**, not a folder, unless the
    /// path ends with a separator. `####` anywhere in the name is replaced
    /// by the frame number at that width, else four digits are appended.
    ///
    /// `-o …\\pose26_phiray_` used to make a *folder* called
    /// `pose26_phiray_` and write `f0001.png` inside it, which is neither
    /// where nor what the caller asked for.
    pub fn frame_path(out: &str, frame: i64) -> (std::path::PathBuf, std::path::PathBuf) {
        let dir_only = out.ends_with('/') || out.ends_with('\\');
        let trimmed = out.trim_end_matches(['/', '\\']);
        let cut = trimmed.rfind(['/', '\\']).map(|i| i + 1).unwrap_or(0);
        let (dir, prefix) = if dir_only { (trimmed, "") } else { (&trimmed[..cut.saturating_sub(1).max(0)], &trimmed[cut..]) };
        let dir = std::path::PathBuf::from(if dir.is_empty() { "." } else { dir });
        // `####` sets its own width; otherwise four digits follow the name.
        let name = match (prefix.find('#'), prefix.rfind('#')) {
            (Some(a), Some(b)) => format!("{}{:0width$}{}.png", &prefix[..a], frame, &prefix[b + 1..], width = b - a + 1),
            _ => format!("{prefix}{frame:04}.png"),
        };
        (dir.clone(), dir.join(name))
    }

    /// Run it. One PNG per frame, named Blender's way ([`Self::frame_path`]),
    /// and a `times.txt` beside them for the comparisons.
    pub fn run(&self) -> Result<Vec<BFrameCost>, String> {
        let (outdir, _) = Self::frame_path(&self.out, 1);
        std::fs::create_dir_all(&outdir).map_err(|e| format!("outdir: {e}"))?;
        let mut model = super::open::open(&self.file).map_err(|e| format!("open: {e}"))?;
        if let Some(camera) = &self.camera {
            if !model.cameras.iter().any(|c| &c.name == camera) {
                return Err(format!("--camera: no camera object named {camera:?} in the scene"));
            }
            model.render.camera = camera.clone();
        }
        if let Some(albedo) = self.look.material {
            BMatStand::set(Some(BMatStand::grey(albedo)));
        }
        if self.look.no_world {
            model.world.background.strength = 0.0;
        }
        if self.look.no_lights {
            model.lights.clear();
        }
        if let Some(view) = &self.look.view {
            model.render.view_transform = view.clone();
        }

        let mut state = BState::new();
        seed_view_layer(&mut state, &model);
        seed_render_props(&mut state, &model);
        // Film ▸ Transparent and the output settings live in the other
        // seed; without it every render painted a background Blender
        // leaves out.
        seed_output_props(&mut state, &model);
        // What the render is given, and which of it Phiray reads
        // (`render_audit`): printed, since a background render has no
        // journal window to read it in.
        for line in crate::v2::scene::render_audit::BRenderAudit::lines(&state) {
            println!("{line}");
        }
        // Shared, not copied: a pose file's model is gigabytes once its
        // meshes are read, and a second copy of it is what ran a 2 GB scene
        // out of memory before a single sample.
        let model = std::sync::Arc::new(model);
        set_loaded_shared(Some(model.clone()));
        let mut set = render_set(&state);
        set.samples = self.sample_count();

        let scale = self.look.scale.max(1);
        let plain = gpui::hsla(0.0, 0.0, 0.8, 1.0);
        let mut cost = Vec::new();
        for &frame in &self.frame {
            let began = web_time::Instant::now();
            // The frame the scene stands at: `current_scene` poses the rig
            // and the objects, `plan_at` puts the camera where it is.
            state.set(CtxKey::Frame, format!("{frame}"));
            // The render's own restrictions — the camera toggle and excluded
            // collections, not the viewport's eye: Blender renders what the
            // viewport hides and leaves out what the camera disables
            // (`render_cast.rs`, the same cast F12 in the app draws).
            let cast = crate::v2::scene::render_cast::BRenderCast::of(&state, Some(&model));
            if let Some(note) = &cast.note {
                println!("{note}");
            }
            let objects: Vec<BObject3D> = cast.objects;
            let posed = began.elapsed();
            // An empty list here is the headless twin of the app's blank
            // Render Result, and was just as silent (`scene::render_cast`).
            let drawn = crate::v2::scene::render_cast::BRenderCast::count_geometry(&objects);
            if drawn == 0 {
                return Err(format!(
                    "frame {frame}: nothing to render — {} objects reached the tracer, none with geometry",
                    objects.len()
                ));
            }
            let look = BRayLook {
                sky: gpui::hsla(0.0, 0.0, 0.14, 1.0),
                grid: gpui::hsla(0.0, 0.0, 0.3, 1.0),
                world: Some(model.world.background.radiance()),
                lights: crate::v2::scene::pose::lights_in_render(&state, crate::v2::scene::motion::lights_at(&model, Some(frame))),
                wire: None,
                view: view_words(&state),
                grid_hide: 0,
                environment: environment().map(|(_, rot)| rot),
                set,
                studio: false,
            };
            let mut plan = plan_at(&model, set, &look, plain, Some(frame)).ok_or("the scene names no camera")?;
            if let Some(denoise) = self.look.denoise {
                plan.denoise = denoise;
            }
            plan.w = (plan.w / scale).max(1);
            plan.h = (plan.h / scale).max(1);
            let shot = web_time::Instant::now();
            // Blender's `-b` prints a line a sample — `Fra:1 | Time:00:12.34 |
            // Remaining:01:02.00 | Sample 3/256` — so a long render shows its
            // pace and what is left; this printed nothing until the end.
            let target = plan.samples;
            // `still` reports the samples taken so far — already a count,
            // and the last one twice (the final sample, then the settled
            // frame) — so the line is printed once a count.
            let mut last = 0;
            let report = |done: u32| {
                if done == last {
                    return;
                }
                last = done;
                let elapsed = shot.elapsed();
                let left = crate::v2::scene::render_status::remaining(elapsed, done, target, false)
                    .map_or(String::new(), |left| format!(" | Remaining:{}", crate::v2::scene::render_status::timecode(left)));
                println!("Fra:{frame} | Time:{}{left} | Sample {done}/{target}", crate::v2::scene::render_status::timecode(elapsed));
            };
            let (pixel, taken) = still(&objects, plan.look, plan.camera, plan.w, plan.h, plan.samples, plan.denoise, plan.adapt, plan.time_limit, report)
                // The tracer's own reason (`render_job::note_failure`), not a
                // guess: headless has no Info editor and no journal, so the
                // 2048-sample run of pose 26 ended after 30 minutes having
                // printed nothing at all.
                .ok_or_else(|| {
                    crate::v2::scene::render_job::failure()
                        .unwrap_or_else(|| "no gpu adapter, or a scene that does not fit".to_string())
                })?;
            let drew = shot.elapsed();
            // Which device drew it, as Blender's `-b` names its Cycles
            // device: a time means nothing without it (a laptop can hand an
            // unlisted exe to its integrated GPU).
            if cost.is_empty()
                && let Some(device) = phiray::Gpu::in_use()
            {
                println!("Device: {} | {} | {} {}", device.name, device.kind_label(), device.backend_label(), device.driver);
            }
            let (_, path) = Self::frame_path(&self.out, frame as i64);
            image::RgbaImage::from_raw(plan.w, plan.h, pixel)
                .ok_or("the pixels do not fill the frame")?
                .save(&path)
                .map_err(|e| format!("save {}: {e}", path.display()))?;
            println!("Saved: {}", path.display());
            cost.push(BFrameCost {
                frame,
                posed_ms: posed.as_secs_f64() * 1000.0,
                drew_ms: drew.as_secs_f64() * 1000.0,
                samples: taken,
            });
        }
        let times: String = cost.iter().map(BFrameCost::line).collect();
        std::fs::write(outdir.join("times.txt"), times).map_err(|e| format!("times: {e}"))?;
        Ok(cost)
    }
}

#[cfg(test)]
#[path = "render_tests.rs"]
mod render_tests;
