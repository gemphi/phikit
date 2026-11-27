//! The salon in the library (`script/salon.py` from `data/once/salon.tsv`,
//! after the user's pictures of 28 November 2026): every piece is a
//! thing of Once City, opens in the 3D view and is drawn there; a seat
//! is as high as the other couches'; the lotus dome holds them. What the
//! view shows is kept beside the system's temporary files
//! (`phiogy-salon-<name>-<width>x<height>.bgra`, blue, green, red, solid),
//! to look at.
use std::path::{Path, PathBuf};
use appkit::pui::core::PMode;
use appkit::pui::state::PState;
use super::library::dice;
use super::{DESK, drawn};
use crate::part::figure::FILM;
const PIECE: [&str; 6] = ["salon-chair-red", "salon-sofa-red", "salon-chaise-red", "salon-meridienne-rose", "salon-chair-rose", "salon-sofa-rose"];
fn once(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("asset").join("once").join(format!("{name}.usda"))
}
#[test]
fn step_67_every_piece_of_the_salon_opens_in_the_view_and_is_drawn() {
    let (app, mut stage, look) = dice(DESK);
    let desk = app.desk();
    let source = crate::part::library::source(desk);
    for name in PIECE.iter().chain(["lotus-dome"].iter()) {
        source.set("ui/item", &format!("dir:{}", once(name).to_string_lossy()));
        let item = desk.library.get(&desk.shelf.get("ui/item")).unwrap_or_else(|| panic!("{name} in the library"));
        let model = desk.library.model(&item.path);
        assert!(model.count > 0 && !model.part.is_empty(), "{name} has no parts");
        stage.lay(DESK, PMode::of_width(DESK.width));
        drawn(&stage, &look);
        let frame = desk.library.film.frame(FILM).expect("drawn");
        // Not one colour: the piece stands in the frame.
        let first = &frame.pixel[..4];
        assert!(frame.pixel.chunks(4).any(|p| p != first), "{name} drew nothing");
        let file = std::env::temp_dir().join(format!("phiogy-salon-{name}-{}x{}.bgra", frame.width, frame.height));
        std::fs::write(file, frame.pixel.as_slice()).expect("kept");
    }
}
#[test]
fn a_seat_of_the_salon_is_as_high_as_the_other_couches() {
    let (app, _, _) = dice(DESK);
    let desk = app.desk();
    for name in PIECE {
        let model = desk.library.model(&once(name));
        // The seat's top: the highest point of the parts named seat. USD
        // stands up y, as Once City's tables do.
        let top = model.part.iter().filter(|part| part.name == "seat").flat_map(|part| part.piece.triangle.iter().flatten().map(|p| p[1])).fold(f32::MIN, f32::max);
        assert!((0.42..=0.5).contains(&top), "{name}: its seat's top at {top}");
    }
}
