//! Constraint context — `bl_ui/properties_object_constraint.py`.
// Constraint context set — Blender ref: `bl_ui/properties_object_constraint.py`.

use super::prop::*;
use super::set::BSectionSpec;
use super::set::sec;

static ADD: &[BPropSpec] = &[e(
    "add_constraint",
    "Add Object Constraint",
    &[
        // Motion Tracking
        en("CAMERA_SOLVER", "Camera Solver"),
        en("FOLLOW_TRACK", "Follow Track"),
        en("OBJECT_SOLVER", "Object Solver"),
        // Transform
        en("COPY_LOCATION", "Copy Location"),
        en("COPY_ROTATION", "Copy Rotation"),
        en("COPY_SCALE", "Copy Scale"),
        en("COPY_TRANSFORMS", "Copy Transforms"),
        en("LIMIT_DISTANCE", "Limit Distance"),
        en("LIMIT_LOCATION", "Limit Location"),
        en("LIMIT_ROTATION", "Limit Rotation"),
        en("LIMIT_SCALE", "Limit Scale"),
        en("MAINTAIN_VOLUME", "Maintain Volume"),
        en("TRANSFORM", "Transformation"),
        en("TRANSFORM_CACHE", "Transform Cache"),
        // Tracking
        en("CLAMP_TO", "Clamp To"),
        en("DAMPED_TRACK", "Damped Track"),
        en("IK", "Inverse Kinematics"),
        en("LOCKED_TRACK", "Locked Track"),
        en("SPLINE_IK", "Spline IK"),
        en("STRETCH_TO", "Stretch To"),
        en("TRACK_TO", "Track To"),
        // Relationship
        en("ACTION", "Action"),
        en("ARMATURE", "Armature"),
        en("CHILD_OF", "Child Of"),
        en("FLOOR", "Floor"),
        en("FOLLOW_PATH", "Follow Path"),
        en("PIVOT", "Pivot"),
        en("SHRINKWRAP", "Shrinkwrap"),
    ],
)];

static STACK: &[BPropSpec] = &[p("constraints", "Constraint Stack", "ObjectConstraints")];

pub static SECTIONS: &[BSectionSpec] =
    &[sec("Add Constraint", ADD), sec("Constraint Stack", STACK)];
