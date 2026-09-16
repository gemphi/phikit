//! Task 14.5 — View3D canvas layout unit test.
//!
//! Feature: phiber-blender-space-parity
//! Validates: Requirement 13.6
//!
//! The View3D center emits (a) a gradient background, (b) two axis
//! lines, (c) an overlay row, (d) a gizmo stack. This unit test asserts
//! the reference-imported view3d subtree exposes the expected static
//! metadata; full rendering validation happens in the buikit-adaptation
//! task family (23.x).

#[test]
fn view3d_module_is_wired() {
    // Compile-time assertion — the view3d custom subtree is reachable
    // through `phiber::v2::part::custom`. If the module is missing or
    // renamed, this test file fails to build.
    #[allow(unused_imports)]
    use phiber::v2::part::custom;
    let _ = std::any::type_name::<custom::BCustomList>();
}

#[test]
fn view3d_center_dispatch_covers_view3d_host() {
    // `hosts_with_center()` must include View3D — else the center
    // dispatcher would produce Empty for View3D areas (Requirement 13.6).
    use phiber::v2::host::BHostType;
    use phiber::v2::part::custom::set::hosts_with_center;
    assert!(
        hosts_with_center().contains(&BHostType::View3D),
        "View3D host missing from hosts_with_center()"
    );
}
