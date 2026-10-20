//! The stepwise still render — the long path that yields partial images
//! while the tracer refines, and the small helpers it keeps to itself
//! (`composite`, `note_render_refused`). Split so each file sits under the
//! 240-line read cap (`tests/level_structure.rs`).

use super::*;



/// The same render, one sample at a time: `step` is called after each with
/// the samples taken so far and what was read back (`BStillAt`): usually
/// nothing, every `PEEK_EVERY` the bytes as they stand, and once the frame
/// has settled the finished image. Returning `false` from it stops the
/// render early, which is how a job is cancelled: the image as far as it got
/// — the bands finished and the one under way at its samples so far — is
/// handed to `step` as the finished one, as Blender keeps a cancelled
/// render's tiles.
///
/// A frame with more pixels than the device binds as one image
/// (`Tracer::max_pixels`) is rendered in horizontal bands, each a tile of
/// the frame (`Camera::tile`), and the bands are joined — so a 4K frame
/// renders whole on a device that binds 128 MiB. The samples reported step
/// over the bands together, so progress still ends at `samples`.
///
/// `still` is this without the reporting; a final render uses this one so the
/// Rendering space can show progress rather than freeze (`scene/render_job.rs`).

/// Why a render produced nothing, said once: the Info editor's report and
/// the session journal both carry it, so a blank Render Result is never the
/// only evidence that something went wrong.
fn note_render_refused(why: String) {
    let line = format!("Render failed: {why}");
    crate::v2::state::log::push(line.clone());
    crate::v2::app::journal::BJournal::note(line);
}

/// A slice's intended length in the final render. Short enough that no
/// single dispatch approaches a GPU watchdog (two seconds on Windows),
/// long enough that the drain between slices stays a small share of the
/// work — `Tracer::sample_in` waits for the device after each one.
const RENDER_SLICE_MS: f64 = 120.0;

/// Slices for the very first sample, before anything has been timed.
const FIRST_SLICES: u32 = 32;

/// The most a sample is cut into; a slice costs one device drain.
const MOST_SLICES: u32 = 64;

/// How many slices a sample takes, from how long one took.
///
/// Settled from the **second** sample, never the first: `Tracer::warm`
/// compiles the other shading modes on another thread and holds the
/// device's pipeline lock while it does, so the first sample can spend
/// minutes blocked on a lock rather than on the GPU.
struct BSliceCut;

impl BSliceCut {
    fn from(took: std::time::Duration) -> u32 {
        let ms = took.as_secs_f64() * 1000.0;
        if ms <= 0.0 {
            return FIRST_SLICES;
        }
        ((ms / RENDER_SLICE_MS).ceil() as u32).clamp(1, MOST_SLICES)
    }
}

pub fn still_stepwise(
    objects: &[BObject3D],
    look: BStillLook,
    cam: phiray::Camera,
    w: u32,
    h: u32,
    samples: u32,
    denoise: bool,
    // Adaptive Sampling as Cycles resolves it (`BAdaptive::of`):
    // `[threshold, min samples, step, on]`. All zero is off, which is what
    // Workbench and the thumbnail paths pass.
    adapt: [f32; 4],
    // Render ▸ Sampling ▸ Time Limit in seconds, `0` for none. Shared out
    // across the bands by their rows, so a frame cut into four bands gives
    // each a quarter of the time rather than the first band all of it and
    // the last none — Cycles has no bands and simply stops when the clock
    // runs out (`Session::run_main_render_loop`).
    time_limit: f32,
    region: super::super::region::BStillRegion,
    // How often the image so far is read back for someone watching
    // (`PEEK_EVERY` for the Rendering space); `None` for a caller that only
    // wants the finished frame. A readback stalls the device and copies the
    // whole frame, so a headless render that drew nothing with them paid
    // for one every sample.
    peek: Option<std::time::Duration>,
    mut step: impl FnMut(u32, BStillAt) -> bool,
) -> Option<u32> {
    use crate::v2::scene::BShading;
    // The process's device, not one of its own: a device opened per
    // render compiled every kernel again before its first sample (seconds
    // on a warm driver cache, half a minute on a cold one) and held a
    // second copy of the scene beside the viewport's. The shared one has
    // the kernels the viewport already built, and is the device Preferences
    // ▸ System chose.
    let Some(gpu) = phiray::Gpu::shared() else {
        note_render_refused("no GPU adapter".to_string());
        return None;
    };
    let (shading, palette) = (look.shading, look.palette());
    let cam = with_studio(cam, shading);
    let built = scene::build(objects, &palette);
    // A scene the device will not take is reported with its reason rather
    // than leaving a blank Render Result behind: pose 26 declined here and
    // the info bar still said "Finished" (`phiray::Tracer::why_not`).
    if let Some(why) = phiray::Tracer::why_not(&built, gpu.max_binding(), phiray::Tracer::chunks_for(&gpu)) {
        note_render_refused(format!("the GPU declined this scene — {why}"));
        return None;
    }
    let mut tracer = phiray::Tracer::from_scene(&gpu, built)?;
    // A Workbench render's `render_aa` is its anti-aliasing count; the
    // path tracer's samples are its own.
    tracer.set_samples(samples);
    tracer.set_aa_samples(samples);
    tracer.set_adaptive(adapt);
    // What Adaptive Sampling resolved to, said once: the three panel rows
    // are "automatic" by default and Cycles' rules turn them into quite
    // different numbers, so without this there is no way to tell from a
    // log which settings a render actually used.
    crate::v2::app::journal::BJournal::note(if adapt[3] != 0.0 {
        format!(
            "Adaptive sampling: threshold {:.4}, min {} samples, checked every {}",
            adapt[0], adapt[1] as u32, adapt[2] as u32,
        )
    } else {
        "Adaptive sampling: off — every pixel takes every sample".to_string()
    });
    tracer.set_mode(match shading {
        BShading::Wireframe => phiray::Mode::Wire,
        BShading::Solid => phiray::Mode::Solid,
        BShading::Material => phiray::Mode::Preview,
        BShading::Rendered => phiray::Mode::Rendered,
    });
    // The pixels drawn: the region of the frame — two pixels wider each
    // way, for the same reason as the bands' overlap, and cut back after —
    // or the frame itself.
    // A band is drawn some rows over into its neighbours and laid in
    // without them, and the region with that margin around it: the effects
    // and the anti-aliasing gather each read a pixel's neighbours and clamp
    // at a tile's edge, and the overlap is what makes the seam invisible —
    // two rows, or SMAA's reach when the single pass leaves it on the
    // finished frame (`Tracer::seam_rows`).
    // A final render keeps the GPU busy far longer than the viewport's
    // five-second readback guard allows, and a readback that gives up
    // returns an EMPTY buffer that every caller reads as "the GPU
    // declined" — which is exactly how pose 26 came back blank while the
    // card was working. The budget is the render's for as long as it runs
    // (`phiray::PatienceFor`), and the viewport keeps its own.
    let _patience = phiray::PatienceFor::render(std::time::Duration::from_secs(600));
    let reach = tracer.seam_rows();
    let (rx, ry, rw, rh) = region.drawn(w, h, reach);
    let bands = BBandList::of(rw, rh, BBandList::most(&gpu), reach);
    let mut drawn = BStill::empty(rw, rh);
    // The first sample is shown at once, then one every `PEEK_EVERY`.
    let mut peeked = std::time::Instant::now() - peek.unwrap_or_default();
    // How a sample is cut into dispatches. The viewport has always done
    // this (`BSlicer`); the final render did not, and that is what killed
    // the app: a compute dispatch that runs past the platform's GPU
    // watchdog — two seconds on Windows — has the display driver reset
    // under it, and wgpu reports the device lost. Pose 26 was ~4 s a
    // sample, so every final render of it was over the limit.
    let mut slices = FIRST_SLICES;
    let mut seen = 0u32;
    let mut told = false;
    let mut told_done = false;
    let mut told_limit = false;
    for (k, (y0, rows)) in bands.list.iter().enumerate() {
        let (over, under) = (reach.min(*y0), reach.min(rh - y0 - rows));
        // This band's share of the time limit, by its rows of the frame.
        let budget = if time_limit > 0.0 { time_limit * (*rows as f32 / rh.max(1) as f32) } else { 0.0 };
        let began = std::time::Instant::now();
        let tiled = bands.list.len() > 1 || region.pixels.is_some();
        let band = phiray::Camera { tile: if tiled { [rx, ry + y0 - over, w, h] } else { [0; 4] }, ..cam };
        loop {
            let at = std::time::Instant::now();
            // `go` always says carry on: a final render is never abandoned
            // for a newer view, so `sample_in` never returns `None` here.
            let (taken, done) = tracer
                .sample_in(&gpu, band, rw, rows + over + under, slices, || true)
                .expect("a final render never abandons a sample");
            let took = at.elapsed();
            let was = slices;
            seen += 1;
            if seen == 2 {
                slices = BSliceCut::from(took);
                if !told {
                    told = true;
                    crate::v2::app::journal::BJournal::note(format!(
                        "Render pace: {:.0}ms a sample over {rw}×{} px, cut into {was} dispatches of about {:.0}ms; the rest take {slices}",
                        took.as_secs_f64() * 1000.0,
                        rows + over + under,
                        took.as_secs_f64() * 1000.0 / f64::from(was.max(1)),
                    ));
                }
            }
            // Every pixel settled: the band is finished, whatever the
            // sample count says. Cycles ends a render here rather than
            // running on.
            let settled = taken > 0
                && (taken & (crate::v2::scene::render_set::BAdaptive::STEP - 1)) == crate::v2::scene::render_set::BAdaptive::STEP - 1
                && tracer.unconverged(&gpu) == Some(0);
            if settled && !told_done {
                told_done = true;
                crate::v2::app::journal::BJournal::note(format!(
                    "Adaptive sampling: every pixel settled at sample {} of {}",
                    taken + 1,
                    tracer.samples(),
                ));
            }
            // The clock: a band past its share of the time limit is
            // finished at this sample, whatever Max Samples says.
            let out_of_time = budget > 0.0 && began.elapsed().as_secs_f32() >= budget;
            if out_of_time && !told_limit && !done && !settled {
                told_limit = true;
                crate::v2::app::journal::BJournal::note(format!(
                    "Time limit: {time_limit:.0}s reached at sample {} of {}",
                    taken + 1,
                    tracer.samples(),
                ));
            }
            let done = done || settled || out_of_time;
            // Reading the image back is the slow part, so progress carries a
            // count, the bytes now and then (`PEEK_EVERY`), and the whole
            // picture only at the end.
            let go_on = done || {
                let at = if peek.is_some_and(|every| peeked.elapsed() >= every) {
                    peeked = std::time::Instant::now();
                    let mut peek = drawn.rgba.clone();
                    BStill::lay_rgba(&mut peek, &tracer.pixels(&gpu), *y0 as usize, *rows as usize, over as usize, rw as usize);
                    BStillAt::Peek(BStill::peek(region.framed_rgba(peek, w, h, reach)))
                } else {
                    BStillAt::Counting
                };
                step(bands.progress(k, taken + 1, tracer.samples()), at)
            };
            if done || !go_on {
                if denoise && done {
                    tracer.denoise(&gpu);
                }
                if done {
                    composite(&gpu, &mut tracer);
                }
                drawn.lay(&BStill::read(&tracer, &gpu), *y0 as usize, *rows as usize, over as usize, rw as usize);
                if !go_on {
                    step(bands.progress(k, taken + 1, tracer.samples()), BStillAt::Settled(region.framed(drawn, w, h, reach)));
                    return Some(taken + 1);
                }
                break;
            }
        }
    }
    step(tracer.samples(), BStillAt::Settled(region.framed(drawn, w, h, reach)));
    Some(tracer.samples())
}

/// The scene's compositor over the finished render, as Blender's render
/// pipeline runs it: on the scene-linear pixels, before the view transform
/// (`scene/composite.rs`). Nothing happens when the file has no tree, it is
/// switched off, or nothing reaches its output. A band renders as its own
/// image, so each band is composited on its own — which is sound only
/// because every node that runs is per-pixel; a node that read a
/// neighbourhood would see a seam, and those are exactly the ones
/// `BCompTree::declined` refuses.
fn composite(gpu: &phiray::Gpu, tracer: &mut phiray::Tracer) {
    use crate::v2::scene::composite::BComposite;
    if let Some(tree) = crate::v2::scene::composite::tree() {
        let run = tracer.composite(gpu, &tree);
        if run.any() {
            crate::v2::app::journal::BJournal::note(run.line());
        }
    }
}

/// Material Preview is lit by the studio image `scene::build` puts in the
/// atlas: a camera that was not told so (built by hand rather than by
/// `lit`) gets the lighting words here, or every surface comes out a dark
/// mirror of the raw image.
pub fn with_studio(mut cam: phiray::Camera, shading: crate::v2::scene::BShading) -> phiray::Camera {
    if shading == crate::v2::scene::BShading::Material && cam.preview == [0; 4] && super::super::studio::environment().is_some() {
        super::super::world::studio_lighting(&mut cam);
    }
    cam
}
