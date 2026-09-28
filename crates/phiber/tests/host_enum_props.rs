//! Property 2 — Host enum round-trip.
//!
//! Feature: phiber-blender-space-parity
//! Validates: Requirements 52.12
//!
//! For every `h` in `BHostType::ALL`, `BHostType::from_slug(h.slug())`
//! returns `Some(h)`. Every host that appears in `SWITCHABLE` also lives
//! in `ALL`.

use phiber::v2::host::BHostType;
use proptest::prelude::*;

proptest! {
    #[test]
    fn round_trip_slug_from_slug(idx in 0usize..BHostType::ALL.len()) {
        let h = BHostType::ALL[idx];
        prop_assert_eq!(BHostType::from_slug(h.slug()), Some(h));
    }

    #[test]
    fn unknown_slug_is_none(garbage in "[a-z0-9]{1,20}") {
        if BHostType::ALL.iter().any(|h| h.slug() == garbage) {
            return Ok(());
        }
        prop_assert_eq!(BHostType::from_slug(&garbage), None);
    }
}

#[test]
fn all_has_twenty_eight_variants() {
    // 27 Blender editor hosts + 1 Phiber-native Activity rail host.
    assert_eq!(BHostType::ALL.len(), 28);
}

#[test]
fn switchable_is_subset_of_all() {
    for h in BHostType::SWITCHABLE {
        assert!(
            BHostType::ALL.contains(h),
            "SWITCHABLE contains {h:?} which is missing from ALL"
        );
    }
}

#[test]
fn switchable_excludes_chrome_hosts() {
    // Topbar and Statusbar are chrome-only per Requirements 42.1 and 42.5.
    assert!(!BHostType::SWITCHABLE.contains(&BHostType::Topbar));
    assert!(!BHostType::SWITCHABLE.contains(&BHostType::Statusbar));
}
