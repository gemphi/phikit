//! 2D + 3D projection functions and the viewport-fit AABB helpers. Split
//! out of `scene/mod.rs` so the parent stays under the 200-line cap.
//!
//! All functions here are pure math — no gpui, no state — and are consumed
//! by the viewport renderer and the outliner hit-testing paths alike.

/// Project a world `(x, y)` to a screen offset in the range roughly
/// `[-0.5, 0.5]` on each axis, given the view yaw. A simple orbit: rotate
/// the ground plane by `yaw` about the origin, then apply a fixed isometric
/// tilt so objects further "back" sit higher on screen.
pub fn project(x: f32, y: f32, yaw_deg: f32) -> (f32, f32) {
    let yaw = yaw_deg.to_radians();
    let (s, c) = (yaw.sin(), yaw.cos());
    // Rotate about the vertical axis.
    let rx = x * c - y * s;
    let ry = x * s + y * c;
    // Isometric tilt: compress the depth axis and push it upward.
    let screen_x = rx * 0.42;
    let screen_y = ry * 0.28 - 0.04; // slight downward bias toward the floor
    (screen_x, screen_y)
}

/// Project with zoom applied — the screen offset scales about the viewport
/// centre by `zoom`. Used by the viewport so scroll dollies the whole scene.
pub fn project_zoom(x: f32, y: f32, yaw_deg: f32, zoom: f32) -> (f32, f32) {
    let (sx, sy) = project(x, y, yaw_deg);
    (sx * zoom, sy * zoom)
}

/// Uniform scale applied to a mesh's local vertex coordinates before it is
/// placed at its object's world `(x, y)` on the ground plane. The phiphx
/// primitive buffers are roughly unit-sized (`[-1, 1]`), so this keeps a
/// projected mesh about the same on-screen footprint the old flat glyph had.
pub const MESH_SCALE: f32 = 0.18;

/// Project a full 3D world point `(x, up, z)` — `x` right, `up` vertical,
/// `z` depth — to a screen offset plus a depth value for painter sorting.
///
/// On the ground plane (`up == 0.0`) this agrees exactly with
/// [`project_zoom`] on `(x, z)` so the existing 2D scene placement is
/// preserved. `up` lifts the point toward the top of the screen (smaller
/// screen-y), and the returned depth is the rotated depth axis (`ry`) so
/// meshes and triangles further "back" sort behind nearer ones.
pub fn project3(x: f32, up: f32, z: f32, yaw_deg: f32, zoom: f32) -> (f32, f32, f32) {
    let yaw = yaw_deg.to_radians();
    let (s, c) = (yaw.sin(), yaw.cos());
    let rx = x * c - z * s;
    let rz = x * s + z * c;
    let screen_x = rx * 0.42;
    let screen_y = rz * 0.28 - 0.04;
    let screen_y = screen_y - up * 0.42;
    (screen_x * zoom, screen_y * zoom, rz)
}

/// Full 3D projection that honours yaw, pitch, pan, and zoom. When `pitch`
/// and `pan` are zero this is exactly equivalent to [`project3`], so
/// existing call sites can migrate incrementally.
///
/// Pitch tilts the view about the horizontal axis: +pitch pushes the ground
/// plane down on screen (looking down onto it), which matches Blender's
/// numpad-8/2 elevation direction.
///
/// Pan translates the whole projected scene laterally after zoom.
pub fn project3_full(
    x: f32,
    up: f32,
    z: f32,
    yaw_deg: f32,
    pitch_deg: f32,
    pan: (f32, f32),
    zoom: f32,
) -> (f32, f32, f32) {
    let yaw = yaw_deg.to_radians();
    let (sy, cy) = (yaw.sin(), yaw.cos());
    // Yaw first: rotate the ground plane about the vertical axis.
    let rx = x * cy - z * sy;
    let rz_yaw = x * sy + z * cy;

    // Pitch: rotate the (up, rz_yaw) plane. +pitch tips the top of the view
    // toward the camera (looking down), so the depth axis lifts on screen.
    let pitch = pitch_deg.to_radians();
    let (sp, cp) = (pitch.sin(), pitch.cos());
    let up_p = up * cp - rz_yaw * sp;
    let rz = up * sp + rz_yaw * cp;

    // Ground-plane screen offset with the pitched depth compression.
    let screen_x = rx * 0.42;
    let screen_y = rz * 0.28 - 0.04 - up_p * 0.42;

    ((screen_x * zoom) + pan.0, (screen_y * zoom) + pan.1, rz)
}

/// Below this, a mesh's largest axis extent is treated as degenerate (a
/// single point or a flat/needle mesh with no measurable size on some axis)
/// and the fit radius falls back to `1.0` so `fit_vertex` never divides by
/// a near-zero `r`.
pub const FIT_EPSILON: f32 = 1e-6;

/// Compute the uniform viewport-fit parameters for a set of object-local
/// vertices: the AABB centre `c` and the fit half-extent `r`.
///
/// `c` is the per-axis midpoint of the axis-aligned bounding box. `r` is
/// half the largest axis extent (`max_axis_extent / 2`), so a cube spanning
/// a full axis maps that axis to `[-MESH_SCALE, MESH_SCALE]` after
/// [`fit_vertex`].
///
/// Degenerate guard (Requirement 4.6): if `vertices` is empty, or the
/// largest axis extent is below [`FIT_EPSILON`], `r` falls back to `1.0` so
/// the subsequent divide in [`fit_vertex`] stays finite and never blows up.
pub fn mesh_bounds(vertices: &[[f32; 3]]) -> ([f32; 3], f32) {
    if vertices.is_empty() {
        return ([0.0, 0.0, 0.0], 1.0);
    }
    let mut min = vertices[0];
    let mut max = vertices[0];
    for v in &vertices[1..] {
        for a in 0..3 {
            if v[a] < min[a] {
                min[a] = v[a];
            }
            if v[a] > max[a] {
                max[a] = v[a];
            }
        }
    }
    let center = [
        (min[0] + max[0]) * 0.5,
        (min[1] + max[1]) * 0.5,
        (min[2] + max[2]) * 0.5,
    ];
    let extent_x = max[0] - min[0];
    let extent_y = max[1] - min[1];
    let extent_z = max[2] - min[2];
    let max_axis_extent = extent_x.max(extent_y).max(extent_z);
    let r = if max_axis_extent < FIT_EPSILON {
        1.0
    } else {
        max_axis_extent * 0.5
    };
    (center, r)
}

/// Map one object-local vertex into the shared viewport-fit space: recentre
/// it about the AABB centre `c`, normalise by the fit half-extent `r`, and
/// scale by [`MESH_SCALE`]. With `r` from [`mesh_bounds`], every vertex of
/// an object lands within `[-MESH_SCALE, MESH_SCALE]` on each axis.
pub fn fit_vertex(v: [f32; 3], center: [f32; 3], r: f32) -> [f32; 3] {
    [
        (v[0] - center[0]) / r * MESH_SCALE,
        (v[1] - center[1]) / r * MESH_SCALE,
        (v[2] - center[2]) / r * MESH_SCALE,
    ]
}

/// Convenience wrapper: fit an entire vertex buffer into the shared viewport
/// bounds in one call. Computes the AABB centre and fit radius via
/// [`mesh_bounds`], then applies [`fit_vertex`] to every vertex.
pub fn fit_vertices(vertices: &[[f32; 3]]) -> Vec<[f32; 3]> {
    let (center, r) = mesh_bounds(vertices);
    vertices.iter().map(|&v| fit_vertex(v, center, r)).collect()
}
