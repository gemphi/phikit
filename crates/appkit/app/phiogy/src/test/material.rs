//! Materials in the library: the inspector shows the kit's sixteen as
//! balls, one chosen is worn by the part, and a press on the model in the
//! 3D view picks the part under it.

use appkit::pui::core::{PEvent, PMark, PMode};
use appkit::pui::state::PState;

use super::library::dice;
use super::{DESK, drawn};
use crate::library::Attribute;
use crate::part::figure::FILM;

#[test]
fn step_66_a_part_wears_a_material_chosen_from_its_balls_and_is_picked_in_the_view() {
    let (app, mut stage, look) = dice(DESK);
    let desk = app.desk();
    // The inspector's material tiles: a ball each, drawn in the film.
    let ball: Vec<String> = drawn(&stage, &look).iter().filter_map(|mark| match mark {
        PMark::Picture { name, .. } if name.starts_with("material/") => Some(name.clone()),
        _ => None,
    }).collect();
    assert_eq!(desk.library.material.len(), 16);
    assert!(!ball.is_empty() && ball.iter().all(|name| desk.library.film.frame(name).is_some()), "{ball:?}");
    // One chosen for the dice's part: its pieces wear it, in its colour.
    let item = desk.library.get(&desk.shelf.get("ui/item")).expect("the dice");
    let source = crate::part::library::source(desk);
    source.set(&format!("look/{}/0/material", item.key), "milanese");
    let piece = desk.library.dressed(&item, &Attribute::new(desk.choice.clone()));
    let milanese = desk.library.material("milanese").expect("milanese");
    assert_eq!(piece[0].material.as_deref(), Some(milanese.as_ref()));
    assert_eq!(piece[0].colour, milanese.colour);
    // Its controls, as Reality Composer Pro's: they stand at its own
    // roughness and metal, and what is set is worn.
    let at = |name: &str| format!("look/{}/0/{name}", item.key);
    assert_eq!((source.get(&at("rough")), source.get(&at("metal")), source.get(&at("tiling"))), ("0.35".to_string(), "1.00".to_string(), "1".to_string()));
    source.set(&at("rough"), "0.80");
    source.set(&at("tiling"), "2.00");
    let tuned = desk.library.dressed(&item, &Attribute::new(desk.choice.clone()));
    let worn = tuned[0].material.as_deref().expect("worn");
    assert_eq!((worn.rough, worn.scale), (0.8, milanese.scale * 2.0));
    stage.lay(DESK, PMode::of_width(DESK.width));
    let slider = drawn(&stage, &look).iter().filter(|mark| matches!(mark, PMark::Text { text, .. } if text == "0.80")).count();
    assert!(slider >= 1, "the roughness slider says its value");
    // A press on the middle of the view picks the part there.
    stage.lay(DESK, PMode::of_width(DESK.width));
    let view = drawn(&stage, &look).iter().find_map(|mark| match mark {
        PMark::Picture { name, rect, .. } if name == FILM => Some(*rect),
        _ => None,
    }).expect("the view");
    desk.shelf.set("ui/part", "");
    stage.send(&PEvent::PointerDown(view.center()));
    assert!(stage.send(&PEvent::PointerUp(view.center())));
    assert_eq!(desk.shelf.get("ui/part"), "0");
    // A press beside it picks nothing.
    desk.shelf.set("ui/part", "");
    let corner = appkit::pui::core::PPoint::new(view.origin.x + 4.0, view.origin.y + 4.0);
    stage.send(&PEvent::PointerDown(corner));
    stage.send(&PEvent::PointerUp(corner));
    assert_eq!(desk.shelf.get("ui/part"), "");
}

/// A couch of the drawing dressed in materials in the 3D view: its frame
/// walnut, its cushions quilted, its rings and feet gold links. What the
/// view shows is kept beside the system's temporary files (blue, green,
/// red, solid, its width and height in the name), to look at.
#[test]
fn a_couch_dressed_in_materials_is_drawn_in_them() {
    let (app, mut stage, look) = dice(DESK);
    let desk = app.desk();
    let couch = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("asset/once/couch-scroll-2.usda");
    let source = crate::part::library::source(desk);
    source.set("ui/item", &format!("dir:{}", couch.to_string_lossy()));
    let item = desk.library.get(&desk.shelf.get("ui/item")).expect("the couch");
    let plain = |stage: &mut appkit::pui::window::PStage| {
        stage.lay(DESK, PMode::of_width(DESK.width));
        drawn(stage, &look);
        desk.library.film.frame(FILM).expect("drawn")
    };
    let before = plain(&mut stage);
    for (index, part) in desk.library.model(&item.path).part.iter().enumerate() {
        let wear = match () {
            _ if part.name.contains("seat") => "quilted",
            _ if ["ring", "foot", "bead", "head"].iter().any(|n| part.name.contains(n)) => "link-bracelet",
            _ => "walnut",
        };
        source.set(&format!("look/{}/{index}/material", item.key), wear);
    }
    desk.shelf.set("ui/part", "");
    let after = plain(&mut stage);
    assert_ne!(after.pixel, before.pixel);
    let file = std::env::temp_dir().join(format!("phiogy-couch-{}x{}.bgra", after.width, after.height));
    std::fs::write(file, after.pixel.as_slice()).expect("kept");
}
