// SPDX-License-Identifier: Apache-2.0
//! The shaders of a scene: `intern/cycles/scene/shader*.cpp` and
//! `svm.cpp`. The graph of nodes a shader is made of (`graph`), and for
//! the nodes gate 1 shades (a diffuse BSDF, an emission and a background
//! with constant inputs) the compiler and the manager that writes the
//! shaders' words (`compile`, `manager`). The shader graph is not compiled
//! as a whole yet, nor are the other node classes.
mod compile;
mod graph;
mod manager;
pub use compile::{JUMP_NODE_SIZE, compile, emission_estimate};
pub use graph::{InputId, OutputId, ShaderBump, ShaderGraph, ShaderInput, ShaderNode, ShaderNodeSpecialType, ShaderOutput};
pub use manager::ShaderManager;
