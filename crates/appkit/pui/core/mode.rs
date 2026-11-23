//! How an app is being used: at a desk, in a hand, or in space. One app
//! runs in all three; what changes is how its window is laid out
//! (`window/place.rs`). The numbers that decide it are `data/mode.tsv`.

use std::sync::OnceLock;

/// How an app is being used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PMode {
    /// A window on a screen, with a pointer and keys.
    Desktop,
    /// A narrow screen held in a hand: one thing at a time.
    Mobile,
    /// A headset: every area is a panel of its own around the person.
    Spatial,
}

impl PMode {
    pub const ALL: [PMode; 3] = [PMode::Desktop, PMode::Mobile, PMode::Spatial];

    /// The mode's name, as a configuration file writes it.
    pub fn name(self) -> &'static str {
        match self {
            PMode::Desktop => "desktop",
            PMode::Mobile => "mobile",
            PMode::Spatial => "spatial",
        }
    }

    /// The mode of that name.
    pub fn named(name: &str) -> Option<PMode> {
        PMode::ALL.into_iter().find(|mode| mode.name() == name)
    }

    /// The mode of a flat window this wide: mobile below the width
    /// `data/mode.tsv` gives, desktop from it on. This is what makes an app
    /// responsive: the same window, narrowed, is laid out as a mobile one.
    /// Space is never guessed from a size; the platform says so.
    pub fn of_width(width: f32) -> PMode {
        match width < setting("mobile.below") {
            true => PMode::Mobile,
            false => PMode::Desktop,
        }
    }
}

/// A number of `data/mode.tsv`, by its name; zero for a name it does not
/// have.
pub fn setting(name: &str) -> f32 {
    static TABLE: OnceLock<Vec<(&'static str, f32)>> = OnceLock::new();
    let table = TABLE.get_or_init(|| {
        let line = include_str!("../../src/data/mode.tsv").lines().filter(|line| !line.trim().is_empty() && !line.starts_with('#'));
        line.filter_map(|line| line.split_once('\t')).filter_map(|(name, value)| Some((name.trim(), value.trim().parse().ok()?))).collect()
    });
    let found = table.iter().find(|(known, _)| *known == name);
    debug_assert!(found.is_some(), "data/mode.tsv has no {name}");
    found.map_or(0.0, |(_, value)| *value)
}
