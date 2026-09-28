//! Window caption — Phiber draws its own title bar in the theme colours
//! (the native one is hidden), so the Topbar carries the platform window
//! controls. Declarative: [`CAPTION`] says which side each platform uses.
//! `CaptionLead` reserves the macOS traffic-light space at the left edge;
//! `CaptionTrail` shows minimize / maximize / close on Windows and Linux.

use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;

/// Where this platform's window controls sit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BCaptionSide {
    Lead,
    Trail,
}

#[cfg(target_os = "macos")]
pub const CAPTION: BCaptionSide = BCaptionSide::Lead;
#[cfg(not(target_os = "macos"))]
pub const CAPTION: BCaptionSide = BCaptionSide::Trail;

#[derive(Clone, Copy, Debug)]
pub struct BCaption {
    pub host: BHostType,
    pub side: BCaptionSide,
}

impl BViewItem for BCaption {
    fn build(&self, _s: &BState) -> BView {
        if self.host != BHostType::Topbar || self.side != CAPTION {
            return BView::empty();
        }
        BView::WindowControls { leading: self.side == BCaptionSide::Lead }
    }
}
