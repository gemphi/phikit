//! The Image editor's images beside the Render Result: Image ▸ Open… reads
//! a file into the session and shows it, New… makes one from its dialog,
//! the Image menu's Invert / Flip / Rotate change the one shown, Copy
//! puts it on the clipboard, Reload reads it again and Save As writes it.

use gpui::{TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::part::file::purpose::BFilePurpose;
use phiber::v2::part::image::shown::BShownImage;
use phiber::v2::scene::image_list::BImageList;
use phiber::v2::state::{BKey as _, ImageKey, ModalKey, OpKey, val::{BValue, V_MODAL_NEW_IMAGE, V_TRUE}};
use phiber::v2::view::BAction;
use phiber::v2::view::draw::canvas::image::BImageView;

#[gpui::test]
async fn an_opened_image_is_shown_changed_copied_and_saved(cx: &mut TestAppContext) {
    let dir = std::env::temp_dir().join(format!("phiber-images-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    // A 4 × 2 picture: the left half red, the right half blue.
    let file = dir.join("halves.png");
    image::RgbaImage::from_fn(4, 2, |x, _| if x < 2 { image::Rgba([255, 0, 0, 255]) } else { image::Rgba([0, 0, 255, 255]) }).save(&file).unwrap();
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1400.0), px(800.0)));
    app.update(cx, |app, _| {
        app.state.remove(ModalKey::Open);
        app.handle_file_commit(&file.to_string_lossy(), BFilePurpose::OpenImage);
    });
    let shown = app.read_with(cx, |a, _| BShownImage::now(&a.state));
    assert_eq!(shown.name(), "halves.png", "named as the file, and shown");
    assert_eq!(BShownImage::name_list(), vec!["halves.png".to_string()], "no render yet: the image alone in the header's list");
    let view = app.read_with(cx, |a, _| BImageView::now(&a.state));
    assert_eq!(view.image(), Some((4, 2)), "the canvas paints the picture at its size");
    assert!(view.status.is_none(), "no render bar over a picture");
    let pixel = |name: &str, x: usize, y: usize| BImageList::get(name).map(|i| i.bgra[4 * (y * i.w as usize + x)..4 * (y * i.w as usize + x) + 4].to_vec()).unwrap();
    assert_eq!(pixel("halves.png", 0, 0), vec![0, 0, 255, 255], "BGRA: red first");

    // Flip Horizontally: blue leads now; Invert Red Channel on the blue
    // pixel gives magenta; Rotate 90° Clockwise swaps the sides.
    let op = |slug: &str| BAction::Set(ImageKey::Op.path(), slug.into());
    app.update(cx, |app, cx| app.handle_action(&op("flip:x"), cx));
    assert_eq!(pixel("halves.png", 0, 0), vec![255, 0, 0, 255], "flipped: the blue half leads");
    app.update(cx, |app, cx| app.handle_action(&op("invert:r"), cx));
    assert_eq!(pixel("halves.png", 0, 0), vec![255, 0, 255, 255], "red inverted on blue: magenta");
    app.update(cx, |app, cx| app.handle_action(&op("rotate:90"), cx));
    let turned = BImageList::get("halves.png").unwrap();
    assert_eq!((turned.w, turned.h), (2, 4), "a quarter turn swaps the sides");
    // Top-left before (magenta) lands top-right.
    assert_eq!(pixel("halves.png", 1, 0), vec![255, 0, 255, 255]);

    // Copy: the picture on the clipboard as a PNG.
    app.update(cx, |app, cx| app.handle_action(&BAction::Set(OpKey::ImageCopy.path(), String::new()), cx));
    let copied = cx.read_from_clipboard().expect("something on the clipboard");
    let png = copied.entries().iter().find_map(|e| match e {
        gpui::ClipboardEntry::Image(img) => Some(img.bytes.clone()),
        _ => None,
    });
    let png = image::load_from_memory(&png.expect("an image entry")).unwrap().to_rgba8();
    assert_eq!(png.dimensions(), (2, 4));
    assert_eq!(png.get_pixel(1, 0).0, [255, 0, 255, 255], "RGBA on the clipboard, as changed");

    // Save As: written where the browser pointed, in the extension's format.
    let out = dir.join("turned.png");
    app.update(cx, |app, _| phiber::v2::part::file::save_image::BImageSave::to(&mut app.state, &out.to_string_lossy()));
    let saved = image::open(&out).unwrap().to_rgba8();
    assert_eq!((saved.dimensions(), saved.get_pixel(1, 0).0), ((2, 4), [255, 0, 255, 255]));

    // Reload: the file's pixels again, the turn undone.
    app.update(cx, |app, cx| app.handle_action(&op("reload"), cx));
    let back = BImageList::get("halves.png").unwrap();
    assert_eq!((back.w, back.h), (4, 2));
    assert_eq!(pixel("halves.png", 0, 0), vec![0, 0, 255, 255]);

    // New…: the dialog's OK makes a UV grid and shows it; the header lists both.
    app.update(cx, |app, cx| {
        app.handle_action(&BAction::Set(ModalKey::Open.path(), V_MODAL_NEW_IMAGE.into()), cx);
        app.handle_action(&BAction::Set(ImageKey::New("width").path(), BValue::Int(64).encode()), cx);
        app.handle_action(&BAction::Set(ImageKey::New("height").path(), BValue::Int(64).encode()), cx);
        app.handle_action(&BAction::Set(ImageKey::New("type").path(), "UV Grid".into()), cx);
        app.handle_action(&BAction::Set(ImageKey::New("commit").path(), V_TRUE.into()), cx);
    });
    let shown = app.read_with(cx, |a, _| BShownImage::now(&a.state));
    assert_eq!(shown.name(), "Untitled");
    let grid = BImageList::get("Untitled").unwrap();
    assert_eq!((grid.w, grid.h), (64, 64));
    assert_eq!(&grid.bgra[..4], &[64, 64, 64, 255], "the UV grid's dark check at the top left");
    assert_eq!(BShownImage::name_list(), vec!["halves.png".to_string(), "Untitled".to_string()]);
    assert_eq!(app.read_with(cx, |a, _| a.state.get(ModalKey::Open).map(str::to_string)).as_deref(), Some(""), "the dialog closed");
    // Back to the first by name, as the header's dropdown does.
    app.update(cx, |app, cx| app.handle_action(&BAction::Set(ImageKey::Shown.path(), "halves.png".into()), cx));
    assert_eq!(app.read_with(cx, |a, _| BImageView::now(&a.state)).image(), Some((4, 2)));
    let _ = std::fs::remove_dir_all(&dir);
}
