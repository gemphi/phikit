//! Glass: what a pane of glass does to what is behind it. The numbers
//! are the theme's; a theme that gives no blur has no glass. The screen
//! hands them to the renderer's backdrop pass (`gpu/gpui_glass`), which
//! blurs, tints and lights what was drawn so far.
//!
//! Glass is for panes: a bar, a panel that floats. A button, a row and
//! an icon are never glass.

use super::PColor;
use crate::theme::PTheme;

/// The theme's tokens for glass: how far the blur reaches, the lift on
/// the colour that shows through, the colour it leans toward and how far,
/// how far the rim bends what is behind it, and the light the rim
/// catches and how much of it.
pub const BLUR: &str = "glass.blur";
pub const SATURATION: &str = "glass.saturation";
pub const TINT: &str = "glass.tint";
pub const SHARE: &str = "glass.share";
pub const LENS: &str = "glass.lens";
pub const EDGE: &str = "glass.edge";
pub const RIM: &str = "glass.rim";
/// Whether the window itself lets what is behind it show, blurred by the
/// system: more than nothing says it does.
pub const WINDOW: &str = "window.glass";

/// What a pane of glass does to what is behind it.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PGlass {
    pub blur: f32,
    pub saturation: f32,
    pub tint: PColor,
    pub share: f32,
    pub lens: f32,
    pub edge: PColor,
    pub rim: f32,
}

impl PGlass {
    /// The theme's glass. `None` for a theme that gives no blur.
    pub fn of(theme: &dyn PTheme) -> Option<PGlass> {
        let blur = theme.size(BLUR).filter(|blur| *blur > 0.0)?;
        let size = |token: &str| theme.size(token).unwrap_or_default();
        let color = |token: &str| theme.color(token).unwrap_or_default();
        Some(PGlass { blur, saturation: theme.size(SATURATION).unwrap_or(1.0), tint: color(TINT), share: size(SHARE), lens: size(LENS), edge: color(EDGE), rim: size(RIM) })
    }

    /// Whether the theme has the window itself show what is behind it.
    pub fn window(theme: &dyn PTheme) -> bool {
        theme.size(WINDOW).is_some_and(|share| share > 0.0)
    }
}
