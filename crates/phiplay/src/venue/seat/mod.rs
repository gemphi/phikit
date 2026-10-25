//! A seat: eight shapes, from the semi-hexagon with a car interior's
//! restraint to the phiegg the demo arena first had. Each is a stem, a
//! pan and a back; the back is what tells them apart (`BBack`).

mod set;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BSeat {
    SemiHex,
    Phiegg,
    Shell,
    Wing,
    Pod,
    Saddle,
    Bench,
    Blade,
}

/// The back's outline.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BBack {
    /// Half a hexagon standing on its long side: a flat top `top` of the
    /// width, the shoulders cut at 60°.
    SemiHex { top: f32 },
    /// The phiegg: a superellipse of power `n`, narrowing above its middle
    /// to `top` of its width.
    Egg { n: f32, top: f32 },
    /// A bucket wrapping round the sitter, `wrap` degrees of a circle.
    Shell { wrap: f32 },
    /// Swept back and up to a point, like a wing tip.
    Wing { sweep: f32 },
    /// A capsule: a rounded rectangle, radius `round` of the width.
    Pod { round: f32 },
    /// A low curve, a saddle's cantle.
    Saddle,
    /// No back: a bench.
    None,
    /// A thin tall slab.
    Blade,
}

#[derive(Clone, Copy, Debug)]
pub struct BSeatSpec {
    pub label: &'static str,
    pub back: BBack,
    /// The back's width and height, metres.
    pub width: f32,
    pub height: f32,
    /// How far the back leans away from the field, degrees.
    pub lean: f32,
    /// How many facets its outline is cut in: low is crisp, high is round.
    pub facets: u8,
}

impl BSeat {
    pub const ALL: [BSeat; 8] = [BSeat::SemiHex, BSeat::Phiegg, BSeat::Shell, BSeat::Wing, BSeat::Pod, BSeat::Saddle, BSeat::Bench, BSeat::Blade];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_seat_fits_a_seat_pitch() {
        for seat in BSeat::ALL {
            let s = seat.spec();
            assert!(s.width <= 0.5 && s.height <= 0.6, "{}: wider than a 0.55 m pitch allows", s.label);
            assert!(s.facets >= 4);
        }
    }
}
