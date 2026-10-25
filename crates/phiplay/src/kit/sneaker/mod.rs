//! A shoe. Eight court shoes, every collar above the ankle bone (about
//! 8 cm up a woman's foot from the floor) — the high-tops basketball is
//! played in, and the mids just over the bone. They take the team's
//! colours: the upper in white or the home colour, the trim where the
//! design puts it, the sole its own.

mod set;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BSneaker {
    Apex,
    Summit,
    Vector,
    Pulse,
    Arc,
    Glide,
    Forge,
    Volt,
}

/// Where the trim goes on the upper.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BShoeTrim {
    /// The toe cap.
    Toe,
    /// The heel counter.
    Heel,
    /// A blade running from the midfoot up to the collar.
    Blade,
    /// A band round the collar.
    Collar,
    /// The upper fading from the trim at the sole to the upper's colour at
    /// the collar.
    Fade,
    /// The upper split: trim on the outside, upper colour inside.
    Split,
    /// Panels round the eyelets and the heel.
    Panels,
    /// The sole lit (a translucent sole with the trim colour glowing).
    GlowSole,
}

#[derive(Clone, Copy, Debug)]
pub struct BSneakerSpec {
    pub label: &'static str,
    /// The collar's height above the floor, metres. Every one is above
    /// the ankle bone (0.08).
    pub collar: f32,
    /// The sole's thickness.
    pub sole: f32,
    /// A strap across the ankle.
    pub strap: bool,
    /// The upper is the team's home colour (else white).
    pub coloured: bool,
    pub trim: BShoeTrim,
}

/// The ankle bone's height above the floor: every collar clears it.
pub const ANKLE: f32 = 0.08;

impl BSneaker {
    pub const ALL: [BSneaker; 8] =
        [BSneaker::Apex, BSneaker::Summit, BSneaker::Vector, BSneaker::Pulse, BSneaker::Arc, BSneaker::Glide, BSneaker::Forge, BSneaker::Volt];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_collar_clears_the_ankle_and_every_design_is_its_own() {
        let mut seen = Vec::new();
        for shoe in BSneaker::ALL {
            let s = shoe.spec();
            assert!(s.collar > ANKLE + 0.02 && s.collar < 0.2, "{}: collar {}", s.label, s.collar);
            assert!(s.sole > 0.015 && s.sole < 0.05);
            assert!(!seen.contains(&s.trim), "{}: the same trim as another", s.label);
            seen.push(s.trim);
        }
    }
}
