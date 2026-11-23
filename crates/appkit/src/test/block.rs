use crate::pui::block::{PBand, PDeck, PFacts, PPill, PText, PTip};
use crate::pui::core::*;
use crate::theme::{PLook, PTheme};

/// A look with light and what a block is drawn with.
const LOOK: &str = "color\tshade.light\t#ffffff\nsize\tshade.share\t0.5\nsize\tshade.rest\t0.5\nsize\tshade.soft\t0.25\ncolor\ttext.ink\t#202020\ncolor\ttext.dim\t#808080\ncolor\tline.ink\t#c0c0c0\nsize\tline.width\t1\nsize\ttext.line\t20\nsize\ttext.advance\t8\nsize\tgap\t16\nsize\ticon.size\t20\nsize\trow.radius\t10\nword\ttext.cut\t...\n";

fn blend(sheet: &PSheet) -> Vec<(PRect, f32, f32)> {
    sheet.mark.iter().filter_map(|mark| match mark {
        PMark::Blend { rect, from, to, .. } => Some((*rect, from.a, to.a)),
        _ => None,
    }).collect()
}

#[test]
fn a_pill_is_as_wide_as_what_it_holds_and_stands_on_a_ground_lit_as_it_is() {
    let look = PLook::read(LOOK);
    let mut sheet = PSheet::new(&look);
    let word = ("", "Follow", false);
    let wide = sheet.measure("Follow");
    // A line and half a gap high; its words and a gap wide.
    assert_eq!(word.size(&sheet), PSize { width: wide + 16.0, height: 28.0 });
    // With an icon, the icon and a quarter of a gap more.
    assert_eq!(("star", "Follow", false).size(&sheet).width, wide + 16.0 + 20.0 + 4.0);
    assert_eq!(("star", "", false).size(&sheet), PSize { width: 36.0, height: 28.0 });
    // At rest a soft ground of its own, lit from the left; on, the strongest.
    let rect = word.draw(&mut sheet, PPoint::new(10.0, 10.0), false);
    assert_eq!(rect, PRect::new(10.0, 10.0, wide + 16.0, 28.0));
    assert_eq!(blend(&sheet), [(rect, 0.125, 0.0)]);
    assert!(sheet.mark.contains(&PMark::Text { at: PPoint::new(18.0, 14.0), text: "Follow".to_string(), color: look.color("text.ink").unwrap_or_default() }));
    let mut sheet = PSheet::new(&look);
    ("", "Follow", true).draw(&mut sheet, PPoint::new(10.0, 10.0), false);
    assert_eq!(blend(&sheet)[0].1, 0.5);
    // Held by a group, it has no ground of its own at rest.
    let mut sheet = PSheet::new(&look);
    word.draw(&mut sheet, PPoint::new(10.0, 10.0), true);
    assert!(blend(&sheet).is_empty());
}

#[test]
fn pills_that_belong_together_stand_on_one_ground_and_the_one_that_is_on_is_lit() {
    let look = PLook::read(LOOK);
    let mut sheet = PSheet::new(&look);
    let list = [("phone", "", false), ("video", "", false), ("star", "", true)];
    // Three icons, a quarter of a gap between and round them.
    assert_eq!(list.span(&sheet), 36.0 * 3.0 + 4.0 * 2.0 + 8.0);
    let rect = list.draw(&mut sheet, PPoint::new(100.0, 50.0));
    assert_eq!(rect, [PRect::new(104.0, 54.0, 36.0, 28.0), PRect::new(144.0, 54.0, 36.0, 28.0), PRect::new(184.0, 54.0, 36.0, 28.0)]);
    // One ground under them all, lit from the right, against what it
    // stands on; and only the one that is on has a light of its own.
    assert_eq!(blend(&sheet), [(PRect::new(100.0, 50.0, 124.0, 36.0), 0.0, 0.25), (rect[2], 0.5, 0.0)]);
    // No pill, no ground.
    let mut sheet = PSheet::new(&look);
    let none: [(&str, &str, bool); 0] = [];
    assert!(none.draw(&mut sheet, PPoint::default()).is_empty() && sheet.mark.is_empty());
    // In less room than they need, each gives up the same share of the
    // room beside what it holds, and the group is no wider than asked.
    let mut sheet = PSheet::new(&look);
    let tight = list.draw_in(&mut sheet, PPoint::new(100.0, 50.0), 100.0);
    assert_eq!(tight.iter().map(|rect| rect.size.width).collect::<Vec<_>>(), [28.0, 28.0, 28.0]);
    assert_eq!(blend(&sheet)[0].0, PRect::new(100.0, 50.0, 100.0, 36.0));
}

#[test]
fn a_table_names_each_thing_at_the_left_and_ends_its_value_at_the_right_over_a_line() {
    let look = PLook::read(LOOK);
    let mut sheet = PSheet::new(&look);
    let row = [("Fabric".to_string(), "silk".to_string()), ("Care".to_string(), "Dry clean only, store flat in tissue".to_string())];
    let end = row.table(&mut sheet, PPoint::new(0.0, 0.0), 200.0);
    // A line and half a gap a row.
    assert_eq!(end, 56.0);
    let text: Vec<(f32, String)> = sheet.mark.iter().filter_map(|mark| match mark {
        PMark::Text { at, text, .. } => Some((at.x, text.clone())),
        _ => None,
    }).collect();
    let silk = sheet.measure("silk");
    assert_eq!(text[0], (0.0, "Fabric".to_string()));
    assert_eq!(text[1], (200.0 - silk, "silk".to_string()));
    // A value too long for its row is cut, and still ends at the right.
    assert!(text[3].1.ends_with("...") && text[3].1.len() < row[1].1.len());
    assert_eq!(text[3].0 + sheet.measure(&text[3].1), 200.0);
    assert_eq!(sheet.mark.iter().filter(|mark| matches!(mark, PMark::Fill { rect, .. } if rect.size == PSize { width: 200.0, height: 1.0 })).count(), 2);
    // A text that fits is as it is.
    assert_eq!("silk".fit(200.0, &sheet), "silk");
}

#[test]
fn a_card_has_a_face_a_name_and_a_line_and_a_deck_folds_its_cards_to_the_width() {
    let look = PLook::read(&format!("{LOOK}size	card.width	100
size	card.face	40"));
    let mut sheet = PSheet::new(&look);
    let gap = look.size("gap").unwrap_or_default();
    let line = look.size("text.line").unwrap_or_default();
    let crowd = [(Some(("faces", 4, 2)), "Lora Vale", "Here now"), (None, "Tavi Reyes", "Away"), (None, "Mina Okoro", "")];
    // In 250 two cards of at least 100 fit, sharing the width; the third folds under.
    let (rect, end) = crowd.draw(&mut sheet, PPoint::new(10.0, 20.0), 250.0);
    let (wide, high) = ((250.0 - gap / 2.0) / 2.0, 40.0 + line * 2.0 + gap * 1.5);
    assert_eq!(rect, [PRect::new(10.0, 20.0, wide, high), PRect::new(10.0 + wide + gap / 2.0, 20.0, wide, high), PRect::new(10.0, 20.0 + high + gap / 2.0, wide, high)]);
    assert_eq!(end, 20.0 + high * 2.0 + gap / 2.0);
    // One with a picture has it round, in the middle at the top; one without has the letter their name begins with, drawn as an icon is.
    let face = PRect::new(rect[0].center().x - 20.0, 20.0 + gap / 2.0, 40.0, 40.0);
    assert!(sheet.mark.contains(&PMark::Picture { name: "faces".to_string(), rect: face, radius: 20.0, fit: PFit::Cell { column: 4, index: 2 } }), "{:?}", sheet.mark);
    let text: Vec<String> = sheet.mark.iter().filter_map(|mark| match mark {
        PMark::Text { text, .. } => Some(text.clone()),
        _ => None,
    }).collect();
    assert_eq!(text, ["Lora Vale", "Here now", "Tavi Reyes", "Away", "Mina Okoro", ""]);
    let letter: Vec<&str> = sheet.mark.iter().filter_map(|mark| match mark {
        PMark::Icon { name, .. } => Some(name.as_str()),
        _ => None,
    }).collect();
    assert_eq!(letter, ["letter-t", "letter-m"]);
    // Every card and every face stands on a ground of its own.
    assert_eq!(blend(&sheet).len(), 6);
    // Too narrow for one card, a card is as wide as the room there is.
    let (narrow, _) = crowd[..1].draw(&mut PSheet::new(&look), PPoint::default(), 60.0);
    assert_eq!(narrow[0].size.width, 60.0);
}

#[test]
fn a_tip_is_said_under_a_place_in_the_upper_half_over_one_in_the_lower_and_inside_the_frame() {
    let look = PLook::read(LOOK);
    let mut sheet = PSheet::new(&look);
    let frame = PRect::new(0.0, 0.0, 400.0, 300.0);
    let wide = sheet.measure("Pin") + 16.0;
    // Under a place in the upper half, in the middle of it.
    let high = "Pin".tip(&mut sheet, PRect::new(100.0, 20.0, 40.0, 30.0), frame);
    assert_eq!(high, Some(PRect::new(120.0 - wide / 2.0, 54.0, wide, 28.0)));
    // Over one in the lower half; and never past the frame's side.
    let low = "Pin".tip(&mut sheet, PRect::new(380.0, 250.0, 40.0, 30.0), frame);
    assert_eq!(low, Some(PRect::new(400.0 - 8.0 - wide, 250.0 - 4.0 - 28.0, wide, 28.0)));
    // It floats: under its light is the window's own ground, so nothing shows through its word.
    assert!(sheet.mark.iter().any(|mark| matches!(mark, PMark::Fill { rect, .. } if Some(*rect) == high)), "{:?}", sheet.mark);
    // No word, no tip.
    let before = sheet.mark.len();
    assert_eq!("".tip(&mut sheet, PRect::new(0.0, 0.0, 10.0, 10.0), frame), None);
    assert_eq!(sheet.mark.len(), before);
}
