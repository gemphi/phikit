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
fn a_look_s_tinted_colours_take_the_light_s_hue_keep_their_lightness_and_follow_the_light_a_person_chooses() {
    use crate::theme::lightness;
    // A dark grey fill, a quarter of it the light's colour; the ink is not tinted.
    let look = PLook::read("color\twindow.fill\t#404040\ncolor\tink\t#000000\ncolor\tshade.light\t#ffffff\nword\ttint.token\twindow.fill\nsize\ttint.share\t0.25\n");
    let shade = PShade::new(vec![("dark".to_string(), look)]);
    let grey = 64.0 / 255.0;
    let near = |one: f32, other: f32| (one - other).abs() < 0.001;
    // A white light gives no hue, and no lightness either: the grey stays.
    let fill = shade.color("window.fill").expect("a fill");
    assert!(near(fill.r, grey) && near(fill.g, grey) && near(fill.b, grey), "{fill:?}");
    assert_eq!(shade.color("ink"), Some(PColor::new(0.0, 0.0, 0.0, 1.0)));
    // The light the person prefers colours the fill: a dark of that colour,
    // as light as the grey was, never lighter (the user, 25 November 2026).
    shade.prefer("color shade.light #ff0000");
    let fill = shade.color("window.fill").expect("a fill");
    assert!(fill.r > fill.g && near(fill.g, fill.b) && near(lightness(fill), lightness(PColor::new(grey, grey, grey, 1.0))), "{fill:?}");
}

#[test]
fn a_look_of_a_table_begins_from_another_and_changes_what_it_says() {
    let base = PLook::read("color\tink\t#000000\nsize\tblur\t1\n");
    let table = "# Looks.\nlight\ndark\tcolor\tink\t#ffffff\ndark\tsize\tblur\t8\nnight\tfrom\tdark\nnight\tsize\tblur\t20\n";
    let shade = PShade::read(&base, table);
    assert_eq!(shade.list(), ["light", "dark", "night"]);
    // Night is Dark's ink, with a blur of its own.
    assert!(shade.turn("night"));
    assert_eq!((shade.color("ink"), shade.size("blur")), (Some(PColor::new(1.0, 1.0, 1.0, 1.0)), Some(20.0)));
    // Dark keeps its own; Light is the theme as it is.
    assert!(shade.turn("dark") && shade.size("blur") == Some(8.0));
    assert!(shade.turn("light") && shade.color("ink") == Some(PColor::new(0.0, 0.0, 0.0, 1.0)));
}

