//! F12 renders on a worker, reports progress, and can be stopped.
//!
//! A full-resolution render takes minutes, so the Rendering space must not
//! wait for it: the job publishes a sample count as it goes and the finished
//! image at the end, and `cancel` stops it after the sample in flight.
//!
//! The job is process-wide (one render at a time, as in Blender), so these
//! run in sequence inside one test rather than as separate ones racing for it.

use phiber::phiphx::scene::SceneCamera;
use phiber::v2::scene::{BObject3D, BRenderPlan, BShading, camera_of, render_job};
use phiber::v2::view::draw::canvas::view3d::rendered::BStillLook;

fn plan(samples: u32) -> BRenderPlan {
    let cam = SceneCamera {
        name: "Camera".into(),
        location: [0.0, -4.0, 0.0],
        matrix_world: [[1.0, 0.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, -1.0, 0.0, 0.0], [0.0, -4.0, 0.0, 1.0]],
        kind: "PERSP".into(),
        focal_length_mm: 50.0,
        sensor_width: 36.0,
        ortho_scale: 6.0,
        clip_start: 0.1,
        clip_end: 100.0,
        display_size: 1.0,
        dof: Default::default(),
    };
    BRenderPlan { camera: camera_of(&cam, 32, 32), w: 32, h: 32, samples, denoise: false, look: BStillLook::new(BShading::Rendered, gpui::hsla(0.0, 0.0, 0.8, 1.0)), region: Default::default() }
}

fn a_cube() -> Vec<BObject3D> {
    vec![BObject3D::Primitive { name: "Cube".into(), shape: phiber::v2::scene::BShape::Cube, x: 0.0, y: 0.0 }]
}

/// Wait for a condition, or give up — a render on a software GPU is slow but
/// not unbounded.
fn until(mut ready: impl FnMut() -> bool) -> bool {
    for _ in 0..600 {
        if ready() {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    false
}

#[test]
fn a_render_reports_progress_then_an_image_and_can_be_cancelled() {
    if phiray::Ray::gpu().is_none() {
        return;
    }

    // A render publishes its target before the first sample, so the
    // Rendering space can show "0 of N" rather than nothing.
    render_job::start(a_cube(), plan(24));
    let (first, _) = render_job::shot();
    let first = first.expect("a shot is published as the job starts");
    assert_eq!((first.w, first.h, first.target), (32, 32, 24));
    assert!(!first.done);
    assert_eq!(first.fraction(), 0.0);

    // Asking again while it runs does not start a second render.
    render_job::start(a_cube(), plan(999));
    assert_eq!(render_job::shot().0.expect("still the first").target, 24, "a second F12 restarted the render");

    assert!(until(|| !render_job::rendering()), "the render never finished");
    let (done, seq) = render_job::shot();
    let done = done.expect("a finished shot");
    assert!(done.done);
    assert!(done.has_image(), "the job finished without an image");
    assert_eq!(done.bgra.len(), 32 * 32 * 4);
    assert_eq!(done.fraction(), 1.0);
    assert_eq!(done.taken, 24, "the render stopped short of its target");
    assert!(seq > 1, "progress was never published, only the result");

    // Cancelling stops a render early: it reaches nothing like its target.
    render_job::start(a_cube(), plan(4000));
    assert!(until(render_job::rendering));
    std::thread::sleep(std::time::Duration::from_millis(200));
    render_job::cancel();
    assert!(until(|| !render_job::rendering()), "cancel did not stop the render");
    let stopped = render_job::shot().0.expect("a shot");
    assert!(stopped.taken < 4000, "cancel was ignored: {} samples taken", stopped.taken);
    assert!(stopped.done && stopped.has_image(), "a cancelled render is finished, with the image as far as it got");
    assert_eq!(stopped.bgra.len(), 32 * 32 * 4);
    assert!(stopped.bgra.chunks_exact(4).any(|p| p[3] != 0 && (p[0], p[1], p[2]) != (0, 0, 0)), "the partial image is drawn, not blank");
    assert!(render_job::cancelled(), "an animation render reads this to stop at the frame");

    // A plain Esc is that cancel wherever it lands — the render window
    // (`render_view_cancel`), the Rendering workspace, a playing viewport —
    // as Blender's `G.is_break` is; with no render under way it closes the
    // window as before.
    use phiber::v2::app::keymap::{BKeystroke, action_for};
    use phiber::v2::view::prim::BAction;
    use phiber::v2::state::{BKey, BState, ModalKey, OpKey, val::V_MODAL_RENDER};
    let mut s = BState::new();
    s.set(ModalKey::Open, V_MODAL_RENDER);
    let esc = BKeystroke::parse("Escape").unwrap();
    assert_eq!(action_for(&esc, &s).unwrap().0, BAction::Set(ModalKey::Open.path(), String::new()), "no render: Esc closes the window");
    render_job::start(a_cube(), plan(4000));
    assert!(until(render_job::rendering));
    assert_eq!(action_for(&esc, &s).unwrap().0, BAction::Set(OpKey::Render.path(), "cancel".to_string()), "rendering: Esc cancels");
    let mut playing = BState::new();
    playing.set(phiber::v2::state::CtxKey::Playing, "true");
    assert_eq!(action_for(&esc, &playing).unwrap().0, BAction::Set(OpKey::Render.path(), "cancel".to_string()), "and in a playing Rendering workspace too, before it stops playback");
    // The Image editor's header shows the render's bar and a Cancel beside
    // the Render Result while it runs — not only the status bar's X.
    use phiber::v2::part::image::header::BImageBlock;
    use phiber::v2::view::{BView, BViewItem};
    let header = BImageBlock::new(phiber::v2::host::BHostType::Image).build(&s);
    let BView::Row { list, .. } = header.shown().clone() else { panic!("a row") };
    let cancel = list.iter().find_map(|v| match v {
        BView::Button(c) if c.label.as_deref() == Some("Cancel") => c.action.clone(),
        _ => None,
    });
    assert_eq!(cancel, Some(BAction::Set(OpKey::Render.path(), "cancel".to_string())), "a Cancel button in the header: {list:?}");
    render_job::cancel();
    assert!(until(|| !render_job::rendering()));
}

/// F12 follows the engine: `BLENDER_WORKBENCH` renders Solid with
/// `Scene.display.shading` and `render_aa` samples, no denoise, the camera
/// carrying the display's words; every other engine renders through the
/// path tracer at the panel's settings.
#[test]
fn a_workbench_file_plans_a_solid_render_from_the_displays_shading() {
    use phiber::phiphx::scene::{SceneDisplayShading, SceneModel};
    use phiber::v2::scene::{BRenderSet, render_plan};
    use phiber::v2::view::draw::canvas::view3d::rendered::BRayLook;
    let mut model = SceneModel::default();
    model.cameras.push(SceneCamera {
        name: "Camera".into(),
        location: [0.0, -4.0, 0.0],
        matrix_world: [[1.0, 0.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, -1.0, 0.0, 0.0], [0.0, -4.0, 0.0, 1.0]],
        kind: "PERSP".into(),
        focal_length_mm: 50.0,
        sensor_width: 36.0,
        ortho_scale: 6.0,
        clip_start: 0.1,
        clip_end: 100.0,
        display_size: 1.0,
        dof: Default::default(),
    });
    model.render.camera = "Camera".into();
    let look = BRayLook { sky: gpui::black(), grid: gpui::white(), grid_hide: 0, world: None, lights: Vec::new(), wire: None, view: [0.0, 0.0, 1.0, 0.0], environment: None, set: Default::default(), studio: false };
    let set = BRenderSet { samples: 64, denoise: true, ..Default::default() };
    let plain = gpui::hsla(0.0, 0.0, 0.8, 1.0);
    let cycles = render_plan(&model, set, &look, plain).expect("a camera");
    assert_eq!((cycles.look.shading, cycles.samples, cycles.denoise), (BShading::Rendered, 64, true));
    model.render.engine = "BLENDER_WORKBENCH".into();
    let mut display = SceneDisplayShading::default();
    display.solid_fx.shadow = true;
    display.solid_fx.cavity = true;
    display.solid_fx.cavity_type = "WORLD";
    display.solid_shade.light = "MATCAP";
    display.solid_shade.color_type = "TEXTURE";
    display.xray.solid = true;
    display.xray.alpha = 0.3;
    display.samples = 16;
    model.render.display = display;
    let wb = render_plan(&model, set, &look, plain).expect("a camera");
    assert_eq!((wb.look.shading, wb.samples, wb.denoise), (BShading::Solid, 16, false), "Workbench: Solid at render_aa, no denoise");
    assert!(wb.look.matcap.is_some() && wb.look.texture, "the matcap and the images go up");
    assert_eq!(wb.camera.shade[0], 1, "MatCap lighting");
    assert_eq!(wb.camera.shadow[3], 0.5, "the display's shadow at its intensity");
    assert_eq!(wb.camera.ssao[0], 16, "Cavity ▸ World's samples");
    assert!((wb.camera.see_through - 0.7).abs() < 1e-6, "X-ray at 0.3");
    assert_eq!(cycles.camera.shade, [0; 4], "the path tracer takes none of it");
    // Depth of Field (`use_dof`) wants the camera's own too: the display's
    // switch alone draws nothing; with the camera's on, `DofPass::init`'s
    // words — focused where the camera says, the sensor the frame's width.
    model.render.display.solid_fx.dof = true;
    assert_eq!(render_plan(&model, set, &look, plain).unwrap().camera.dof, [0.0; 4], "the camera's depth of field off");
    model.cameras[0].dof = phiber::phiphx::blend::BlendDof { on: true, fstop: 2.0, focus_distance: 5.0, ..Default::default() };
    let dof = render_plan(&model, set, &look, plain).unwrap().camera.dof;
    assert!((dof[0] - 0.0125 * (0.05 / 4.95)).abs() < 1e-7 && dof[1] == -5.0 && (dof[2] - 1.0 / 0.036).abs() < 1e-3 && dof[3] == -100.0, "{dof:?}");
    assert_eq!(cycles.camera.dof, [0.0; 4], "the path tracer blurs with its own aperture");
}
