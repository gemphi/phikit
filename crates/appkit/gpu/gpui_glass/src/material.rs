//! How a piece of glass looks. Each style's numbers are a row of
//! `data/style.tsv`; colours come from the caller's theme through
//! [`GlassMaterial::themed`], so none is fixed in this crate.

use std::sync::OnceLock;

/// `NSGlassEffectView.Style`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GlassStyle {
    /// Frosted: a wide blur and a real tint, for panels with text on them.
    #[default]
    Regular,
    /// Barely frosted: the backdrop stays readable, the rim does the work.
    Clear,
}

impl GlassStyle {
    /// The name the style table spells it with.
    pub fn name(self) -> &'static str {
        match self {
            Self::Regular => "regular",
            Self::Clear => "clear",
        }
    }
}

/// Everything the shading stage needs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlassMaterial {
    /// Blur reach in pixels; the Gaussian's sigma is half of this.
    pub blur_radius: f32,
    /// 1 leaves colour alone. Blurring greys colour out; this brings it back.
    pub saturation: f32,
    pub tint: [f32; 3],
    /// How far colour moves toward `tint`, 0..=1.
    pub tint_amount: f32,
    /// How far the rim pulls its sample inward, in pixels.
    pub lens: f32,
    pub edge: [f32; 3],
    pub edge_strength: f32,
    pub shadow: [f32; 3],
    pub shadow_strength: f32,
}

impl GlassMaterial {
    /// The style's shape, without colours: a neutral tint and edge. A style
    /// the table does not name is glass that does nothing.
    pub fn of(style: GlassStyle) -> Self {
        Self::named(style.name()).unwrap_or(Self::shaped([0.0; 6]))
    }

    /// Any row of `data/style.tsv` by its name: the two glass styles, and the
    /// materials Apple names by thickness (`ultra_thin` to `ultra_thick`, `bar`).
    pub fn named(name: &str) -> Option<Self> {
        table().iter().find(|(known, _)| known == name).map(|(_, number)| Self::shaped(*number))
    }

    fn shaped(number: [f32; 6]) -> Self {
        let [blur_radius, saturation, tint_amount, lens, edge_strength, shadow_strength] = number;
        Self {
            blur_radius,
            saturation,
            tint: [0.0; 3],
            tint_amount,
            lens,
            edge: [1.0; 3],
            edge_strength,
            shadow: [0.0; 3],
            shadow_strength,
        }
    }

    /// The style in a theme's colours. `tint` carries its own alpha;
    /// `opacity` is the theme's share of tint over a Regular backdrop, the
    /// style's own when the theme has none. Clear keeps its smaller part of it.
    pub fn themed(style: GlassStyle, tint: [f32; 4], opacity: Option<f32>, edge: [f32; 3], shadow: [f32; 3]) -> Self {
        let base = Self::of(style);
        let regular = Self::of(GlassStyle::Regular).tint_amount;
        let opacity = opacity.unwrap_or(base.tint_amount);
        let share = match style {
            GlassStyle::Regular => opacity,
            GlassStyle::Clear => opacity * base.tint_amount / regular.max(f32::EPSILON),
        };
        Self {
            tint: [tint[0], tint[1], tint[2]],
            tint_amount: (share * tint[3]).clamp(0.0, 1.0),
            edge,
            shadow,
            ..base
        }
    }
}

/// `data/style.tsv`, read once: a style's name and its six numbers.
fn table() -> &'static [(String, [f32; 6])] {
    static TABLE: OnceLock<Vec<(String, [f32; 6])>> = OnceLock::new();
    TABLE.get_or_init(|| include_str!("../data/style.tsv").lines().filter_map(row).collect())
}

/// One line of the table, or nothing for a blank line or a comment.
fn row(line: &str) -> Option<(String, [f32; 6])> {
    let mut cell = line.split('\t');
    let name = cell.next().filter(|name| !name.is_empty() && !name.starts_with('#'))?;
    let mut number = [0.0f32; 6];
    for slot in number.iter_mut() {
        *slot = cell.next()?.trim().parse().ok()?;
    }
    Some((name.to_string(), number))
}
