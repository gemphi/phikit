//! Which compositor node a `bNode` is, and the settings it carries.
//!
//! Blender's compositor is one node tree per scene over the render's pixels.
//! Every node in this file is **per-pixel**: its output at a pixel depends on
//! its inputs at that same pixel and nothing else, which is what lets the
//! whole tree run as a chain of buffer passes (`eval.rs`) rather than as a
//! tiled scheduler. The nodes that read a *neighbourhood* — Blur, Glare,
//! Defocus, Denoise, Dilate/Erode, Kuwahara — and the ones that move pixels
//! about — Translate, Scale, Rotate, Transform, Crop, Flip, Lens Distortion —
//! are [`BCompKind::Declined`]: named, counted and reported, never guessed at.
//!
//! Ported from Blender 5.2.0, `source/blender/nodes/composite/nodes/`. Each
//! node there carries a `node_build_multi_function` that is its CPU
//! implementation; those lambdas are what `pixel.rs` holds.

use crate::blend::shade::graph::BlendNode;
use crate::blend::shade::socket::BSocketDefault;

/// A compositor node, by what it does to a pixel.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BCompKind {
    /// `CompositorNodeRLayers` — the render itself, this tree's input.
    RenderLayer,
    /// `CompositorNodeImage` — a file on disk. Read as the tree's input too:
    /// one image in, one image out is what a viewport composite is.
    Image,
    /// `CompositorNodeRGB` / `CompositorNodeValue` — a constant.
    Constant,
    /// `CompositorNodeComposite` — the tree's result.
    Composite,
    /// `CompositorNodeViewer` — the Image editor's Viewer Node pass.
    Viewer,
    /// `NodeGroupOutput` — what 5.x ends a compositing tree with. There is
    /// no Composite node any more: the scene points at a node *group* and
    /// the group's output socket is the result.
    GroupOutput,
    /// `CompositorNodeBrightContrast`.
    BrightContrast,
    /// `CompositorNodeExposure`.
    Exposure,
    /// `ShaderNodeGamma` — shared with the shader trees since 5.x.
    Gamma,
    /// `CompositorNodeInvert`.
    Invert,
    /// `CompositorNodeHueSat`.
    HueSat,
    /// `CompositorNodePosterize`.
    Posterize,
    /// `CompositorNodeRGBToBW`.
    RgbToBw,
    /// `CompositorNodeSetAlpha`.
    SetAlpha,
    /// `CompositorNodePremulKey` — premultiply / unpremultiply.
    AlphaConvert,
    /// `CompositorNodeAlphaOver`.
    AlphaOver,
    /// `CompositorNodeMixRGB` (4.x) / `ShaderNodeMix` (5.x).
    Mix,
    /// `NodeReroute` — passes its input through.
    Reroute,
    /// A node this pass does not run: it reads more than its own pixel, or
    /// moves pixels, or needs a datablock the reader does not carry.
    #[default]
    Declined,
}

impl BCompKind {
    /// The kind for a node's `idname`.
    pub fn of(idname: &str) -> Self {
        match idname {
            "CompositorNodeRLayers" => Self::RenderLayer,
            "CompositorNodeImage" => Self::Image,
            "CompositorNodeRGB" | "CompositorNodeValue" => Self::Constant,
            "CompositorNodeComposite" => Self::Composite,
            "CompositorNodeViewer" => Self::Viewer,
            "NodeGroupOutput" => Self::GroupOutput,
            "CompositorNodeBrightContrast" => Self::BrightContrast,
            "CompositorNodeExposure" => Self::Exposure,
            "CompositorNodeGamma" | "ShaderNodeGamma" => Self::Gamma,
            "CompositorNodeInvert" => Self::Invert,
            "CompositorNodeHueSat" => Self::HueSat,
            "CompositorNodePosterize" => Self::Posterize,
            "CompositorNodeRGBToBW" => Self::RgbToBw,
            "CompositorNodeSetAlpha" => Self::SetAlpha,
            "CompositorNodePremulKey" => Self::AlphaConvert,
            "CompositorNodeAlphaOver" => Self::AlphaOver,
            "CompositorNodeMixRGB" | "ShaderNodeMix" => Self::Mix,
            "NodeReroute" => Self::Reroute,
            _ => Self::Declined,
        }
    }

    /// Whether this pass runs the node at all.
    pub fn runs(self) -> bool {
        self != Self::Declined
    }

    /// Whether the node ends a tree: the colour it is handed is the result.
    pub fn output(self) -> bool {
        matches!(self, Self::Composite | Self::Viewer | Self::GroupOutput)
    }

    /// Whether the node starts one: it takes the render's pixel rather than
    /// another node's.
    pub fn source(self) -> bool {
        matches!(self, Self::RenderLayer | Self::Image)
    }
}

/// What a node reads off its own sockets, whatever the file's version
/// spells those as.
///
/// A socket is matched by **identifier** first and then by name, and each
/// lookup takes a *list* of spellings, because 5.x renames sockets freely:
/// a compositor Mix's `Image` / `Image_001` / `Fac` became
/// `ShaderNodeMix`'s `A_Color` / `B_Color` / `Factor_Float`, and a group
/// output's socket is `Socket_1` with the name `Image`. A reader that takes
/// one spelling reads the node's *default* instead of the file's value and
/// looks like a maths bug.
pub trait BCompSetting {
    /// The first of `id` that the node has, as a number: a float socket's
    /// own value, a boolean's 0 or 1, a menu's enum, or a colour's first
    /// channel.
    fn number(&self, id: &[&str]) -> Option<f32>;
    /// The same as a colour, a float broadcast to RGB with alpha 1.
    fn color(&self, id: &[&str]) -> Option<[f32; 4]>;
    /// The value, else `fallback`.
    fn number_or(&self, id: &[&str], fallback: f32) -> f32 {
        self.number(id).unwrap_or(fallback)
    }
}

/// The first available input socket under any of `id`.
fn input_of<'a>(node: &'a BlendNode, id: &[&str]) -> Option<&'a crate::blend::shade::socket::BlendSocket> {
    id.iter()
        .find_map(|want| node.inputs.iter().find(|s| s.available() && s.identifier == *want))
        .or_else(|| id.iter().find_map(|want| node.inputs.iter().find(|s| s.available() && s.name == *want)))
}

impl BCompSetting for BlendNode {
    fn number(&self, id: &[&str]) -> Option<f32> {
        Some(match input_of(self, id)?.default.as_ref()? {
            BSocketDefault::Float(v) => *v,
            BSocketDefault::Bool(v) => f32::from(u8::from(*v)),
            BSocketDefault::Int(v) => *v as f32,
            BSocketDefault::Color(c) => c[0],
            BSocketDefault::Vector(v) => v[0],
        })
    }

    fn color(&self, id: &[&str]) -> Option<[f32; 4]> {
        Some(match input_of(self, id)?.default.as_ref()? {
            BSocketDefault::Color(c) => *c,
            BSocketDefault::Vector(v) => [v[0], v[1], v[2], 1.0],
            BSocketDefault::Float(v) => [*v, *v, *v, 1.0],
            BSocketDefault::Bool(v) => {
                let c = f32::from(u8::from(*v));
                [c, c, c, 1.0]
            }
            BSocketDefault::Int(v) => [*v as f32, *v as f32, *v as f32, 1.0],
        })
    }
}

/// The socket spellings each input goes by, across versions.
pub mod sock {
    /// A node's one image input. `Color` is Invert's and Gamma's; `A_Color`
    /// is `ShaderNodeMix`'s first.
    pub const IMAGE: &[&str] = &["Image", "Color", "A_Color"];
    /// A second image: Mix's and Alpha Over's.
    pub const IMAGE_2: &[&str] = &["Image_001", "B_Color", "Color2"];
    /// A blend factor.
    pub const FAC: &[&str] = &["Fac", "Factor", "Factor_Float"];
    pub const BRIGHT: &[&str] = &["Bright", "Brightness"];
    pub const CONTRAST: &[&str] = &["Contrast"];
    pub const EXPOSURE: &[&str] = &["Exposure"];
    pub const GAMMA: &[&str] = &["Gamma"];
    pub const HUE: &[&str] = &["Hue"];
    pub const SATURATION: &[&str] = &["Saturation"];
    pub const VALUE: &[&str] = &["Value"];
    pub const STEPS: &[&str] = &["Steps"];
    pub const ALPHA: &[&str] = &["Alpha"];
    pub const STRAIGHT: &[&str] = &["Straight Alpha", "Convert Premultiplied"];
    pub const CLAMP: &[&str] = &["Clamp Result"];
    pub const INVERT_COLOR: &[&str] = &["Invert Color"];
    pub const INVERT_ALPHA: &[&str] = &["Invert Alpha"];
    pub const SET_ALPHA_TYPE: &[&str] = &["Type"];
    pub const ALPHA_MAPPING: &[&str] = &["Mapping"];
    pub const OVER_OPERATION: &[&str] = &["Operation"];
}

/// A node's settings that are not on a socket: what `read.rs` digs out of
/// `bNode.storage` while it still has the file and the DNA.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BCompSet {
    /// `MA_RAMP_*`, for Mix.
    pub blend: i32,
    /// Mix's Clamp Result.
    pub clamp: bool,
}
