use crate::pui::core::light::{edge, ink, label, pane, surface};
use crate::pui::core::*;
use crate::theme::{PLook, PTheme};

/// A look with light: white, a half of it at the most, and the shares
/// puikit gives a shade at rest and a soft one.
const LIT: &str = "color\tshade.light\t#ffffff\nsize\tshade.share\t0.5\nsize\tshade.rest\t0.72\nsize\tshade.soft\t0.48\ncolor\twindow.fill\t#101010\ncolor\ttext.ink\t#202020\n";
/// A flat look: a fill and an ink of its own, and no light.
const FLAT: &str = "color\tcard.fill\t#ff0000\ncolor\tcard.ink\t#00ff00\n";
/// What makes a look's panes glass.
const GLASS: &str = "size\tglass.blur\t24\nsize\tglass.saturation\t1.2\ncolor\tglass.tint\t#101010\nsize\tglass.share\t0.35\nsize\tglass.lens\t26\ncolor\tglass.edge\t#ffffff\nsize\tglass.rim\t0.3\nsize\twindow.glass\t1\n";

const RECT: PRect = PRect { origin: PPoint { x: 10.0, y: 20.0 }, size: PSize { width: 100.0, height: 40.0 } };

fn white(alpha: f32) -> PColor {
    PColor::new(1.0, 1.0, 1.0, alpha)
}

#[test]
fn a_shade_is_the_theme_s_light_at_one_of_three_strengths() {
    let look = PLook::read(LIT);
    let alpha = [PLight::Strong, PLight::Rest, PLight::Soft].map(|light| light.shade(&look).map(|shade| shade.a));
    assert_eq!(alpha, [Some(0.5), Some(0.5 * 0.72), Some(0.5 * 0.48)]);
    // A theme that names no light has no shade.
    assert_eq!(PLight::Rest.shade(&PLook::read(FLAT)), None);
}

#[test]
fn a_surface_is_lit_from_its_side_and_a_label_from_its_left_or_its_right() {
    let look = PLook::read(LIT);
    let mut sheet = PSheet::new(&look);
    surface(&mut sheet, RECT, 8.0, PLight::Strong, "card.fill");
    label(&mut sheet, RECT, 8.0, PLight::Strong, false, "card.fill");
    label(&mut sheet, RECT, 8.0, PLight::Strong, true, "card.fill");
    // The light fades to itself with nothing of it left, so no other
    // colour shows on the way.
    let blend = |from: f32, to: f32, turn: f32| PMark::Blend { rect: RECT, radius: 8.0, from: white(from), to: white(to), turn, stop: [0.0, 1.0] };
    assert_eq!(sheet.mark, [blend(0.5, 0.0, 90.0), blend(0.5, 0.0, 90.0), blend(0.0, 0.5, 90.0)]);
}

#[test]
fn content_runs_side_to_side_whatever_the_turn_and_only_a_pane_runs_up_or_down() {
    let turned = |turn: f32| PLook::read(&format!("{LIT}size\tsurface.turn\t{turn}\nsize\tlabel.turn\t{turn}\n"));
    let turn = |sheet: &PSheet| sheet.mark.iter().filter_map(|mark| match mark {
        PMark::Blend { from, to, turn, .. } => Some((*turn, from.a > to.a)),
        _ => None,
    }).collect::<Vec<_>>();
    for (look, side, pane_turn, mirrored) in [(turned(0.0), 90.0, 0.0, false), (turned(180.0), 90.0, 180.0, false), (turned(270.0), 270.0, 270.0, true), (turned(90.0), 90.0, 90.0, false)] {
        let mut sheet = PSheet::new(&look);
        surface(&mut sheet, RECT, 8.0, PLight::Strong, "");
        label(&mut sheet, RECT, 8.0, PLight::Strong, false, "");
        pane(&mut sheet, RECT, 8.0, "");
        // The surface side to side; the label lit from its left, or from
        // its right where the look turns the light to the left; the pane
        // as the look turns it.
        let got = turn(&sheet);
        assert_eq!(got[0], (side, true));
        assert_eq!(got[1], (90.0, !mirrored));
        assert_eq!(got.last().copied(), Some((pane_turn, true)));
    }
}

#[test]
fn a_flat_theme_fills_with_the_token_it_is_given_and_an_empty_token_draws_nothing() {
    let look = PLook::read(FLAT);
    let mut sheet = PSheet::new(&look);
    surface(&mut sheet, RECT, 8.0, PLight::Rest, "card.fill");
    label(&mut sheet, RECT, 8.0, PLight::Rest, true, "card.fill");
    pane(&mut sheet, RECT, 8.0, "card.fill");
    surface(&mut sheet, RECT, 8.0, PLight::Rest, "");
    let fill = PMark::Fill { rect: RECT, radius: 8.0, color: PColor::new(1.0, 0.0, 0.0, 1.0) };
    assert_eq!(sheet.mark, [fill.clone(), fill.clone(), fill]);
    // The ink on it is the token's there, and the theme's own on light.
    assert_eq!(ink(&sheet, "card.ink"), PColor::new(0.0, 1.0, 0.0, 1.0));
    assert_eq!(ink(&PSheet::new(&PLook::read(LIT)), "card.ink"), sheet_ink());
}

fn sheet_ink() -> PColor {
    PLook::read(LIT).color("text.ink").expect("in the look")
}

#[test]
fn a_pane_stands_on_the_window_s_ground_or_is_glass_where_the_theme_has_glass() {
    let look = PLook::read(LIT);
    let mut sheet = PSheet::new(&look);
    pane(&mut sheet, RECT, 8.0, "");
    let ground = PMark::Fill { rect: RECT, radius: 8.0, color: look.color("window.fill").expect("in the look") };
    let lit = PMark::Blend { rect: RECT, radius: 8.0, from: white(0.5 * 0.72), to: white(0.0), turn: 0.0, stop: [0.0, 1.0] };
    assert_eq!(sheet.mark, [ground, lit.clone()]);
    // With glass, what is behind it shows through, and it has no ground.
    let glassy = look.with(GLASS);
    let mut sheet = PSheet::new(&glassy);
    pane(&mut sheet, RECT, 8.0, "");
    let glass = PGlass { blur: 24.0, saturation: 1.2, tint: look.color("window.fill").expect("in the look"), share: 0.35, lens: 26.0, edge: white(1.0), rim: 0.3 };
    assert_eq!(sheet.mark, [PMark::Glass { rect: RECT, radius: 8.0, glass }, lit]);
}

#[test]
fn a_theme_has_glass_when_it_gives_a_blur_and_a_glass_window_when_it_says_so() {
    let (plain, glassy) = (PLook::read(LIT), PLook::read(LIT).with(GLASS));
    assert_eq!((PGlass::of(&plain), PGlass::window(&plain)), (None, false));
    assert!(PGlass::of(&glassy).is_some() && PGlass::window(&glassy));
    // No blur is no glass, and a painter draws none.
    assert_eq!(PGlass::of(&glassy.with("size\tglass.blur\t0\n")), None);
    let mut sheet = PSheet::new(&plain);
    sheet.glass(RECT, 8.0);
    assert!(sheet.mark.is_empty());
}

#[test]
fn a_painter_with_one_colour_to_a_fill_blends_with_the_one_that_shows_more() {
    let mut board = crate::test::core::Board::default();
    board.blend(RECT, 8.0, white(0.0), white(0.5), 90.0);
    board.blend(RECT, 8.0, white(0.7), white(0.0), 0.0);
    assert_eq!(board.fill, [(RECT, white(0.5)), (RECT, white(0.7))]);
}

#[test]
fn a_pane_at_an_edge_of_the_window_is_lit_toward_the_page() {
    let look = PLook::read(LIT);
    let mut sheet = PSheet::new(&look);
    edge(&mut sheet, RECT, 0.0, false, "bar.fill");
    edge(&mut sheet, RECT, 0.0, true, "bar.fill");
    // On the window's ground; at the left edge nothing at its left and a
    // shade at rest at its right, at the right edge the other way.
    let ground = PMark::Fill { rect: RECT, radius: 0.0, color: look.color("window.fill").expect("in the look") };
    let blend = |from: f32, to: f32| PMark::Blend { rect: RECT, radius: 0.0, from: white(from), to: white(to), turn: 90.0, stop: [0.0, 1.0] };
    assert_eq!(sheet.mark, [ground.clone(), blend(0.0, 0.5 * 0.72), ground, blend(0.5 * 0.72, 0.0)]);
    // Glass where the theme has glass, and in a flat theme its own fill.
    let glass = PLook::read(&format!("{LIT}{GLASS}"));
    let mut pane = PSheet::new(&glass);
    edge(&mut pane, RECT, 0.0, false, "bar.fill");
    assert!(matches!(pane.mark[0], PMark::Glass { .. }) && pane.mark.len() == 2);
    let flat = PLook::read(FLAT);
    let mut plain = PSheet::new(&flat);
    edge(&mut plain, RECT, 0.0, false, "card.fill");
    assert_eq!(plain.mark, [PMark::Fill { rect: RECT, radius: 0.0, color: flat.color("card.fill").expect("in the look") }]);
}
