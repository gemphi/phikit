//! Saving a model from the 3D view: one package (.usdz) of it, turned as
//! the hand turned it, and the person asked where a copy is to be kept.

use appkit::pui::core::{PEvent, PMark, PMode, PPoint};
use appkit::pui::window::PDeed;

use super::library::dice;
use super::{DESK, drawn};
use crate::part::figure::FILM;

#[test]
fn step_65_save_keeps_the_model_as_a_usdz_turned_as_it_is_seen() {
    let (_app, mut stage, look) = dice(DESK);
    let tool = |stage: &appkit::pui::window::PStage, name: &str| drawn(stage, &look).iter().rev().find_map(|mark| match mark {
        PMark::Icon { name: known, rect, .. } if known == name => Some(rect.center()),
        _ => None,
    }).expect(name);
    let save = |stage: &mut appkit::pui::window::PStage| {
        assert!(stage.send(&PEvent::PointerDown(tool(stage, "download"))));
        let asked = stage.errand();
        let [PDeed::Save(file)] = asked.as_slice() else { panic!("{asked:?}") };
        assert!(file.ends_with("dice.usdz"), "{file}");
        std::fs::read(file).expect("the package")
    };
    let first = save(&mut stage);
    // A zip, the dice's own layer first, turned.
    assert_eq!(&first[..4], &0x0403_4b50u32.to_le_bytes());
    let name = u16::from_le_bytes([first[26], first[27]]) as usize;
    assert_eq!(&first[30..30 + name], b"dice.usda");
    assert!(String::from_utf8_lossy(&first).contains("xformOp:transform:turn"));
    // Turned by a drag across the view, it is kept turned otherwise.
    let view = drawn(&stage, &look).iter().find_map(|mark| match mark {
        PMark::Picture { name, rect, .. } if name == FILM => Some(*rect),
        _ => None,
    }).expect("the view");
    let at = view.center();
    stage.send(&PEvent::PointerDown(at));
    stage.send(&PEvent::PointerMove(PPoint::new(at.x + view.size.width / 4.0, at.y)));
    stage.send(&PEvent::PointerUp(PPoint::new(at.x + view.size.width / 4.0, at.y)));
    stage.lay(DESK, PMode::of_width(DESK.width));
    drawn(&stage, &look);
    assert_ne!(save(&mut stage), first);
}
