//! # SCNShader Modular Implementation
//!
//! Mirrors SceneKit's shader system with WGSL compilation.

pub mod modifiers;
pub mod templates;
pub mod compiler;
pub mod ssao_shaders;

use std::collections::HashMap;
pub use modifiers::*;

/// Complete shader program with vertex and fragment stages
#[derive(Debug, Clone)]
pub struct SCNProgram {
    pub vertex_shader: Option<String>,
    pub fragment_shader: Option<String>,
    pub vertex_function_name: String,
    pub fragment_function_name: String,
    pub is_opaque: bool,
    pub shader_modifiers: HashMap<SCNShaderModifierEntryPoint, String>,
}

impl SCNProgram {
    pub fn new() -> Self {
        Self {
            vertex_shader: None,
            fragment_shader: None,
            vertex_function_name: "vs_main".to_string(),
            fragment_function_name: "fs_main".to_string(),
            is_opaque: true,
            shader_modifiers: HashMap::new(),
        }
    }
}

pub trait SCNShadable {
    fn program(&self) -> Option<&SCNProgram>;
    fn set_program(&mut self, program: SCNProgram);
}
