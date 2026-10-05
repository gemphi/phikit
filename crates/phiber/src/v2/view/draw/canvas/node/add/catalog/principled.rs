//! Principled BSDF — `node_shader_bsdf_principled.cc`, in its own file for its
//! size. Sockets in the source's own order (the output is declared first there
//! too); the gpu-internal `Weight` input is omitted, as it is on every BSDF.
//!
//! Blender groups most of these inputs into collapsible panels (Subsurface,
//! Specular, Coat, Sheen, Emission, Thin Film). phiber has no node panels yet,
//! so they are declared as a flat list — the sockets and their order are the
//! same, only the panel folds are missing.

use super::{BAddCategory, BNodeDecl, BShaderNode, SOCK_BOOLEAN, SOCK_FLOAT, SOCK_RGBA, SOCK_SHADER, SOCK_VECTOR};

pub struct PrincipledBsdf;
impl BShaderNode for PrincipledBsdf {
    fn idname(&self) -> &'static str {
        "ShaderNodeBsdfPrincipled"
    }
    fn label(&self) -> &'static str {
        "Principled BSDF"
    }
    fn category(&self) -> BAddCategory {
        BAddCategory::Shader
    }
    fn declare(&self, b: &mut BNodeDecl) {
        b.output("BSDF", SOCK_SHADER);
        b.input("Base Color", SOCK_RGBA);
        b.input("Metallic", SOCK_FLOAT);
        b.input("Roughness", SOCK_FLOAT);
        b.input("IOR", SOCK_FLOAT);
        b.input("Alpha", SOCK_FLOAT);
        b.input("Thin Wall", SOCK_BOOLEAN);
        b.input("Normal", SOCK_VECTOR);
        b.input("Diffuse Roughness", SOCK_FLOAT);
        b.input("Subsurface Weight", SOCK_FLOAT);
        b.input("Subsurface Radius", SOCK_VECTOR);
        b.input("Subsurface Scale", SOCK_FLOAT);
        b.input("Subsurface IOR", SOCK_FLOAT);
        b.input("Subsurface Anisotropy", SOCK_FLOAT);
        b.input("Specular IOR Level", SOCK_FLOAT);
        b.input("Specular Tint", SOCK_RGBA);
        b.input("Anisotropic", SOCK_FLOAT);
        b.input("Anisotropic Rotation", SOCK_FLOAT);
        b.input("Tangent", SOCK_VECTOR);
        b.input("Transmission Weight", SOCK_FLOAT);
        b.input("Coat Weight", SOCK_FLOAT);
        b.input("Coat Roughness", SOCK_FLOAT);
        b.input("Coat IOR", SOCK_FLOAT);
        b.input("Coat Tint", SOCK_RGBA);
        b.input("Coat Normal", SOCK_VECTOR);
        b.input("Sheen Weight", SOCK_FLOAT);
        b.input("Sheen Roughness", SOCK_FLOAT);
        b.input("Sheen Tint", SOCK_RGBA);
        b.input("Emission Color", SOCK_RGBA);
        b.input("Emission Strength", SOCK_FLOAT);
        b.input("Thin Film Thickness", SOCK_FLOAT);
        b.input("Thin Film IOR", SOCK_FLOAT);
    }
}
