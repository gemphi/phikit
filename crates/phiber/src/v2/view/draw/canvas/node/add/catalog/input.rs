//! The Input submenu — `node_shader_rgb.cc`, `node_shader_value.cc`,
//! `node_shader_tex_coord.cc`.

use super::{BAddCategory, BNodeDecl, BShaderNode, SOCK_FLOAT, SOCK_RGBA, SOCK_VECTOR};

const INPUT: BAddCategory = BAddCategory::Input;

/// `ShaderNodeRGB` — a constant colour. Its `ui_name` is just "Color".
pub struct Rgb;
impl BShaderNode for Rgb {
    fn idname(&self) -> &'static str {
        "ShaderNodeRGB"
    }
    fn label(&self) -> &'static str {
        "Color"
    }
    fn category(&self) -> BAddCategory {
        INPUT
    }
    fn declare(&self, b: &mut BNodeDecl) {
        b.output("Color", SOCK_RGBA);
    }
}

pub struct Value;
impl BShaderNode for Value {
    fn idname(&self) -> &'static str {
        "ShaderNodeValue"
    }
    fn label(&self) -> &'static str {
        "Value"
    }
    fn category(&self) -> BAddCategory {
        INPUT
    }
    fn declare(&self, b: &mut BNodeDecl) {
        b.output("Value", SOCK_FLOAT);
    }
}

pub struct TextureCoordinate;
impl BShaderNode for TextureCoordinate {
    fn idname(&self) -> &'static str {
        "ShaderNodeTexCoord"
    }
    fn label(&self) -> &'static str {
        "Texture Coordinate"
    }
    fn category(&self) -> BAddCategory {
        INPUT
    }
    fn declare(&self, b: &mut BNodeDecl) {
        b.output("Generated", SOCK_VECTOR);
        b.output("Normal", SOCK_VECTOR);
        b.output("UV", SOCK_VECTOR);
        b.output("Object", SOCK_VECTOR);
        b.output("Camera", SOCK_VECTOR);
        b.output("Window", SOCK_VECTOR);
        b.output("Reflection", SOCK_VECTOR);
    }
}
