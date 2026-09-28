//! Physics context — `bl_ui/properties_physics_*.py`.
// Physics context set — Blender ref: `bl_ui/properties_physics_*.py`.

use super::prop::*;
use super::set::BSectionSpec;
use super::set::sec;

static ADD: &[BPropSpec] = &[e(
    "add_physics",
    "Add Physics",
    &[
        en("FORCE_FIELD", "Force Field"),
        en("COLLISION", "Collision"),
        en("CLOTH", "Cloth"),
        en("DYNAMIC_PAINT", "Dynamic Paint"),
        en("SOFT_BODY", "Soft Body"),
        en("FLUID", "Fluid"),
        en("RIGID_BODY", "Rigid Body"),
        en("RIGID_BODY_CONSTRAINT", "Rigid Body Constraint"),
    ],
)];

static RIGID_BODY: &[BPropSpec] = &[
    e(
        "rigid_body.type",
        "Type",
        &[en("ACTIVE", "Active"), en("PASSIVE", "Passive")],
    ),
    f("rigid_body.mass", "Mass", BUnit::Mass),
    f("rigid_body.friction", "Friction", BUnit::None),
    f("rigid_body.restitution", "Bounciness", BUnit::None),
    b("rigid_body.use_deactivation", "Enable Deactivation"),
];

static COLLISION: &[BPropSpec] = &[
    f("collision.damping", "Damping", BUnit::None),
    f("collision.friction_factor", "Friction Factor", BUnit::None),
    f("collision.stickiness", "Stickiness", BUnit::None),
    b("collision.use_particle_kill", "Kill Particles"),
];

static CLOTH: &[BPropSpec] = &[
    f("cloth.settings.quality", "Quality Steps", BUnit::None),
    f("cloth.settings.mass", "Vertex Mass", BUnit::Mass),
    f("cloth.settings.air_damping", "Air Viscosity", BUnit::None),
    f("cloth.settings.tension_stiffness", "Tension", BUnit::None),
    f(
        "cloth.settings.compression_stiffness",
        "Compression",
        BUnit::None,
    ),
    f("cloth.settings.shear_stiffness", "Shear", BUnit::None),
];

pub static SECTIONS: &[BSectionSpec] = &[
    sec("Add Physics", ADD),
    sec("Rigid Body", RIGID_BODY),
    sec("Collision", COLLISION),
    sec("Cloth", CLOTH),
];
