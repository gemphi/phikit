//! The eight uniforms.

use super::{BCollar, BPattern, BUniform, BUniformSpec};

impl BUniform {
    pub fn spec(self) -> &'static BUniformSpec {
        match self {
            BUniform::Classic => &CLASSIC,
            BUniform::Panel => &PANEL,
            BUniform::Sash => &SASH,
            BUniform::Stripe => &STRIPE,
            BUniform::Chevron => &CHEVRON,
            BUniform::Fade => &FADE,
            BUniform::Yoke => &YOKE,
            BUniform::Hoops => &HOOPS,
        }
    }
}

static CLASSIC: BUniformSpec = BUniformSpec { label: "Classic", pattern: BPattern::Piping, collar: BCollar::Crew, side_stripe: true };
static PANEL: BUniformSpec = BUniformSpec { label: "Panel", pattern: BPattern::Panels { width: 0.14 }, collar: BCollar::Scoop, side_stripe: true };
static SASH: BUniformSpec = BUniformSpec { label: "Sash", pattern: BPattern::Sash { width: 0.16, rise: 0.35 }, collar: BCollar::Vee, side_stripe: false };
static STRIPE: BUniformSpec = BUniformSpec { label: "Stripe", pattern: BPattern::Stripes { width: 0.05, apart: 0.62 }, collar: BCollar::Crew, side_stripe: true };
static CHEVRON: BUniformSpec = BUniformSpec { label: "Chevron", pattern: BPattern::Chevron { v: 0.86, width: 0.08 }, collar: BCollar::Vee, side_stripe: false };
static FADE: BUniformSpec = BUniformSpec { label: "Fade", pattern: BPattern::Fade { v: 0.45 }, collar: BCollar::Scoop, side_stripe: false };
static YOKE: BUniformSpec = BUniformSpec { label: "Yoke", pattern: BPattern::Yoke { v: 0.82 }, collar: BCollar::Crew, side_stripe: true };
static HOOPS: BUniformSpec = BUniformSpec { label: "Hoops", pattern: BPattern::Hoops { count: 4 }, collar: BCollar::Crew, side_stripe: false };
