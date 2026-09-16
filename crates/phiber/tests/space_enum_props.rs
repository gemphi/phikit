//! Property 1 — Space enum round-trip.
//!
//! Feature: phiber-blender-space-parity
//! Validates: Requirements 1.4, 1.5, 52.11
//!
//! For every `s` in `BSpaceType::EVERY`, `BSpaceType::from_slug(s.slug())`
//! returns `Some(s)`. For any string not in the slug set, `from_slug`
//! returns `None`.

use phiber::v2::space::BSpaceType;
use proptest::prelude::*;

fn arb_space() -> impl Strategy<Value = BSpaceType> {
    prop_oneof![
        Just(BSpaceType::Layout),
        Just(BSpaceType::Modeling),
        Just(BSpaceType::Sculpting),
        Just(BSpaceType::UvEditing),
        Just(BSpaceType::TexturePaint),
        Just(BSpaceType::Shading),
        Just(BSpaceType::Animation),
        Just(BSpaceType::Rendering),
        Just(BSpaceType::Compositing),
        Just(BSpaceType::GeometryNodes),
        Just(BSpaceType::Scripting),
    ]
}

proptest! {
    #[test]
    fn round_trip_slug_from_slug(s in arb_space()) {
        prop_assert_eq!(BSpaceType::from_slug(s.slug()), Some(s));
    }

    #[test]
    fn unknown_slug_is_none(garbage in "[a-z0-9]{1,20}") {
        // Reject any accidental match against a real slug.
        if BSpaceType::EVERY.iter().any(|t| t.slug() == garbage) {
            return Ok(());
        }
        prop_assert_eq!(BSpaceType::from_slug(&garbage), None);
    }
}

#[test]
fn every_has_eleven_variants() {
    assert_eq!(BSpaceType::EVERY.len(), 11);
}

/// The tab strip is a subset of the spaces that exist: Modeling and
/// Sculpting still resolve by slug but are no longer offered as tabs.
#[test]
fn tab_strip_excludes_modeling_and_sculpting() {
    assert_eq!(BSpaceType::ALL.len(), 9);
    assert!(!BSpaceType::ALL.contains(&BSpaceType::Modeling));
    assert!(!BSpaceType::ALL.contains(&BSpaceType::Sculpting));
    assert_eq!(
        BSpaceType::from_slug("Modeling"),
        Some(BSpaceType::Modeling)
    );
    assert!(!BSpaceType::Modeling.is_tab());
    assert!(BSpaceType::Layout.is_tab());
}
