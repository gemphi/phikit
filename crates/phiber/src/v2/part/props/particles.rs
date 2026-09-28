//! Particles context — `bl_ui/properties_particle.py`.
// Particles context set — Blender ref: `bl_ui/properties_particle.py`.

use super::prop::*;
use super::set::BSectionSpec;
use super::set::sec;

static PARTICLE_TYPE: &[BEnumItem] = &[en("EMITTER", "Emitter"), en("HAIR", "Hair")];

static SYSTEM: &[BPropSpec] = &[
    e("type", "Type", PARTICLE_TYPE),
    i("count", "Number", 0, 10_000_000),
    i("frame_start", "Frame Start", -100_000, 100_000),
    i("frame_end", "Frame End", -100_000, 100_000),
    f("lifetime", "Lifetime", BUnit::Time),
    f("lifetime_random", "Lifetime Randomness", BUnit::None),
];

static EMISSION: &[BPropSpec] = &[
    e(
        "emit_from",
        "Emit From",
        &[
            en("VERT", "Vertices"),
            en("FACE", "Faces"),
            en("VOLUME", "Volume"),
        ],
    ),
    b("use_emit_random", "Random"),
    b("use_even_distribution", "Even Distribution"),
];

static PHYSICS: &[BPropSpec] = &[
    e(
        "physics_type",
        "Physics Type",
        &[
            en("NEWTON", "Newtonian"),
            en("KEYED", "Keyed"),
            en("BOIDS", "Boids"),
            en("FLUID", "Fluid"),
            en("NO", "None"),
        ],
    ),
    f("mass", "Mass", BUnit::Mass),
    f("particle_size", "Size", BUnit::Length),
    f("size_random", "Randomness", BUnit::None),
    f("brownian_factor", "Brownian", BUnit::None),
    f("damping", "Damp", BUnit::None),
];

static RENDER: &[BPropSpec] = &[
    e(
        "render_type",
        "Render As",
        &[
            en("NONE", "None"),
            en("HALO", "Halo"),
            en("LINE", "Line"),
            en("PATH", "Path"),
            en("OBJECT", "Object"),
            en("COLLECTION", "Collection"),
        ],
    ),
    b("show_unborn", "Unborn"),
    b("use_dead", "Dead"),
];

static VIEWPORT: &[BPropSpec] = &[
    e(
        "display_method",
        "Display",
        &[
            en("NONE", "None"),
            en("RENDER", "Rendered"),
            en("DOT", "Point"),
            en("CIRC", "Circle"),
            en("CROSS", "Cross"),
            en("AXIS", "Axis"),
        ],
    ),
    i("display_size", "Size", 1, 1000),
    i("display_percentage", "Amount", 0, 100),
];

pub static SECTIONS: &[BSectionSpec] = &[
    sec("System", SYSTEM),
    sec("Emission", EMISSION),
    sec("Physics", PHYSICS).closed(),
    sec("Render", RENDER).closed(),
    sec("Viewport Display", VIEWPORT).closed(),
];
