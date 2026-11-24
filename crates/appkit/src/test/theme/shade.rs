use crate::pui::core::PColor;
use crate::theme::{PLook, PShade, PTheme};

/// A day look, and what a night look changes of it.
const DAY: &str = "color\tink\t#000000\nsize\tgap\t16\nword\tface\tPhi\n";
const NIGHT: &str = "color\tink\t#ffffff\n";

fn shade() -> PShade {
    let day = PLook::read(DAY);
    PShade::new(vec![("night".to_string(), day.with(NIGHT)), ("day".to_string(), day)])
}

#[test]
fn a_shade_is_its_first_look_until_it_is_turned_to_another() {
    let shade = shade();
    assert_eq!((shade.list(), shade.name()), (vec!["night", "day"], "night"));
    assert_eq!(shade.color("ink"), Some(PColor::new(1.0, 1.0, 1.0, 1.0)));
    assert!(shade.turn("day") && shade.name() == "day");
    assert_eq!(shade.color("ink"), Some(PColor::new(0.0, 0.0, 0.0, 1.0)));
    // A look it does not have changes nothing.
    assert!(!shade.turn("dusk") && shade.name() == "day");
}

#[test]
fn a_look_made_from_another_keeps_what_its_rows_do_not_change() {
    let shade = shade();
    assert_eq!((shade.size("gap"), shade.word("face"), shade.color("paper")), (Some(16.0), Some("Phi"), None));
    // A shade of no looks has nothing.
    let none = PShade::default();
    assert_eq!((none.name(), none.color("ink"), none.size("gap"), none.word("face")), ("", None, None, None));
}

#[test]
fn what_is_worn_takes_the_place_of_the_look_s_own_whichever_look_is_shown() {
    let shade = shade();
    let (night, plain) = (shade.color("window.fill"), shade.size("gap"));
    shade.wear("color\twindow.fill\t#102030\n");
    let worn = shade.color("window.fill");
    assert!(worn.is_some() && worn != night);
    // What it does not name is the look's; another look wears it too.
    assert_eq!(shade.size("gap"), plain);
    assert!(shade.turn("day"));
    assert_eq!(shade.color("window.fill"), worn);
    // A table with no rows takes it off.
    shade.wear("");
    assert_ne!(shade.color("window.fill"), worn);
}

#[test]
fn a_look_s_tinted_colours_are_mixed_with_the_light_s_and_follow_the_light_a_person_chooses() {
    // A black fill, a quarter of it the light's colour; the ink is not tinted.
    let look = PLook::read("color\twindow.fill\t#000000\ncolor\tink\t#000000\ncolor\tshade.light\t#ffffff\nword\ttint.token\twindow.fill\nsize\ttint.share\t0.25\n");
    let shade = PShade::new(vec![("dark".to_string(), look)]);
    assert_eq!(shade.color("window.fill"), Some(PColor::new(0.25, 0.25, 0.25, 1.0)));
    assert_eq!(shade.color("ink"), Some(PColor::new(0.0, 0.0, 0.0, 1.0)));
    // The light the person prefers colours the fill: a dark of that colour, not a bare black.
    shade.prefer("color shade.light #ff0000");
    assert_eq!(shade.color("window.fill"), Some(PColor::new(0.25, 0.0, 0.0, 1.0)));
}
