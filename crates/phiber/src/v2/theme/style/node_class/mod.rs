//! A node's header colour, by the class of node it is.
//!
//! phiber coloured a node header by looking for substrings in its `idname` and
//! reaching for whatever theme colour was nearest — the accent for anything with
//! "Output" in it, the status green for a shader, the X axis's red for a texture.
//! Those were three colours invented on the spot. Blender has **seventeen**, they
//! are theme fields with factory values, and they are the first thing anyone
//! reads off a graph.
//!
//! The mapping is `node_get_colorid` (`editors/space_node/node_draw.cc:1388`)
//! through `ui_theme_color_ptr` (`editors/interface/resources.cc:655`), and the
//! values are `userdef_default_theme.c`'s `.space_node`:
//!
//! | class | theme field | factory |
//! |---|---|---|
//! | (default, `TH_NODE`) | `syntaxl` | `0x303030` |
//! | `NODE_CLASS_INPUT` | `syntaxn` | `0x82354c` |
//! | `NODE_CLASS_OUTPUT` | `nodeclass_output` | `0x3e232a` |
//! | `NODE_CLASS_OP_COLOR` | `syntaxb` | `0x6e6e23` |
//! | `NODE_CLASS_OP_VECTOR` | `nodeclass_vector` | `0x3c3c83` |
//! | `NODE_CLASS_OP_FILTER` | `nodeclass_filter` | `0x412b51` |
//! | `NODE_CLASS_CONVERTER` | `syntaxv` | `0x246283` |
//! | `NODE_CLASS_GROUP` | `syntaxc` | `0x374725` |
//! | `NODE_CLASS_INTERFACE` | `console_output` | `0x1d1d1d` |
//! | `NODE_CLASS_MATTE` | `syntaxs` | `0x5a3838` |
//! | `NODE_CLASS_DISTORT` | `syntaxd` | `0x3e5a5b` |
//! | `NODE_CLASS_TEXTURE` | `nodeclass_texture` | `0x79461d` |
//! | `NODE_CLASS_SHADER` | `nodeclass_shader` | `0x2b652b` |
//! | `NODE_CLASS_SCRIPT` | `nodeclass_script` | `0x203c3c` |
//! | `NODE_CLASS_GEOMETRY` | `nodeclass_geometry` | `0x1d725e` |
//! | `NODE_CLASS_ATTRIBUTE` | `nodeclass_attribute` | `0x1d2546` |
//! | `NODE_CLASS_LAYOUT` (a frame) | `movie` | `0x0f0f0f` |
//!
//! **What is taken and what is not.** Blender's values are one dark theme's, and
//! `0x303030` on a light palette is a black bar. What carries the meaning is the
//! HUE — shader green, texture orange, vector blue, output maroon — not the
//! lightness, which is just a header against a dark background. So
//! [`class_color`] keeps Blender's hue and saturation and takes its lightness
//! from the palette's own header shade. One rule, seventeen hues, and a light
//! palette gets pale green shader headers rather than near-black ones. This is a
//! divergence and it is the deliberate kind: the socket colours next door are NOT
//! treated this way, because there the exact colour is the socket's identity,
//! while a header's exact lightness is the theme's business.
//!
//! **What is inferred.** `node_get_colorid` reads `node.typeinfo->nclass`, and
//! `typeinfo` is a registry pointer — nothing of it is in the .blend. The one
//! thing in the file that says what a node is, is its `idname`, which IS the
//! type's registered string (`ShaderNodeBsdfPrincipled`, `ShaderNodeTexImage`).
//! So [`class_of`] reads the idname. Every colour here is Blender's to the digit;
//! the CLASSIFICATION is a stand-in for the registry and will mis-sort an
//! unusual node. It is also the only thing a file reader can do, and it is
//! stated here so nobody mistakes it for ported fact.

use crate::v2::theme::palette::BColor;
use gpui::{Hsla, Rgba};

/// `NODE_DO_OUTPUT` (`DNA_node_types.h:211`).
pub const NODE_DO_OUTPUT: i32 = 1 << 6;

/// `eNodeClass`, as far as the header colour cares.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BNodeClass {
    /// `TH_NODE` — the shade a node with no class of its own takes, and also
    /// what an INACTIVE output node takes (see [`class_of`]).
    Plain,
    Input,
    Output,
    Color,
    Vector,
    Filter,
    Converter,
    Group,
    Interface,
    Matte,
    Distort,
    Texture,
    Shader,
    Script,
    Geometry,
    Attribute,
    Frame,
}

impl BNodeClass {
    /// The factory hex of the theme field this class reads.
    const fn hex(self) -> u32 {
        match self {
            Self::Plain => 0x303030,
            Self::Input => 0x82354c,
            Self::Output => 0x3e232a,
            Self::Color => 0x6e6e23,
            Self::Vector => 0x3c3c83,
            Self::Filter => 0x412b51,
            Self::Converter => 0x246283,
            Self::Group => 0x374725,
            Self::Interface => 0x1d1d1d,
            Self::Matte => 0x5a3838,
            Self::Distort => 0x3e5a5b,
            Self::Texture => 0x79461d,
            Self::Shader => 0x2b652b,
            Self::Script => 0x203c3c,
            Self::Geometry => 0x1d725e,
            Self::Attribute => 0x1d2546,
            Self::Frame => 0x0f0f0f,
        }
    }

    pub const ALL: &'static [Self] = &[
        Self::Plain,
        Self::Input,
        Self::Output,
        Self::Color,
        Self::Vector,
        Self::Filter,
        Self::Converter,
        Self::Group,
        Self::Interface,
        Self::Matte,
        Self::Distort,
        Self::Texture,
        Self::Shader,
        Self::Script,
        Self::Geometry,
        Self::Attribute,
        Self::Frame,
    ];
}

/// Blender's hue and saturation for `class`, at `header`'s lightness and alpha.
///
/// A class whose factory colour has no hue worth keeping — `Plain`, `Interface`
/// and `Frame` are all neutral greys in Blender — comes back as the header shade
/// itself, which is the same statement in phiber's terms: no class, no tint.
pub fn class_color(class: BNodeClass, header: BColor) -> BColor {
    let hex = class.hex();
    let channel = |shift: u32| ((hex >> shift) & 0xff) as f32 / 255.0;
    let hsla: Hsla = Rgba { r: channel(16), g: channel(8), b: channel(0), a: 1.0 }.into();
    let saturation = hsla.s * 100.0;
    // Blender's neutral classes carry no hue; tinting by a rounding error's
    // worth of saturation would be inventing one.
    if saturation < 1.0 {
        return header;
    }
    (hsla.h * 360.0, saturation, header.2, header.3)
}

/// Which class a node is — the inferred half, kept in its own file so the
/// boundary between ported fact and inference is a file boundary.
pub mod infer;

pub use infer::class_of;

#[cfg(test)]
mod tests;
