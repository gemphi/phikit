// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! The scene as the renderer holds it: `intern/cycles/scene`. So far the
//! passes, the integrator, the film, the camera, the objects and the meshes
//! with their attributes and their trees, the graph a shader is made of,
//! and the device scene they are flattened into.

mod attribute;
mod background;
mod camera;
mod device;
mod film;
pub(crate) mod geometry;
mod integrator;
pub(crate) mod mesh;
mod object;
mod param;
mod pass;
mod scene;
mod shader;
mod table;
mod update;

pub use attribute::{Attribute, AttributeData, AttributeDescriptor, AttributeElement, AttributePrimitive, AttributeRequest, AttributeRequestSet, AttributeSet, AttributeStandard, AttributeType, KernelDataType};
pub use background::Background;
pub use camera::{Camera, MotionType};
pub use device::DeviceScene;
pub use film::{Film, FilmPassChange, FilmPassInput, PassLayout, filter_table};
pub use geometry::{AttributeIdMap, Geometry, GeometryInput, GeometryKind, GeometryManager, Shader, find_attribute};
pub use integrator::{GuidingParam, HaltonSequence, Integrator, IntegratorInput, MAX_SAMPLE, fill_sample_pattern, tabulated_sobol_generate_4d};
pub use mesh::{Mesh, MeshSize, SubdFace, Triangle};
pub use object::{Object, ObjectInput, ObjectManager};
pub use param::SceneParam;
pub use shader::{JUMP_NODE_SIZE, ShaderManager, compile, emission_estimate, InputId, OutputId, ShaderBump, ShaderGraph, ShaderInput, ShaderNode, ShaderNodeSpecialType, ShaderOutput};
pub use pass::{Pass, PassInfo, PassMode, PassType};
pub use scene::{Scene, Surface, World};
pub use table::LookupTableSet;
