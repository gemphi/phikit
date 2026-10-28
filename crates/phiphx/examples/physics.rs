//! Print the physics settings a `.blend` carries (`blend/physics/`):
//! `cargo run -p phiphx --example physics -- file.blend`
//!
//! Settings only, as the Physics tabs show them. Nothing simulates them yet.
use phiphx::blend::physics::BlendPhysics;

fn main() {
    for path in std::env::args().skip(1) {
        let t = std::time::Instant::now();
        let model = phiphx::phx::Phx::open(std::path::Path::new(&path)).expect("open");
        println!("{path}: {} objects, opened in {:.2?}", model.objects.len(), t.elapsed());
        print(&model.physics);
    }
}

fn print(p: &BlendPhysics) {
    if p.is_empty() {
        println!("  no physics");
        return;
    }
    println!("  gravity {:?}", p.gravity);
    match &p.world {
        Some(w) => println!(
            "  Rigid Body World: collection '{}', constraints '{}', {}, substeps {}, iterations {}, time scale {}, split impulse {}, cache {}-{} step {} baked {}, gravity weight {}, all {}",
            w.collection, w.constraints, on(w.enabled), w.substeps_per_frame, w.solver_iterations, w.time_scale, w.split_impulse,
            w.cache.frame_start, w.cache.frame_end, w.cache.step, w.cache.baked, w.effector_weights.gravity, w.effector_weights.all
        ),
        None => println!("  no Rigid Body World"),
    }
    for b in &p.bodies {
        println!(
            "  body {:<10} {:<8} {:<12} source {:?} dynamic {} animated {} mass {} friction {} bounce {} margin {}{} damping {}/{} deactivation {}{} sleep {}/{} collections {:#07x}",
            b.object, b.kind.blender(), b.shape.blender(), b.mesh_source, b.enabled, b.kinematic, b.mass, b.friction, b.restitution,
            b.margin, if b.use_margin { " (on)" } else { "" }, b.linear_damping, b.angular_damping, b.use_deactivation,
            if b.start_deactivated { " (starts asleep)" } else { "" }, b.linear_sleep, b.angular_sleep, b.collections
        );
    }
    for j in &p.joints {
        println!(
            "  joint {:<8} {:<14} {} - {} {} collisions {} breaking {:?} iterations {:?} spring {:?} motor lin {:?} ang {:?}",
            j.object, j.kind.blender(), j.first, j.second, on(j.enabled),
            if j.disable_collisions { "off" } else { "on" },
            j.use_breaking.then_some(j.breaking_threshold),
            j.override_iterations.then_some(j.solver_iterations),
            j.spring_kind,
            j.use_motor_linear.then_some((j.motor_linear_velocity, j.motor_linear_impulse)),
            j.use_motor_angular.then_some((j.motor_angular_velocity, j.motor_angular_impulse)),
        );
        for (space, axes) in [("lin", &j.linear), ("ang", &j.angular)] {
            for (k, a) in axes.iter().enumerate() {
                if a.use_limit || a.use_spring {
                    println!(
                        "      {space} {} limit {:?} spring {:?}",
                        ["x", "y", "z"][k],
                        a.use_limit.then_some((a.lower, a.upper)),
                        a.use_spring.then_some((a.stiffness, a.damping))
                    );
                }
            }
        }
    }
    for c in &p.cloths {
        println!(
            "  cloth {:<8} '{}' quality {} mass {} air {} stiffness {}/{}/{}/{} damping {}/{}/{}/{} {} pin '{}' x{} pressure {:?} sewing {} shrink {:?}",
            c.object, c.modifier, c.quality, c.mass, c.air_damping, c.tension, c.compression, c.shear, c.bending,
            c.tension_damping, c.compression_damping, c.shear_damping, c.bending_damping,
            if c.angular_bending { "angular" } else { "linear" }, c.pin_group, c.pin_stiffness,
            c.pressure.then_some((c.pressure_force, c.target_volume)), c.sewing, c.shrink
        );
        let k = &c.collision;
        println!(
            "      collisions {} quality {} distance {} friction {} clamp {} collection '{}'; self {} distance {} friction {} clamp {}",
            on(k.enabled), k.quality, k.distance, k.friction, k.impulse_clamp, k.collection, on(k.self_enabled),
            k.self_distance, k.self_friction, k.self_impulse_clamp
        );
    }
    for c in &p.colliders {
        println!(
            "  collider {:<8} damping {} thickness {}/{} cloth friction {} single sided {} override normals {} permeability {} stickiness {} kill {} particle damping {}+{} friction {}+{}",
            c.object, c.damping, c.thickness_outer, c.thickness_inner, c.cloth_friction, c.single_sided, c.override_normals,
            c.permeability, c.stickiness, c.kill_particles, c.damping_factor, c.damping_random, c.friction_factor, c.friction_random
        );
    }
    for f in &p.fields {
        println!(
            "  field {:<8} {} {:?} strength {} flow {} noise {} seed {} power {} max {:?} min {:?}",
            f.object, f.kind.blender(), f.shape, f.strength, f.flow, f.noise, f.seed, f.power, f.max_distance, f.min_distance
        );
    }
    for s in &p.softs {
        println!(
            "  soft {:<8} friction {} mass {} '{}' speed {} goal {} {} ({}..{}) spring {} friction {} '{}' edges {} pull {} push {} damp {} plastic {} bend {} length {} shear {} quads {} '{}' self {} {:?} ball {} stiff {} damp {} face {} edge {} steps {}..{} auto {} error {} choke {} fuzzy {} cache {}-{}",
            s.object, s.friction, s.mass, s.mass_group, s.speed, on(s.goal), s.goal_default, s.goal_min, s.goal_max, s.goal_spring,
            s.goal_friction, s.goal_group, on(s.edges), s.pull, s.push, s.damping, s.plastic, s.bending, s.spring_length, s.shear,
            s.stiff_quads, s.spring_group, on(s.self_collision), s.collision, s.ball_size, s.ball_stiffness, s.ball_damping,
            s.face_collision, s.edge_collision, s.step_min, s.step_max, s.auto_step, s.error_limit, s.choke, s.fuzzy,
            s.cache.frame_start, s.cache.frame_end
        );
    }
}

fn on(b: bool) -> &'static str {
    match b {
        true => "on",
        false => "off",
    }
}
