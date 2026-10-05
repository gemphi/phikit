//! The Color submenu — `node_shader_mix_rgb.cc`, `node_shader_invert.cc`,
//! `node_shader_gamma.cc`, `node_shader_brightness.cc`.

use super::{BAddCategory, BNodeDecl, BShaderNode, SOCK_FLOAT, SOCK_RGBA};

const COLOR: BAddCategory = BAddCategory::Color;

/// `ShaderNodeMixRGB` — the legacy mix; its `ui_name` is "Mix (Legacy)" now
/// that `ShaderNodeMix` is the one the menu points at.
pub struct MixLegacy;
impl BShaderNode for MixLegacy {
    fn idname(&self) -> &'static str {
        "ShaderNodeMixRGB"
    }
    fn label(&self) -> &'static str {
        "Mix (Legacy)"
    }
    fn category(&self) -> BAddCategory {
        COLOR
    }
    fn declare(&self, b: &mut BNodeDecl) {
        b.input("Factor", SOCK_FLOAT);
        b.input("Color1", SOCK_RGBA);
        b.input("Color2", SOCK_RGBA);
        b.output("Color", SOCK_RGBA);
    }
}

pub struct InvertColor;
impl BShaderNode for InvertColor {
    fn idname(&self) -> &'static str {
        "ShaderNodeInvert"
    }
    fn label(&self) -> &'static str {
        "Invert Color"
    }
    fn category(&self) -> BAddCategory {
        COLOR
    }
    fn declare(&self, b: &mut BNodeDecl) {
        b.input("Factor", SOCK_FLOAT);
        b.input("Color", SOCK_RGBA);
        b.output("Color", SOCK_RGBA);
    }
}

pub struct Gamma;
impl BShaderNode for Gamma {
    fn idname(&self) -> &'static str {
        "ShaderNodeGamma"
    }
    fn label(&self) -> &'static str {
        "Gamma"
    }
    fn category(&self) -> BAddCategory {
        COLOR
    }
    fn declare(&self, b: &mut BNodeDecl) {
        b.input("Color", SOCK_RGBA);
        b.input("Gamma", SOCK_FLOAT);
        b.output("Color", SOCK_RGBA);
    }
}

/// `ShaderNodeBrightContrast` — `ui_name` "Brightness/Contrast". Its first input
/// is labelled "Brightness" but its identifier is "Bright" — the one shader node
/// in the seed whose socket name and link identifier differ.
pub struct BrightContrast;
impl BShaderNode for BrightContrast {
    fn idname(&self) -> &'static str {
        "ShaderNodeBrightContrast"
    }
    fn label(&self) -> &'static str {
        "Brightness/Contrast"
    }
    fn category(&self) -> BAddCategory {
        COLOR
    }
    fn declare(&self, b: &mut BNodeDecl) {
        b.input("Color", SOCK_RGBA);
        b.input_id("Brightness", "Bright", SOCK_FLOAT);
        b.input("Contrast", SOCK_FLOAT);
        b.output("Color", SOCK_RGBA);
    }
}
