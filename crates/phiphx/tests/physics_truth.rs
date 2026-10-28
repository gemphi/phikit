//! The Physics tabs, read from the file (`blend/physics/`) out of
//! `physics.blend` (`fixtures/make_physics.py`, Blender 5.x through pip
//! bpy). Every setting in the fixture is away from its default, so a value
//! that comes back as the default is a field that was not read.

use phiphx::blend::physics::*;
use phiphx::blend::BlendFile;
use std::path::PathBuf;

fn physics() -> BlendPhysics {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/physics.blend");
    BlendFile::parse_bytes(&std::fs::read(&path).unwrap()).unwrap().extract_scene().unwrap().props.physics
}

fn near(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-5
}

#[test]
fn the_world_and_the_scene_gravity() {
    let p = physics();
    assert!(near(p.gravity[2], -5.0), "{:?}", p.gravity);
    let w = p.world.as_ref().expect("a Rigid Body World");
    assert_eq!(w.collection, "RigidBodyWorld");
    assert_eq!(w.constraints, "RigidBodyConstraints");
    assert!(w.enabled);
    assert_eq!((w.substeps_per_frame, w.solver_iterations), (20, 25));
    assert!(near(w.time_scale, 1.5));
    assert!(w.split_impulse);
    assert_eq!((w.cache.frame_start, w.cache.frame_end, w.cache.baked), (5, 120, false));
    assert!(near(w.effector_weights.gravity, 0.8));
    // Wind is `weight[4]`: All, Force, Vortex, Magnetic, Wind.
    assert!(near(w.effector_weights.weights[3], 0.5), "{:?}", w.effector_weights.weights);
    assert!(near(w.effector_weights.all, 1.0));
}

#[test]
fn every_body_shape_and_setting() {
    let p = physics();
    let of = |name: &str| p.bodies.iter().find(|b| b.object == name).unwrap_or_else(|| panic!("{name}"));

    let ground = of("Ground");
    assert_eq!((ground.kind, ground.shape), (BRigidKind::Passive, BRigidShape::Mesh));
    assert!(near(ground.friction, 0.9));

    let crate_ = of("Crate");
    assert_eq!((crate_.kind, crate_.shape), (BRigidKind::Active, BRigidShape::Box));
    assert!(near(crate_.mass, 2.5) && near(crate_.friction, 0.3) && near(crate_.restitution, 0.6));
    assert!(crate_.use_margin && near(crate_.margin, 0.02));
    assert!(near(crate_.linear_damping, 0.1) && near(crate_.angular_damping, 0.2));
    assert!(crate_.use_deactivation && crate_.start_deactivated);
    assert!(near(crate_.linear_sleep, 0.5) && near(crate_.angular_sleep, 0.6));
    assert_eq!(crate_.collections, (1 << 0) | (1 << 3));
    assert!(crate_.enabled && !crate_.kinematic);

    let ball = of("Ball");
    assert_eq!(ball.shape, BRigidShape::Sphere);
    assert!(near(ball.mass, 0.7) && ball.kinematic);

    let pill = of("Pill");
    assert_eq!((pill.shape, pill.mesh_source, pill.enabled), (BRigidShape::Capsule, BMeshSource::Final, false));

    assert_eq!(of("Can").shape, BRigidShape::Cylinder);
    assert_eq!(of("Funnel").shape, BRigidShape::Cone);
    assert_eq!(of("Rock").shape, BRigidShape::ConvexHull);
    assert_eq!((of("Shell").shape, of("Shell").kind), (BRigidShape::Mesh, BRigidKind::Active));
    assert_eq!(p.bodies.len(), 8);
}

#[test]
fn every_joint_kind_and_setting() {
    let p = physics();
    let of = |name: &str| p.joints.iter().find(|j| j.object == name).unwrap_or_else(|| panic!("{name}"));

    let hinge = of("Hinge");
    assert_eq!((hinge.kind, hinge.first.as_str(), hinge.second.as_str()), (BJointKind::Hinge, "Crate", "Ball"));
    assert!(hinge.enabled && !hinge.disable_collisions);
    assert!(hinge.use_breaking && near(hinge.breaking_threshold, 15.0));
    assert!(hinge.override_iterations && hinge.solver_iterations == 30);
    let z = hinge.angular[2];
    assert!(z.use_limit && near(z.lower, -0.5) && near(z.upper, 1.0), "{z:?}");
    assert!(!hinge.angular[0].use_limit && !hinge.linear[0].use_limit);

    let spring = of("Spring");
    assert_eq!((spring.kind, spring.spring_kind), (BJointKind::GenericSpring, BSpringKind::Spring1));
    let x = spring.linear[0];
    assert!(x.use_spring && near(x.stiffness, 12.5) && near(x.damping, 0.7), "{x:?}");
    assert!(x.use_limit && near(x.lower, -0.2) && near(x.upper, 0.3));
    assert!(!spring.linear[1].use_spring);

    let motor = of("Motor");
    assert_eq!(motor.kind, BJointKind::Motor);
    assert!(motor.use_motor_linear && near(motor.motor_linear_velocity, 2.0) && near(motor.motor_linear_impulse, 50.0));
    assert!(motor.use_motor_angular && near(motor.motor_angular_velocity, 3.0) && near(motor.motor_angular_impulse, 20.0));

    let fixed = of("Fixed");
    assert_eq!((fixed.kind, fixed.enabled), (BJointKind::Fixed, false));
    assert!(fixed.disable_collisions, "the default");
    assert_eq!(of("Point").kind, BJointKind::Point);
    assert_eq!(of("Slider").kind, BJointKind::Slider);
    assert_eq!(of("Piston").kind, BJointKind::Piston);
    assert_eq!(of("Generic").kind, BJointKind::Generic);
    assert_eq!(p.joints.len(), 8);
}

#[test]
fn the_cloth_and_its_collisions() {
    let p = physics();
    let [c] = p.cloths.as_slice() else { panic!("one cloth, got {}", p.cloths.len()) };
    assert_eq!((c.object.as_str(), c.modifier.as_str()), ("Flag", "Cloth"));
    assert_eq!(c.quality, 7);
    assert!(near(c.mass, 0.4) && near(c.air_damping, 2.0));
    assert!(near(c.tension, 25.0) && near(c.compression, 20.0) && near(c.shear, 8.0) && near(c.bending, 1.5));
    assert!(near(c.tension_damping, 7.0) && near(c.compression_damping, 6.0));
    assert!(near(c.shear_damping, 4.0) && near(c.bending_damping, 0.8));
    assert!(c.angular_bending);
    assert_eq!(c.pin_group, "Pin");
    assert!(near(c.pin_stiffness, 3.0));
    assert!(c.pressure && near(c.pressure_force, 2.0) && c.target_volume.is_none());
    assert!(!c.sewing && !c.dynamic_mesh && !c.internal_springs);
    let k = &c.collision;
    assert!(k.enabled && k.quality == 4 && near(k.distance, 0.02) && near(k.friction, 4.0) && near(k.impulse_clamp, 0.5));
    assert!(k.self_enabled && near(k.self_distance, 0.01) && near(k.self_friction, 7.0));
}

#[test]
fn the_collider_the_field_and_the_soft_body() {
    let p = physics();
    let [c] = p.colliders.as_slice() else { panic!("one collider, got {}", p.colliders.len()) };
    assert_eq!(c.object, "Floor");
    assert!(near(c.damping, 0.3) && near(c.thickness_outer, 0.05) && near(c.thickness_inner, 0.3));
    assert!(near(c.cloth_friction, 6.0));
    assert!(!c.single_sided && c.override_normals);
    assert!(near(c.permeability, 0.2) && near(c.stickiness, 0.4) && c.kill_particles);
    assert!(near(c.damping_factor, 0.25) && near(c.friction_factor, 0.35));

    let [f] = p.fields.as_slice() else { panic!("one field, got {}", p.fields.len()) };
    assert_eq!((f.object.as_str(), f.kind, f.shape), ("Gust", BFieldKind::Wind, BFieldShape::Plane));
    assert!(near(f.strength, 4.5) && near(f.flow, 0.3) && near(f.noise, 0.2) && f.seed == 7);
    assert!(near(f.power, 1.5));
    assert_eq!(f.max_distance.map(|d| near(d, 12.0)), Some(true));
    assert!(f.min_distance.is_none());

    let [s] = p.softs.as_slice() else { panic!("one soft body, got {}", p.softs.len()) };
    assert_eq!(s.object, "Jelly");
    assert!(near(s.friction, 0.7) && near(s.mass, 2.0) && near(s.speed, 1.2));
    assert!(s.goal && near(s.goal_default, 0.6) && near(s.goal_min, 0.1) && near(s.goal_max, 0.9));
    assert!(near(s.goal_spring, 0.4) && near(s.goal_friction, 0.3));
    assert_eq!(s.goal_group, "Goal");
    assert!(s.edges && near(s.pull, 0.6) && near(s.push, 0.5) && near(s.damping, 0.7));
    assert!(near(s.bending, 0.3) && near(s.shear, 0.8));
    assert_eq!((s.plastic, s.spring_length), (2, 50));
    assert!(s.self_collision && near(s.ball_size, 0.3));
    assert_eq!(s.collision, BSoftCollision::Manual);
    assert_eq!((s.step_min, s.step_max, s.choke), (15, 350, 2));
    // The switches the fixture leaves alone, as bpy reports them for this
    // file: Auto-Step on (`solverflags & SBSO_OLDERR`, bit 1, not bit 0),
    // edge and face collision off, Stiff Quads off.
    assert!(s.auto_step && !s.edge_collision && !s.face_collision && !s.stiff_quads);
}

#[test]
fn a_file_without_physics_reads_as_empty() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/LIGHTS.blend");
    let p = BlendFile::parse_bytes(&std::fs::read(&path).unwrap()).unwrap().extract_scene().unwrap().props.physics;
    assert!(p.is_empty(), "{p:?}");
    assert!(near(p.gravity[2], -9.81));
}
