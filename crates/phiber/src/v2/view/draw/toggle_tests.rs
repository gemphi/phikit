//! Tests for `toggle` — split out so the module itself sits under the
//! 240-line read cap (`tests/level_structure.rs`).

use super::*;

/// The frame is the window and the third is the zone: toggling moves the
/// one that is in question and leaves the one that is not.
/// Five places a zone can be, five pictures. They shared arms before, so
/// a Centre toggle drew as a Left one and said the wrong zone.
#[test]
fn each_of_the_five_places_draws_its_own_picture() {
    let place = |zone| BZoneGlyph { zone, visible: true }.place();
    let five = [BZoneType::Left, BZoneType::Center, BZoneType::Right, BZoneType::Top, BZoneType::Bottom];
    let mut seen: Vec<(BAxis, BJustify, BAlign)> = Vec::new();
    for zone in five {
        let at = place(zone);
        assert!(!seen.contains(&at), "{zone:?} draws where another zone already does");
        seen.push(at);
    }
    assert_eq!(seen.len(), 5, "left, centre, right, top, bottom");
    // The two across and the two down are each other's opposite end.
    assert_eq!(place(BZoneType::Left).1, BJustify::Start);
    assert_eq!(place(BZoneType::Right).1, BJustify::End);
    assert_eq!(place(BZoneType::Top).1, BJustify::Start);
    assert_eq!(place(BZoneType::Bottom).1, BJustify::End);
    // And the middle is pushed to neither end, on either axis.
    assert_eq!(place(BZoneType::Center), (BAxis::Row, BJustify::Center, BAlign::Center));
}

#[test]
fn only_the_zone_s_third_changes_ink() {
    for zone in [BZoneType::Left, BZoneType::Right, BZoneType::Bottom, BZoneType::Top] {
        let shown = BZoneGlyph { zone, visible: true };
        let hidden = BZoneGlyph { zone, visible: false };
        assert_eq!(shown.frame_ink(), hidden.frame_ink(), "{zone:?}: the window does not move");
        assert_eq!(shown.frame_ink(), BInk::Text, "{zone:?}: and it is ordinary ink, like the icons beside it");
        assert_eq!(shown.zone_ink(), BInk::Accent, "{zone:?}: shown is the accent");
        assert_eq!(hidden.zone_ink(), BInk::Dim, "{zone:?}: hidden falls back");
        assert_ne!(shown.zone_ink(), hidden.zone_ink(), "{zone:?}: the third is what answers");
    }
}
