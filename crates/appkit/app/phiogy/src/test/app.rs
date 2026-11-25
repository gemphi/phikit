//! Phiogy as appkit sees it: what it supplies, where its parts go, and
//! that everything it is drawn with is its theme's.

use std::cell::RefCell;
use std::rc::Rc;

use appkit::pui::PApp;
use appkit::pui::core::{PColor, PMark, PMode, PRect, PSheet};
use appkit::pui::window::{PDeed, PLevel};
use appkit::theme::{PLook, PTheme};

use super::{DESK, HAND, drawn, opened, part, rect, staged};
use crate::Phiogy;
use crate::board::lock;

#[test]
fn phiogy_supplies_everything_its_window_names_and_a_view_for_every_part() {
    let app = Phiogy::new(Rc::new(Phiogy::look("")));
    assert_eq!((app.name(), app.front(), app.side()), ("Phiogy", "room", "guide"));
    assert!(app.missing().is_empty(), "{:?}", app.missing());
    let bare: Vec<&str> = app.part().list().iter().map(|part| part.slug()).filter(|part| app.content(part).is_none()).collect();
    assert!(bare.is_empty(), "{bare:?}");
    assert!(app.content("nothing").is_none());
}

#[test]
fn at_a_desk_the_room_has_the_body_and_my_rooms_are_in_the_bar_s_menu() {
    let (_, stage, _) = staged(DESK);
    assert_eq!(part(&stage), ["title", "room", "status", "guide"]);
    // The caption is the window's first 44. Under it the bar is 56 wide
    // and its menu 200, 8 clear of the window's side and foot; there
    // are no tabs. The room has what is left: its header
    // and its footer have the heights the theme gives.
    assert_eq!(stage.frame().caption, PRect::new(0.0, 0.0, 1040.0, 44.0));
    assert_eq!(rect(&stage, "title"), PRect::new(264.0, 44.0, 776.0, 56.0));
    assert_eq!(rect(&stage, "room"), PRect::new(264.0, 100.0, 776.0, 564.0));
    assert_eq!(rect(&stage, "status"), PRect::new(264.0, 664.0, 776.0, 56.0));
    assert_eq!(stage.frame().tab.size.height, 0.0);
    let guide = rect(&stage, "guide");
    assert!(stage.frame().menu.contains(guide.origin) && guide.size.height > 300.0, "{guide:?}");
}

#[test]
fn narrowed_it_is_a_mobile_app_and_shows_the_room_alone() {
    let (_, stage, _) = staged(HAND);
    assert_eq!(part(&stage), ["title", "room", "status"]);
    // The bar lies under the room, 8 clear of the window's foot and sides.
    assert_eq!(stage.frame().activity, PRect::new(8.0, 736.0, 374.0, 56.0));
}

/// A theme that writes down every token it is asked for and does not have.
struct Strict<'a> {
    look: &'a PLook,
    missed: RefCell<Vec<String>>,
}

impl Strict<'_> {
    fn note<T>(&self, token: &str, found: Option<T>) -> Option<T> {
        if found.is_none() {
            self.missed.borrow_mut().push(token.to_string());
        }
        found
    }
}

impl PTheme for Strict<'_> {
    fn color(&self, token: &str) -> Option<PColor> {
        self.note(token, self.look.color(token))
    }

    fn size(&self, token: &str) -> Option<f32> {
        self.note(token, self.look.size(token))
    }
}

#[test]
fn every_colour_and_size_phiogy_is_drawn_with_is_in_its_theme() {
    for size in [DESK, HAND] {
        let (app, mut stage, look) = staged(size);
        let strict = Strict { look: look.as_ref(), missed: RefCell::new(Vec::new()) };
        // A room with nobody on its line, and one with a caller and a
        // page that only has it open.
        stage.draw(&mut PSheet::new(&strict));
        {
            let mut board = lock(&app.desk().board);
            let hold = opened(&mut board, "phiogy/lora", "peer-aaaaaaaa", 1);
            opened(&mut board, "phiogy/lora", "peer-bbbbbbbb", 2);
            crate::system::join::join(&mut board, hold.line).expect("it joins");
        }
        stage.draw(&mut PSheet::new(&strict));
        // The hosts, each menu of the bar, and the profile.
        assert!(stage.window.apply(&PDeed::Show("venue".to_string())));
        stage.lay(size, PMode::of_width(size.width));
        stage.draw(&mut PSheet::new(&strict));
        for row in ["room", "venue", "menu"] {
            stage.window.apply(&PDeed::Pick(row.to_string()));
            stage.lay(size, PMode::of_width(size.width));
            stage.draw(&mut PSheet::new(&strict));
        }
        stage.window.apply(&PDeed::Open("profile".to_string()));
        stage.lay(size, PMode::of_width(size.width));
        stage.tick();
        assert!(stage.panel().is_some());
        stage.draw(&mut PSheet::new(&strict));
        assert!(strict.missed.borrow().is_empty(), "not in data/theme.tsv: {:?}", strict.missed.borrow());
    }
}

#[test]
fn nothing_phiogy_draws_has_a_fill_of_its_own() {
    for size in [DESK, HAND] {
        let (app, mut stage, look) = staged(size);
        {
            let mut board = lock(&app.desk().board);
            let hold = opened(&mut board, "phiogy/lora", "peer-aaaaaaaa", 1);
            crate::system::join::join(&mut board, hold.line).expect("it joins");
        }
        for (space, least) in [("room", 4), ("venue", 4)] {
            assert!(stage.window.apply(&PDeed::Show(space.to_string())));
            stage.lay(size, PMode::of_width(size.width));
            let mark = drawn(&stage, &look);
            // What is flat is the window's ground, a rule, the line and the
            // caret of the field that has the keys, and the veil
            // behind a modal. Everything else wears the light.
            let plain = ["window.fill", "line.ink", "control.accent", "modal.veil"].map(|token| look.color(token).expect("in the theme"));
            let flat: Vec<&PMark> = mark.iter().filter(|mark| matches!(mark, PMark::Fill { color, .. } if !plain.contains(color))).collect();
            assert!(flat.is_empty(), "{flat:?}");
            let lit = mark.iter().filter(|mark| matches!(mark, PMark::Blend { .. })).count();
            assert!(lit >= least, "{space}: {lit}");
        }
    }
}

#[test]
fn every_icon_a_table_names_has_a_drawing_on_disk() {
    use crate::data;
    // The tables that name an icon, and the cell of a row it is in.
    let named = [(data::ICON, 1), (data::ACT, 1), (data::TOOL, 1), (data::ACTIVITY, 2), (data::MENU, 5), (data::MODAL, 2), (data::SPACE, 2), (data::HOST, 2), (data::PART, 2)];
    let icon = named.iter().flat_map(|(table, cell)| data::row(table).into_iter().filter_map(|row| row.get(*cell).map(|name| name.to_string())).collect::<Vec<_>>());
    // Phiogy's own first, then the ones every app shares.
    let own = concat!(env!("CARGO_MANIFEST_DIR"), "/asset/icon");
    let shared = concat!(env!("CARGO_MANIFEST_DIR"), "/../../asset/icon");
    let missing: Vec<String> = icon.filter(|name| !name.is_empty()).filter(|name| ![own, shared].iter().any(|folder| std::path::Path::new(&format!("{folder}/{name}.svg")).is_file())).collect();
    assert!(missing.is_empty(), "no drawing for: {missing:?}");
}

#[test]
fn in_every_dark_look_the_ground_is_whole_and_no_lighter_than_its_own_colour_whatever_the_light() {
    use appkit::theme::{PTheme, lightness};
    let look = crate::app::Phiogy::look("");
    let shade = crate::app::Phiogy::shade(&look);
    // The brightest light a person may give: a hard green.
    shade.put("color", "shade.light", "#57ffb7");
    for name in shade.list().iter().map(|name| name.to_string()).collect::<Vec<_>>() {
        assert!(shade.turn(&name));
        let ground = shade.color("window.fill").expect("a ground");
        let ink = shade.color("text.ink").expect("an ink");
        // A dark look: light ink on its ground.
        if lightness(ink) > 0.5 {
            assert_eq!(ground.a, 1.0, "{name}: its ground is see-through");
            let own = crate::app::Phiogy::shade(&look);
            own.turn(&name);
            let bare = own.color("window.fill").expect("a ground");
            assert!(lightness(ground) <= lightness(bare) + 0.005, "{name}: the light makes its ground lighter ({} > {})", lightness(ground), lightness(bare));
        }
    }
}
