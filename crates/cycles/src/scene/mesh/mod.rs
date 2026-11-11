// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/mesh.h` and `mesh.cpp` (`Mesh`):
//! triangles over vertices. The positions and normals are attributes of
//! its geometry; the triangles, their shaders and whether they are smooth
//! are sockets of the `mesh` type of `data/node.tsv`.
//!
//! Not ported: subdivision (`tessellate`, `mesh_subdivision.cpp`),
//! displacement (`mesh_displace.cpp`) and tangents (`update_tangents`,
//! which needs MikkTSpace). A mesh keeps the faces to subdivide and their
//! attributes, but nothing dices them.

mod attribute;
mod bound;
mod normal;
mod pack;
mod triangle;

pub use triangle::Triangle;

use super::{AttributeElement, AttributePrimitive, AttributeSet, AttributeStandard, Geometry, GeometryKind};
use crate::graph::Value;

/// How many of each element a mesh has: what `Attribute::element_size`
/// asks a mesh for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MeshSize {
    pub vert: usize,
    pub triangle: usize,
    /// `get_num_subd_base_verts`: the vertices before subdivision added any.
    pub subd_base_vert: usize,
    pub subd_face: usize,
    /// The corners of the faces to subdivide.
    pub subd_corner: usize,
}

impl MeshSize {
    /// `Attribute::element_size`, for a mesh: how many values an attribute
    /// on `element` has.
    pub fn element_size(&self, element: AttributeElement, prim: AttributePrimitive) -> usize {
        let of = AttributeElement::of;
        let subd = prim == AttributePrimitive::Subd;
        match (element.domain(), subd) {
            (domain, _) if domain == of("OBJECT") || domain == of("MESH") || domain == of("VOXEL") => 1,
            (domain, true) if domain == of("VERTEX") => self.subd_base_vert,
            (domain, false) if domain == of("VERTEX") => self.vert,
            (domain, true) if domain == of("FACE") => self.subd_face,
            (domain, false) if domain == of("FACE") => self.triangle,
            (domain, true) if domain == of("CORNER") => self.subd_corner,
            (domain, false) if domain == of("CORNER") => self.triangle * 3,
            _ => 0,
        }
    }
}

/// `Mesh::SubdFace`: a face to subdivide.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubdFace {
    pub start_corner: i32,
    /// `num_corners`.
    pub corner_count: i32,
    pub shader: i32,
    pub smooth: bool,
    pub ptex_offset: i32,
}

impl SubdFace {
    pub fn is_quad(&self) -> bool {
        self.corner_count == 4
    }

    /// `num_ptex_faces`: a quad is one, any other face one for each corner.
    pub fn ptex_face_count(&self) -> i32 {
        match self.is_quad() {
            true => 1,
            false => self.corner_count,
        }
    }
}

/// `Mesh`.
#[derive(Debug, Clone, PartialEq)]
pub struct Mesh {
    /// What it has in common with every geometry.
    pub geometry: Geometry,
    /// `subd_attributes`: the attributes of the faces to subdivide.
    pub subd_attribute: AttributeSet,
    /// Its first face and corner to subdivide, among every mesh's.
    pub face_offset: usize,
    pub corner_offset: usize,
    /// `num_subd_added_verts`.
    subd_added_vert: usize,
    /// `num_subd_faces`.
    subd_face: usize,
}

impl Default for Mesh {
    fn default() -> Self {
        Self::new()
    }
}

impl Mesh {
    /// An empty mesh, with its position attribute.
    pub fn new() -> Mesh {
        let geometry = Geometry::new(GeometryKind::Mesh).expect("data/node.tsv has the mesh");
        let subd_attribute = AttributeSet::new(GeometryKind::Mesh, AttributePrimitive::Subd);
        let mut mesh = Mesh { geometry, subd_attribute, face_offset: 0, corner_offset: 0, subd_added_vert: 0, subd_face: 0 };
        mesh.add_builtin_attribute();
        mesh
    }

    /// `add_builtin_attributes`.
    fn add_builtin_attribute(&mut self) {
        self.add_attribute(AttributeStandard::of("POSITION"));
    }

    /// `num_verts`.
    pub fn vert_count(&self) -> usize {
        self.geometry.position_attribute().map(|attribute| attribute.size()).unwrap_or(0)
    }

    /// `num_triangles`.
    pub fn triangle_count(&self) -> usize {
        self.geometry.node.int_array("triangles").len() / 3
    }

    /// `get_triangle`.
    pub fn triangle(&self, index: usize) -> Triangle {
        let corner = self.geometry.node.int_array("triangles");
        Triangle { v: [0, 1, 2].map(|at| corner.get(index * 3 + at).copied().unwrap_or(0)) }
    }

    /// `get_num_subd_faces`.
    pub fn subd_face_count(&self) -> usize {
        self.subd_face
    }

    /// How many of each element it has.
    pub fn size(&self) -> MeshSize {
        let vert = self.vert_count();
        let subd_corner = self.geometry.node.int_array("subd_face_corners").len();
        MeshSize { vert, triangle: self.triangle_count(), subd_base_vert: vert.saturating_sub(self.subd_added_vert), subd_face: self.subd_face, subd_corner }
    }

    /// `attributes.add(std)`: the standard attribute, with a value for
    /// each of its element the mesh has.
    pub fn add_attribute(&mut self, std: AttributeStandard) -> &mut super::Attribute {
        let size = self.size();
        let count = std.element_on(GeometryKind::Mesh).map(|element| size.element_size(element, AttributePrimitive::Geometry)).unwrap_or(0);
        self.geometry.attribute.add_standard(std, "", count)
    }

    /// `subd_attributes.add(std)`.
    pub fn add_subd_attribute(&mut self, std: AttributeStandard) -> &mut super::Attribute {
        let size = self.size();
        let count = std.element_on(GeometryKind::Mesh).map(|element| size.element_size(element, AttributePrimitive::Subd)).unwrap_or(0);
        self.subd_attribute.add_standard(std, "", count)
    }

    fn resize_int(&mut self, name: &str, count: usize) {
        let mut value = self.geometry.node.int_array(name).to_vec();
        value.resize(count, 0);
        self.geometry.node.set(name, Value::IntArray(value));
    }

    fn resize_boolean(&mut self, name: &str, count: usize) {
        let mut value = self.geometry.node.boolean_array(name).to_vec();
        value.resize(count, false);
        self.geometry.node.set(name, Value::BooleanArray(value));
    }

    /// `resize_mesh`: room for so many vertices and triangles.
    pub fn resize_mesh(&mut self, vert: usize, triangle: usize) {
        self.add_attribute(AttributeStandard::of("POSITION")).resize(vert);
        self.resize_int("triangles", triangle * 3);
        self.resize_int("shader", triangle);
        self.resize_boolean("smooth", triangle);
        let size = self.size();
        self.geometry.attribute.resize(|element| size.element_size(element, AttributePrimitive::Geometry));
    }

    /// `resize_subd_faces`: room for so many faces to subdivide.
    pub fn resize_subd_face(&mut self, face: usize, corner: usize) {
        ["subd_start_corner", "subd_num_corners", "subd_shader", "subd_ptex_offset"].iter().for_each(|name| self.resize_int(name, face));
        self.resize_boolean("subd_smooth", face);
        self.resize_int("subd_face_corners", corner);
        self.subd_face = face;
        let size = self.size();
        self.subd_attribute.resize(|element| size.element_size(element, AttributePrimitive::Subd));
    }

    /// `clear`: no vertices, no triangles, no faces, no attribute but the
    /// position.
    pub fn clear(&mut self, preserve_shader: bool) {
        self.geometry.clear(preserve_shader);
        let int = ["triangles", "shader", "subd_start_corner", "subd_num_corners", "subd_shader", "subd_ptex_offset", "subd_face_corners", "subd_creases_edge"];
        int.iter().for_each(|name| self.resize_int(name, 0));
        ["smooth", "subd_smooth"].iter().for_each(|name| self.resize_boolean(name, 0));
        self.geometry.node.set("subd_creases_weight", Value::FloatArray(Vec::new()));
        self.subd_attribute.clear(false);
        self.geometry.attribute.clear(false);
        self.add_builtin_attribute();
        self.geometry.node.set_enum("subdivision_type", "none");
        self.geometry.clear(true);
        (self.subd_added_vert, self.subd_face) = (0, 0);
    }

    /// `get_subd_face`.
    pub fn subd_face(&self, index: usize) -> SubdFace {
        let int = |name: &str| self.geometry.node.int_array(name).get(index).copied().unwrap_or(0);
        let smooth = self.geometry.node.boolean_array("subd_smooth").get(index).copied().unwrap_or(false);
        SubdFace { start_corner: int("subd_start_corner"), corner_count: int("subd_num_corners"), shader: int("subd_shader"), smooth, ptex_offset: int("subd_ptex_offset") }
    }

    /// `add_edge_crease`.
    pub fn add_edge_crease(&mut self, v0: i32, v1: i32, weight: f32) {
        let mut edge = self.geometry.node.int_array("subd_creases_edge").to_vec();
        edge.extend([v0, v1]);
        let mut sharp = self.geometry.node.float_array("subd_creases_weight").to_vec();
        sharp.push(weight);
        self.geometry.node.set("subd_creases_edge", Value::IntArray(edge));
        self.geometry.node.set("subd_creases_weight", Value::FloatArray(sharp));
    }

    /// `add_vertex_crease`.
    pub fn add_vertex_crease(&mut self, vert: i32, weight: f32) {
        let mut crease = self.geometry.node.int_array("subd_vert_creases").to_vec();
        crease.push(vert);
        let mut sharp = self.geometry.node.float_array("subd_vert_creases_weight").to_vec();
        sharp.push(weight);
        self.geometry.node.set("subd_vert_creases", Value::IntArray(crease));
        self.geometry.node.set("subd_vert_creases_weight", Value::FloatArray(sharp));
    }

    /// `need_tesselation`: it is subdivided, and what the dicing depends
    /// on changed.
    pub fn need_tesselation(&self) -> bool {
        let node = &self.geometry.node;
        let diced = ["subd_dicing_rate", "subd_adaptive_space", "subd_objecttoworld", "subd_max_level"].iter().any(|name| node.socket_is_modified(name));
        node.int("subdivision_type") != 0 && (self.geometry.position_is_modified() || diced)
    }
}
