//! Backend adapters for resolved, kit-neutral colors and gradients.

use crate::v2::theme::{BGradientColors, palette::BColor};
use gpui::{Background, Hsla, hsla, linear_color_stop, linear_gradient};

pub fn to_hsla(color: BColor) -> Hsla {
    hsla(color.0 / 360.0, color.1 / 100.0, color.2 / 100.0, color.3)
}

pub fn resolved_gradient(colors: BGradientColors) -> Background {
    if colors.enabled {
        linear_gradient(
            180.0,
            linear_color_stop(to_hsla(colors.top), 0.0),
            linear_color_stop(to_hsla(colors.bottom), 1.0),
        )
    } else {
        to_hsla(colors.bottom).into()
    }
}
