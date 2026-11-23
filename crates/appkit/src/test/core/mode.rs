use crate::pui::core::*;

#[test]
fn a_mode_has_a_name_a_configuration_file_can_write() {
    let name = PMode::ALL.map(PMode::name);
    assert_eq!(name, ["desktop", "mobile", "spatial"]);
    assert!(PMode::ALL.iter().all(|mode| PMode::named(mode.name()) == Some(*mode)));
    assert_eq!(PMode::named("tablet"), None);
}

#[test]
fn a_narrow_window_is_a_mobile_one_and_a_wide_one_a_desktop_one() {
    let edge = setting("mobile.below");
    assert!(edge > 0.0, "data/mode.tsv gives the width");
    assert_eq!((PMode::of_width(edge - 1.0), PMode::of_width(edge), PMode::of_width(edge * 2.0)), (PMode::Mobile, PMode::Desktop, PMode::Desktop));
    // The same window dragged narrower changes mode: that is responsive.
    let mode: Vec<PMode> = [1280.0, 900.0, 600.0, 320.0].map(PMode::of_width).to_vec();
    assert_eq!(mode, [PMode::Desktop, PMode::Desktop, PMode::Mobile, PMode::Mobile]);
}

#[test]
fn space_is_never_guessed_from_a_width() {
    assert!([0.0, 500.0, 5000.0].iter().all(|width| PMode::of_width(*width) != PMode::Spatial));
}

#[test]
fn the_settings_of_space_are_in_the_data_file() {
    assert!(setting("spatial.width") > 0.0 && setting("spatial.height") > 0.0 && setting("spatial.turn") > 0.0);
}
