use crate::pui::core::*;
use crate::screen::play::inside;

#[test]
fn a_clip_holds_the_marks_up_to_its_own_end() {
    let (rect, fill) = (PRect::new(0.0, 0.0, 10.0, 10.0), PMark::Fill { rect: PRect::default(), radius: 0.0, color: PColor::default() });
    // After a clip has begun: a fill, a clip inside it with a fill, its
    // end, a fill, then the end of the first, and a fill outside it.
    let mark = [fill.clone(), PMark::Clip(rect), fill.clone(), PMark::Unclip, fill.clone(), PMark::Unclip, fill.clone()];
    assert_eq!(inside(&mark), 5);
    // A clip that is never ended holds everything after it.
    assert_eq!(inside(&mark[..3]), 3);
}

#[test]
fn a_picture_s_file_is_decoded_as_the_renderer_keeps_it() {
    use crate::screen::album::decode;
    // A picture two across and one high: red, then blue, written as a
    // PNG file.
    let picture = image::RgbaImage::from_raw(2, 1, vec![255, 0, 0, 255, 0, 0, 255, 255]).expect("two pixels");
    let mut file = std::io::Cursor::new(Vec::new());
    picture.write_to(&mut file, image::ImageFormat::Png).expect("a PNG");
    let decoded = decode(file.get_ref()).expect("it is a picture");
    assert_eq!((decoded.size(0).width.0, decoded.size(0).height.0), (2, 1));
    // The renderer keeps blue first: red and blue have changed places.
    assert_eq!(decoded.as_bytes(0), Some(&[0u8, 0, 255, 255, 255, 0, 0, 255][..]));
    // What is no picture is none.
    assert!(decode(b"not a picture").is_none());
}

#[test]
fn a_copy_kept_under_a_name_with_no_ending_is_given_the_file_s_own() {
    use crate::screen::errand::place;
    use std::path::{Path, PathBuf};
    let from = Path::new("art/one.png");
    let folder = std::env::temp_dir();
    // A system that hides endings gives back a name without one.
    assert_eq!(place(from, folder.join("appkit-no-such-copy")), folder.join("appkit-no-such-copy.png"));
    // An ending the person gave is theirs; a file with none gives none.
    assert_eq!(place(from, PathBuf::from("kept/copy.jpg")), PathBuf::from("kept/copy.jpg"));
    assert_eq!(place(Path::new("art/one"), PathBuf::from("kept/copy")), PathBuf::from("kept/copy"));
    // Nothing that stands there already is written over unasked: this
    // file's own folder has a `mod.rs`.
    let here = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/test/screen/mod");
    assert_eq!(place(Path::new("any.rs"), here.clone()), here);
}

#[test]
fn an_album_knows_a_film_s_pictures_by_their_names() {
    use crate::screen::album::{Album, frame};
    let film = PFilm::default();
    let album = Album::of(Some(film.clone()));
    assert!(!album.moving() && album.size("far").is_none());
    // Two across, one high: blue, then red, as the renderer keeps them.
    film.show("far", 2, 1, vec![255, 0, 0, 255, 0, 0, 255, 255]);
    assert!(album.moving());
    assert_eq!(album.size("far"), Some((2.0, 1.0)));
    let made = frame(&film.frame("far").expect("it was shown")).expect("a frame of its own size");
    assert_eq!(made.as_bytes(0), Some(&[255u8, 0, 0, 255, 0, 0, 255, 255][..]));
    // Ended, it is no picture; and nothing was drawn, so nothing is stale.
    film.end("far");
    assert!(!album.moving() && album.size("far").is_none() && album.stale().is_empty());
    // An album with no film has no moving pictures.
    assert!(!Album::of(None).moving());
}
