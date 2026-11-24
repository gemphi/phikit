use super::activity::{LOOK, bar, picked};
use crate::pui::core::*;
use crate::pui::window::*;
use crate::theme::{PLook, PTheme};

const SIZE: PSize = PSize { width: 800.0, height: 600.0 };

/// The bar's look with a caption 40 high.
fn look() -> PLook {
    PLook::read(&format!("{LOOK}size\tcaption.height\t40\n"))
}

/// The frame of a window with the tests' bar, as a stage lays it.
fn frame(window: &PWindow, mode: PMode, look: &PLook) -> PFrame {
    PFrame::of(SIZE, mode, look, bar().thick(look), bar().wide(window, mode == PMode::Mobile, look), false)
}

fn icon(sheet: &PSheet) -> Vec<String> {
    let name = |mark: &PMark| match mark {
        PMark::Icon { name, .. } => Some(name.clone()),
        _ => None,
    };
    sheet.mark.iter().filter_map(name).collect()
}

#[test]
fn the_mark_is_over_the_bar_and_is_the_pin_of_the_bar_s_menu() {
    let (look, mut window) = (look(), picked("layout"));
    let laid = frame(&window, PMode::Desktop, &look);
    assert_eq!((laid.caption, laid.activity, laid.menu), (PRect::new(0.0, 0.0, 800.0, 40.0), PRect::new(0.0, 40.0, 50.0, 560.0), PRect::new(50.0, 40.0, 200.0, 560.0)));
    // The mark is the pin, and the only place on the caption a press
    // does something: at a desk, it puts the menu away.
    let spot = PCaption::default().spot(&laid, &bar(), &look);
    assert_eq!(spot, [(PRect::new(0.0, 0.0, 50.0, 40.0), PDeed::Fold)]);
    // With the menu put away it is where it was, and brings it back.
    assert!(window.apply(&PDeed::Fold));
    let spot = PCaption::default().spot(&frame(&window, PMode::Desktop, &look), &bar(), &look);
    assert_eq!(spot, [(PRect::new(0.0, 0.0, 50.0, 40.0), PDeed::Fold)]);
    // In a hand the bar lies at the foot: the mark is a square at the
    // caption's start, and draws the menu out and puts it away.
    let spot = PCaption::default().spot(&frame(&window, PMode::Mobile, &look), &bar(), &look);
    assert_eq!(spot, [(PRect::new(0.0, 0.0, 40.0, 40.0), PDeed::Side)]);
}

#[test]
fn the_mark_does_what_the_table_names_and_there_is_no_pin_without_a_menu() {
    let (look, window) = (look(), picked(""));
    let bar = PActivity::read("book\tBook\tfeather\thead\tspace\tlayout\nlayout\tLayout\tgrid\ttop\tspace\tlayout\n");
    assert_eq!(bar.home, Some(PDeed::Show("layout".to_string())));
    let laid = PFrame::of(SIZE, PMode::Desktop, &look, bar.thick(&look), 0.0, false);
    assert_eq!(PCaption::default().spot(&laid, &bar, &look), [(PRect::new(0.0, 0.0, 50.0, 40.0), PDeed::Show("layout".to_string()))]);
    // An app with no bar has neither, and a theme with no caption none.
    assert!(PCaption::default().spot(&laid, &PActivity::default(), &look).is_empty());
    let plain = PLook::read(LOOK);
    assert!(PCaption::default().spot(&frame(&window, PMode::Desktop, &plain), &self::bar(), &plain).is_empty());
}

#[test]
fn bare_it_carries_the_window_s_buttons_and_is_moved_by_what_is_between_its_presses() {
    let (look, window) = (look(), picked("layout"));
    let laid = frame(&window, PMode::Desktop, &look);
    let grip = PCaption { bare: true, over: false }.grip(&laid, &bar(), &look);
    let knob = [(PRect::new(760.0, 0.0, 40.0, 40.0), PGrip::Close), (PRect::new(720.0, 0.0, 40.0, 40.0), PGrip::Max), (PRect::new(680.0, 0.0, 40.0, 40.0), PGrip::Min)];
    // From the mark to the buttons, where the name is.
    let drag = (PRect::new(50.0, 0.0, 630.0, 40.0), PGrip::Drag);
    assert_eq!(grip, [knob[0], knob[1], knob[2], drag]);
    // Under the system's own bar it is told nothing.
    assert!(PCaption::default().grip(&laid, &bar(), &look).is_empty());
}

#[test]
fn it_draws_the_mark_the_name_and_the_buttons_the_pin_under_the_pointer_and_the_bar_has_no_head() {
    let (look, mut window) = (look(), picked("layout"));
    let laid = frame(&window, PMode::Desktop, &look);
    let mut sheet = PSheet::new(&look);
    PCaption { bare: true, over: false }.draw(&laid, &bar(), &window, &mut sheet);
    // With no logo yet the mark is the first letter of the app's name;
    // the name starts a gap inside the menu's cell, on the caption's
    // middle line.
    assert_eq!((sheet.text_list(), icon(&sheet)), (vec!["Book"], ["letter-b", "x", "square", "minus"].map(str::to_string).to_vec()));
    assert!(sheet.mark.contains(&PMark::Text { at: PPoint::new(60.0, 10.0), text: "Book".to_string(), color: PColor::default() }));
    // With the pointer over the mark, the pin stands in its place, in
    // the middle of the mark's cell.
    let mut over = PSheet::new(&look);
    PCaption { bare: true, over: true }.draw(&laid, &bar(), &window, &mut over);
    assert_eq!((over.text_list(), icon(&over)), (vec!["Book"], ["pin-open", "x", "square", "minus"].map(str::to_string).to_vec()));
    assert!(over.mark.contains(&PMark::Icon { name: "pin-open".to_string(), rect: PRect::new(15.0, 10.0, 20.0, 20.0), color: PColor::default() }));
    // The bar and its menu draw neither the mark nor the name nor a
    // button of their own, and their first row is 5 under the caption.
    let mut under = PSheet::new(&look);
    bar().draw(laid.activity, laid.menu, &window, None, &mut under);
    assert_eq!((icon(&under), under.text_list()), (["grid", "palette", "box", "book", "soft-fill", "letter-a", "plus", "folder", "settings"].map(str::to_string).to_vec(), vec!["New", "Open", "Preferences"]));
    assert_eq!(bar().spot(laid.activity, laid.menu, &window, &look)[0], (PRect::new(0.0, 45.0, 50.0, 40.0), PDeed::Fold));
    // Put away, the name goes, and under the pointer the pin is the
    // narrow one; under the system's bar there are no buttons.
    assert!(window.apply(&PDeed::Fold));
    let mut shut = PSheet::new(&look);
    PCaption::default().draw(&frame(&window, PMode::Desktop, &look), &bar(), &window, &mut shut);
    assert_eq!((shut.text_list(), icon(&shut)), (Vec::<&str>::new(), vec!["letter-b".to_string()]));
    let mut shut = PSheet::new(&look);
    PCaption { bare: false, over: true }.draw(&frame(&window, PMode::Desktop, &look), &bar(), &window, &mut shut);
    assert_eq!((shut.text_list(), icon(&shut)), (Vec::<&str>::new(), vec!["pin-shut".to_string()]));
    // A theme with no caption draws none.
    let (plain, mut none) = (PLook::read(LOOK), PSheet::new(&look));
    PCaption::default().draw(&frame(&window, PMode::Desktop, &plain), &bar(), &window, &mut none);
    assert!(none.mark.is_empty());
}

#[test]
fn the_app_s_mark_is_as_large_as_the_theme_says_and_as_an_icon_without() {
    let window = picked("layout");
    let mut bar = bar();
    bar.logo.icon = "book".to_string();
    let mark = |look: &PLook| {
        let mut sheet = PSheet::new(look);
        PCaption { bare: true, over: false }.draw(&frame(&window, PMode::Desktop, look), &bar, &window, &mut sheet);
        sheet.mark.iter().find_map(|mark| match mark {
            PMark::Icon { name, rect, .. } if name == "book" => Some(*rect),
            _ => None,
        })
    };
    // Without a word from the theme, an icon's size, in the middle of its cell.
    assert_eq!(mark(&look()), Some(PRect::new(15.0, 10.0, 20.0, 20.0)));
    // With one, that share of its ground's side, in whole dots, about
    // the same middle.
    let large = PLook::read(&format!("{LOOK}size\tcaption.height\t40\nsize\tcaption.mark\t0.8\n"));
    let ground = large.size("activity.row").unwrap_or_default() - large.size("gap").unwrap_or_default() / 4.0;
    let side = (ground.min(40.0) * 0.8).round();
    assert!(side > 20.0, "{side}");
    assert_eq!(mark(&large), Some(PRect::new(25.0 - side / 2.0, 20.0 - side / 2.0, side, side)));
}
