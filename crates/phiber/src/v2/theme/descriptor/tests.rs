//! Descriptor catalog tests. Split out of `descriptor.rs` so the parent stays
//! under the 200-line cap. Enabled only under `#[cfg(test)]`.

use super::{
    BResolvedUiStyle, BShadowStyle, BStyleDescriptor, BStyleParseError, BStyleParser, BStyleValue,
    UI_STYLE_DESCRIPTORS,
};
use crate::v2::theme::style::APPLE_UI_STYLE_DEFAULTS;
use std::collections::HashSet;

fn alternate(descriptor: &BStyleDescriptor, current: BStyleValue) -> BStyleValue {
    match (descriptor.parser, current) {
        (BStyleParser::Number { min, max }, BStyleValue::Number(value)) => {
            BStyleValue::Number(if value != min { min } else { max })
        }
        (BStyleParser::Bool, BStyleValue::Bool(value)) => BStyleValue::Bool(!value),
        (BStyleParser::ColorHsla, BStyleValue::Color((h, s, l, a))) => {
            BStyleValue::Color((if h < 360.0 { h + 0.5 } else { 0.0 }, s, l, a))
        }
        (BStyleParser::Shadow, BStyleValue::Shadow(BShadowStyle::None)) => {
            BStyleValue::Shadow(BShadowStyle::ExtraLarge)
        }
        (BStyleParser::Shadow, BStyleValue::Shadow(BShadowStyle::ExtraLarge)) => {
            BStyleValue::Shadow(BShadowStyle::None)
        }
        _ => panic!("descriptor parser and target type disagree"),
    }
}

#[test]
fn catalog_declares_every_style_field_with_unique_key_and_target() {
    assert_eq!(UI_STYLE_DESCRIPTORS.len(), 99);
    let mut keys = HashSet::new();
    let mut targets = HashSet::new();
    for descriptor in UI_STYLE_DESCRIPTORS {
        let encoded = descriptor.key.encode();
        assert!(
            encoded.starts_with("ctx.ui_style."),
            "non-canonical key: {encoded}"
        );
        assert_eq!(
            descriptor.key.segs.len(),
            3,
            "key must have domain and token: {encoded}"
        );
        assert!(
            keys.insert(encoded.clone()),
            "duplicate descriptor key: {encoded}"
        );
        assert!(
            targets.insert(descriptor.target),
            "duplicate target: {}",
            descriptor.target
        );
        assert!(!descriptor.preference.section.is_empty());
        assert!(!descriptor.preference.label.is_empty());
    }
}

#[test]
fn every_descriptor_reads_its_default_and_owns_only_its_target() {
    let baseline = BResolvedUiStyle::from(&APPLE_UI_STYLE_DEFAULTS);
    for descriptor in UI_STYLE_DESCRIPTORS {
        assert_eq!(
            descriptor.value(&baseline),
            descriptor.default_value(&APPLE_UI_STYLE_DEFAULTS)
        );
        let replacement = alternate(descriptor, descriptor.value(&baseline));
        let mut changed = baseline;
        assert!(descriptor.apply_value(&mut changed, replacement));
        assert_eq!(descriptor.value(&changed), replacement);
        for unrelated in UI_STYLE_DESCRIPTORS {
            if unrelated.target != descriptor.target {
                assert_eq!(
                    unrelated.value(&changed),
                    unrelated.value(&baseline),
                    "{} changed unrelated field {}",
                    descriptor.target,
                    unrelated.target,
                );
            }
        }
    }
}

#[test]
fn parsers_reject_malformed_non_finite_and_out_of_bounds_values() {
    let numeric = BStyleParser::Number {
        min: 0.0,
        max: 10.0,
    };
    assert_eq!(numeric.parse("bad"), Err(BStyleParseError::Malformed));
    assert_eq!(numeric.parse("NaN"), Err(BStyleParseError::NonFinite));
    assert_eq!(numeric.parse("11"), Err(BStyleParseError::OutOfBounds));
    assert_eq!(numeric.parse("4.5"), Ok(BStyleValue::Number(4.5)));

    assert_eq!(
        BStyleParser::ColorHsla.parse("220, 50, 25, 0.5"),
        Ok(BStyleValue::Color((220.0, 50.0, 25.0, 0.5)))
    );
    assert_eq!(
        BStyleParser::ColorHsla.parse("361, 50, 25, 1"),
        Err(BStyleParseError::OutOfBounds)
    );
    assert_eq!(
        BStyleParser::Bool.parse("yes"),
        Err(BStyleParseError::Malformed)
    );
    assert_eq!(
        BStyleParser::Shadow.parse("large"),
        Err(BStyleParseError::Malformed)
    );
}
