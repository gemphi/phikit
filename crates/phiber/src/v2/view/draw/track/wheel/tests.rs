use super::*;
use gpui::{Size, px};

fn bounds(x: f32, y: f32, w: f32, h: f32) -> Bounds<Pixels> {
    Bounds { origin: Point { x: px(x), y: px(y) }, size: Size { width: px(w), height: px(h) } }
}

fn written(action: BAction) -> String {
    let BAction::Adjust(_, value) = action else { panic!("a wheel adjusts, got {action:?}") };
    value
}

#[test]
fn the_wheel_is_blenders_way_round() {
    // `hsvcircle_vals_from_pos`: cyan at the top, red at the bottom, and
    // hue rising clockwise from there — blue-magenta on the right.
    let near = |(h, s): (f32, f32), want: f32| (h - want).abs() < 1e-5 && (s - 1.0).abs() < 1e-5;
    assert!(near(BWheel::hs_at(0.5, 0.0), 0.5), "top is cyan: {:?}", BWheel::hs_at(0.5, 0.0));
    assert!(near(BWheel::hs_at(0.5, 1.0), 0.0) || near(BWheel::hs_at(0.5, 1.0), 1.0), "bottom is red");
    assert!(near(BWheel::hs_at(1.0, 0.5), 0.75), "right is 0.75: {:?}", BWheel::hs_at(1.0, 0.5));
    assert!(near(BWheel::hs_at(0.0, 0.5), 0.25), "left is 0.25");
    // The centre is grey, and outside the circle is its rim.
    assert_eq!(BWheel::hs_at(0.5, 0.5).1, 0.0);
    assert_eq!(BWheel::hs_at(-3.0, 0.5).1, 1.0);
}

#[test]
fn the_cursor_is_where_the_colour_reads_back_from() {
    let key = BPath::owned(vec!["c"]);
    for hsv in [[0.0, 1.0, 1.0], [0.3, 0.4, 0.8], [0.75, 0.9, 0.2], [0.999, 0.05, 1.0]] {
        let w = BWheel { key: key.clone(), hsv, alpha: 1.0, space: BPickSpace::Rgba };
        let (across, down) = w.marker();
        let (h, s) = BWheel::hs_at(across, down);
        let dh = (h - hsv[0]).abs().min(1.0 - (h - hsv[0]).abs());
        assert!(dh < 1e-4 && (s - hsv[1]).abs() < 1e-4, "{hsv:?} drew at {across},{down} and read back {h},{s}");
    }
}

#[test]
fn a_scene_colour_is_written_back_as_rgba_and_a_theme_colour_as_hsl() {
    let b = bounds(10.0, 20.0, 100.0, 100.0);
    // The right edge, full saturation: hue 0.75 at the value the colour had.
    let scene = BWheel::of_rgba(BPath::owned(vec!["light"]), [0.5, 0.5, 0.5, 1.0]);
    let raw = written(scene.adjust_at(b, Point { x: px(110.0), y: px(70.0) }));
    let Some(crate::v2::state::BValue::Color([r, g, bl, a])) = crate::v2::state::BValue::parse(crate::v2::state::val::BValueKind::Color, &raw) else {
        panic!("{raw:?} is not an RGBA the control reads")
    };
    assert!((r - 0.25).abs() < 1e-4 && g.abs() < 1e-4 && (bl - 0.5).abs() < 1e-4 && a == 1.0, "{raw}");
    let theme = BWheel::of_hsl(BPath::owned(vec!["ui_style", "colors", "bg_app"]), (210.0, 40.0, 60.0, 1.0));
    let raw = written(theme.adjust_at(b, Point { x: px(60.0), y: px(70.0) }));
    assert!(crate::v2::theme::parse_hsla(&raw).is_ok(), "{raw:?} is a colour the theme field accepts");
}

#[test]
fn the_value_bar_is_black_at_the_bottom_and_keeps_the_hue() {
    let b = bounds(0.0, 0.0, 16.0, 100.0);
    let w = BWheel::of_rgba(BPath::owned(vec!["c"]), [1.0, 0.0, 0.0, 0.5]);
    let bottom = written(w.value_at(b, Point { x: px(8.0), y: px(100.0) }));
    assert_eq!(bottom, "0 0 0 0.5");
    let half = written(w.value_at(b, Point { x: px(8.0), y: px(50.0) }));
    assert_eq!(half, "0.5 0 0 0.5", "red at half value, the alpha untouched");
}

#[test]
fn a_grey_theme_colour_keeps_its_hue_through_the_wheel() {
    // A grey has no hue in RGB; read from HSL it keeps the one it was given,
    // so lifting its value does not turn it red.
    let w = BWheel::of_hsl(BPath::owned(vec!["c"]), (210.0, 0.0, 24.0, 1.0));
    assert!((w.hsv[0] - 210.0 / 360.0).abs() < 1e-5);
    assert!((w.hsv[2] - 0.24).abs() < 1e-5 && w.hsv[1] == 0.0);
}

#[test]
fn hsv_and_rgb_round_trip() {
    for rgb in [[1.0, 0.0, 0.0], [0.2, 0.6, 0.4], [0.9, 0.9, 0.1], [0.0, 0.0, 0.0], [0.3, 0.3, 0.3], [2.0, 1.0, 0.5]] {
        let back = hsv_to_rgb(rgb_to_hsv(rgb));
        assert!(rgb.iter().zip(back).all(|(a, b)| (a - b).abs() < 1e-5), "{rgb:?} came back {back:?}");
    }
}

#[test]
fn a_node_socket_colour_is_written_into_the_editors_value_overlay() {
    use crate::phiphx::blend::shade::socket::BSocketDefault;
    use crate::v2::view::draw::canvas::node::edit::{BValueEdits, decode};
    // An overlay already holding another socket's edit keeps it.
    let mut edits = BValueEdits::default();
    edits.set("Mix", "Factor", BSocketDefault::Float(0.25));
    let space = BPickSpace::Socket { node: "Principled BSDF".to_string(), socket: "Base Color".to_string(), edits };
    let w = BWheel::of_socket(BPath::owned(vec!["shader", "node", "values"]), [0.8, 0.8, 0.8, 1.0], space);
    let raw = written(w.value_at(bounds(0.0, 0.0, 16.0, 100.0), Point { x: px(8.0), y: px(100.0) }));
    let back = decode(&raw);
    assert_eq!(back.get("Mix", "Factor"), Some(&BSocketDefault::Float(0.25)), "the other edit survives");
    assert_eq!(back.get("Principled BSDF", "Base Color"), Some(&BSocketDefault::Color([0.0, 0.0, 0.0, 1.0])), "value 0 is black");
}
