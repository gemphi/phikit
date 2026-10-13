//! Non-mesh objects the viewport overlay draws: cameras, lights, empties.
//! Field names follow Blender's DNA (`Camera`, `Light`, `Object.empty_*`).

/// A camera object — Blender `Camera` DNA plus the owning object's transform.
#[derive(Clone, Debug, PartialEq)]
pub struct SceneCamera {
    pub name: String,
    /// Owning Object's location (kept for panels).
    pub location: [f32; 3],
    /// World matrix (Blender `m[col][row]` layout).
    pub matrix_world: [[f32; 4]; 4],
    /// `"PERSP"`, `"ORTHO"` or `"PANO"`.
    pub kind: String,
    /// Focal length in millimetres (`Camera.lens`).
    pub focal_length_mm: f32,
    pub sensor_width: f32,
    pub ortho_scale: f32,
    /// Near / far clip distances.
    pub clip_start: f32,
    pub clip_end: f32,
    /// Viewport display size (`Camera.drawsize`).
    pub display_size: f32,
    /// Depth of field (`blend/object/dof.rs`). Off by default, which makes a
    /// pinhole camera the ordinary case rather than a special one.
    pub dof: crate::blend::BlendDof,
}

/// A light object — Blender `Light` DNA plus the owning object's transform.
#[derive(Clone, Debug, PartialEq)]
pub struct SceneLight {
    pub name: String,
    pub location: [f32; 3],
    pub matrix_world: [[f32; 4]; 4],
    pub color: [f32; 3],
    /// Blender's `Light.energy`, **as the file stores it**: watts for
    /// POINT/SPOT, unitless irradiance for SUN, and 4/pi of the watts the
    /// UI shows for AREA (`AREA_ENERGY_SCALE`, `blend/object/light_driver.rs`).
    /// The tracer's record converts an area light's back to watts
    /// (`view3d/rendered/lamp.rs`), which is the boundary the two units meet at.
    pub energy: f32,
    /// `"POINT"`, `"SUN"`, `"SPOT"`, `"AREA"`.
    pub kind: String,
    /// Spot cone angle in radians (`Light.spotsize`).
    pub spot_size: f32,
    /// Spot Blend, 0 ..= 1 (`Light.spotblend`): how much of the cone's
    /// edge fades. 0 is a hard edge.
    pub spot_blend: f32,
    /// The light's shape, which decides how soft its shadow is: a point's
    /// or spot's radius, an area light's edge, or a sun's angular diameter
    /// in radians (`Light.sun_angle`).
    pub size: f32,
    /// An area light's edge across its own Y; equal to `size` for the
    /// shapes Blender gives one size (Square, Disk).
    pub size_y: f32,
    /// An area light's shape: `"SQUARE"`, `"RECTANGLE"`, `"DISK"`,
    /// `"ELLIPSE"` (`Light.area_shape`).
    pub shape: String,
}

/// An empty — Blender `Object.empty_drawtype` / `empty_drawsize`.
#[derive(Clone, Debug, PartialEq)]
pub struct SceneEmpty {
    pub name: String,
    pub matrix_world: [[f32; 4]; 4],
    /// `"PLAIN_AXES"`, `"ARROWS"`, `"SINGLE_ARROW"`, `"CIRCLE"`, `"CUBE"`,
    /// `"SPHERE"`, `"CONE"`, `"IMAGE"`.
    pub display_type: String,
    pub display_size: f32,
}
