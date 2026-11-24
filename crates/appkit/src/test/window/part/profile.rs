use std::cell::RefCell;
use std::rc::Rc;

use crate::pui::core::*;
use crate::pui::window::*;
use crate::service::PTable;
use crate::theme::{PLook, PShade};

/// A flat look for the tests: a gap of 10, a line of 20, a mark of 60 and
/// letters of 8.
const LOOK: &str = "size\tgap\t10\nsize\ttext.line\t20\nsize\tprofile.avatar\t60\nsize\ttext.advance\t8\n";

/// Where a line of text was drawn.
fn at(sheet: &PSheet, wanted: &str) -> Option<PPoint> {
    sheet.mark.iter().find_map(|mark| match mark {
        PMark::Text { at, text, .. } if text == wanted => Some(*at),
        _ => None,
    })
}

#[test]
fn a_profile_shows_who_the_user_is_and_every_other_row_of_their_table() {
    let look = PLook::read(LOOK);
    let mut profile = PProfile::new(PTable::read("name\tAda Lovelace\nmail\tada@example.com\nmodal\tprofile\nModel\tnone\nPlan\tfree\n"));
    profile.set_frame(PRect::new(100.0, 100.0, 300.0, 260.0));
    let mut sheet = PSheet::new(&look);
    show(&profile, &mut sheet, PPoint::default());
    // Their mark, their name and address, then the rows: the modal the
    // table names is not a row.
    assert_eq!(sheet.text_list(), ["Ada Lovelace", "ada@example.com", "Model", "none", "Plan", "free"]);
    // The mark stands on Phi-Soft's own shape.
    let shape = sheet.mark.iter().find(|mark| matches!(mark, PMark::Icon { .. }));
    assert_eq!(shape, Some(&PMark::Icon { name: "soft-fill".to_string(), rect: PRect::new(110.0, 110.0, 60.0, 60.0), color: PColor::default() }));
    // It draws inside its own rectangle and nowhere else.
    assert_eq!((sheet.mark.first(), sheet.mark.last()), (Some(&PMark::Clip(PRect::new(100.0, 100.0, 300.0, 260.0))), Some(&PMark::Unclip)));
}

#[test]
fn a_row_too_long_for_the_profile_is_set_in_several_lines_and_the_next_begins_under_them() {
    let look = PLook::read(LOOK);
    let mut profile = PProfile::new(PTable::read("name\tAda\nChats\tkept on this computer until the page closes\nPlan\tfree\n"));
    profile.set_frame(PRect::new(0.0, 0.0, 300.0, 260.0));
    let mut sheet = PSheet::new(&look);
    show(&profile, &mut sheet, PPoint::default());
    // The rows are one block, a gap in from the profile's sides, which
    // begins two gaps and the mark down. A name is a gap into the block;
    // what it stands for begins four tenths of the block's width after
    // that, and has what is left less a gap: 148, eighteen letters of 8.
    assert_eq!(at(&sheet, "Chats"), Some(PPoint::new(20.0, 85.0)));
    assert_eq!(["kept on this", "computer until the", "page closes"].map(|line| at(&sheet, line)), [85.0, 105.0, 125.0].map(|down| Some(PPoint::new(132.0, down))));
    // Three lines and half a line under the row's top.
    assert_eq!(at(&sheet, "Plan"), Some(PPoint::new(20.0, 155.0)));
}

#[test]
fn what_is_true_now_is_asked_once_a_frame_and_shown_under_the_table_s_rows() {
    let look = PLook::read(LOOK);
    let count = Rc::new(RefCell::new(1));
    let told = Rc::clone(&count);
    let fact: PFact = Rc::new(move || PTable::read(&format!("Chats\t{}\n", told.borrow())));
    let mut profile = PProfile::new(PTable::read("name\tAda\nabout\tAbout\nPlan\tfree\n")).live(Some(fact));
    profile.set_frame(PRect::new(0.0, 0.0, 300.0, 260.0));
    let text = |profile: &PProfile| {
        let mut sheet = PSheet::new(&look);
        show(profile, &mut sheet, PPoint::default());
        sheet.text_list().into_iter().map(str::to_string).collect::<Vec<_>>()
    };
    // Nothing was asked before the first frame; the heading is the word
    // the table gives for it, and is not a row.
    assert_eq!(text(&profile), ["Ada", "", "About", "Plan", "free"]);
    assert!(!profile.tick());
    assert_eq!(text(&profile)[3..], ["Plan", "free", "Chats", "1"]);
    *count.borrow_mut() = 7;
    profile.tick();
    assert_eq!(text(&profile)[5..], ["Chats", "7"]);
}

#[test]
fn it_has_a_pill_for_each_look_and_a_press_turns_the_window_and_tells_the_app() {
    let look = PLook::read(LOOK);
    let shade = Rc::new(PShade::new(vec![("light".to_string(), look.clone()), ("dark".to_string(), look.clone())]));
    let wish = PWish::default();
    let user = PTable::read("name\tAda\nappearance\tAppearance\nlook.light\tLight\n");
    let mut profile = PProfile::new(user).dressed(Some(Rc::clone(&shade)), Some(wish.clone()));
    profile.set_frame(PRect::new(100.0, 100.0, 300.0, 260.0));
    let mut sheet = PSheet::new(&look);
    show(&profile, &mut sheet, PPoint::default());
    // The heading, then a pill a look: one the table has a word for reads
    // that word, the other its own name. Neither is a row.
    assert_eq!(sheet.text_list()[2..], ["Appearance", "Light", "dark"]);
    // A press on the second turns the window to it and tells the app.
    let pill = at(&sheet, "dark").expect("a pill");
    assert!(send(&mut profile, &PEvent::PointerDown(pill.plus(PPoint::new(2.0, 2.0)))));
    assert_eq!((shade.name(), wish.borrow().as_slice()), ("dark", [PDeed::Tell("look dark".to_string())].as_slice()));
    // A press anywhere else does nothing.
    assert!(!send(&mut profile, &PEvent::PointerDown(PPoint::new(110.0, 110.0))) && wish.borrow().len() == 1);
    // A profile that was given no looks has no heading for them.
    let mut bare = PProfile::new(PTable::read("name\tAda\nappearance\tAppearance\n"));
    bare.set_frame(PRect::new(0.0, 0.0, 300.0, 260.0));
    let mut plain = PSheet::new(&look);
    show(&bare, &mut plain, PPoint::default());
    assert_eq!(plain.text_list(), ["Ada", ""]);
}

#[test]
fn in_a_theme_with_light_its_rows_are_one_surface_and_the_look_shown_is_lit_the_most() {
    let look = PLook::read(&format!("{LOOK}color\tshade.light\t#ffffff\nsize\tshade.share\t0.5\nsize\tshade.soft\t0.5\n"));
    let shade = Rc::new(PShade::new(vec![("light".to_string(), look.clone()), ("dark".to_string(), look.clone())]));
    let mut profile = PProfile::new(PTable::read("name\tAda\nPlan\tfree\n")).dressed(Some(shade), None);
    profile.set_frame(PRect::new(0.0, 0.0, 300.0, 260.0));
    let mut sheet = PSheet::new(&look);
    show(&profile, &mut sheet, PPoint::default());
    let lit: Vec<(f32, f32, f32)> = sheet.mark.iter().filter_map(|mark| match mark {
        PMark::Blend { rect, from, to, turn, .. } => Some((rect.size.width, from.a.max(to.a), *turn)),
        _ => None,
    }).collect();
    // The group of rows, a surface as wide as the profile less its
    // gaps, its row's name on a ground of its own; the group of looks, as wide; then the pills on it, lit
    // against it, the look shown the most and the other the least. The
    // mark is Phi-Soft's shape in the light's colour, no light of its own.
    assert_eq!(lit, [(280.0, 0.5, 90.0), (112.0, 0.25, 90.0), (280.0, 0.5, 90.0), (60.0, 0.5, 90.0), (52.0, 0.25, 90.0)]);
}

#[test]
fn a_profile_longer_than_its_panel_is_scrolled_with_the_wheel() {
    let look = PLook::read(LOOK);
    let shade = Rc::new(PShade::new(vec![("light".to_string(), look.clone())]));
    let mut profile = PProfile::new(PTable::read("name\tAda\nOne\t1\nTwo\t2\nThree\t3\n")).dressed(Some(shade), None);
    // Its mark, three rows and its pill come to 220; the panel has 150.
    profile.set_frame(PRect::new(0.0, 0.0, 300.0, 150.0));
    let down = |profile: &PProfile, text: &str| {
        let mut sheet = PSheet::new(&look);
        show(profile, &mut sheet, PPoint::default());
        at(&sheet, text).map(|at| at.y)
    };
    assert_eq!((down(&profile, "Ada"), down(&profile, "light")), (Some(20.0), Some(195.0)));
    assert!(send(&mut profile, &PEvent::Scroll(PPoint::new(10.0, 10.0), PPoint::new(0.0, -100000.0))));
    // As far as its foot and no further: 90, its groups' headings and grounds included.
    assert_eq!((down(&profile, "Ada"), down(&profile, "light")), (Some(-70.0), Some(105.0)));
    // One that fits does not scroll.
    profile.set_frame(PRect::new(0.0, 0.0, 300.0, 400.0));
    down(&profile, "Ada");
    assert!(!send(&mut profile, &PEvent::Scroll(PPoint::new(10.0, 10.0), PPoint::new(0.0, -100.0))));
}
