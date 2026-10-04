//! End to end: the Rendering workspace is Blender's — the Image editor with
//! `IMAGE_HT_header`'s View and Image menus (no Select or UV outside the UV
//! editor), New / Open until there is an image, and once a render has run
//! the Render Result in the window at `image_view_all`'s zoom with
//! `draw_render_info`'s bar over it, and the slot / view layer / pass /
//! channels dropdowns in the header.

use gpui::{TestAppContext, px, size};
use phiber::phiphx::scene::SceneCamera;
use phiber::v2::app::BPhiberApp;
use phiber::v2::scene::{BObject3D, BRenderPlan, BShading, camera_of, render_job};
use phiber::v2::view::draw::canvas::view3d::rendered::BStillLook;
use phiber::v2::space::BSpaceType;
use phiber::v2::state::CtxKey;

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
    BRenderPlan { camera: camera_of(&cam, 64, 32), w: 64, h: 32, samples, denoise: false, look: BStillLook::new(BShading::Rendered, gpui::hsla(0.0, 0.0, 0.8, 1.0)) }
}

fn selectors(cx: &mut gpui::VisualTestContext) -> Vec<String> {
    cx.all_debug_bounds().into_iter().map(|(selector, _)| selector).collect()
}

#[gpui::test]
async fn the_rendering_workspace_is_the_image_editor_with_blenders_header(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1400.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state.remove(phiber::v2::state::ModalKey::Open);
        app.state.set(CtxKey::Space, BSpaceType::Rendering.slug());
        cx.notify();
    });
    cx.run_until_parked();

    let window = cx.debug_bounds("image-window").expect("the Image editor window is painted");
    let header = cx.debug_bounds("region-Header-Image").expect("the Image editor header is painted");
    assert!(window.origin.y > header.origin.y);
    let buts = cx.debug_bounds("region-Header-Buts").expect("Properties runs down the right");
    let time = cx.debug_bounds("region-Header-Time").expect("a Timeline strip runs along the bottom");
    // No tool header above the header: Blender's Rendering workspace keeps
    // the Image editor's tool header hidden in View mode, so the header sits
    // level with the Properties header beside it …
    assert_eq!(header.origin.y, buts.origin.y, "the Image header is the first strip of its area");
    assert_eq!(window.origin.y, header.origin.y + header.size.height, "the window starts right under its header");
    // … and the window reaches the Timeline: no asset-shelf drawer reserving
    // an empty band between them.
    let gap = f32::from(time.origin.y) - f32::from(window.origin.y + window.size.height);
    let seam = phiber::v2::theme::ui_style_defaults(phiber::v2::theme::BPaletteName::DEFAULT).resize.gap;
    assert!(gap >= 0.0 && gap <= seam + 2.0, "the Image window ends a seam above the Timeline, not {gap} px (seam {seam})");
    let names = selectors(cx);
    let has = |needle: &str| names.iter().any(|s| s.contains(needle));
    assert!(has("ctx.modal-file_browser"), "without an image the header offers Open (the file browser): {names:?}");
    assert!(!has("ctx.menu.image.slot"), "no render slots before a render");
    assert!(has("ctx.menu.image.View") && has("ctx.menu.image.Image"), "the View and Image menus: {names:?}");
    assert!(!has("ctx.menu.image.Select") && !has("ctx.menu.image.UV"), "no Select or UV menu outside the UV editor");

    if phiray::Ray::gpu().is_none() {
        return;
    }
    render_job::start(
        vec![BObject3D::Primitive { name: "Cube".into(), shape: phiber::v2::scene::BShape::Cube, x: 0.0, y: 0.0 }],
        plan(4),
    );
    let mut ready = false;
    for _ in 0..600 {
        if render_job::shot().0.is_some_and(|shot| shot.done && shot.has_image()) {
            ready = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    assert!(ready, "the render finishes");
    app.update(cx, |_, cx| cx.notify());
    cx.run_until_parked();

    let names = selectors(cx);
    let has = |needle: &str| names.iter().any(|s| s.contains(needle));
    assert!(!has("ctx.modal-file_browser"), "the Open button gives way to the Render Result data-block");
    assert!(
        has("ctx.menu.image.slot") && has("ctx.menu.image.view_layer") && has("ctx.menu.image.pass") && has("ctx.menu.image.channels"),
        "slot, view layer, pass and display-channel dropdowns: {names:?}"
    );
    // The picture: a 64 × 32 render fits the window, so it shows at 1:1,
    // centred; the info bar is a scrim across the top of the window
    // (`draw_render_info`).
    let scale = cx.update(|window, _| window.scale_factor());
    let painted = cx.scene_fingerprint();
    let picture = painted.iter().find(|line| line.starts_with("image ")).expect("the render is painted");
    let sizes: Vec<f32> = picture.split(' ').nth(2).unwrap().split('x').map(|v| v.parse::<f32>().unwrap() / scale).collect();
    assert_eq!(sizes, vec![64.0, 32.0], "1:1 — `image_view_all` keeps a fitting image at its size: {picture}");
    // Found by geometry, not by colour. Blender lays a quarter-black over the
    // picture; `paint_info` lays the theme's own dim there instead, so the
    // scrim follows the palette — and a test that pinned `l: 0.0, a: 0.25`
    // pinned the one thing about the bar that is meant to change.
    let height = phiber::v2::view::draw::canvas::image::INFO_BAR_HEIGHT;
    // `quad <x>,<y> <w>x<h> clip …`
    let top = |line: &str| line.split(' ').nth(1).unwrap().split(',').nth(1).unwrap().parse::<f32>().unwrap() / scale;
    let size_of = |line: &str, n: usize| line.split(' ').nth(2).unwrap().split('x').nth(n).unwrap().parse::<f32>().unwrap() / scale;
    let bar = painted
        .iter()
        .find(|line| {
            line.starts_with("quad ")
                && (size_of(line, 1) - height).abs() < 1.0
                && (top(line) - f32::from(window.origin.y)).abs() < 1.0
                && (size_of(line, 0) - f32::from(window.size.width)).abs() < 1.0
        })
        .unwrap_or_else(|| panic!("the render info bar, {height} px across the top of the window"));
    assert!(bar.starts_with("quad "), "{bar}");
}
