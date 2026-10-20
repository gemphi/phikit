//! Material Preview and Rendered shading through phiray: the scene is
//! ray traced on a render thread ([`worker`]) and painted as one image that
//! refines while the view is still. Material Preview uses the studio rig
//! (like Blender's default HDRI preview); Rendered uses the file's lights and
//! world. Falls back to the mesh painter when
//! there is no GPU adapter.

mod atlas;
mod bands;
mod build;
mod clock;
mod compile;
mod job;
mod coarse;
mod key;
mod pace;
mod place;
mod posted;
mod prepare;
mod reframe;
mod scene;
mod settle;
mod region;
mod still;
pub mod studio;
mod vert;
mod view;
mod wake;
mod worker;
mod lamp;
mod slot;
mod stand;
mod state;
mod surface;
mod world;
pub use self::region::BStillRegion;
pub(in crate::v2::view::draw::canvas) use self::prepare::BRayDeclined;
pub(in crate::v2::view::draw::canvas::view3d) use self::prepare::prepare;
pub use self::still::{BStill, BStillAt, BStillLook, PEEK_EVERY, still, still_stepwise};
pub use self::lamp::BLamp;
pub use self::stand::BMatStand;
pub use self::world::{BRayLook, ENV_CDF_SLOT, ENV_IRRADIANCE_SLOT, ENV_PREFILTERED_SLOT, ENV_SLOT, environment, lit, studio_lighting, view_words};
#[cfg(test)]
mod paint_tests;
#[cfg(test)]
mod still_tests;
#[cfg(test)]
mod surface_tests;
#[cfg(test)]
mod tests;

use gpui::{Bounds, Corners, Pixels, RenderImage, Window};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use self::reframe::BShotView;
use self::state::{MAX_SIDE, POSTED, STATE};
pub use self::state::{shown_image, views_posted};
use self::worker::{BRayJob, BRayShot, BRayWorker};
pub(in crate::v2::view::draw::canvas::view3d) use self::view::BRayView;
pub use self::wake::{drawing, landed, refining, set_ticking, take_fresh, ticking};
pub(super) use self::wake::REFINING;


/// Paint the traced frame. Answers whether an image actually landed: the
/// tracer is asynchronous, so the first frames after a change — and every
/// frame on a machine whose adapter went away — have nothing to show, and
/// the caller has to draw something rather than leave the canvas empty.
pub(super) fn paint(bounds: Bounds<Pixels>, view: BRayView, corners: Corners<Pixels>, window: &mut Window) -> bool {
    let (bw, bh) = (f32::from(bounds.size.width), f32::from(bounds.size.height));
    if bw < 1.0 || bh < 1.0 {
        return false;
    }
    let dpr = window.scale_factor();
    let fit = (MAX_SIDE / (bw.max(bh) * dpr)).min(1.0);
    // And no more pixels than the device can bind as one image
    // (`Tracer::max_pixels`): a larger view is traced smaller and stretched.
    let most = phiray::Gpu::opened().map_or(f32::MAX, |gpu| phiray::Tracer::max_pixels(&gpu) as f32);
    let fit = fit * (most / (bw * bh * dpr * dpr * fit * fit)).sqrt().min(1.0);
    let live = ((bw * dpr * fit) as u32, (bh * dpr * fit) as u32);
    // While the viewport is being dragged, trace at the size already in hand
    // and stretch it; take the new size once the drag stops (`settle`).
    // This viewport's slot (`state::BRaySlot`), found by where it is painted.
    let (slot, ((w, h), waiting)) = STATE.with(|s| {
        let mut s = s.borrow_mut();
        let (slot, gone) = s.slot((f32::from(bounds.origin.x), f32::from(bounds.origin.y)));
        gone.into_iter().for_each(|img| drop(window.drop_image(img)));
        (slot, s.slots[slot].settle.size(live, std::time::Instant::now()))
    });
    // The viewport's own render region: the frame is still `w × h`, the
    // image traced is the region's pixels of it (`Camera::tile`), drawn in
    // its place and nothing outside (`ED_view3d_calc_render_border`).
    let part = view.border.and_then(|rect| crate::v2::scene::view_border::BViewBorder::pixels_of(rect, w, h));
    let (tile, (tw, th)) = match part {
        Some((x, y, rw, rh)) => ([x, y, w, h], (rw, rh)),
        None => ([0; 4], (w, h)),
    };
    let look = BRayLook {
        sky: view.sky,
        grid: view.grid,
        grid_hide: view.grid_hide,
        set: view.set,
        // Workbench (Solid, Wireframe) draws the theme's viewport shade behind
        // the scene, whatever the file's world says (`View3DShading.
        // background_type` THEME); only Rendered looks through to the world.
        world: match (view.studio, view.mode) {
            (_, phiray::Mode::Wire | phiray::Mode::Solid) | (true, _) => None,
            (false, _) => crate::v2::scene::source::with_loaded(|m| m.map(|m| m.world.background.radiance())),
        },
        environment: match (view.studio, view.mode) {
            (_, phiray::Mode::Wire | phiray::Mode::Solid) => None,
            // Material Preview: Blender's studio light, whatever the file's world says.
            (true, _) => self::studio::environment().map(|_| self::studio::ROTATION),
            (false, _) => self::world::environment().map(|(_, rot)| rot),
        },
        lights: view.lights.clone(),
        wire: (view.mode == phiray::Mode::Wire).then_some(view.wire),
        view: view.view,
        studio: view.studio,
    };
    // The camera frames the size being traced, not the live bounds: the image
    // is stretched to fit, so a drag shows the old frame widened rather than
    // a new one squeezed into the old pixels.
    let cam = phiray::Camera {
        see_through: view.see_through,
        fx: view.fx,
        cavity: view.cavity,
        ssao: view.ssao,
        shadow: view.shadow,
        shadow_fade: view.shadow_fade,
        matcap: view.matcap,
        outline_fx: view.outline_fx,
        outline_pick: view.outline_pick,
        outline_active: view.outline_active,
        outline_move: view.outline_move,
        outline: view.outline,
        shade: view.shade,
        single: view.single,
        dof: view.dof,
        tile,
        ..self::scene::camera(view.yaw, view.pitch, view.pan, view.zoom, view.setup, w as f32 / h.max(1) as f32, &look)
    };
    let (image, settled) = STATE.with(|s| {
        let (mut held, mut hasher) = (s.borrow_mut(), std::collections::hash_map::DefaultHasher::new());
        let s = &mut *held;
        // The sample target and the denoise flag are not part of `Camera`,
        // so they join the key by hand — changing either has to restart the
        // refinement, not carry on with the old one.
        // The selection rides outside `Camera` — so that changing it does
        // not restart the refinement — and so joins the key by hand.
        (s.scene, w, h, view.mode as u32, bytemuck::bytes_of(&cam), view.set.preview_samples, view.aa, view.set.preview_denoise, view.set.preview_denoise_from, &*view.outline_class)
            .hash(&mut hasher);
        let key = hasher.finish();
        let shot = |aspect| self::reframe::BShotView {
            zoom: view.zoom,
            pan: view.pan,
            aspect,
            orbit: (view.yaw, view.pitch, view.setup.pivot, view.setup.perspective),
            part: part.map(|(x, y, rw, rh)| (x as f32 / w as f32, y as f32 / h as f32, rw as f32 / w as f32, rh as f32 / h as f32)),
        };
        let post = s.should_post(slot, key);
        let Some(Some(worker)) = s.worker.as_ref() else { return (None, true) };
        if post {
            worker.send(BRayJob::View(
                key,
                BRayShot { cam, w: tw, h: th, mode: view.mode, samples: view.set.preview_samples, aa: view.aa, denoise_from: view.set.preview_denoise.then_some(view.set.preview_denoise_from), outline: Arc::clone(&view.outline_class) },
            ));
        }
        let frame = worker.take(s.slots[slot].view, s.taken);
        if post {
            s.shots.note(key, shot(w as f32 / h.max(1) as f32));
            s.clock.posted(key);
            POSTED.with(|n| n.set(n.get() + 1));
            s.posted(slot, key);
        }
        // An empty frame is one the kernel never wrote (`worker`): nothing to
        // show, but the view has settled — the theme canvas stays.
        let mut blank = false;
        if let Some(f) = frame {
            if let Some(line) = s.clock.landed(f.view, f.done) {
                crate::v2::app::journal::BJournal::note(line);
            }
            blank = f.bgra.is_empty() && f.done && f.view == key;
            let img = image::RgbaImage::from_raw(f.w, f.h, f.bgra).map(|buf| Arc::new(RenderImage::new(smallvec::smallvec![image::Frame::new(buf)])));
            if let Some(old) = s.land(f.seq, f.view, f.done, img) {
                let _ = window.drop_image(old);
            }
        }
        // The image where its own view puts it in this one (`reframe.rs`).
        let place = |v: u64| s.shots.get(v).map_or(bounds, |old| old.place(&BShotView { part: None, ..shot(bw / bh) }, bounds));
        match &s.slots[slot].image {
            Some((_, v, done, img)) => (Some((img.clone(), place(*v))), *done && *v == key),
            None => (None, blank),
        }
    });
    let painted = match image {
        Some((img, at)) => {
            let _ = window.paint_image(bounds, at, corners, img, 0, false);
            true
        }
        None => false,
    };
    // While a new size is waiting, draw again so the end of the drag is
    // seen; while the view refines, the app draws when a frame lands (the
    // flag is cleared as each window draw begins, `wake::drawing`, so any
    // one unsettled viewport keeps it).
    if !settled {
        REFINING.store(true, std::sync::atomic::Ordering::Release);
    }
    // No ticker yet to carry the frames the tracer is about to land: one
    // more frame, in which `render` sees the flag and starts it (`wake`).
    if waiting || (!settled && !self::wake::ticking()) {
        window.request_animation_frame();
    }
    painted
}

#[path = "fit.rs"]
mod fit;
pub use fit::scene_fit;
