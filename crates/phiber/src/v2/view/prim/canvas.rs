//! Canvas primitives — host-specific canvases (viewport / tree / node
//! graph / text buffer) and the progress bar.

use crate::v2::host::BHostType;
use crate::v2::scene::camera_view::BCameraSlot;
use crate::v2::view::BView;

impl BView {
    pub fn canvas(host: BHostType) -> Self {
        Self::Canvas { host, camera: BCameraSlot::none() }
    }
    /// The 3D viewport in an area: `camera` says whether it looks through
    /// the scene camera and which key toggles that.
    pub fn view3d_canvas(camera: BCameraSlot) -> Self {
        Self::Canvas { host: BHostType::View3D, camera }
    }
    /// A determinate progress bar with a leading status label. `fraction` is
    /// clamped to `0.0..=1.0` at render time.
    pub fn progress_bar(fraction: f32, label: impl Into<String>) -> Self {
        Self::ProgressBar {
            fraction,
            label: label.into(),
        }
    }

    pub fn log(open: bool, toggle: super::BControl) -> Self {
        Self::Log { open, toggle }
    }
}
