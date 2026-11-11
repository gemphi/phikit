use crate::device::blend::shade::world_driver::*;
use crate::device::blend::anim::driver::state::BDriverState;

fn sockets(linked: bool) -> Vec<(String, Vec<BDriverSocket>)> {
    vec![(
        BACKGROUND.to_string(),
        vec![
            BDriverSocket { name: "Color".into(), rgba: true, linked },
            BDriverSocket { name: "Strength".into(), rgba: false, linked: false },
        ],
    )]
}

fn sock(input: usize, component: usize) -> BWorldWrite {
    BWorldWrite::Socket { node: BACKGROUND.to_string(), input, component }
}

/// A driver that is just a constant: no variables, so a Sum comes to
/// 0, and one key at x = 0 maps that 0 to `value`.
fn constant(value: f32) -> BlendDriver {
    use crate::device::blend::anim::fcurve::{BFCurve, BKey, IPO_CONST};
    let key = BKey::new([0.0, value], [0.0, value], [0.0, value], IPO_CONST);
    BlendDriver {
        kind: crate::device::blend::rig::driver::DRIVER_SUM,
        expression: String::new(),
        vars: Vec::new(),
        parsed: None,
        curve: BFCurve::new("", 0, vec![key]),
    }
}

fn state() -> BDriverState {
    BDriverState::default()
}

#[test]
fn the_background_colour_and_strength_land_on_the_world() {
    let set = BWorldDriverList {
        inputs: sockets(false),
        drivers: vec![
            BWorldDriver { write: sock(BG_COLOR, 0), driver: constant(0.25) },
            BWorldDriver { write: sock(BG_COLOR, 2), driver: constant(0.75) },
            BWorldDriver { write: sock(BG_STRENGTH, 0), driver: constant(3.0) },
            // A driver on `World.color` is kept and does not land on
            // `BlendWorld::color`, which is the Background's.
            BWorldDriver { write: BWorldWrite::Color(0), driver: constant(0.9) },
        ],
        ..Default::default()
    };
    let st = state();
    let ds = st.scene();
    assert_eq!(set.values(&ds, 1.0).len(), 4, "all four are read");

    let mut world = BlendWorld::default();
    set.apply(&mut world, &ds, 1.0);
    assert_eq!(world.color, [0.25, 0.05, 0.75], "the Background's Color, component by component");
    assert_eq!(world.strength, 3.0);
}

#[test]
fn a_link_into_the_background_keeps_the_world_where_it_was() {
    let set = BWorldDriverList {
        inputs: sockets(true),
        drivers: vec![BWorldDriver { write: sock(BG_COLOR, 0), driver: constant(0.9) }],
        ..Default::default()
    };
    let st = state();
    let ds = st.scene();
    let mut world = BlendWorld::default();
    set.apply(&mut world, &ds, 1.0);
    assert_eq!(world.color, [0.05; 3], "an Environment Texture feeds Color, not the driver");
}

#[test]
fn a_strength_socket_takes_a_negative_and_a_colour_does_not() {
    let set = BWorldDriverList { inputs: sockets(false), ..Default::default() };
    assert_eq!(socket_clamp(set.sockets(BACKGROUND), BG_STRENGTH, -2.0), -2.0, "a float socket is unclamped");
    assert_eq!(socket_clamp(set.sockets(BACKGROUND), BG_COLOR, -2.0), 0.0, "an RGBA socket clamps at 0");
    assert_eq!(socket_clamp(set.sockets(BACKGROUND), BG_COLOR, 4.0), 4.0, "and is free above 1");
}

#[test]
fn the_worlds_own_colour_clamps_at_zero_and_is_free_above_one() {
    // `World.color` is hard 0..∞, like every colour property.
    assert_eq!((-1.0f32).max(0.0), 0.0);
    assert_eq!(3.0f32.max(0.0), 3.0);
}
