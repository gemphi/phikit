//! The overlays' sizes, as Blender draws them at a UI scale of one.

/// `U.widget_unit` (`18 · scale + 2 · pixelsize`): the 3D cursor's size.
pub const WIDGET_UNIT: f32 = 20.0;

/// The cursor's ring (`cursor_circle`): twelve segments, alternately white
/// and red, half a widget unit across the radius.
pub const RING_SEGMENTS: usize = 12;
pub const RING_RADIUS: f32 = 0.5;

/// The cursor's crosshair (`cursor_lines`), in widget units along each
/// axis: coloured from `COLOR_LIMIT` out to `OUTER`, the overlay's white in
/// from there to `INNER`.
pub const OUTER: f32 = 1.0;
pub const COLOR_LIMIT: f32 = 0.85;
pub const INNER: f32 = 0.25;

/// An origin's dot: `TH_OBCENTER_DIA` (6) plus one across, ringed by one
/// pixel (`overlay_extra_point_vert.glsl`).
pub const ORIGIN_DIAMETER: f32 = 7.0;
pub const ORIGIN_RING: f32 = 1.0;
