use super::{BCorners, BSurfaceOpacity};

/// Radius for a descendant inset from an ancestor's rounded edge.
pub fn compatible_inner_radius(ancestor_radius: f32, edge_inset: f32) -> f32 {
    (ancestor_radius.max(0.0) - edge_inset.max(0.0)).max(0.0)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BChildCornerSupport {
    #[default]
    Propagated,
    Unsupported,
}

/// Paint treatment for a descendant touching inherited corners.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BChildSurfacePlan {
    pub corners: BCorners,
    pub radius: f32,
    pub inset: f32,
}

/// Propagates a compatible radius or keeps an unsupported opaque child out of
/// the ancestor's corner arc. Transparent/non-touching children need neither.
pub fn child_surface_plan(
    corners: BCorners,
    opacity: BSurfaceOpacity,
    support: BChildCornerSupport,
    ancestor_radius: f32,
    edge_inset: f32,
) -> BChildSurfacePlan {
    if !opacity.is_opaque() || !corners.any() {
        return BChildSurfacePlan::default();
    }
    match support {
        BChildCornerSupport::Propagated => BChildSurfacePlan {
            corners,
            radius: compatible_inner_radius(ancestor_radius, edge_inset),
            inset: 0.0,
        },
        BChildCornerSupport::Unsupported => BChildSurfacePlan {
            corners: BCorners::NONE,
            radius: 0.0,
            inset: ancestor_radius.max(0.0),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compatible_radius_is_bounded_by_ancestor() {
        assert_eq!(compatible_inner_radius(14.0, 1.0), 13.0);
        assert_eq!(compatible_inner_radius(4.0, 8.0), 0.0);
        assert_eq!(compatible_inner_radius(-2.0, 1.0), 0.0);
    }

    #[test]
    fn unsupported_opaque_child_uses_inset_policy() {
        let plan = child_surface_plan(
            BCorners::new(true, false, false, false),
            BSurfaceOpacity::Opaque,
            BChildCornerSupport::Unsupported,
            14.0,
            1.0,
        );
        assert_eq!(plan.corners, BCorners::NONE);
        assert_eq!((plan.radius, plan.inset), (0.0, 14.0));
    }

    #[test]
    fn propagated_child_gets_compatible_radius() {
        let corners = BCorners::new(false, true, false, false);
        let plan = child_surface_plan(
            corners,
            BSurfaceOpacity::Opaque,
            BChildCornerSupport::Propagated,
            14.0,
            1.0,
        );
        assert_eq!(plan.corners, corners);
        assert_eq!((plan.radius, plan.inset), (13.0, 0.0));
    }
}
