use crate::pui::core::figure::{end, size, start};
use crate::pui::core::*;
use crate::theme::PLook;

/// A look with light and a figure's measures.
const LIT: &str = "color\tshade.light\t#ffffff\nsize\tshade.share\t0.5\nsize\tshade.rest\t0.5\ncolor\ttext.ink\t#202020\nsize\ttext.line\t20\nsize\ttext.advance\t8\nsize\tfigure.height\t28\nsize\tfigure.inset\t10\nsize\tfigure.radius\t9\n";
/// A look that gives a figure no ground.
const BARE: &str = "color\ttext.ink\t#202020\nsize\ttext.line\t20\n";

#[test]
fn a_figure_stands_on_a_ground_of_its_own_with_its_text_in_the_middle() {
    let look = PLook::read(LIT);
    let mut sheet = PSheet::new(&look);
    let wide = sheet.measure("2,500");
    // As wide as its text and the room beside it, as high as the theme says.
    assert_eq!(size(&sheet, "2,500"), PSize { width: wide + 20.0, height: 28.0 });
    let rect = end(&mut sheet, PPoint::new(300.0, 100.0), "2,500", false);
    assert_eq!(rect, PRect::new(300.0 - wide - 20.0, 86.0, wide + 20.0, 28.0));
    // The ground is a label's light, from the left; then the text.
    let shade = PColor::new(1.0, 1.0, 1.0, 0.25);
    assert_eq!(sheet.mark[0], PMark::Blend { rect, radius: 9.0, from: shade, to: PColor { a: 0.0, ..shade }, turn: 90.0, stop: [0.0, 1.0] });
    assert!(matches!(&sheet.mark[1], PMark::Text { at, text, .. } if text == "2,500" && *at == PPoint::new(rect.origin.x + 10.0, 90.0)), "{:?}", sheet.mark[1]);
    assert_eq!(sheet.mark.len(), 2);
}

#[test]
fn a_figure_on_a_ground_runs_against_it_and_one_digit_stands_on_a_round_one() {
    let look = PLook::read(LIT);
    let mut sheet = PSheet::new(&look);
    let rect = start(&mut sheet, PPoint::new(40.0, 50.0), "3", true);
    // Never narrower than it is high; it begins where it was asked to.
    assert_eq!((rect.origin.x, rect.size.width.max(28.0), rect.size.height), (40.0, rect.size.width, 28.0));
    // Against the ground it stands on: the light comes from the right.
    let shade = PColor::new(1.0, 1.0, 1.0, 0.25);
    assert_eq!(sheet.mark[0], PMark::Blend { rect, radius: 9.0, from: PColor { a: 0.0, ..shade }, to: shade, turn: 90.0, stop: [0.0, 1.0] });
}

#[test]
fn a_figure_is_bare_where_the_theme_gives_it_no_ground_and_nothing_with_no_text() {
    let look = PLook::read(BARE);
    let mut sheet = PSheet::new(&look);
    end(&mut sheet, PPoint::new(300.0, 100.0), "12", false);
    assert!(matches!(sheet.mark.as_slice(), [PMark::Text { text, .. }] if text == "12"), "{:?}", sheet.mark);
    let mut sheet = PSheet::new(&look);
    assert_eq!(size(&sheet, ""), PSize::default());
    end(&mut sheet, PPoint::new(300.0, 100.0), "", false);
    assert!(sheet.mark.is_empty());
}
