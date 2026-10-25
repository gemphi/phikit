//! The eight shoes. Names are the set's own; none is a brand's.

use super::{BShoeTrim, BSneaker, BSneakerSpec};

impl BSneaker {
    pub fn spec(self) -> &'static BSneakerSpec {
        match self {
            BSneaker::Apex => &APEX,
            BSneaker::Summit => &SUMMIT,
            BSneaker::Vector => &VECTOR,
            BSneaker::Pulse => &PULSE,
            BSneaker::Arc => &ARC,
            BSneaker::Glide => &GLIDE,
            BSneaker::Forge => &FORGE,
            BSneaker::Volt => &VOLT,
        }
    }
}

static APEX: BSneakerSpec = BSneakerSpec { label: "Apex", collar: 0.16, sole: 0.028, strap: false, coloured: false, trim: BShoeTrim::Toe };
static SUMMIT: BSneakerSpec = BSneakerSpec { label: "Summit", collar: 0.17, sole: 0.030, strap: true, coloured: true, trim: BShoeTrim::Collar };
static VECTOR: BSneakerSpec = BSneakerSpec { label: "Vector", collar: 0.12, sole: 0.026, strap: false, coloured: false, trim: BShoeTrim::Blade };
static PULSE: BSneakerSpec = BSneakerSpec { label: "Pulse", collar: 0.15, sole: 0.030, strap: false, coloured: true, trim: BShoeTrim::Fade };
static ARC: BSneakerSpec = BSneakerSpec { label: "Arc", collar: 0.15, sole: 0.027, strap: false, coloured: false, trim: BShoeTrim::Heel };
static GLIDE: BSneakerSpec = BSneakerSpec { label: "Glide", collar: 0.11, sole: 0.024, strap: false, coloured: true, trim: BShoeTrim::Split };
static FORGE: BSneakerSpec = BSneakerSpec { label: "Forge", collar: 0.17, sole: 0.038, strap: true, coloured: false, trim: BShoeTrim::Panels };
static VOLT: BSneakerSpec = BSneakerSpec { label: "Volt", collar: 0.14, sole: 0.034, strap: false, coloured: false, trim: BShoeTrim::GlowSole };
