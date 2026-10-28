"""physics.blend: every physics setting phiphx reads (`blend/physics/`), each
set away from its default so `tests/physics_truth.rs` can tell a read value
from a fallback.

  Rigid Body World  substeps 20, solver iterations 25, time scale 1.5, split
                    impulse on, cache frames 5-120, gravity weight 0.8, wind
                    weight 0.5; the scene's gravity (0, 0, -5)
  Ground            passive MESH, friction 0.9
  Crate             active BOX, mass 2.5, friction 0.3, bounciness 0.6, margin
                    0.02 (on), damping 0.1 / 0.2, deactivation on at 0.5 / 0.6,
                    starts deactivated, collision collections 0 and 3
  Ball              active SPHERE, mass 0.7, animated (kinematic)
  Pill              active CAPSULE, mesh source FINAL, dynamic off
  Can / Funnel / Rock / Shell   CYLINDER / CONE / CONVEX_HULL / MESH (active)
  Hinge             HINGE Crate-Ball, Z angle limit -0.5..1.0, breaking at 15,
                    collisions NOT disabled, 30 solver iterations
  Spring            GENERIC_SPRING Crate-Can, X spring 12.5 / damping 0.7,
                    X linear limit -0.2..0.3, SPRING1
  Motor             MOTOR Ball-Can, linear 2.0 (max impulse 50), angular 3.0
                    (max impulse 20)
  Fixed (disabled), Point, Slider, Piston, Generic
  Flag              Cloth: quality 7, mass 0.4, air 2.0, tension 25,
                    compression 20, shear 8, bending 1.5, dampings 7 / 6 / 4 /
                    0.8, pin stiffness 3 and group "Pin", pressure 2,
                    collision: distance 0.02, quality 4, friction 4, impulse
                    clamp 0.5, self collision at 0.01 with friction 7
  Floor             Collision: damping 0.3, thickness 0.05 / 0.3, cloth
                    friction 6, single sided off, override normals on,
                    permeability 0.2, stickiness 0.4, kill particles on
  Gust              Force field WIND: strength 4.5, flow 0.3, noise 0.2, seed
                    7, power 1.5, max distance 12, shape PLANE
  Jelly             Soft Body: friction 0.7, mass 2, speed 1.2, goal 0.6
                    (0.1..0.9, spring 0.4, friction 0.3, group "Goal"), pull
                    0.6, push 0.5, damping 0.7, bend 0.3, shear 0.8, plastic 2,
                    spring length 50, self collision, ball 0.3, MANUAL, steps
                    15..350, choke 2

Run with bpy available (pip install bpy):
  python make_physics.py
"""

import os

import bpy

bpy.ops.wm.read_factory_settings(use_empty=True)
scene = bpy.context.scene


def mesh(name, at, kind="cube", size=1.0):
    match kind:
        case "cube":
            bpy.ops.mesh.primitive_cube_add(size=size, location=at)
        case "sphere":
            bpy.ops.mesh.primitive_uv_sphere_add(radius=size, location=at)
        case "plane":
            bpy.ops.mesh.primitive_plane_add(size=size, location=at)
        case "grid":
            bpy.ops.mesh.primitive_grid_add(x_subdivisions=4, y_subdivisions=4, size=size, location=at)
    ob = bpy.context.active_object
    ob.name = name
    return ob


def empty(name, at):
    bpy.ops.object.empty_add(location=at)
    ob = bpy.context.active_object
    ob.name = name
    return ob


def activate(ob):
    bpy.context.view_layer.objects.active = ob
    for o in bpy.context.view_layer.objects:
        o.select_set(o == ob)


def rigid(ob, kind, shape, **fields):
    activate(ob)
    bpy.ops.rigidbody.object_add(type=kind)
    rb = ob.rigid_body
    rb.collision_shape = shape
    for k, v in fields.items():
        setattr(rb, k, v)
    return rb


def joint(name, at, kind, a, b, **fields):
    ob = empty(name, at)
    activate(ob)
    bpy.ops.rigidbody.constraint_add(type=kind)
    con = ob.rigid_body_constraint
    con.object1 = a
    con.object2 = b
    for k, v in fields.items():
        setattr(con, k, v)
    return con


# ── Rigid bodies ─────────────────────────────────────────────────────────
ground = mesh("Ground", (0, 0, -1), "plane", 10)
rigid(ground, "PASSIVE", "MESH", friction=0.9)

crate = mesh("Crate", (0, 0, 2))
rigid(
    crate, "ACTIVE", "BOX", mass=2.5, friction=0.3, restitution=0.6, use_margin=True,
    collision_margin=0.02, linear_damping=0.1, angular_damping=0.2, use_deactivation=True,
    deactivate_linear_velocity=0.5, deactivate_angular_velocity=0.6, use_start_deactivated=True,
)
crate.rigid_body.collision_collections = [i in (0, 3) for i in range(20)]

ball = mesh("Ball", (2, 0, 2), "sphere", 0.5)
rigid(ball, "ACTIVE", "SPHERE", mass=0.7, kinematic=True)

pill = mesh("Pill", (4, 0, 2))
rigid(pill, "ACTIVE", "CAPSULE", mesh_source="FINAL", enabled=False)

can = mesh("Can", (6, 0, 2))
rigid(can, "ACTIVE", "CYLINDER")
funnel = mesh("Funnel", (8, 0, 2))
rigid(funnel, "ACTIVE", "CONE")
rock = mesh("Rock", (10, 0, 2))
rigid(rock, "ACTIVE", "CONVEX_HULL")
shell = mesh("Shell", (12, 0, 2))
rigid(shell, "ACTIVE", "MESH")

# ── The world ────────────────────────────────────────────────────────────
rbw = scene.rigidbody_world
rbw.substeps_per_frame = 20
rbw.solver_iterations = 25
rbw.time_scale = 1.5
rbw.use_split_impulse = True
rbw.point_cache.frame_start = 5
rbw.point_cache.frame_end = 120
rbw.effector_weights.gravity = 0.8
rbw.effector_weights.wind = 0.5
scene.gravity = (0.0, 0.0, -5.0)

# ── Joints ───────────────────────────────────────────────────────────────
joint(
    "Hinge", (1, 0, 2), "HINGE", crate, ball, use_limit_ang_z=True, limit_ang_z_lower=-0.5,
    limit_ang_z_upper=1.0, use_breaking=True, breaking_threshold=15.0, disable_collisions=False,
    use_override_solver_iterations=True, solver_iterations=30,
)
joint(
    "Spring", (3, 0, 2), "GENERIC_SPRING", crate, can, use_spring_x=True, spring_stiffness_x=12.5,
    spring_damping_x=0.7, use_limit_lin_x=True, limit_lin_x_lower=-0.2, limit_lin_x_upper=0.3,
    spring_type="SPRING1",
)
joint(
    "Motor", (5, 0, 2), "MOTOR", ball, can, use_motor_lin=True, motor_lin_target_velocity=2.0,
    motor_lin_max_impulse=50.0, use_motor_ang=True, motor_ang_target_velocity=3.0,
    motor_ang_max_impulse=20.0,
)
joint("Fixed", (7, 0, 2), "FIXED", can, funnel, enabled=False)
joint("Point", (9, 0, 2), "POINT", funnel, rock)
joint("Slider", (11, 0, 2), "SLIDER", rock, shell)
joint("Piston", (13, 0, 2), "PISTON", shell, crate)
joint("Generic", (15, 0, 2), "GENERIC", ball, shell)

# ── Cloth ────────────────────────────────────────────────────────────────
flag = mesh("Flag", (0, 4, 2), "grid", 2)
pin = flag.vertex_groups.new(name="Pin")
pin.add([0, 4], 1.0, "REPLACE")
cloth = flag.modifiers.new("Cloth", "CLOTH")
s = cloth.settings
s.quality = 7
s.mass = 0.4
s.air_damping = 2.0
s.tension_stiffness = 25.0
s.compression_stiffness = 20.0
s.shear_stiffness = 8.0
s.bending_stiffness = 1.5
s.tension_damping = 7.0
s.compression_damping = 6.0
s.shear_damping = 4.0
s.bending_damping = 0.8
s.pin_stiffness = 3.0
s.vertex_group_mass = "Pin"
s.use_pressure = True
s.uniform_pressure_force = 2.0
c = cloth.collision_settings
c.use_collision = True
c.distance_min = 0.02
c.collision_quality = 4
c.friction = 4.0
c.impulse_clamp = 0.5
c.use_self_collision = True
c.self_distance_min = 0.01
c.self_friction = 7.0

# ── Collision ────────────────────────────────────────────────────────────
floor = mesh("Floor", (0, 4, 0), "plane", 6)
floor.modifiers.new("Collision", "COLLISION")
col = floor.collision
col.damping = 0.3
col.thickness_outer = 0.05
col.thickness_inner = 0.3
col.cloth_friction = 6.0
col.use_culling = False
col.use_normal = True
col.permeability = 0.2
col.stickiness = 0.4
col.damping_factor = 0.25
col.friction_factor = 0.35
col.use_particle_kill = True

# ── Force field ──────────────────────────────────────────────────────────
gust = empty("Gust", (0, 8, 1))
activate(gust)
bpy.ops.object.forcefield_toggle()
f = gust.field
f.type = "WIND"
f.strength = 4.5
f.flow = 0.3
f.noise = 0.2
f.seed = 7
f.falloff_power = 1.5
f.use_max_distance = True
f.distance_max = 12.0
f.shape = "PLANE"

# ── Soft body ────────────────────────────────────────────────────────────
jelly = mesh("Jelly", (4, 4, 2))
goal = jelly.vertex_groups.new(name="Goal")
goal.add([0, 1], 0.5, "REPLACE")
jelly.modifiers.new("Soft", "SOFT_BODY")
sb = jelly.soft_body
sb.friction = 0.7
sb.mass = 2.0
sb.speed = 1.2
sb.use_goal = True
sb.goal_default = 0.6
sb.goal_min = 0.1
sb.goal_max = 0.9
sb.goal_spring = 0.4
sb.goal_friction = 0.3
sb.vertex_group_goal = "Goal"
sb.use_edges = True
sb.pull = 0.6
sb.push = 0.5
sb.damping = 0.7
sb.bend = 0.3
sb.shear = 0.8
sb.plastic = 2
sb.spring_length = 50
sb.use_self_collision = True
sb.ball_size = 0.3
sb.collision_type = "MANUAL"
sb.step_min = 15
sb.step_max = 350
sb.choke = 2

out = os.path.join(os.path.dirname(os.path.abspath(__file__)), "physics.blend")
bpy.ops.wm.save_as_mainfile(filepath=out)
print("wrote", out)
