//! Static unit shapes for empties — Blender `overlay_shape.cc` (Blender 5.2).
//! Segments are `(from, to)` in local space, scaled by `empty_display_size`.
//! Circles / rings are generated from `RING_SEGMENTS`.

pub type BSeg = ([f32; 3], [f32; 3]);

/// `plain_axes`: ±1 on each axis.
pub static PLAIN_AXES: &[BSeg] = &[
    ([-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]),
    ([0.0, -1.0, 0.0], [0.0, 1.0, 0.0]),
    ([0.0, 0.0, -1.0], [0.0, 0.0, 1.0]),
];

/// `arrows`: origin → +X / +Y / +Z.
pub static ARROWS: &[BSeg] = &[
    ([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]),
    ([0.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
    ([0.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
];

/// `single_arrow`: +Z shaft with a four-line head.
pub static SINGLE_ARROW: &[BSeg] = &[
    ([0.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
    ([0.0, 0.0, 1.0], [0.035, 0.0, 0.75]),
    ([0.0, 0.0, 1.0], [-0.035, 0.0, 0.75]),
    ([0.0, 0.0, 1.0], [0.0, 0.035, 0.75]),
    ([0.0, 0.0, 1.0], [0.0, -0.035, 0.75]),
];

/// `cube`: the 12 edges of [-1, 1]³.
pub static CUBE: &[BSeg] = &[
    ([-1.0, -1.0, -1.0], [1.0, -1.0, -1.0]),
    ([-1.0, 1.0, -1.0], [1.0, 1.0, -1.0]),
    ([-1.0, -1.0, 1.0], [1.0, -1.0, 1.0]),
    ([-1.0, 1.0, 1.0], [1.0, 1.0, 1.0]),
    ([-1.0, -1.0, -1.0], [-1.0, 1.0, -1.0]),
    ([1.0, -1.0, -1.0], [1.0, 1.0, -1.0]),
    ([-1.0, -1.0, 1.0], [-1.0, 1.0, 1.0]),
    ([1.0, -1.0, 1.0], [1.0, 1.0, 1.0]),
    ([-1.0, -1.0, -1.0], [-1.0, -1.0, 1.0]),
    ([1.0, -1.0, -1.0], [1.0, -1.0, 1.0]),
    ([-1.0, 1.0, -1.0], [-1.0, 1.0, 1.0]),
    ([1.0, 1.0, -1.0], [1.0, 1.0, 1.0]),
];

/// Segment count for rings (Blender uses 64 for `circle`, 8 for `empty_cone`).
pub const RING_SEGMENTS: usize = 32;
pub const CONE_SEGMENTS: usize = 8;

/// Axis pairs a ring can lie in: XZ (empty circle), XY, YZ (sphere rings).
pub const PLANE_XZ: (usize, usize) = (0, 2);
pub const PLANE_XY: (usize, usize) = (0, 1);
pub const PLANE_YZ: (usize, usize) = (1, 2);
