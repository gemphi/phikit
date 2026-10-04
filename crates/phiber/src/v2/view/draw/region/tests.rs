//! Which regions own a host's corners.
//!
//! A region claims a corner only when something inside it actually paints
//! there; the host keeps the rest.

use super::{BSurfaceOpacity, surface_opacity};
use crate::v2::host::BHostType;
use crate::v2::region::BRegionType;

#[test]
fn only_intentionally_painted_regions_claim_host_corners() {
    for kind in [
        BRegionType::Tools,
        BRegionType::ToolProps,
        BRegionType::UI,
        BRegionType::AssetShelf,
    ] {
        assert_eq!(
            surface_opacity(kind, BHostType::View3D),
            BSurfaceOpacity::Transparent
        );
    }
    // A footer is a strip like the header: painted, so it owns the host's
    // bottom corners as the header owns the top ones.
    for kind in [BRegionType::Header, BRegionType::Footer] {
        assert_eq!(
            surface_opacity(kind, BHostType::View3D),
            BSurfaceOpacity::Opaque
        );
    }
    assert_eq!(
        surface_opacity(BRegionType::Header, BHostType::Topbar),
        BSurfaceOpacity::Transparent
    );
    // A vertical rail paints the strip shade — the same one the horizontal
    // strips under a header take. It used to paint nothing, which left the rail
    // and the panel beside it one flat field, so the active tab had no edge to
    // be cut out of and the strip read as part of the content (`region_fill`).
    //
    // It still claims no corners: what it paints is a shade, not the host's
    // rounded outline, and the area's body is what reaches the corners.
    assert_eq!(
        surface_opacity(BRegionType::NavBar, BHostType::Buts),
        BSurfaceOpacity::Transparent
    );
    assert_eq!(super::region_fill(BRegionType::NavBar), super::BFill::HostRail);
    // The execute bar is a strip: the header surface, owning its corners.
    assert_eq!(super::region_fill(BRegionType::Execute), super::region_fill(BRegionType::Header));
    assert_eq!(surface_opacity(BRegionType::Execute, BHostType::File), BSurfaceOpacity::Opaque);
}

#[test]
fn a_canvas_host_hands_its_corners_to_the_region_that_holds_the_canvas() {
    // The canvas is what paints to the edges, so the region around it
    // owns the corners on its behalf; a host that stacks content keeps
    // them itself, because nothing inside reaches them.
    assert_eq!(
        surface_opacity(BRegionType::Center, BHostType::View3D),
        BSurfaceOpacity::Opaque
    );
    assert_eq!(
        surface_opacity(BRegionType::Center, BHostType::Shader),
        BSurfaceOpacity::Opaque
    );
    assert_eq!(
        surface_opacity(BRegionType::Center, BHostType::Buts),
        BSurfaceOpacity::Transparent
    );
    assert_eq!(
        surface_opacity(BRegionType::Center, BHostType::Outliner),
        BSurfaceOpacity::Transparent
    );
}

#[test]
fn the_viewports_canvas_inherits_the_zones_bottom_corners() {
    use super::super::surface::{BCellPosition, BCorners, BRowPosition};

    // A View3D host is a header row above a content band, so the band is the
    // last row: its region — and through it the canvas — owns the two bottom
    // corners. This is the whole chain that decides whether the blue surface
    // paints an arc or squares off the zone it sits in.
    let rows = 2;
    let opacity = surface_opacity(BRegionType::Center, BHostType::View3D);
    let ownership = BCorners::ALL.assign_cell(
        BCellPosition::at(1, rows, 0, 1),
        opacity,
    );
    assert_eq!(
        ownership.child,
        BCorners::new(false, false, true, true),
        "the content region must carry the bottom corners to its canvas"
    );

    // The header above it keeps the two at the top.
    let header = BCorners::ALL.assign_row(
        BRowPosition::at(0, rows),
        surface_opacity(BRegionType::Header, BHostType::View3D),
    );
    assert_eq!(header.child, BCorners::new(true, true, false, false));
}
