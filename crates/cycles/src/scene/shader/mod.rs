// SPDX-License-Identifier: Apache-2.0
//! The shaders of a scene: `intern/cycles/scene/shader*.cpp` and
//! `svm.cpp`. So far the graph of nodes a shader is made of (`graph`).
//! The shader itself, the shader manager, the SVM compiler and what each
//! kind of node compiles to are not ported yet.

mod graph;

pub use graph::{InputId, OutputId, ShaderBump, ShaderGraph, ShaderInput, ShaderNode, ShaderNodeSpecialType, ShaderOutput};
