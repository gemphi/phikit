//! A form is only its table and its source: blocks shown from lists, a
//! press kept at its block's path, a block with an empty list not shown.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::pui::core::*;
use crate::pui::form::{PChoice, PForm, PSource};
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
