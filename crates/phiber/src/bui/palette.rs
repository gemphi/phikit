//! Shared color palette for all BUI elements (Blender dark theme).

use gpui::{Hsla, hsla};

pub const BG_APP: Hsla = hsla(220. / 360., 0.14, 0.12, 1.);
pub const BG_TOP: Hsla = hsla(220. / 360., 0.14, 0.16, 1.);
pub const BG_PANEL: Hsla = hsla(220. / 360., 0.12, 0.18, 1.);
pub const BG_HEADER: Hsla = hsla(220. / 360., 0.14, 0.20, 1.);
pub const BG_TOOLBAR: Hsla = hsla(220. / 360., 0.10, 0.16, 1.);
pub const BG_VIEWPORT: Hsla = hsla(210. / 360., 0.18, 0.10, 1.);
pub const BG_STATUS: Hsla = hsla(220. / 360., 0.14, 0.14, 1.);
pub const BG_ACTIVE: Hsla = hsla(210. / 360., 0.18, 0.28, 1.);

pub const BORDER: Hsla = hsla(0., 0., 0.06, 1.);
pub const TEXT: Hsla = hsla(0., 0., 0.85, 1.);
pub const TEXT_DIM: Hsla = hsla(0., 0., 0.55, 1.);
