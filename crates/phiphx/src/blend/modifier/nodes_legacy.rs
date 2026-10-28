//! The legacy "Auto Smooth" conversion — Blender's
//! `BKE_main_mesh_legacy_convert_auto_smooth`
//! (`blenkernel/intern/mesh_legacy_convert.cc`), run on **load** for every
//! file saved before 4.2.22.
//!
//! Until 4.1 a mesh carried its own auto-smooth: `Mesh.flag &
//! ME_AUTOSMOOTH` with an angle in `Mesh.smoothresh`, and the viewport
//! split the normals at edges sharper than that. 4.1 removed it and
//! Blender now *adds a Geometry Nodes modifier* instead, built in code by
//! `add_auto_smooth_node_tree` — the same ten-node "Auto Smooth" group
//! (Set Shade Smooth on the edge domain selected by Is Edge Smooth, whose
//! Shade Smooth is `Unsigned Angle <= Angle AND Is Shade Smooth`, then Set
//! Shade Smooth on the face domain) that the 5.x asset ships as "Smooth by
//! Angle". The modifier goes at the TAIL of the object's stack
//! (`BLI_addtail`) with `Angle` set to the mesh's saved `smoothresh`.
//!
//! So an older file has NO `NodesModifierData` on disk at all: the
//! modifier phiphx must evaluate is one Blender invents while reading. Of
//! the user's files, every one with this shading comes through here —
//! BOZO-CAR's `CAR-HOOD` among them, whose stack is saved as Solidify
//! alone and loads as Solidify + "Auto Smooth".
//!
//! The conversion is skipped exactly where Blender skips it: a mesh with
//! custom normals (the old auto-smooth did no edge tagging then), an
//! object carrying a Weighted Normal or Normal Edit modifier or a Bevel
//! with Harden Normals (they generate custom normals), and an object that
//! already has an auto-smooth node modifier.

use super::nodes::{self, BNodes};
use super::nodes_field::BInput;
use crate::blend::file::types::*;
use crate::blend::shade::graph::{BlendNode, BlendNodeGraph, BlendNodeLink};
use crate::blend::shade::socket::{BSocketDefault, BlendSocket};
use std::collections::HashMap;

/// `ME_AUTOSMOOTH_LEGACY` (`DNA_mesh_types.h:94`).
const ME_AUTOSMOOTH: u32 = 1 << 5;
/// `CD_CUSTOMLOOPNORMAL` (`DNA_customdata_types.h:96`).
const CD_CUSTOMLOOPNORMAL: i32 = 41;
/// `eModifierType_NormalEdit`, `_WeightedNormal`, `_Bevel`.
const MOD_NORMAL_EDIT: u32 = 50;
const MOD_WEIGHTED_NORMAL: u32 = 54;
const MOD_BEVEL: u32 = 24;
/// `MOD_BEVEL_HARDEN_NORMALS` (`DNA_modifier_types.h`).
const MOD_BEVEL_HARDEN_NORMALS: u32 = 1 << 14;
/// `AttrDomain::Edge` / `::Face` as `bNode.custom1`.
const DOMAIN_EDGE: i32 = 1;
const DOMAIN_FACE: i32 = 2;
/// `NODE_COMPARE_LESS_EQUAL`.
const COMPARE_LESS_EQUAL: i32 = 1;

impl BlendFile {
    /// The "Auto Smooth" modifier Blender adds to `ob` at load, or `None`
    /// when this object is not converted (`BKE_main_mesh_legacy_convert_auto_smooth`).
    pub(crate) fn legacy_auto_smooth(&self, ob: &BlendBlock, dna: &DnaType) -> Option<BNodes> {
        let me = self.mesh_block_of(ob, dna)?;
        if self.int_field(me, "Mesh", "flag", dna) & ME_AUTOSMOOTH == 0 {
            return None;
        }
        // `const float angle = mesh->smoothresh_legacy` — named without the
        // suffix in the files that still carry the flag.
        let angle = match self.off("Mesh", "smoothresh_legacy", dna) {
            Some(_) => self.float_field(me, "Mesh", "smoothresh_legacy", dna, 0.0),
            None => self.float_field(me, "Mesh", "smoothresh", dna, 0.0),
        };
        if self.mesh_has_custom_normals(me, dna) || self.modifiers_make_custom_normals(ob, dna) {
            return None;
        }
        let inputs = HashMap::from([("Socket_2".to_string(), BInput::Float(angle))]);
        let compares = HashMap::from([("Compare".to_string(), super::nodes_field::BCompare { operation: COMPARE_LESS_EQUAL, data_type: 0, mode: 0 })]);
        nodes::from_parts(auto_smooth_graph(), inputs, compares)
    }

    /// `mesh->attribute_storage.lookup("custom_normal")`, and the layer the
    /// older files keep them in (`CD_CUSTOMLOOPNORMAL`, named
    /// `.corner_normal` from 4.1).
    fn mesh_has_custom_normals(&self, me: &BlendBlock, dna: &DnaType) -> bool {
        self.scoped(me, || {
            if self.mesh_attribute(me, dna, &|n, _, _| n == "custom_normal").is_some() {
                return true;
            }
            let ps = self.header.pointer_size as usize;
            let at = ["corner_data", "ldata"].iter().find_map(|f| Self::struct_field_offset_exact("Mesh", f, dna, ps));
            at.and_then(|cd| self.customdata_layer(me, cd, dna, &|t, n| t == CD_CUSTOMLOOPNORMAL || n == b".corner_normal")).is_some()
        })
    }

    /// The modifiers that made the old auto-smooth skip an object: Weighted
    /// Normal, Normal Edit, a Bevel with Harden Normals — and an
    /// auto-smooth node modifier already there.
    fn modifiers_make_custom_normals(&self, ob: &BlendBlock, dna: &DnaType) -> bool {
        let first = self.ptr(ob, self.off("Object", "modifiers", dna));
        let next = self.off("ModifierData", "*next", dna);
        self.scoped(ob, || self.list(first, next)).into_iter().any(|md| {
            match self.int_field(md, "ModifierData", "type", dna) {
                MOD_WEIGHTED_NORMAL | MOD_NORMAL_EDIT => true,
                MOD_BEVEL => self.int_field(md, "BevelModifierData", "flags", dna) & MOD_BEVEL_HARDEN_NORMALS != 0,
                nodes::DNA_TYPE => self.ptr(md, self.off("NodesModifierData", "*node_group", dna)) != 0,
                _ => false,
            }
        })
    }
}

fn socket(id: &str, default: Option<BSocketDefault>) -> BlendSocket {
    BlendSocket { identifier: id.into(), name: id.into(), default, ..Default::default() }
}

fn node(name: &str, idname: &str, custom1: i32, inputs: &[(&str, Option<BSocketDefault>)], outputs: &[&str]) -> BlendNode {
    BlendNode {
        name: name.into(),
        idname: idname.into(),
        custom1,
        inputs: inputs.iter().map(|(id, d)| socket(id, d.clone())).collect(),
        outputs: outputs.iter().map(|id| socket(id, None)).collect(),
        ..Default::default()
    }
}

fn link(from: &str, from_socket: &str, to: &str, to_socket: &str) -> BlendNodeLink {
    BlendNodeLink { from: from.into(), from_socket: from_socket.into(), to: to.into(), to_socket: to_socket.into() }
}

/// `add_auto_smooth_node_tree` (`mesh_legacy_convert.cc:2053`), node for
/// node and link for link, with the socket identifiers it gives them
/// (`Socket_1` the mesh, `Socket_2` the angle, `Socket_0` the output).
pub(crate) fn auto_smooth_graph() -> BlendNodeGraph {
    let yes = Some(BSocketDefault::Bool(true));
    let zero = Some(BSocketDefault::Float(0.0));
    let nodes = vec![
        node("Group Output", "NodeGroupOutput", 0, &[("Socket_0", None)], &[]),
        node("Group Input", "NodeGroupInput", 0, &[], &["Socket_1", "Socket_2"]),
        node("Group Input.001", "NodeGroupInput", 0, &[], &["Socket_1", "Socket_2"]),
        node("Set Shade Smooth", "GeometryNodeSetShadeSmooth", DOMAIN_EDGE, &[("Geometry", None), ("Selection", yes.clone()), ("Shade Smooth", yes.clone())], &["Geometry"]),
        node("Set Shade Smooth.001", "GeometryNodeSetShadeSmooth", DOMAIN_FACE, &[("Geometry", None), ("Selection", yes.clone()), ("Shade Smooth", yes)], &["Geometry"]),
        node("Edge Angle", "GeometryNodeInputMeshEdgeAngle", 0, &[], &["Unsigned Angle", "Signed Angle"]),
        node("Is Edge Smooth", "GeometryNodeInputEdgeSmooth", 0, &[], &["Smooth"]),
        node("Is Face Smooth", "GeometryNodeInputShadeSmooth", 0, &[], &["Smooth"]),
        node("Boolean Math", "FunctionNodeBooleanMath", 0, &[("Boolean", None), ("Boolean_001", None)], &["Boolean"]),
        node("Compare", "FunctionNodeCompare", 0, &[("A", zero.clone()), ("B", zero)], &["Result"]),
    ];
    let links = vec![
        link("Edge Angle", "Unsigned Angle", "Compare", "A"),
        link("Set Shade Smooth.001", "Geometry", "Group Output", "Socket_0"),
        link("Group Input", "Socket_2", "Compare", "B"),
        link("Compare", "Result", "Boolean Math", "Boolean"),
        link("Is Face Smooth", "Smooth", "Boolean Math", "Boolean_001"),
        link("Group Input.001", "Socket_1", "Set Shade Smooth", "Geometry"),
        link("Is Edge Smooth", "Smooth", "Set Shade Smooth", "Selection"),
        link("Set Shade Smooth", "Geometry", "Set Shade Smooth.001", "Geometry"),
        link("Boolean Math", "Boolean", "Set Shade Smooth", "Shade Smooth"),
    ];
    BlendNodeGraph { nodes, links }
}
