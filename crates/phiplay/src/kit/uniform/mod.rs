//! A uniform's pattern: what the trim colour does on the body colour. Eight
//! of them, all clean American-Olympic silhouettes — one idea each, no
//! badges, the number the loudest thing on the shirt.

mod set;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BUniform {
    Classic,
    Panel,
    Sash,
    Stripe,
    Chevron,
    Fade,
    Yoke,
    Hoops,
}

/// Where the trim goes, in the shirt's own frame: `u` across the body from
/// the wearer's right (0) to left (1), `v` from the hem (0) to the
/// shoulders (1), on the front; the back mirrors it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BPattern {
    /// Piping round the neck and the arm holes only.
    Piping,
    /// Side panels `width` of the body's width.
    Panels { width: f32 },
    /// A diagonal band rising from the wearer's right side to her left
    /// shoulder, `rise` above the line from hem to shoulder corner, so it
    /// passes over the number rather than through it.
    Sash { width: f32, rise: f32 },
    /// Two vertical stripes either side of the number.
    Stripes { width: f32, apart: f32 },
    /// A V across the chest at height `v`.
    Chevron { v: f32, width: f32 },
    /// The trim fading into the body from the hem to `v`.
    Fade { v: f32 },
    /// The shoulders in trim down to `v`.
    Yoke { v: f32 },
    /// Horizontal bands, `count` of them.
    Hoops { count: u8 },
}

/// The neckline.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BCollar {
    Crew,
    Vee,
    Scoop,
}

#[derive(Clone, Copy, Debug)]
pub struct BUniformSpec {
    pub label: &'static str,
    pub pattern: BPattern,
    pub collar: BCollar,
    /// The shorts carry a side stripe of the trim.
    pub side_stripe: bool,
}

impl BUniform {
    pub const ALL: [BUniform; 8] =
        [BUniform::Classic, BUniform::Panel, BUniform::Sash, BUniform::Stripe, BUniform::Chevron, BUniform::Fade, BUniform::Yoke, BUniform::Hoops];
}

impl BPattern {
    /// Whether the point (`u`, `v`) of the shirt is trim — what the
    /// uniform's texture is painted from.
    pub fn is_trim(&self, u: f32, v: f32) -> bool {
        match *self {
            BPattern::Piping => false,
            BPattern::Panels { width } => u < width || u > 1.0 - width,
            BPattern::Sash { width, rise } => ((v - u - rise).abs() * std::f32::consts::FRAC_1_SQRT_2) < width / 2.0,
            BPattern::Stripes { width, apart } => {
                let d = (u - 0.5).abs();
                (d - apart / 2.0).abs() < width / 2.0
            }
            BPattern::Chevron { v: at, width } => {
                let line = at - 0.5 * (u - 0.5).abs();
                (v - line).abs() < width / 2.0
            }
            BPattern::Fade { v: at } => v < at * 0.5,
            BPattern::Yoke { v: at } => v > at,
            BPattern::Hoops { count } => (v * f32::from(count) * 2.0).floor() as u32 % 2 == 1,
        }
    }

    /// How much trim a point carries, for patterns that blend (the fade);
    /// 0 or 1 for the rest.
    pub fn trim_weight(&self, u: f32, v: f32) -> f32 {
        match *self {
            BPattern::Fade { v: at } => (1.0 - v / at).clamp(0.0, 1.0),
            _ => f32::from(u8::from(self.is_trim(u, v))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_pattern_but_piping_puts_trim_somewhere_and_leaves_room_for_the_number() {
        for uniform in BUniform::ALL {
            let spec = uniform.spec();
            let mut trim = 0;
            for i in 0..50 {
                for j in 0..50 {
                    trim += usize::from(spec.pattern.trim_weight(i as f32 / 49.0, j as f32 / 49.0) > 0.5);
                }
            }
            let share = trim as f32 / 2500.0;
            if spec.pattern == BPattern::Piping {
                assert_eq!(trim, 0);
            } else {
                assert!(share > 0.05 && share < 0.6, "{}: {share:.2} trim", spec.label);
            }
            // The chest's middle, where the number goes, is body colour on
            // every pattern but the hoops (which run behind it).
            if !matches!(spec.pattern, BPattern::Hoops { .. }) {
                assert!(spec.pattern.trim_weight(0.5, 0.6) < 0.5, "{}: the number sits on body colour", spec.label);
            }
        }
    }
}
