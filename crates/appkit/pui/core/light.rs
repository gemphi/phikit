//! Light: the one way a surface is lit, in every app. It is puikit's
//! (`src/tokens/tokens.scss`): one light, three strengths of it, and two
//! ways of laying it.
//!
//! * The surface: light at the foot fading to nothing at the head, so a
//!   heading at the top of a surface stands free. A card, a field, a
//!   panel and a user's mark all wear it.
//! * The label: light from the left, gone before the right; mirrored for
//!   what sits at the right, such as a person's own side of a chat. A
//!   button, a row that is chosen, a pill and a turn wear it. A bar at
//!   the window's side wears it toward the page (`edge`), so that what
//!   stands on the bar runs against it.
//!
//! Nothing here has a colour of its own: a look changes every surface at
//! once by changing the light. A theme that names no light is a flat
//! one, and what is drawn is filled with the token it is given instead.

use super::{PColor, PGlass, PPaint, PRect};
use crate::theme::PTheme;

/// The theme's tokens for the light: its colour, how much of it the
/// strongest shade is, and the share of that a shade at rest and a soft
/// one have.
pub const LIGHT: &str = "shade.light";
pub const SHARE: &str = "shade.share";
pub const REST: &str = "shade.rest";
pub const SOFT: &str = "shade.soft";
/// Where the light fades to, in degrees: 0 is up, 90 to the right.
pub const SURFACE: &str = "surface.turn";
pub const LABEL: &str = "label.turn";
/// What a pane that is not glass stands on, and the ink on anything lit.
pub const GROUND: &str = "window.fill";
pub const INK: &str = "text.ink";

/// How strong a light is: the faintest for what is at rest, the
/// strongest for what was chosen or is pressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PLight {
    Soft,
    Rest,
    Strong,
}

impl PLight {
    /// The shade: the theme's light at this strength. `None` in a theme
    /// that names no light.
    pub fn shade(self, theme: &dyn PTheme) -> Option<PColor> {
        let light = theme.color(LIGHT)?;
        let part = match self {
            PLight::Strong => 1.0,
            PLight::Rest => theme.size(REST).unwrap_or(1.0),
            PLight::Soft => theme.size(SOFT).unwrap_or(1.0),
        };
        Some(PColor { a: light.a * theme.size(SHARE).unwrap_or(1.0) * part, ..light })
    }
}

/// Fill with the colour of a token, as a flat theme does. A token that
/// is empty draws nothing.
fn flat(paint: &mut dyn PPaint, rect: PRect, radius: f32, token: &str) {
    if !token.is_empty() {
        let color = paint.color(token);
        paint.round(rect, radius, color);
    }
}

/// The surface: light at the foot, nothing at the head.
pub fn surface(paint: &mut dyn PPaint, rect: PRect, radius: f32, light: PLight, token: &str) {
    match light.shade(paint.theme()) {
        Some(shade) => {
            let turn = paint.size(SURFACE);
            paint.blend(rect, radius, shade, PColor { a: 0.0, ..shade }, turn);
        }
        None => flat(paint, rect, radius, token),
    }
}

/// The theme's token for how much of a label's width its light
/// reaches: 0.7 has it gone seven tenths of the way across. A theme
/// without it has the light reach the far side.
pub const REACH: &str = "shade.reach";

/// The label: light from the left, gone at the right, or sooner where
/// the theme says how far it reaches. With `end` it runs the other way.
pub fn label(paint: &mut dyn PPaint, rect: PRect, radius: f32, light: PLight, end: bool, token: &str) {
    let reach = paint.theme().size(REACH).unwrap_or(1.0).clamp(0.0, 1.0);
    let wide = rect.size.width * reach;
    match (light.shade(paint.theme()), end) {
        (Some(shade), false) => {
            let turn = paint.theme().size(LABEL).unwrap_or(90.0);
            paint.blend(PRect::new(rect.origin.x, rect.origin.y, wide, rect.size.height), radius, shade, PColor { a: 0.0, ..shade }, turn);
        }
        (Some(shade), true) => {
            let turn = paint.theme().size(LABEL).unwrap_or(90.0);
            paint.blend(PRect::new(rect.origin.x + rect.size.width - wide, rect.origin.y, wide, rect.size.height), radius, PColor { a: 0.0, ..shade }, shade, turn);
        }
        (None, _) => flat(paint, rect, radius, token),
    }
}

/// A pane: a piece of the window's own furniture, a bar or a panel that
/// floats. Where the theme has glass it is glass, and what is behind it
/// shows through blurred; where it has none it stands on the window's
/// ground. Either way it wears the surface's light.
pub fn pane(paint: &mut dyn PPaint, rect: PRect, radius: f32, token: &str) {
    match PLight::Rest.shade(paint.theme()) {
        Some(_) => {
            under(paint, rect, radius);
            surface(paint, rect, radius, PLight::Rest, token);
        }
        None => flat(paint, rect, radius, token),
    }
}

/// A pane that stands at an edge of the window, as a bar at its side
/// does: it disappears toward that edge and is lit toward the page.
/// What is pressed inside it runs the other way, so the two lights cross
/// and nothing is filled from edge to edge. `end` is for one at the
/// right edge.
pub fn edge(paint: &mut dyn PPaint, rect: PRect, radius: f32, end: bool, token: &str) {
    match PLight::Rest.shade(paint.theme()) {
        Some(_) => {
            under(paint, rect, radius);
            label(paint, rect, radius, PLight::Rest, !end, token);
        }
        None => flat(paint, rect, radius, token),
    }
}

/// A bar and the column it opens: one pane, one shape with the bar's
/// corners, and two lights woven: the bar's from its right, the column,
/// its child, against it from its left, each gone by its middle, so
/// neither needs corners of its own. What stands in the column is lit
/// against the column, from the right, and so on down.
pub fn bond(paint: &mut dyn PPaint, first: PRect, second: PRect, radius: f32, token: &str) {
    let whole = PRect::new(first.origin.x, first.origin.y, first.size.width + second.size.width, first.size.height.max(second.size.height));
    match PLight::Rest.shade(paint.theme()) {
        Some(_) => {
            under(paint, whole, radius);
            label(paint, first, 0.0, PLight::Rest, true, token);
            label(paint, second, 0.0, PLight::Rest, false, token);
        }
        None => flat(paint, whole, radius, token),
    }
}

/// What a pane has under its light: glass where the theme has glass, the
/// window's ground where it has none.
fn under(paint: &mut dyn PPaint, rect: PRect, radius: f32) {
    match PGlass::of(paint.theme()) {
        Some(_) => paint.glass(rect, radius),
        None => flat(paint, rect, radius, GROUND),
    }
}

/// The ink on something lit: the theme's own ink, since light has no
/// colour to set another against. In a flat theme, the token's.
pub fn ink(paint: &dyn PPaint, token: &str) -> PColor {
    match paint.theme().color(LIGHT) {
        Some(_) => paint.color(INK),
        None => paint.color(token),
    }
}
