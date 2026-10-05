//! The Output submenu — `node_shader_output_material.cc`.

use super::{BAddCategory, BNodeDecl, BShaderNode, SOCK_FLOAT, SOCK_SHADER, SOCK_VECTOR};

/// `ShaderNodeOutputMaterial` — the sink of a shader graph.
pub struct MaterialOutput;
impl BShaderNode for MaterialOutput {
    fn idname(&self) -> &'static str {
        "ShaderNodeOutputMaterial"
    }
    fn label(&self) -> &'static str {
        "Material Output"
    }
    fn category(&self) -> BAddCategory {
        BAddCategory::Output
    }
    fn declare(&self, b: &mut BNodeDecl) {
        // Thickness is `.hide_value()`, not unavailable, so it is still a drawn
        // socket. There is no output — an output node is where the graph ends.
        b.input("Surface", SOCK_SHADER);
        b.input("Volume", SOCK_SHADER);
        b.input("Displacement", SOCK_VECTOR);
        b.input("Thickness", SOCK_FLOAT);
    }
}
