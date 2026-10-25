//! What a field is made of: maple boards, a polymer court, an LED glass
//! floor, rubber, natural grass three ways, sand. Each sport has its own by
//! default (`BSportSpec::turf`); the picker may lay any of them.

mod set;

use crate::colour::Rgb;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BTurf {
    Maple,
    Polymer,
    Glass,
    Rubber,
    Grass,
    Stripes,
    Checks,
    Sand,
}

/// The surface's pattern, which the field texture draws under the lines.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BGrain {
    /// Boards along the length, `width` metres wide, joints staggered.
    Planks { width: f32 },
    /// One colour with a fine speckle.
    Speckle,
    /// A grid of glass tiles `size` metres square, each lit on its own.
    Tiles { size: f32 },
    /// Grass mown in bands `width` metres across the pitch.
    Mown { width: f32 },
    /// Grass mown in squares `size` metres a side.
    Chequer { size: f32 },
    /// Rippled sand.
    Ripples,
}

#[derive(Clone, Copy, Debug)]
pub struct BTurfSpec {
    pub label: &'static str,
    pub base: Rgb,
    /// The second tone of the pattern.
    pub alt: Rgb,
    pub grain: BGrain,
    pub roughness: f32,
    /// Lit from within (the glass floor).
    pub emissive: bool,
}

impl BTurf {
    pub const ALL: [BTurf; 8] = [BTurf::Maple, BTurf::Polymer, BTurf::Glass, BTurf::Rubber, BTurf::Grass, BTurf::Stripes, BTurf::Checks, BTurf::Sand];
}
