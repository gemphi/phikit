//! Parsing + resolved-value application for `BStyleDescriptor`.
//!
//! Split out of `descriptor.rs` to keep the parent under the 200-line cap.
//! `impl BStyleParser::parse`, `impl BStyleDescriptor::*` and the free
//! `parse_hsla` helper live here; they are pulled back into the parent's
//! public surface via `mod parser;` at the top of `descriptor.rs`.

use super::{
    BColor, BFontFamily, BResolvedUiStyle, BShadowStyle, BStyleChoice, BStyleDescriptor,
    BStyleParseError, BStyleParser, BStyleValue, BUiStyleDefaults, FONT_CHOICES, SHADOW_CHOICES,
};

impl BStyleParser {
    pub fn parse(self, raw: &str) -> Result<BStyleValue, BStyleParseError> {
        let raw = raw.trim();
        match self {
            Self::Number { min, max } => {
                let value = raw
                    .parse::<f32>()
                    .map_err(|_| BStyleParseError::Malformed)?;
                if !value.is_finite() {
                    return Err(BStyleParseError::NonFinite);
                }
                if value < min || value > max {
                    return Err(BStyleParseError::OutOfBounds);
                }
                Ok(BStyleValue::Number(value))
            }
            Self::Bool => match raw {
                "true" => Ok(BStyleValue::Bool(true)),
                "false" => Ok(BStyleValue::Bool(false)),
                _ => Err(BStyleParseError::Malformed),
            },
            Self::ColorHsla => parse_color(raw).map(BStyleValue::Color),
            Self::Shadow => BShadowStyle::from_slug(raw)
                .map(BStyleValue::Shadow)
                .ok_or(BStyleParseError::Malformed),
            Self::Font => BFontFamily::from_slug(raw)
                .map(BStyleValue::Font)
                .ok_or(BStyleParseError::Malformed),
        }
    }
}

/// A colour in either spelling it has: `"h, s, l, a"`, or a hex code.
///
/// Blender's picker has a Hex field beside its sliders
/// (`interface_region_color_picker.cc:1008`, `hex_to_rgba`), and a hex code is how
/// a colour arrives from a browser, a palette or another app. Accepting it here
/// rather than only in the picker means the field already beside every swatch
/// takes one — the colour has two spellings and one parser, the same reason
/// `parse_hsla` is public.
///
/// A comma decides which: `"h, s, l, a"` always has three, and no hex code has
/// any. So this cannot swallow an HSLA string, and `#3D3D3D`, `3d3d3d` and `50`
/// are all read as hex, which is what Blender reads them as too.
///
/// One divergence, deliberate: Blender's hex field sits beside the colour it
/// edits, so a form that states no alpha (`#RRGGBB`) keeps the colour's own. This
/// parser is context-free — a descriptor validating a string has no current colour
/// to keep an alpha from — so a hex without an alpha is opaque. The picker softens
/// it by spelling a transparent colour's alpha out (`format_hex` writes eight
/// digits whenever a colour is not opaque), so the digits are on screen to keep.
pub fn parse_color(raw: &str) -> Result<BColor, BStyleParseError> {
    if raw.contains(',') {
        return parse_hsla(raw);
    }
    super::hex::parse_hex(raw, (0.0, 0.0, 0.0, 1.0)).ok_or(BStyleParseError::Malformed)
}

/// Parse an `"h, s, l, a"` string into a `BColor`, validating each channel's
/// bounds independently (HSLA: h∈[0,360], s∈[0,100], l∈[0,100], a∈[0,1]).
///
/// Public because the renderer needs it too: a colour control carries its value
/// in this spelling, and its swatch has to become that colour rather than guess.
/// One parser for the string, so what the swatch shows cannot disagree with what
/// the descriptor would accept.
pub fn parse_hsla(raw: &str) -> Result<BColor, BStyleParseError> {
    let mut parts = raw.split(',').map(str::trim);
    let mut next = || {
        parts
            .next()
            .ok_or(BStyleParseError::Malformed)?
            .parse::<f32>()
            .map_err(|_| BStyleParseError::Malformed)
    };
    let color = (next()?, next()?, next()?, next()?);
    if parts.next().is_some() {
        return Err(BStyleParseError::Malformed);
    }
    if ![color.0, color.1, color.2, color.3]
        .iter()
        .all(|v| v.is_finite())
    {
        return Err(BStyleParseError::NonFinite);
    }
    if !(0.0..=360.0).contains(&color.0)
        || !(0.0..=100.0).contains(&color.1)
        || !(0.0..=100.0).contains(&color.2)
        || !(0.0..=1.0).contains(&color.3)
    {
        return Err(BStyleParseError::OutOfBounds);
    }
    Ok(color)
}

impl BStyleDescriptor {
    pub fn parse_and_validate(&self, raw: &str) -> Result<BStyleValue, BStyleParseError> {
        self.parser.parse(raw)
    }
    pub fn format_value(&self, value: BStyleValue) -> Option<String> {
        let encoded = match value {
            BStyleValue::Number(value) => value.to_string(),
            BStyleValue::Bool(value) => value.to_string(),
            BStyleValue::Color((h, s, l, a)) => format!("{h}, {s}, {l}, {a}"),
            BStyleValue::Shadow(shadow) => shadow.slug().to_string(),
            BStyleValue::Font(family) => family.slug().to_string(),
        };
        (self.parse_and_validate(&encoded).ok() == Some(value)).then_some(encoded)
    }
    pub fn choices(&self) -> &'static [BStyleChoice] {
        match self.parser {
            BStyleParser::Shadow => SHADOW_CHOICES,
            BStyleParser::Font => &FONT_CHOICES,
            _ => &[],
        }
    }
    pub fn default_value(&self, defaults: &BUiStyleDefaults) -> BStyleValue {
        (self.default)(defaults)
    }
    pub fn apply_value(&self, style: &mut BResolvedUiStyle, value: BStyleValue) -> bool {
        (self.apply)(style, value)
    }
    pub fn value(&self, style: &BResolvedUiStyle) -> BStyleValue {
        (self.read)(style)
    }
}
