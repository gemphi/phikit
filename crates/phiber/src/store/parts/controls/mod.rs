//! Controls part - header control cluster entries (Blender space_view3d header).
//!
//! Pure static data: orientation/snap dropdown items, toggle buttons, and the
//! shading quad. The leaf view (`bui/part/controls`) only matches and renders.

pub mod def;
pub mod set;

/// A single control entry (dropdown item or toggle button).
#[derive(Clone, Copy, Debug)]
pub struct CtrlEntry {
    pub idx: &'static str,
    pub label: &'static str,
    pub icon: &'static str,
    pub active: bool,
}

/// Control cluster spec for a host's header.
#[derive(Clone, Copy, Debug)]
pub struct CtrlCluster {
    pub orientation: &'static [CtrlEntry],
    pub snap: &'static [CtrlEntry],
    pub toggles: &'static [CtrlEntry],
    pub shading: &'static [CtrlEntry],
    pub view_toggles: &'static [CtrlEntry],
}

pub static EMPTY_CLUSTER: CtrlCluster = CtrlCluster {
    orientation: &[],
    snap: &[],
    toggles: &[],
    shading: &[],
    view_toggles: &[],
};
