//! Parsing + resolved-value application for `BStyleDescriptor`.
//!
//! Split out of `descriptor.rs` to keep the parent under the 200-line cap.
//! `impl BStyleParser::parse`, `impl BStyleDescriptor::*` and the free
//! `parse_hsla` helper live here; they are pulled back into the parent's
//! public surface via `mod parser;` at the top of `descriptor.rs`.

use super::{
    BColor, BResolvedUiStyle, BShadowStyle, BStyleChoice, BStyleDescriptor, BStyleParseError,
    BStyleParser, BStyleValue, BUiStyleDefaults, SHADOW_CHOICES,
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
            Self::ColorHsla => parse_hsla(raw).map(BStyleValue::Color),
            Self::Shadow => match raw {
                "none" => Ok(BStyleValue::Shadow(BShadowStyle::None)),
                "extra_large" => Ok(BStyleValue::Shadow(BShadowStyle::ExtraLarge)),
                _ => Err(BStyleParseError::Malformed),
            },
        }
    }
}

/// Parse an `"h, s, l, a"` string into a `BColor`, validating each channel's
/// bounds independently (HSLA: h∈[0,360], s∈[0,100], l∈[0,100], a∈[0,1]).
pub(super) fn parse_hsla(raw: &str) -> Result<BColor, BStyleParseError> {
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
            BStyleValue::Shadow(BShadowStyle::None) => "none".to_string(),
            BStyleValue::Shadow(BShadowStyle::ExtraLarge) => "extra_large".to_string(),
        };
        (self.parse_and_validate(&encoded).ok() == Some(value)).then_some(encoded)
    }
    pub fn choices(&self) -> &'static [BStyleChoice] {
        match self.parser {
            BStyleParser::Shadow => SHADOW_CHOICES,
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
