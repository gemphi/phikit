//! The Converter submenu — `node_shader_math.cc`, `node_shader_color_ramp.cc`.

use super::{BAddCategory, BNodeDecl, BShaderNode, SOCK_FLOAT, SOCK_RGBA};

const CONVERTER: BAddCategory = BAddCategory::Converter;

/// `ShaderNodeMath` — three value inputs (the third is used only by the ternary
/// operations, but it is declared for all) and one value output.
pub struct Math;
impl BShaderNode for Math {
    fn idname(&self) -> &'static str {
        "ShaderNodeMath"
    }
    fn label(&self) -> &'static str {
        "Math"
    }
    fn category(&self) -> BAddCategory {
        CONVERTER
    }
    fn declare(&self, b: &mut BNodeDecl) {
        b.input("Value", SOCK_FLOAT);
        b.input("Value", SOCK_FLOAT);
        b.input("Value", SOCK_FLOAT);
        b.output("Value", SOCK_FLOAT);
    }
}

/// `ShaderNodeValToRGB` — `ui_name` "Color Ramp".
pub struct ColorRamp;
impl BShaderNode for ColorRamp {
    fn idname(&self) -> &'static str {
        "ShaderNodeValToRGB"
    }
    fn label(&self) -> &'static str {
        "Color Ramp"
    }
    fn category(&self) -> BAddCategory {
        CONVERTER
    }
    fn declare(&self, b: &mut BNodeDecl) {
        b.input("Factor", SOCK_FLOAT);
        b.output("Color", SOCK_RGBA);
        b.output("Alpha", SOCK_FLOAT);
    }
}
