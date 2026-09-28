//! Backend adapters for resolved, kit-neutral colors and gradients.

use crate::v2::theme::{BGradientColors, palette::BColor};
use gpui::{Background, Hsla, hsla, linear_color_stop, linear_gradient};

pub fn to_hsla(color: BColor) -> Hsla {
    hsla(color.0 / 360.0, color.1 / 100.0, color.2 / 100.0, color.3)
}

pub fn resolved_gradient(colors: BGradientColors) -> Background {
    if colors.enabled {
        // Degrees clockwise from "up", so 180 sweeps down the element and 90
        // sweeps across it to the right.
        let degrees = match colors.axis {
            crate::v2::theme::BGradientAxis::Down => 180.0,
            crate::v2::theme::BGradientAxis::Trailing => 90.0,
            crate::v2::theme::BGradientAxis::Leading => 270.0,
        };
        // The base is reached at `stop` and holds from there.
        linear_gradient(
            degrees,
            linear_color_stop(to_hsla(colors.top), 0.0),
            linear_color_stop(to_hsla(colors.bottom), colors.stop),
        )
    } else {
        to_hsla(colors.bottom).into()
    }
}
