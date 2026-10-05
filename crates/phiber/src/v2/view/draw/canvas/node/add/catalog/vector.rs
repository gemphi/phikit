//! The Vector submenu — `node_shader_mapping.cc`, `node_shader_normal_map.cc`,
//! `node_shader_bump.cc`.

use super::{BAddCategory, BNodeDecl, BShaderNode, SOCK_FLOAT, SOCK_RGBA, SOCK_VECTOR};

const VECTOR: BAddCategory = BAddCategory::Vector;

pub struct Mapping;
impl BShaderNode for Mapping {
    fn idname(&self) -> &'static str {
        "ShaderNodeMapping"
    }
    fn label(&self) -> &'static str {
        "Mapping"
    }
    fn category(&self) -> BAddCategory {
        VECTOR
    }
    fn declare(&self, b: &mut BNodeDecl) {
        b.input("Vector", SOCK_VECTOR);
        b.input("Location", SOCK_VECTOR);
        b.input("Rotation", SOCK_VECTOR);
        b.input("Scale", SOCK_VECTOR);
        b.output("Vector", SOCK_VECTOR);
    }
}

pub struct NormalMap;
impl BShaderNode for NormalMap {
    fn idname(&self) -> &'static str {
        "ShaderNodeNormalMap"
    }
    fn label(&self) -> &'static str {
        "Normal Map"
    }
    fn category(&self) -> BAddCategory {
        VECTOR
    }
    fn declare(&self, b: &mut BNodeDecl) {
        b.input("Strength", SOCK_FLOAT);
        b.input("Color", SOCK_RGBA);
        b.output("Normal", SOCK_VECTOR);
    }
}

pub struct Bump;
impl BShaderNode for Bump {
    fn idname(&self) -> &'static str {
        "ShaderNodeBump"
    }
    fn label(&self) -> &'static str {
        "Bump"
    }
    fn category(&self) -> BAddCategory {
        VECTOR
    }
    fn declare(&self, b: &mut BNodeDecl) {
        b.input("Strength", SOCK_FLOAT);
        b.input("Distance", SOCK_FLOAT);
        b.input("Filter Width", SOCK_FLOAT);
        b.input("Height", SOCK_FLOAT);
        b.input("Normal", SOCK_VECTOR);
        b.output("Normal", SOCK_VECTOR);
    }
}
