//! A node socket's colour, by the type of data that flows through it.
//!
//! Ported verbatim from Blender 5.2.0, `editors/space_node/drawnode.cc:987`,
//! `std_node_socket_colors` — one RGBA per `eNodeSocketDatatype`, indexed by the
//! type itself (`std_node_socket_colors_get(socket_type, r_color)`).
//!
//! These are the one set of colours in the node editor that is NOT themed, in
//! Blender or here, and the reason is that a socket's colour is the only thing
//! naming its type on the card. Yellow IS colour, grey IS a number, green IS a
//! shader; people read a graph by them. Re-deriving them per palette would make
//! the sixteen palettes disagree about what a shader socket looks like, which
//! costs the reader the one cue the shape gives them for free. So they sit here
//! as Blender's own floats, with the citation, rather than on the ladder.
//!
//! The table is kept in RGB because that is the form Blender states it in and
//! the form it can be checked against; [`socket_color`] converts to the theme's
//! HSLA `BColor` through the same GPUI conversion the hex parser uses, so there
//! is one conversion in the crate and not two.

use crate::v2::theme::palette::BColor;
use gpui::{Hsla, Rgba};

/// `std_node_socket_colors`, index = `eNodeSocketDatatype`.
///
/// Index 5 is Blender's own `UNUSED` hole — there is no socket type 5 — and it
/// is kept so every other index lines up with the enum. Its alpha is zero in
/// Blender too, so a file that somehow carried type 5 would draw nothing rather
/// than a wrong colour.
const SOCKET_COLORS: [[f32; 4]; 25] = [
    [0.63, 0.63, 0.63, 1.0], // SOCK_FLOAT
    [0.39, 0.39, 0.78, 1.0], // SOCK_VECTOR
    [0.78, 0.78, 0.16, 1.0], // SOCK_RGBA
    [0.39, 0.78, 0.39, 1.0], // SOCK_SHADER
    [0.80, 0.65, 0.84, 1.0], // SOCK_BOOLEAN
    [0.00, 0.00, 0.00, 0.0], // UNUSED
    [0.35, 0.55, 0.36, 1.0], // SOCK_INT
    [0.44, 0.70, 1.00, 1.0], // SOCK_STRING
    [0.93, 0.62, 0.36, 1.0], // SOCK_OBJECT
    [0.39, 0.22, 0.39, 1.0], // SOCK_IMAGE
    [0.00, 0.84, 0.64, 1.0], // SOCK_GEOMETRY
    [0.96, 0.96, 0.96, 1.0], // SOCK_COLLECTION
    [0.62, 0.31, 0.64, 1.0], // SOCK_TEXTURE
    [0.92, 0.46, 0.51, 1.0], // SOCK_MATERIAL
    [0.65, 0.39, 0.78, 1.0], // SOCK_ROTATION
    [0.40, 0.40, 0.40, 1.0], // SOCK_MENU
    [0.72, 0.20, 0.52, 1.0], // SOCK_MATRIX
    [0.30, 0.50, 0.50, 1.0], // SOCK_BUNDLE
    [0.49, 0.49, 0.23, 1.0], // SOCK_CLOSURE
    [0.39, 0.34, 0.26, 1.0], // SOCK_FONT
    [0.00, 0.00, 0.00, 1.0], // SOCK_SCENE
    [0.00, 0.00, 0.00, 1.0], // SOCK_TEXT_ID
    [0.00, 0.00, 0.00, 1.0], // SOCK_MASK
    [0.39, 0.34, 0.26, 1.0], // SOCK_SOUND
    [0.36, 0.47, 0.61, 1.0], // SOCK_INT_VECTOR
];

/// `virtual_node_socket_color` (`drawnode.cc:984`) — what a socket with no type
/// is drawn in. `SOCK_CUSTOM` is `-1`, so it has no row in the table, and a type
/// from a newer file than this table knows is in the same position.
const VIRTUAL: [f32; 4] = [0.2, 0.2, 0.2, 1.0];

/// The colour of a socket carrying `kind`, an `eNodeSocketDatatype`.
///
/// Out-of-range is not an error here. Blender asserts the index is in range
/// because its own enum and its own table are compiled together; a FILE can
/// carry a type this table has never heard of (one added after 5.2.0, or
/// `SOCK_CUSTOM` from a plugin's node), and the honest answer then is the
/// virtual-socket grey Blender already has for exactly that case.
pub fn socket_color(kind: i32) -> BColor {
    let rgba = usize::try_from(kind)
        .ok()
        .and_then(|i| SOCKET_COLORS.get(i))
        .copied()
        .unwrap_or(VIRTUAL);
    let [r, g, b, a] = rgba;
    let hsla: Hsla = Rgba { r, g, b, a }.into();
    (hsla.h * 360.0, hsla.s * 100.0, hsla.l * 100.0, a)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `eNodeSocketDatatype`, for the types a shader graph actually contains.
    const FLOAT: i32 = 0;
    const VECTOR: i32 = 1;
    const RGBA: i32 = 2;
    const SHADER: i32 = 3;
    const CUSTOM: i32 = -1;

    /// Back to RGB, so a ported float can be compared against Blender's own.
    fn rgb(color: BColor) -> [f32; 3] {
        let rgba: Rgba =
            Hsla { h: color.0 / 360.0, s: color.1 / 100.0, l: color.2 / 100.0, a: color.3 }.into();
        [rgba.r, rgba.g, rgba.b]
    }

    #[test]
    fn a_sockets_colour_is_blenders_own_float_for_its_type() {
        // The round trip through HSLA must not move the colour: these are the
        // numbers in `std_node_socket_colors`, and a socket people identify by
        // its colour cannot drift.
        for (kind, expected) in [
            (FLOAT, [0.63, 0.63, 0.63]),
            (VECTOR, [0.39, 0.39, 0.78]),
            (RGBA, [0.78, 0.78, 0.16]),
            (SHADER, [0.39, 0.78, 0.39]),
        ] {
            let ours = rgb(socket_color(kind));
            for c in 0..3 {
                assert!(
                    (ours[c] - expected[c]).abs() < 1e-3,
                    "socket type {kind}: {ours:?} against Blender's {expected:?}"
                );
            }
        }
    }

    #[test]
    fn the_shader_socket_is_green_and_the_colour_socket_is_yellow() {
        // Stated as hues rather than channels, because this is the fact a
        // reader of a graph actually uses, and a table shifted by one index
        // would still pass a channel-by-channel check on a shuffled table.
        let hue = |kind| socket_color(kind).0;
        assert!((90.0..150.0).contains(&hue(SHADER)), "shader hue {}", hue(SHADER));
        assert!((45.0..75.0).contains(&hue(RGBA)), "colour hue {}", hue(RGBA));
        assert!((210.0..270.0).contains(&hue(VECTOR)), "vector hue {}", hue(VECTOR));
        // A float is grey, so it has no hue worth asserting — it has no
        // saturation, which is the claim.
        assert!(socket_color(FLOAT).1 < 1.0, "a float socket is grey");
    }

    #[test]
    fn an_unknown_type_falls_back_to_the_virtual_socket_rather_than_panicking() {
        // `SOCK_CUSTOM` is -1, and a file newer than this table carries types
        // past its end. Blender asserts; a reader of other people's files
        // cannot afford to.
        for kind in [CUSTOM, -99, 25, 1000] {
            let ours = rgb(socket_color(kind));
            for c in 0..3 {
                assert!((ours[c] - 0.2).abs() < 1e-3, "type {kind} → {ours:?}, not the virtual grey");
            }
        }
    }

    #[test]
    fn blenders_unused_hole_keeps_every_other_index_aligned() {
        // Type 5 does not exist; the row is kept so SOCK_INT stays at 6. If the
        // hole were dropped, every type from 6 up would take its neighbour's
        // colour — which is exactly the kind of off-by-one a colour table hides.
        assert_eq!(SOCKET_COLORS.len(), 25);
        assert_eq!(SOCKET_COLORS[5][3], 0.0, "the hole draws nothing");
        assert_eq!(socket_color(5).3, 0.0);
        let int = rgb(socket_color(6));
        assert!((int[1] - 0.55).abs() < 1e-3, "SOCK_INT is at index 6: {int:?}");
    }
}
