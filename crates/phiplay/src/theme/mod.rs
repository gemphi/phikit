//! A theme: the palette a venue is dressed in — the paint on the floor, the
//! boards, the seats, the light show on the façade — and the light itself.
//!
//! Sets name colours by role, never by value, so one court read through
//! eight themes is eight courts. Teams carry their own colours (`team`);
//! a theme dresses everything that is not a player.

mod set;

use crate::colour::Rgb;

/// The eight themes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BTheme {
    Bloom,
    DesertDusk,
    Lagoon,
    NeonNight,
    Savanna,
    Glacier,
    Carnival,
    Studio,
}

/// What a colour is for. Fields, venues and boards ask for a role.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BRole {
    /// The main colour: a key, a third, the boards' ground.
    Primary,
    /// The second colour: the centre circle, the alternate seats.
    Secondary,
    /// A small, strong colour: rims, the light show's peaks.
    Accent,
    /// The apron and run-off round the field.
    Apron,
    /// The lines.
    Line,
    /// The façade's light, at rest.
    Glow,
}

/// One theme's palette and light.
#[derive(Clone, Copy, Debug)]
pub struct BThemeSpec {
    pub label: &'static str,
    /// One line on what it is, for the picker's caption.
    pub caption: &'static str,
    pub primary: Rgb,
    pub secondary: Rgb,
    pub accent: Rgb,
    pub apron: Rgb,
    pub line: Rgb,
    pub glow: Rgb,
    /// The seat colours, cycled tier by tier.
    pub seats: [Rgb; 4],
    /// The sky or roof light's colour and the key light's.
    pub sky: Rgb,
    pub key: Rgb,
    /// The light show's tempo, beats a minute.
    pub tempo: f32,
}

impl BTheme {
    pub const ALL: [BTheme; 8] = [
        BTheme::Bloom,
        BTheme::DesertDusk,
        BTheme::Lagoon,
        BTheme::NeonNight,
        BTheme::Savanna,
        BTheme::Glacier,
        BTheme::Carnival,
        BTheme::Studio,
    ];
}

impl BThemeSpec {
    /// The colour a role names.
    pub fn colour(&self, role: BRole) -> Rgb {
        match role {
            BRole::Primary => self.primary,
            BRole::Secondary => self.secondary,
            BRole::Accent => self.accent,
            BRole::Apron => self.apron,
            BRole::Line => self.line,
            BRole::Glow => self.glow,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_theme_has_a_spec_and_its_lines_read_on_its_floor() {
        for theme in BTheme::ALL {
            let spec = theme.spec();
            assert!(!spec.label.is_empty() && !spec.caption.is_empty());
            // A line has to show against the apron it runs beside.
            let (l, a) = (spec.line.luminance(), spec.apron.luminance());
            let contrast = (l.max(a) + 0.05) / (l.min(a) + 0.05);
            assert!(contrast > 2.0, "{}: lines {contrast:.2}:1 on the apron", spec.label);
            assert!(spec.tempo > 40.0 && spec.tempo < 180.0);
        }
    }
}
