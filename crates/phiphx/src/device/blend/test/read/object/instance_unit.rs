use super::*;

#[test]
fn an_instance_stands_where_its_instancer_puts_the_member() {
    let mut member = BlendSceneObject::default();
    member.name = "Fan".into();
    member.vertices = vec![[0.0, 0.0, 1.0]];
    let world = Mat4::from_translation(Vec3::new(5.0, 2.0, 0.0)) * Mat4::from_rotation_z(std::f32::consts::FRAC_PI_2);
    let copy = instance_of(&member, "Seat.004", world);
    assert_eq!(copy.name, "Seat.004/Fan");
    assert_eq!((copy.block_address, copy.parent_address), (0, 0));
    assert!(copy.is_instance);
    assert_eq!(copy.vertices, member.vertices, "the member's own geometry");
    assert!((Vec3::from(copy.location) - Vec3::new(5.0, 2.0, 0.0)).length() < 1e-6);
    assert!((copy.quaternion[0] - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-6, "turned a quarter about Z");
}
