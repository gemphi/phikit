//! The eleven palettes, each in two modes. Every one is `BPalette::on` —
//! the same eighteen slots on its own ladder — with the accent that means
//! "selected" in it; the light mode is the ladder the other way up on the
//! theme's light seed (`shade::*_LIGHT`), with the same accent unless the
//! theme has a light one of its own.
//!
//! Accents are (hue, saturation, lightness). Sources: Sphere's `#478AD1`
//! and Blender's orange; Tokyo Night's `#7AA2F7`; the concept car's magenta
//! for Onyx; Boldland's orange for Grey and Tesla's UI blue for its light
//! mode; Apple's system blue (`#0A84FF` dark, `#007AFF` light); the `phi`
//! appkit's variants (`appkit/src/theme/variants/*.rs`) for Swift, Barbie,
//! Fenty, Cyberpunk and Chanel.

use super::super::BPaletteName;
use super::super::set::{BAccents, BLENDER_MEANINGS, BPalette};
use super::super::shade;

/// The sphere's ladder with phiber's selection blue, `#478AD1`.
pub static SPHERE: BPalette = BPalette::on(BPaletteName::Sphere, shade::SPHERE, BLENDER_MEANINGS);
pub static SPHERE_LIGHT: BPalette = BPalette::on(BPaletteName::Sphere, shade::SPHERE_LIGHT, BLENDER_MEANINGS);

/// Blender's grey ladder with Blender's selection orange.
const GRAPHITE_ACCENTS: BAccents = BAccents { accent: (28.0, 95.0, 54.0, 1.0), online: (142.0, 70.0, 45.0, 1.0), ..BLENDER_MEANINGS };
pub static GRAPHITE: BPalette = BPalette::on(BPaletteName::Graphite, shade::GRAPHITE, GRAPHITE_ACCENTS);
pub static GRAPHITE_LIGHT: BPalette = BPalette::on(BPaletteName::Graphite, shade::GRAPHITE_LIGHT, GRAPHITE_ACCENTS);

/// Tokyo: Tokyo Night's ladder with its blue, `#7AA2F7`.
const TOKYO_ACCENTS: BAccents = BAccents { accent: (220.8, 88.7, 72.4, 1.0), online: (145.0, 61.0, 60.0, 1.0), axis_z: (217.0, 100.0, 62.0, 1.0), ..BLENDER_MEANINGS };
pub static TOKYO: BPalette = BPalette::on(BPaletteName::Tokyo, shade::TOKYO, TOKYO_ACCENTS);
pub static TOKYO_LIGHT: BPalette = BPalette::on(BPaletteName::Tokyo, shade::TOKYO_LIGHT, TOKYO_ACCENTS);

/// Black: a near-black ladder with the concept car's magenta, `#D6247A`.
pub static ONYX: BPalette = BPalette::accented(BPaletteName::Onyx, shade::ONYX, (331.0, 71.2, 49.0));
pub static ONYX_LIGHT: BPalette = BPalette::accented(BPaletteName::Onyx, shade::ONYX_LIGHT, (331.0, 71.2, 49.0));

/// Grey: a mid-grey ladder with Boldland's orange, `#E8912B`; its light
/// mode is the silver ladder with Tesla's UI blue, `#3E6AE1`.
pub static GREY: BPalette = BPalette::accented(BPaletteName::Grey, shade::GREY, (32.4, 80.4, 53.9));
pub static GREY_LIGHT: BPalette = BPalette::accented(BPaletteName::Grey, shade::GREY_LIGHT, (223.8, 73.1, 56.3));

/// Apple: macOS's dark window grey with its dark-mode blue, `#0A84FF`; its
/// light mode the light window grey with the system blue, `#007AFF`.
pub static APPLE: BPalette = BPalette::accented(BPaletteName::Apple, shade::APPLE, (210.1, 100.0, 52.0));
pub static APPLE_LIGHT: BPalette = BPalette::accented(BPaletteName::Apple, shade::APPLE_LIGHT, (211.3, 100.0, 50.0));

/// Swift: a warm dark ladder with Swift's orange, `#FF5933`.
pub static SWIFT: BPalette = BPalette::accented(BPaletteName::Swift, shade::SWIFT, (11.2, 100.0, 60.0));
pub static SWIFT_LIGHT: BPalette = BPalette::accented(BPaletteName::Swift, shade::SWIFT_LIGHT, (11.2, 100.0, 60.0));

/// Barbie: a pink-tinted dark ladder with hot pink, `#ED1C8C`.
pub static BARBIE: BPalette = BPalette::accented(BPaletteName::Barbie, shade::BARBIE, (327.8, 85.3, 52.0));
pub static BARBIE_LIGHT: BPalette = BPalette::accented(BPaletteName::Barbie, shade::BARBIE_LIGHT, (327.8, 85.3, 52.0));

/// Fenty: an earth ladder with bronze, `#D18C59`.
pub static FENTY: BPalette = BPalette::accented(BPaletteName::Fenty, shade::FENTY, (25.5, 56.6, 58.4));
pub static FENTY_LIGHT: BPalette = BPalette::accented(BPaletteName::Fenty, shade::FENTY_LIGHT, (25.5, 56.6, 58.4));

/// Cyberpunk: a purple-black ladder with neon cyan, `#00FFE6`.
pub static CYBERPUNK: BPalette = BPalette::accented(BPaletteName::Cyberpunk, shade::CYBERPUNK, (174.1, 100.0, 50.0));
pub static CYBERPUNK_LIGHT: BPalette = BPalette::accented(BPaletteName::Cyberpunk, shade::CYBERPUNK_LIGHT, (174.1, 100.0, 50.0));

/// Chanel: a mushroom-forest ladder with champagne, `#CCB88C`.
pub static CHANEL: BPalette = BPalette::accented(BPaletteName::Chanel, shade::CHANEL, (41.2, 38.6, 67.5));
pub static CHANEL_LIGHT: BPalette = BPalette::accented(BPaletteName::Chanel, shade::CHANEL_LIGHT, (41.2, 38.6, 67.5));
