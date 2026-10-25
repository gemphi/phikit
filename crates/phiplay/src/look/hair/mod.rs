//! A hairstyle, as the parameters it is grown from rather than a model:
//! a shell over the scalp (its thickness, its curl), the shapes standing
//! off it (a puff, a bun, a top knot), and the strands that hang (a
//! ponytail, braids, locs, twists). The generator (`build::hair`) reads these
//! and builds the mesh on any head, so sixteen styles on eight looks on a
//! crowd of three thousand cost sixteen sets of numbers.

mod set;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BHair {
    Buzz,
    Crop,
    Pixie,
    Bob,
    Waves,
    Afro,
    Puff,
    DoublePuff,
    Bun,
    TopKnot,
    Ponytail,
    Cornrows,
    BoxBraids,
    Locs,
    Twists,
    Fade,
}

/// Something standing off the scalp.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BVolume {
    /// Nothing: the shell is the style.
    None,
    /// A ball of `radius` at the crown (`at` is up the head's back, 0 the
    /// nape and 1 the top), or two of them at the sides.
    Puff { radius: f32, at: f32, pair: bool },
}

/// What hangs from the scalp.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BStrands {
    None,
    /// One gathered tail from the back of the crown.
    Tail { length: f32, width: f32 },
    /// Many strands all round below a parting line, each `width` thick,
    /// `length` long, plaited (`braid`) or not.
    Hanging { count: u16, length: f32, width: f32, braid: bool },
    /// Rows laid flat front to back on the scalp.
    Rows { count: u8 },
}

#[derive(Clone, Copy, Debug)]
pub struct BHairSpec {
    pub label: &'static str,
    /// How far the shell stands off the scalp, metres.
    pub shell: f32,
    /// How low the shell comes down the back and sides, as a share of the
    /// head from the crown (0) to the nape (1).
    pub reach: f32,
    /// 0 straight to 1 tightly coiled: how rough the shell's surface is.
    pub curl: f32,
    pub volume: BVolume,
    pub strands: BStrands,
    /// The sides cut close under the shell (a fade).
    pub faded_sides: bool,
}

impl BHair {
    pub const ALL: [BHair; 16] = [
        BHair::Buzz,
        BHair::Crop,
        BHair::Pixie,
        BHair::Bob,
        BHair::Waves,
        BHair::Afro,
        BHair::Puff,
        BHair::DoublePuff,
        BHair::Bun,
        BHair::TopKnot,
        BHair::Ponytail,
        BHair::Cornrows,
        BHair::BoxBraids,
        BHair::Locs,
        BHair::Twists,
        BHair::Fade,
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sixteen_styles_each_its_own() {
        for (i, a) in BHair::ALL.iter().enumerate() {
            let s = a.spec();
            assert!(s.shell >= 0.0 && s.shell < 0.12, "{}", s.label);
            assert!((0.0..=1.0).contains(&s.reach) && (0.0..=1.0).contains(&s.curl));
            for b in &BHair::ALL[i + 1..] {
                let t = b.spec();
                let same = s.shell == t.shell && s.reach == t.reach && s.curl == t.curl && s.volume == t.volume && s.strands == t.strands && s.faded_sides == t.faded_sides;
                assert!(!same, "{} and {} are the same style", s.label, t.label);
            }
        }
    }
}
