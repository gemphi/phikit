//! The spatial view's table: what each class of node is as a solid.
//!
//! Blender has no 3D node editor, so nothing here is ported — every value is
//! phiber's own and this file is where all of them live, as a table read down
//! rather than a rule hidden in drawing code (the proposal's board 2 and
//! board 5, and `theme/style/node_class` for the class a node belongs to).
//!
//! Two axes are ours to assign — a node's `x` and `y` come from the file:
//!
//! * **Corner radius by class** — what a node IS reads off its silhouette
//!   before its colour does: a source is round, a terminal is sharp, an
//!   operator is in between, and the field class (attribute / geometry) is a
//!   hex prism, the one shape that is not a rounded box at all.
//! * **Height by class** — the relief that makes a forty-node chain readable
//!   end to end. Sources sit low, operators higher, shaders higher still, and
//!   the output stands tallest: the graph rises toward where it is going.
//!   Fold distance, never order — a tier is a height, not a layer index.

use crate::v2::theme::BNodeClass;

/// How a solid's footprint is cut.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BSolidShape {
    /// A rounded box; the radius is `BSolidSpec::radius`.
    Rounded,
    /// A hexagonal prism — the field class.
    Hex,
}

/// One class's solid.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BSolidSpec {
    pub shape: BSolidShape,
    /// Corner radius in view units; one of the four class radii.
    pub radius: f32,
    /// Height in tiers of [`RELIEF`] × the card header height.
    pub tier: u8,
}

/// The four class radii, in view units (the proposal's 2 / 6 / 14 / 26).
pub const SHARP: f32 = 2.0;
pub const SOFT: f32 = 6.0;
pub const ROUND: f32 = 14.0;
pub const PILL: f32 = 26.0;

/// One tier of height as a fraction of the card header's height, so the
/// relief scales with the theme like everything else on the card.
pub const RELIEF: f32 = 0.6;

/// A frame's floor plate, as a fraction of the card header's height — thin
/// enough to read as a terrace, thick enough to cast an edge.
pub const PLATE: f32 = 0.15;

const fn rounded(radius: f32, tier: u8) -> BSolidSpec {
    BSolidSpec { shape: BSolidShape::Rounded, radius, tier }
}

const fn hex(tier: u8) -> BSolidSpec {
    BSolidSpec { shape: BSolidShape::Hex, radius: 0.0, tier }
}

/// The table. Read down: a class not named takes the plain row.
const BY_CLASS: &[(BNodeClass, BSolidSpec)] = &[
    (BNodeClass::Output, rounded(SHARP, 4)),
    (BNodeClass::Shader, rounded(SOFT, 3)),
    (BNodeClass::Group, rounded(SOFT, 3)),
    (BNodeClass::Script, rounded(SOFT, 3)),
    (BNodeClass::Texture, rounded(ROUND, 2)),
    (BNodeClass::Color, rounded(ROUND, 2)),
    (BNodeClass::Vector, rounded(ROUND, 2)),
    (BNodeClass::Converter, rounded(ROUND, 2)),
    (BNodeClass::Filter, rounded(ROUND, 2)),
    (BNodeClass::Distort, rounded(ROUND, 2)),
    (BNodeClass::Matte, rounded(ROUND, 2)),
    (BNodeClass::Input, rounded(PILL, 1)),
    (BNodeClass::Interface, rounded(PILL, 1)),
    (BNodeClass::Attribute, hex(2)),
    (BNodeClass::Geometry, hex(2)),
    // A frame is a floor plate, not a solid: `tier` 0, and `layout.rs` gives
    // it `PLATE` of height rather than a tier.
    (BNodeClass::Frame, rounded(SOFT, 0)),
];

const PLAIN: BSolidSpec = rounded(SOFT, 1);

impl BSolidSpec {
    /// The solid a class is drawn as.
    pub fn of(class: BNodeClass) -> Self {
        BY_CLASS.iter().find(|(c, _)| *c == class).map_or(PLAIN, |(_, spec)| *spec)
    }

    /// The solid's height in view units, for a header of `header` units.
    pub fn height(&self, header: f32) -> f32 {
        if self.tier == 0 { header * PLATE } else { header * RELIEF * self.tier as f32 }
    }
}

/// How many straight segments one rounded corner is cut into.
pub const CORNER_SEGMENTS: usize = 4;

/// How many straight segments a link's tube is cut into.
pub const TUBE_SEGMENTS: usize = 16;

/// How far above its two ends a link's tube rises at its middle, as a
/// fraction of the card header — so a tube over a node it passes reads as over
/// it rather than through it.
pub const TUBE_LIFT: f32 = 0.5;

/// A tube's thickness in view units: Blender's `LINK_WIDTH` read as node
/// space, so a zoomed-in tube is as thick as the flat noodle it stands for.
pub const TUBE_WIDTH: f32 = super::super::paint::LINK_WIDTH;

/// The light the solids are lit by, in graph space — from the viewer's left
/// and front, mostly from above. Fixed in the world rather than to the camera
/// so turning the view turns the shading with it, which is what makes the
/// relief read as relief.
pub const LIGHT: [f32; 3] = [-0.40, -0.30, 0.87];

/// How dark a face turned fully away from the light goes, as a fraction of its
/// lit lightness; the rest is the lambert term.
pub const AMBIENT: f32 = 0.55;

/// The orbit the view opens on, in degrees: a little off straight-on so the
/// relief shows, and steep enough that the graph still reads left to right.
pub const OPENING_YAW: f32 = -24.0;
pub const OPENING_PITCH: f32 = 58.0;

/// The steepest and shallowest the camera may look: never straight along the
/// floor (nothing would be seen but edges) and never past the vertical.
pub const PITCH_MIN: f32 = 8.0;
pub const PITCH_MAX: f32 = 90.0;

/// Degrees of turn per canvas width of drag, for yaw and for pitch.
pub const TURN_PER_WIDTH: f32 = 180.0;

/// Below this share of the canvas's width a solid's top is too small for its
/// title — Flat's own `LABEL_FLOOR` (`node/set.rs`), the one measure of "how
/// small on screen" the two modes share. Blender stops drawing a label rather
/// than shrinking it to nothing.
pub const TITLE_FLOOR: f32 = 0.05;

/// A title's size as a fraction of the header's projected height, and the
/// pixel range it is held to: the text follows the perspective, but never
/// below legible nor above the flat header's own.
pub const TITLE_SCALE: f32 = 0.5;
pub const TITLE_PX: (f32, f32) = (7.0, 14.0);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_class_has_a_row_and_the_output_stands_tallest() {
        let tallest = BNodeClass::ALL.iter().map(|c| BSolidSpec::of(*c).tier).max().unwrap_or(0);
        assert_eq!(BSolidSpec::of(BNodeClass::Output).tier, tallest, "the graph rises toward its output");
        assert_eq!(BSolidSpec::of(BNodeClass::Frame).tier, 0, "a frame is a plate");
        assert_eq!(BSolidSpec::of(BNodeClass::Attribute).shape, BSolidShape::Hex, "the field class is the hex prism");
    }

    #[test]
    fn a_plate_is_thinner_than_any_tier() {
        let header = 30.0;
        let plate = BSolidSpec::of(BNodeClass::Frame).height(header);
        let lowest = BSolidSpec::of(BNodeClass::Input).height(header);
        assert!(plate > 0.0 && plate < lowest, "{plate} < {lowest}");
    }
}
