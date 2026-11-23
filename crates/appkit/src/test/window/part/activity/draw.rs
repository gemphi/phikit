use super::{BAR, LOOK, SIDE, USER, bar, picked};
use crate::pui::core::*;
use crate::pui::window::part::initial;
use crate::pui::window::*;
use crate::service::PTable;
use crate::test::window::window;
use crate::theme::{PLook, PTheme};

fn icon(sheet: &PSheet) -> Vec<String> {
    let name = |mark: &PMark| match mark {
        PMark::Icon { name, .. } => Some(name.clone()),
        _ => None,
    };
    sheet.mark.iter().filter_map(name).collect()
}

#[test]
fn it_draws_the_mark_its_icons_the_user_and_the_menu_shown_under_the_app_s_name() {
    let look = PLook::read(LOOK);
    let mut sheet = PSheet::new(&look);
    bar().draw(BAR, SIDE, &picked("layout"), None, &mut sheet);
    // The bar's icons, the shape the user's mark stands on, the button
    // that puts the menu away, the menu's.
    assert_eq!(icon(&sheet), ["grid", "palette", "box", "book", "soft-fill", "panel-left", "plus", "folder", "settings"]);
    // With no logo yet the mark is the first letter of the app's name.
    assert_eq!(sheet.text_list(), ["B", "AL", "Book", "New", "Open", "Preferences"]);
    // With no menu shown there is the bar alone.
    let mut alone = PSheet::new(&look);
    bar().draw(BAR, PRect::default(), &picked(""), None, &mut alone);
    assert_eq!((icon(&alone), alone.text_list()), (["grid", "palette", "box", "book", "soft-fill"].map(str::to_string).to_vec(), vec!["B", "AL"]));
    // A mark with an icon draws it in place of the letter; lying there
    // is no mark at all.
    let mut logo = PSheet::new(&look);
    PActivity::read("book\tBook\tfeather\thead\n").with(&PTable::read(USER)).draw(BAR, PRect::default(), &window(), None, &mut logo);
    assert_eq!((icon(&logo), logo.text_list()), (vec!["feather".to_string(), "soft-fill".to_string()], vec!["AL"]));
    let mut lying = PSheet::new(&look);
    bar().draw(PRect::new(0.0, 550.0, 400.0, 50.0), PRect::default(), &window(), None, &mut lying);
    assert_eq!(lying.text_list(), ["AL"]);
    assert_eq!((initial("ada"), initial("Ada Byron King")), ("a".to_string(), "AB".to_string()));
}

#[test]
fn in_a_theme_with_light_the_bar_and_its_menu_are_one_pane_lit_toward_the_page_and_every_icon_has_a_chip() {
    let look = PLook::read(&format!("{LOOK}color\tshade.light\t#ffffff\nsize\tshade.share\t0.5\nsize\tshade.rest\t0.5\nsize\tshade.soft\t0.25\ncolor\twindow.fill\t#eeeeee\nsize\tactivity.radius\t12\nsize\tactivity.corner\t16\n"));
    let mut sheet = PSheet::new(&look);
    bar().draw(BAR, SIDE, &picked("layout"), None, &mut sheet);
    // The pane: the window's ground under the bar and the menu together,
    // its corners as round as the theme says.
    let whole = PRect::new(0.0, 0.0, 250.0, 600.0);
    assert_eq!(sheet.mark.first(), Some(&PMark::Fill { rect: whole, radius: 16.0, color: look.color("window.fill").expect("in the look") }));
    let lit: Vec<(PRect, f32, f32, f32)> = sheet.mark.iter().filter_map(|mark| match mark {
        PMark::Blend { rect, from, to, turn, .. } => Some((*rect, from.a, to.a, *turn)),
        _ => None,
    }).collect();
    // Its light is nothing at the window's edge and a shade at rest
    // toward the page. What stands on it runs the other way: the mark and
    // the row whose menu is shown at full strength, the other rows at the
    // faintest, each a chip 37.5 square in the middle of its cell.
    let chip = |top: f32, from: f32| (PRect::new(6.25, top, 37.5, 37.5), from, 0.0, 90.0);
    assert_eq!(lit[..6], [(whole, 0.0, 0.25, 90.0), chip(6.25, 0.5), chip(51.25, 0.5), chip(91.25, 0.125), chip(281.25, 0.125), chip(511.25, 0.125)]);
    // The user's mark is no light of its own but Phi-Soft's shape, as
    // large as a chip, in the light's colour at full strength; then
    // the menu's three groups.
    assert!(sheet.mark.iter().any(|mark| matches!(mark, PMark::Icon { name, rect, color } if name == "soft-fill" && *rect == PRect::new(6.25, 556.25, 37.5, 37.5) && color.a == 0.5)));
    assert_eq!(lit.len(), 9);
}
