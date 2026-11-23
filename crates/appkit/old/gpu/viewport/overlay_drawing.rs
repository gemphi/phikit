//! Overlay drawing — gizmo axis hit-testing and selection ray casting.

use crate::ecs::{Entity, Scene};
use crate::prelude::Vec3;
use std::sync::{Arc, Mutex};

/// Which gizmo axis is being dragged (if any).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GizmoDragAxis {
    None,
    X,
    Y,
    Z,
}

/// Result of a gizmo hit test against the three translation axes.
pub(super) struct GizmoHitResult {
    pub axis: GizmoDragAxis,
}

/// Test whether a ray hits one of the gizmo axes for the given entity.
///
/// Returns the closest axis hit, or `GizmoDragAxis::None` if no axis was hit.
pub(super) fn test_gizmo_hit(
    near_pt: Vec3,
    ray_dir: Vec3,
    origin: Vec3,
) -> GizmoDragAxis {
    let gizmo_len = 1.75f32; // shaft + arrowhead (matches mesh_sync: 1.5 + 0.25)
    let hit_radius = 0.15f32; // slightly larger for easier picking

    // Test each axis: find closest point on ray to axis line
    let axis_hit = |axis_dir: Vec3| -> Option<f32> {
        let d1 = ray_dir;
        let d2 = axis_dir;
        let r = near_pt - origin;
        let a = d1.dot(d1);
        let b = d1.dot(d2);
        let c = d2.dot(d2);
        let d = d1.dot(r);
        let e = d2.dot(r);
        let denom = a * c - b * b;
        if denom.abs() < 1e-6 {
            return None;
        }
        let t_ray = (b * e - c * d) / denom;
        let t_axis = (a * e - b * d) / denom;
        if t_ray < 0.0 || t_axis < 0.0 || t_axis > gizmo_len {
            return None;
        }
        let p_ray = near_pt + d1 * t_ray;
        let p_axis = origin + d2 * t_axis;
        let dist = (p_ray - p_axis).length();
        if dist < hit_radius {
            Some(dist)
        } else {
            None
        }
    };

    let x_dist = axis_hit(Vec3::X);
    let y_dist = axis_hit(Vec3::Y);
    let z_dist = axis_hit(Vec3::Z);

    // Pick closest axis
    let mut best_axis = GizmoDragAxis::None;
    let mut _best_dist = f32::MAX;
    if let Some(d) = x_dist {
        if d < _best_dist {
            _best_dist = d;
            best_axis = GizmoDragAxis::X;
        }
    }
    if let Some(d) = y_dist {
        if d < _best_dist {
            _best_dist = d;
            best_axis = GizmoDragAxis::Y;
        }
    }
    if let Some(d) = z_dist {
        if d < _best_dist {
            #[allow(unused_assignments)]
            {
                _best_dist = d;
            }
            best_axis = GizmoDragAxis::Z;
        }
    }

    best_axis
}

/// Perform selection ray cast against the scene and invoke the selection callback.
pub(super) fn perform_selection_ray_cast(
    scene: &Scene,
    near_pt: Vec3,
    ray_dir: Vec3,
    on_selection_changed: &Option<Arc<dyn Fn(Option<Entity>) + Send + Sync>>,
) {
    let hit = scene.ray_cast(near_pt, ray_dir);
    if let Some(ref handler) = on_selection_changed {
        handler(hit);
    }
}
