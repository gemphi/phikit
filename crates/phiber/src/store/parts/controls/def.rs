//! Controls part constants (Blender View3D header control labels/icons).

// Transform orientation (Blender: orientation dropdown, default Global).
pub const ORIENT_GLOBAL: (&'static str, &'static str, &'static str) = ("global", "Global", "world");
pub const ORIENT_LOCAL: (&'static str, &'static str, &'static str) = ("local", "Local", "axis");
pub const ORIENT_NORMAL: (&'static str, &'static str, &'static str) =
    ("normal", "Normal", "normal");
pub const ORIENT_GIMBAL: (&'static str, &'static str, &'static str) =
    ("gimbal", "Gimbal", "gimbal");
pub const ORIENT_VIEW: (&'static str, &'static str, &'static str) = ("view", "View", "view");

// Snap element (Blender: snap dropdown, default Increment).
pub const SNAP_INCREMENT: (&'static str, &'static str, &'static str) =
    ("increment", "Increment", "grid");
pub const SNAP_VERTEX: (&'static str, &'static str, &'static str) = ("vertex", "Vertex", "dot");
pub const SNAP_EDGE: (&'static str, &'static str, &'static str) = ("edge", "Edge", "minus");
pub const SNAP_FACE: (&'static str, &'static str, &'static str) = ("face", "Face", "square");
pub const SNAP_VOLUME: (&'static str, &'static str, &'static str) = ("volume", "Volume", "box");

// Editing toggles (Blender: snap magnet, proportional editing).
pub const TOGGLE_SNAP: (&'static str, &'static str, &'static str) = ("snap", "Snapping", "magnet");
pub const TOGGLE_PROPORTIONAL: (&'static str, &'static str, &'static str) =
    ("proportional", "Proportional Editing", "target");

// Shading quad (Blender: wireframe / solid / material preview / rendered).
pub const SHADE_WIREFRAME: (&'static str, &'static str, &'static str) =
    ("wireframe", "Wireframe", "grid");
pub const SHADE_SOLID: (&'static str, &'static str, &'static str) = ("solid", "Solid", "sphere");
pub const SHADE_MATERIAL: (&'static str, &'static str, &'static str) =
    ("material", "Material Preview", "material");
pub const SHADE_RENDERED: (&'static str, &'static str, &'static str) =
    ("rendered", "Rendered", "render");

// View toggles (Blender: overlays, gizmos).
pub const VIEW_OVERLAYS: (&'static str, &'static str, &'static str) =
    ("overlays", "Overlays", "eye");
pub const VIEW_GIZMOS: (&'static str, &'static str, &'static str) = ("gizmos", "Gizmos", "move");
