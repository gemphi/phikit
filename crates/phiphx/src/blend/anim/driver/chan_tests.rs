use super::*;
use crate::blend::rig::math::BEulerOrder;
use glam::{Mat4, Quat, Vec3};

fn near(a: f32, b: f32) -> bool {
    (a - b).abs() < 1.0e-4
}

#[test]
fn a_location_channel_is_the_matrix_translation_on_that_axis() {
    let m = Mat4::from_translation(Vec3::new(1.5, -2.0, 3.25));
    assert!(near(m.trans_chan(0, ROTMODE_AUTO, 1, None), 1.5));
    assert!(near(m.trans_chan(1, ROTMODE_AUTO, 1, None), -2.0));
    assert!(near(m.trans_chan(2, ROTMODE_AUTO, 1, None), 3.25));
}

#[test]
fn a_scale_channel_is_the_length_of_that_column_and_the_average_is_the_cube_root_of_the_volume() {
    let m = Mat4::from_scale(Vec3::new(2.0, 3.0, 4.0));
    assert!(near(m.trans_chan(TRANSCHAN_SCALEX, ROTMODE_AUTO, 1, None), 2.0));
    assert!(near(m.trans_chan(TRANSCHAN_SCALEX + 1, ROTMODE_AUTO, 1, None), 3.0));
    assert!(near(m.trans_chan(TRANSCHAN_SCALEX + 2, ROTMODE_AUTO, 1, None), 4.0));
    assert!(near(m.trans_chan(TRANSCHAN_SCALE_AVG, ROTMODE_AUTO, 1, None), 24.0f32.cbrt()), "the geometric mean of the three");
}

#[test]
fn an_auto_rotation_channel_is_the_euler_in_the_targets_own_order() {
    let eul = Vec3::new(0.3, -0.4, 0.5);
    let m = Mat4::from_mat3(BEulerOrder(ROT_MODE_EUL).to_mat3(eul));
    for (i, want) in [eul.x, eul.y, eul.z].into_iter().enumerate() {
        let got = m.trans_chan(TRANSCHAN_ROTX + i as u32, ROTMODE_AUTO, 1, None);
        assert!(near(got, want), "axis {i}: {got} vs {want}");
    }
}

#[test]
fn a_quaternion_channel_is_the_component_and_rotw_is_the_first() {
    let q = Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), 0.8);
    let m = Mat4::from_quat(q);
    assert!(near(m.trans_chan(TRANSCHAN_ROTW, ROTMODE_QUATERNION, 1, None), q.w));
    assert!(near(m.trans_chan(TRANSCHAN_ROTX + 2, ROTMODE_QUATERNION, 1, None), q.z));
}

#[test]
fn a_swing_twist_channel_splits_the_turn_around_its_axis() {
    // A pure turn about X is all twist and no swing, in Swing/Twist X.
    let m = Mat4::from_quat(Quat::from_axis_angle(Vec3::X, 0.7));
    assert!(near(m.trans_chan(TRANSCHAN_ROTX, ROTMODE_SWING_TWIST_X, 1, None), 0.7), "the twist is the angle itself");
    assert!(near(m.trans_chan(TRANSCHAN_ROTX + 1, ROTMODE_SWING_TWIST_X, 1, None), 0.0), "no swing left over");
    assert!(near(m.trans_chan(TRANSCHAN_ROTX + 2, ROTMODE_SWING_TWIST_X, 1, None), 0.0));
}

#[test]
fn a_channel_past_the_last_one_reads_zero() {
    assert!(near(Mat4::IDENTITY.trans_chan(MAX_TRANSCHAN, ROTMODE_AUTO, 1, None), 0.0));
}

#[test]
fn an_auto_euler_is_pulled_towards_the_targets_saved_one() {
    // 350° about Z decomposes to -0.17rad; against a saved 2π-ish Euler the
    // compatible solution is the one a whole turn up.
    let m = Mat4::from_mat3(BEulerOrder(ROT_MODE_EUL).to_mat3(Vec3::new(0.0, 0.0, -0.1745)));
    let old = Vec3::new(0.0, 0.0, 6.1);
    let got = m.trans_chan(TRANSCHAN_ROTX + 2, ROTMODE_AUTO, ROT_MODE_EUL, Some(old));
    assert!(got > 5.0, "wrapped towards the saved value, not left at -0.17: {got}");
}
