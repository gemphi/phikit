//! A form is only its table and its source: blocks shown from lists, a
//! press kept at its block's path, a block with an empty list not shown.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::pui::core::*;
use crate::pui::form::{PChoice, PDo, PForm, PSource};
use crate::pui::state::PState;
use crate::service::PTable;
use crate::theme::{PLook, PTheme};

const LOOK: &str = "color\tshade.light\t#ffffff\nsize\tshade.share\t0.5\nsize\tshade.rest\t0.5\nsize\tshade.soft\t0.25\ncolor\ttext.ink\t#202020\nsize\ttext.line\t20\nsize\ttext.advance\t8\nsize\tgap\t16\nsize\ticon.size\t20\nsize\trow.radius\t10\nsize\tswatch.size\t30\nsize\tswatch.ring\t3\n";

const TABLE: &str = "# A form of tabs over a band, an empty band, and a swatch of its own.\nmine\ttabs\tkind\t-\tkind\tkind\t-\nmine\tband\tsize\tsize\tsize\tsize\tkind\nmine\tband\tnone\tnone\tnone\tnone\tkind\nmine\tswatch\tpaint\tpaint\tpaint\tpaint\t-\nother\tband\tx\t-\tsize\tx\t-\n";

/// A source of fixed lists that keeps what is set.
#[derive(Default)]
struct Fixed(RefCell<HashMap<String, String>>);

impl PState for Fixed {
    fn get(&self, path: &str) -> String {
        self.0.borrow().get(path).cloned().unwrap_or_default()
    }

    fn set(&self, path: &str, value: &str) {
        self.0.borrow_mut().insert(path.to_string(), value.to_string());
    }
}

impl PSource for Fixed {
    fn list(&self, name: &str) -> Vec<PChoice> {
        let word = |w: &str| PChoice { value: w.to_lowercase(), label: w.to_string(), ..PChoice::default() };
        match name {
            "kind" => vec![word("Cat"), word("Dog")],
            "size" => vec![word("Small"), word("Large")],
            "paint" => vec![PChoice { value: "red".to_string(), colour: Some(PColor::new(1.0, 0.0, 0.0, 1.0)), ..PChoice::default() }],
            "look" => vec![PChoice { value: "coat".to_string(), picture: "coat.png".to_string(), deed: vec![PDo { word: "like".to_string(), icon: "heart".to_string(), on: true }, PDo { word: "full".to_string(), icon: "maximize".to_string(), on: false }], ..PChoice::default() }],
            _ => Vec::new(),
        }
    }
}

#[test]
fn a_form_is_its_table_s_blocks_and_a_press_keeps_the_value_at_its_path() {
    let look: Rc<dyn PTheme> = Rc::new(PLook::read(LOOK));
    let source = Rc::new(Fixed::default());
    let text = Rc::new(PTable::read("size\tSize\npaint\tPaint\nnone\tNone\n"));
    let mut form = PForm::new("mine", TABLE, source.clone(), text, look.clone());
    form.set_frame(PRect::new(0.0, 0.0, 300.0, 600.0));
    let mut sheet = PSheet::new(look.as_ref());
    form.draw(&mut sheet, PPoint::new(0.0, 0.0));
    let said = sheet.text_list();
    // Its tabs, the band under them with its heading, the swatch's
    // heading; not the band whose list is empty, nor another form's.
    for word in ["Cat", "Dog", "Size", "Small", "Large", "Paint"] {
        assert!(said.contains(&word), "{word}: {said:?}");
    }
    assert!(!said.contains(&"None"));
    // A press on Large keeps `large` at the band's path.
    let large = sheet.mark.iter().find_map(|mark| match mark {
        PMark::Text { at, text, .. } if text == "Large" => Some(PPoint::new(at.x + 2.0, at.y + 2.0)),
        _ => None,
    });
    assert!(form.respond(&PEvent::PointerDown(large.expect("drawn"))));
    assert_eq!(source.get("size"), "large");
}

/// A source of people: rows with a note and a level, cards, tiles, text.
struct People;

impl PState for People {
    fn get(&self, _: &str) -> String {
        String::new()
    }

    fn set(&self, _: &str, _: &str) {}
}

impl PSource for People {
    fn list(&self, name: &str) -> Vec<PChoice> {
        let one = |label: &str, note: &str| PChoice { value: label.to_lowercase(), label: label.to_string(), note: note.to_string(), ..PChoice::default() };
        match name {
            "say" => vec![one("Nobody is here.", "Open the room to join.")],
            "call" => vec![PChoice { end: "connected".to_string(), level: Some(0.5), icon: "mic".to_string(), ..one("Ada", "ada.one") }],
            "people" => vec![one("Bea", "away"), one("Cy", "here")],
            _ => Vec::new(),
        }
    }
}

#[test]
fn a_form_flows_its_heading_text_rows_cards_and_tiles_and_scrolls_when_long() {
    let look: Rc<dyn PTheme> = Rc::new(PLook::read(&format!("{LOOK}size\thead.line\t30\nsize\tform.meter\t40\nsize\tform.bar\t4\nsize\tcard.width\t100\nsize\tcard.face\t40\n")));
    let text = Rc::new(PTable::read("title\tOn the line\ncall\tIn the call\npeople\tMembers\n"));
    let table = "on\thead\t-\ttitle\t-\t-\t-\t-\non\ttext\tsay\t-\tsay\t-\t-\t-\non\trows\tcall\tcall\tcall\t-\t-\tstrong\non\tcards\tpeople\tpeople\tpeople\twho\t-\t-\non\tgrid\ttiles\tpeople\tpeople\twho\t-\t-\n";
    let mut form = PForm::new("on", table, Rc::new(People), text, look.clone());
    form.set_frame(PRect::new(0.0, 0.0, 300.0, 200.0));
    let mut sheet = PSheet::new(look.as_ref());
    form.draw(&mut sheet, PPoint::new(0.0, 0.0));
    let said = sheet.text_list();
    for word in ["On the line", "Nobody is here.", "Open the room to join.", "In the call", "Ada", "ada.one", "connected", "Members", "Bea", "Cy"] {
        assert!(said.contains(&word), "{word}: {said:?}");
    }
    // The level's bar: half of 40 long, 4 high.
    assert!(sheet.mark.iter().any(|mark| matches!(mark, PMark::Blend { rect, .. } if rect.size.width == 20.0 && rect.size.height == 4.0)));
    // More than the pane holds: the wheel moves it.
    assert!(form.respond(&PEvent::Scroll(PPoint::new(10.0, 10.0), PPoint::new(0.0, -40.0))));
}

/// A form with a side, a heading of its list, and a band beside it.
const SIDED: &str = "mine\thead\t-\t-\tkind\t-\t-\t-\nmine\tband\tsize\t-\tsize\tsize\t-\tside\nmine\tband\tkind\t-\tkind\tkind\t-\t-\n";

#[test]
fn a_side_stands_at_the_left_held_in_view_or_at_the_top_when_narrow() {
    let look: Rc<dyn PTheme> = Rc::new(PLook::read(&format!("{LOOK}size\tform.side\t0.4\nsize\tform.split\t500\n")));
    let source = Rc::new(Fixed::default());
    let mut form = PForm::new("mine", SIDED, source, Rc::new(PTable::read("")), look.clone());
    let mut at = |width: f32| {
        form.set_frame(PRect::new(0.0, 0.0, width, 400.0));
        let mut sheet = PSheet::new(look.as_ref());
        form.draw(&mut sheet, PPoint::new(0.0, 0.0));
        let place = |wanted: &str| sheet.mark.iter().find_map(|mark| match mark {
            PMark::Text { at, text, .. } | PMark::Head { at, text, .. } if text == wanted => Some(*at),
            _ => None,
        }).unwrap_or_else(|| panic!("{wanted}"));
        // The heading is its list's first choice; the side's band, the
        // other band.
        (place("Cat"), place("Small"), place("Dog"))
    };
    // Wide: the side at the left, from the top; the heading and the
    // rest at the right of it.
    let (head, side, rest) = at(800.0);
    assert!(side.x < 320.0 && head.x >= 320.0 && rest.x >= 320.0 && side.y < rest.y, "{head:?} {side:?} {rest:?}");
    // Narrow: the side at the top, under the heading, over the rest.
    let (head, side, rest) = at(400.0);
    assert!(head.y < side.y && side.y < rest.y && side.x < 100.0, "{head:?} {side:?} {rest:?}");
}

#[test]
fn a_picture_has_what_can_be_done_with_it_floating_down_its_corner() {
    let look: Rc<dyn PTheme> = Rc::new(PLook::read(LOOK));
    let source = Rc::new(Fixed::default());
    let mut form = PForm::new("mine", "mine	picture	look	-	look	look/open	-	-
", source.clone(), Rc::new(PTable::read("")), look.clone());
    form.set_frame(PRect::new(0.0, 0.0, 300.0, 600.0));
    let mut sheet = PSheet::new(look.as_ref());
    form.draw(&mut sheet, PPoint::new(0.0, 0.0));
    let picture = sheet.mark.iter().find_map(|mark| match mark {
        PMark::Picture { name, rect, .. } if name == "coat.png" => Some(*rect),
        _ => None,
    }).expect("the picture");
    let icon = |wanted: &str| sheet.mark.iter().find_map(|mark| match mark {
        PMark::Icon { name, rect, .. } if name == wanted => Some(*rect),
        _ => None,
    }).unwrap_or_else(|| panic!("{wanted}"));
    // Round icons down its top left corner, in their order.
    let (like, full) = (icon("heart"), icon("maximize"));
    assert!(picture.contains(like.center()) && like.center().x < picture.center().x && like.center().y < full.center().y, "{like:?} {full:?} {picture:?}");
    // A press on one keeps the choice's value at its word, under the
    // block's state; on the picture, at the block's path.
    assert!(form.respond(&PEvent::PointerDown(full.center())));
    assert_eq!(source.get("look/full"), "coat");
    assert!(form.respond(&PEvent::PointerDown(picture.center())));
    assert_eq!(source.get("look/open"), "coat");
}

#[test]
fn a_grid_widens_to_the_whole_form_and_gives_it_back() {
    let look: Rc<dyn PTheme> = Rc::new(PLook::read(LOOK));
    let source = Rc::new(Fixed::default());
    let table = "mine	grid	size	size	size	size	-	layout
mine	band	kind	kind	kind	kind	-	-
";
    let mut form = PForm::new("mine", table, source.clone(), Rc::new(PTable::read("size	Size
kind	Kind
")), look.clone());
    form.set_frame(PRect::new(0.0, 0.0, 300.0, 600.0));
    let drawn = |form: &PForm| {
        let mut sheet = PSheet::new(look.as_ref());
        form.draw(&mut sheet, PPoint::new(0.0, 0.0));
        sheet
    };
    let sheet = drawn(&form);
    let widen = sheet.mark.iter().find_map(|mark| match mark {
        PMark::Icon { name, rect, .. } if name == "maximize" => Some(*rect),
        _ => None,
    }).expect("the switch that widens it");
    assert!(sheet.text_list().contains(&"Kind"));
    // Widened, it alone stands; the switch then gives the form back.
    assert!(form.respond(&PEvent::PointerDown(widen.center())));
    let sheet = drawn(&form);
    assert!(sheet.text_list().contains(&"Size") && !sheet.text_list().contains(&"Kind"));
    let back = sheet.mark.iter().find_map(|mark| match mark {
        PMark::Icon { name, rect, .. } if name == "x" => Some(*rect),
        _ => None,
    }).expect("the switch that gives it back");
    assert!(form.respond(&PEvent::PointerDown(back.center())));
    assert!(drawn(&form).text_list().contains(&"Kind"));
}

