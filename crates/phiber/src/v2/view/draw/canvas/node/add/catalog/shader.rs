//! The Shader submenu — `node_shader_bsdf_diffuse.cc`, `node_shader_emission.cc`,
//! `node_shader_background.cc`, `node_shader_bsdf_transparent.cc`,
//! `node_shader_mix_shader.cc`, `node_shader_add_shader.cc`.
//!
//! Principled BSDF is large enough to keep in its own file (`principled.rs`).
//! The gpu-internal `Weight` input each of these declares is left off — it is
//! `.available(is_gpu_internal)`, never drawn in the editor.

use super::{BAddCategory, BNodeDecl, BShaderNode, SOCK_FLOAT, SOCK_RGBA, SOCK_SHADER, SOCK_VECTOR};

const SHADER: BAddCategory = BAddCategory::Shader;

pub struct DiffuseBsdf;
impl BShaderNode for DiffuseBsdf {
    fn idname(&self) -> &'static str {
        "ShaderNodeBsdfDiffuse"
    }
    fn label(&self) -> &'static str {
        "Diffuse BSDF"
    }
    fn category(&self) -> BAddCategory {
        SHADER
    }
    fn declare(&self, b: &mut BNodeDecl) {
        b.input("Color", SOCK_RGBA);
        b.input("Roughness", SOCK_FLOAT);
        b.input("Normal", SOCK_VECTOR);
        b.output("BSDF", SOCK_SHADER);
    }
}

pub struct Emission;
impl BShaderNode for Emission {
    fn idname(&self) -> &'static str {
        "ShaderNodeEmission"
    }
    fn label(&self) -> &'static str {
        "Emission"
    }
    fn category(&self) -> BAddCategory {
        SHADER
    }
    fn declare(&self, b: &mut BNodeDecl) {
        b.input("Color", SOCK_RGBA);
        b.input("Strength", SOCK_FLOAT);
        b.output("Emission", SOCK_SHADER);
    }
}

pub struct Background;
impl BShaderNode for Background {
    fn idname(&self) -> &'static str {
        "ShaderNodeBackground"
    }
    fn label(&self) -> &'static str {
        "Background"
    }
    fn category(&self) -> BAddCategory {
        SHADER
    }
    fn declare(&self, b: &mut BNodeDecl) {
        b.input("Color", SOCK_RGBA);
        b.input("Strength", SOCK_FLOAT);
        b.output("Background", SOCK_SHADER);
    }
}

pub struct TransparentBsdf;
impl BShaderNode for TransparentBsdf {
    fn idname(&self) -> &'static str {
        "ShaderNodeBsdfTransparent"
    }
    fn label(&self) -> &'static str {
        "Transparent BSDF"
    }
    fn category(&self) -> BAddCategory {
        SHADER
    }
    fn declare(&self, b: &mut BNodeDecl) {
        b.input("Color", SOCK_RGBA);
        b.output("BSDF", SOCK_SHADER);
    }
}

pub struct MixShader;
impl BShaderNode for MixShader {
    fn idname(&self) -> &'static str {
        "ShaderNodeMixShader"
    }
    fn label(&self) -> &'static str {
        "Mix Shader"
    }
    fn category(&self) -> BAddCategory {
        SHADER
    }
    fn declare(&self, b: &mut BNodeDecl) {
        // Two inputs named "Shader": the file tells them apart by identifier,
        // which Blender makes unique (`Shader`, `Shader_001`). phiber draws by
        // name and a two-node graph never links both, so the shared name stands.
        b.input("Factor", SOCK_FLOAT);
        b.input("Shader", SOCK_SHADER);
        b.input("Shader", SOCK_SHADER);
        b.output("Shader", SOCK_SHADER);
    }
}

pub struct AddShader;
impl BShaderNode for AddShader {
    fn idname(&self) -> &'static str {
        "ShaderNodeAddShader"
    }
    fn label(&self) -> &'static str {
        "Add Shader"
    }
    fn category(&self) -> BAddCategory {
        SHADER
    }
    fn declare(&self, b: &mut BNodeDecl) {
        b.input("Shader", SOCK_SHADER);
        b.input("Shader", SOCK_SHADER);
        b.output("Shader", SOCK_SHADER);
    }
}
