//! The Render menu's entries do what Blender's do — on a fresh session
//! with no file open as well: Render Image renders the default scene
//! through the startup camera, View Render shows the window without a new
//! render and puts it away again, View Animation plays the frames an
//! animation render wrote, Render Audio… writes the mixdown the browser
//! names, and Lock Interface holds edits back while a render runs. Its
//! own binary, since the render job is one per process.

use gpui::{TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::part::file::purpose::BFilePurpose;
use phiber::v2::scene::render_job;
use phiber::v2::state::{BKey as _, ButsKey, CtxKey, ModalKey, OpKey, ScreenKey, val::{V_MODAL_RENDER, V_TRUE}};
use phiber::v2::view::BAction;

fn render(value: &str) -> BAction {
    BAction::Set(OpKey::Render.path(), value.into())
}

fn modal(app: &gpui::Entity<BPhiberApp>, cx: &gpui::VisualTestContext) -> Option<String> {
    app.read_with(cx, |a, _| a.state.get(ModalKey::Open).filter(|m| !m.is_empty()).map(str::to_string))
}

fn settle(cx: &mut gpui::VisualTestContext) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    while render_job::rendering() && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(50));
        cx.executor().advance_clock(std::time::Duration::from_millis(8));
        cx.run_until_parked();
    }
    assert!(!render_job::rendering(), "the render never finished");
}

#[gpui::test]
async fn the_render_menu_works_on_a_fresh_session(cx: &mut TestAppContext) {
    if phiray::Ray::gpu().is_none() {
        return;
    }
    let dir = std::env::temp_dir().join(format!("phiber-render-menu-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1400.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state.remove(ModalKey::Open);
        // Where Output points: a folder of this test's own, two frames.
        app.state.set(ButsKey::Value("filepath"), dir.join("frame_####").to_string_lossy().to_string());
        app.state.set(ButsKey::Value("cycles.samples"), phiber::v2::state::val::BValue::Int(1).encode());
        app.state.set(CtxKey::FrameStart, "1");
        app.state.set(CtxKey::FrameEnd, "2");
        cx.notify();
    });
    cx.run_until_parked();

    // View Render (F11) with nothing rendered: the window, empty, and no
    // render started; F11 again puts it away.
    app.update(cx, |app, cx| app.handle_action(&render("view"), cx));
    cx.run_until_parked();
    assert_eq!(modal(&app, cx).as_deref(), Some(V_MODAL_RENDER));
    assert!(render_job::shot().0.is_none() && !render_job::rendering(), "View Render renders nothing");
    app.update(cx, |app, cx| app.handle_action(&render("view"), cx));
    assert_eq!(modal(&app, cx), None, "F11 on the open window closes it");

    // Render Image (F12) with no file open: the default scene through the
    // startup camera, into the window.
    app.update(cx, |app, cx| app.handle_action(&render("image"), cx));
    cx.run_until_parked();
    assert_eq!(modal(&app, cx).as_deref(), Some(V_MODAL_RENDER));
    let (shot, _) = render_job::shot();
    let shot = shot.expect("F12 renders the default scene");
    assert!(shot.w > 0 && shot.h > 0);

    // Lock Interface while it renders: an edit does not go through; the
    // view and the render's own cancel do.
    app.update(cx, |app, cx| {
        app.state.set(ScreenKey::LockInterface, V_TRUE);
        app.state.set(CtxKey::Frame, "1");
        let before = app.state.get(CtxKey::Frame).map(str::to_string);
        if render_job::rendering() {
            app.handle_action(&BAction::Set(CtxKey::Frame.path(), "7".into()), cx);
            assert_eq!(app.state.get(CtxKey::Frame).map(str::to_string), before, "locked: the frame does not change");
            app.handle_action(&BAction::Set(ModalKey::Open.path(), String::new()), cx);
            assert_eq!(app.state.get(ModalKey::Open).map(str::to_string), Some(String::new()), "the window may still close");
        }
        app.state.remove(ScreenKey::LockInterface);
    });
    settle(cx);
    app.update(cx, |app, cx| app.handle_action(&BAction::Set(CtxKey::Frame.path(), "1".into()), cx));
    assert_eq!(app.read_with(cx, |a, _| a.state.get(CtxKey::Frame).map(str::to_string)).as_deref(), Some("1"), "unlocked: edits go through");
    let (result, _) = render_job::shot();
    let result = result.expect("the Render Result");
    assert!(result.done && result.has_image());

    // Render slots: the header's Slot 2 is empty but the Render Result
    // stays the data-block; F12 renders into it — at half the size, so the
    // two can be told apart — and Slot 1 keeps the first render. J and
    // Alt J step between the two slots that hold one.
    use phiber::v2::scene::render_slot::BRenderSlot;
    use phiber::v2::state::ImageKey;
    app.update(cx, |app, cx| app.handle_action(&BAction::Set(ImageKey::Slot.path(), "Slot 2".into()), cx));
    assert_eq!(BRenderSlot::active(), 1);
    assert!(render_job::shot().0.is_none() && BRenderSlot::any_filled(), "an empty slot of an existing Render Result");
    app.update(cx, |app, cx| {
        app.state.set(ButsKey::Value("resolution_percentage"), phiber::v2::state::val::BValue::Int(50).encode());
        app.handle_action(&render("image"), cx);
    });
    cx.run_until_parked();
    settle(cx);
    let (second, _) = render_job::shot();
    let second = second.expect("rendered into Slot 2");
    assert_eq!((second.w, second.h), (result.w / 2, result.h / 2), "the second render, half size, in the second slot");
    assert_eq!(BRenderSlot::last(), 1);
    app.update(cx, |app, cx| app.handle_action(&BAction::Set(ImageKey::SlotCycle.path(), "next".into()), cx));
    assert_eq!((BRenderSlot::active(), app.read_with(cx, |a, _| a.state.get(ImageKey::Slot).map(str::to_string))), (0, Some("Slot 1".to_string())), "J: the next slot with a render, the dropdown with it");
    let (first_again, _) = render_job::shot();
    assert_eq!(first_again.map(|s| (s.w, s.h)), Some((result.w, result.h)), "Slot 1 kept the first render");
    app.update(cx, |app, cx| app.handle_action(&BAction::Set(ImageKey::SlotCycle.path(), "previous".into()), cx));
    assert_eq!(BRenderSlot::active(), 1, "Alt J: back to the second");
    app.update(cx, |app, cx| {
        app.state.set(ButsKey::Value("resolution_percentage"), phiber::v2::state::val::BValue::Int(100).encode());
        app.handle_action(&BAction::Set(ImageKey::Slot.path(), "Slot 1".into()), cx);
    });
    let (result, _) = render_job::shot();
    let result = result.expect("the first render, shown again");

    // Render Region: the right half's upper half, cropped — F12 fills a
    // quarter-size result; without Crop, the frame's size with the rest
    // black; Clear puts the whole frame back.
    let region = |v: &str| BAction::Set(OpKey::RenderBorder.path(), v.into());
    app.update(cx, |app, cx| {
        app.handle_action(&region("set:0.5,1,0.5,1"), cx);
        app.handle_action(&BAction::Set(ButsKey::Value("use_crop_to_border").path(), phiber::v2::state::val::BValue::Bool(true).encode()), cx);
        app.handle_action(&render("image"), cx);
    });
    cx.run_until_parked();
    settle(cx);
    let (quarter, _) = render_job::shot();
    let quarter = quarter.expect("the cropped render");
    assert_eq!((quarter.w, quarter.h), (result.w / 2, result.h / 2), "cropped to the region");
    app.update(cx, |app, cx| {
        app.handle_action(&BAction::Set(ButsKey::Value("use_crop_to_border").path(), phiber::v2::state::val::BValue::Bool(false).encode()), cx);
        app.handle_action(&render("image"), cx);
    });
    cx.run_until_parked();
    settle(cx);
    let (framed, _) = render_job::shot();
    let framed = framed.expect("the framed render");
    assert_eq!((framed.w, framed.h), (result.w, result.h), "the frame's size");
    assert_eq!(&framed.bgra[..4], &[0, 0, 0, 0], "outside the region: black, alpha 0");
    let inside = 4 * ((framed.h as usize / 4) * framed.w as usize + framed.w as usize * 3 / 4);
    assert_ne!(framed.bgra[inside + 3], 0, "inside the region: drawn");
    app.update(cx, |app, cx| {
        app.handle_action(&region("clear"), cx);
        app.handle_action(&render("image"), cx);
    });
    cx.run_until_parked();
    settle(cx);
    let (result, before_play) = render_job::shot();
    let result = result.expect("the whole frame again");
    assert_ne!(&result.bgra[..4], &[0, 0, 0, 0], "cleared: the corner is drawn");

    // View Animation (Ctrl F11) with no frames written: Blender's warning.
    app.update(cx, |app, cx| app.handle_action(&render("play"), cx));
    cx.run_until_parked();
    let reports = app.read_with(cx, |a, _| phiber::v2::part::text::info::reports(&a.state));
    assert!(reports.iter().any(|(_, m)| m.starts_with("File '") && m.contains("frame_0001.png' not found")), "{reports:?}");
    assert_eq!(render_job::shot().1, before_play, "nothing shown");

    // Two frames written where Output points: they play in the render
    // window in turn, and the Render Result comes back when it closes.
    for (frame, grey) in [(1u32, 40u8), (2, 200)] {
        image::RgbaImage::from_pixel(6, 4, image::Rgba([grey, grey, grey, 255])).save(dir.join(format!("frame_{frame:04}.png"))).unwrap();
    }
    app.update(cx, |app, cx| {
        app.state.set(ModalKey::Open, String::new());
        app.handle_action(&render("play"), cx);
    });
    cx.run_until_parked();
    assert_eq!(modal(&app, cx).as_deref(), Some(V_MODAL_RENDER), "the player is the render window");
    let mut seen = std::collections::BTreeSet::new();
    for _ in 0..12 {
        cx.executor().advance_clock(std::time::Duration::from_millis(45));
        cx.run_until_parked();
        if let (Some(shot), _) = render_job::shot() {
            if shot.w == 6 {
                seen.insert(shot.bgra[0]);
            }
        }
    }
    assert_eq!(seen.into_iter().collect::<Vec<_>>(), vec![40, 200], "both frames shown, 6 × 4 as written");
    app.update(cx, |app, cx| app.handle_action(&BAction::Set(ModalKey::Open.path(), String::new()), cx));
    for _ in 0..4 {
        cx.executor().advance_clock(std::time::Duration::from_millis(45));
        cx.run_until_parked();
    }
    let (back, _) = render_job::shot();
    let back = back.expect("the Render Result is back");
    assert_eq!((back.w, back.h), (result.w, result.h), "the player put the render back");

    // Render Audio…: the browser's commit writes the range's silence.
    let target = dir.join("mix.wav");
    app.update(cx, |app, cx| app.handle_file_commit(&target.to_string_lossy(), BFilePurpose::Mixdown, cx));
    // The encode is on the background pool since batch 149, so the write
    // lands when the executor is driven, not inside the click.
    cx.run_until_parked();
    let bytes = std::fs::read(&target).expect("the mixdown");
    // Two frames at 24 fps: a twelfth of a second of 16-bit stereo at 48 kHz.
    assert_eq!(bytes.len(), 44 + 4000 * 4);
    assert_eq!(&bytes[0..4], b"RIFF");
    let _ = std::fs::remove_dir_all(&dir);
}
