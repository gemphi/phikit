use crate::scene::ui::*;

#[test]
fn front_view_looks_along_plus_y_level() {
    // Numpad 1: back axis −Y.
    let (yaw, pitch) = SceneView::angles([0.0, -1.0, 0.0]);
    assert!(yaw.abs() < 1e-3 && pitch.abs() < 1e-3, "{yaw} {pitch}");
}

#[test]
fn top_view_pitches_straight_down() {
    let (_, pitch) = SceneView::angles([0.0, 0.0, 1.0]);
    assert!((pitch - 90.0).abs() < 1e-3);
}

#[test]
fn right_view_yaws_a_quarter_turn() {
    // Numpad 3 looks along −X: back axis +X.
    let (yaw, _) = SceneView::angles([1.0, 0.0, 0.0]);
    assert!((yaw - 270.0).abs() < 1e-3, "{yaw}");
}

#[test]
fn the_solid_effects_read_their_flag_bits() {
    let sh = crate::device::blend::BlendShading { flag: (1 << 0) | (1 << 2) | (1 << 10) | (1 << 11), ..Default::default() };
    let fx = SceneSolidFx::of(&sh);
    assert!(fx.outline && fx.shadow && fx.backface_culling && fx.dof && !fx.cavity && !fx.specular);
    assert!(!SceneSolidFx::of(&Default::default()).dof, "Blender's default view has no depth of field");
}
