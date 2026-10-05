//! The addable shader-node types — the Rust face of Blender's `bNodeType`.
//!
//! A Blender node type carries a `declare` callback (`NodeDeclarationBuilder`)
//! that builds a fresh node's sockets, plus its idname, its `ui_name` and the
//! Add-menu category it lives under. [`BShaderNode`] is that, as a trait: each
//! type is a unit struct whose `declare` reads like the `node_shader_*.cc` it is
//! ported from, so a socket list is checked against Blender's source rather than
//! remembered. The types are grouped into one file per Add-menu submenu, the way
//! Blender keeps one file per node.
//!
//! What is deliberately left off a declared node: the gpu-internal `Weight`
//! input every shader node carries (`.available(is_gpu_internal)` — never shown
//! in the editor), and, for now, the node types whose definition has not been
//! ported in yet. A socket that is declared here is one that was read off the
//! source, not approximated.

mod color;
mod converter;
mod input;
mod output;
mod principled;
mod shader;
mod vector;

/// `eNodeSocketDatatype` (`DNA_node_types.h`) — the same index
/// `theme::socket_color` is keyed by, so a declared socket colours like a file
/// one.
pub const SOCK_FLOAT: i32 = 0;
pub const SOCK_VECTOR: i32 = 1;
pub const SOCK_RGBA: i32 = 2;
pub const SOCK_SHADER: i32 = 3;
pub const SOCK_BOOLEAN: i32 = 4;

/// Which submenu of the Add menu a type lives under (`node_add_menu_shader`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BAddCategory {
    Input,
    Output,
    Shader,
    Color,
    Vector,
    Converter,
}

/// One socket a fresh node is declared with.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BAddSocket {
    /// What the row prints (`add_input<T>(name, …)`).
    pub name: &'static str,
    /// What a link resolves the socket by (`bNodeSocket.identifier`). Equal to
    /// the name for almost every shader node; a few — Brightness/Contrast's
    /// `Bright` — declare it apart, and a link to the wrong one draws to nowhere.
    pub identifier: &'static str,
    pub kind: i32,
}

impl BAddSocket {
    /// The file-shaped socket this becomes on an added node.
    pub fn to_socket(&self) -> crate::phiphx::blend::shade::socket::BlendSocket {
        crate::phiphx::blend::shade::socket::BlendSocket {
            identifier: self.identifier.into(),
            name: self.name.into(),
            kind: self.kind,
            ..Default::default()
        }
    }
}

/// The sockets a node declares, built up by [`BShaderNode::declare`] — Blender's
/// `NodeDeclarationBuilder`, which an `add_input`/`add_output` call at a time
/// appends to.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BNodeDecl {
    pub inputs: Vec<BAddSocket>,
    pub outputs: Vec<BAddSocket>,
}

impl BNodeDecl {
    /// `b.add_input<kind>(name)` — identifier is the name, as it is for almost
    /// every shader node.
    pub fn input(&mut self, name: &'static str, kind: i32) -> &mut Self {
        self.inputs.push(BAddSocket { name, identifier: name, kind });
        self
    }

    /// `b.add_input<kind>(name, identifier)` — the rare socket whose link name
    /// differs from its label.
    pub fn input_id(&mut self, name: &'static str, identifier: &'static str, kind: i32) -> &mut Self {
        self.inputs.push(BAddSocket { name, identifier, kind });
        self
    }

    /// `b.add_output<kind>(name)`.
    pub fn output(&mut self, name: &'static str, kind: i32) -> &mut Self {
        self.outputs.push(BAddSocket { name, identifier: name, kind });
        self
    }
}

/// A shader-node type: its identity and the sockets it is born with.
///
/// The trait is Blender's `bNodeType` reduced to what adding a node needs — the
/// `declare` callback and the three fields the Add menu reads. A type is a unit
/// struct, so the catalog is a list of `&dyn BShaderNode` with no state of its
/// own.
pub trait BShaderNode {
    /// `bNode.idname` — `"ShaderNodeBsdfDiffuse"` and so on.
    fn idname(&self) -> &'static str;
    /// `ntype.ui_name` — what the menu row and the node's header read.
    fn label(&self) -> &'static str;
    fn category(&self) -> BAddCategory;
    /// Build this type's sockets — Blender's `declare(NodeDeclarationBuilder&)`.
    fn declare(&self, b: &mut BNodeDecl);

    /// This type's sockets, declared.
    fn sockets(&self) -> BNodeDecl {
        let mut b = BNodeDecl::default();
        self.declare(&mut b);
        b
    }
}

/// Every addable type, in the Add menu's own category order.
pub const CATALOG: &[&dyn BShaderNode] = &[
    &output::MaterialOutput,
    &shader::DiffuseBsdf,
    &principled::PrincipledBsdf,
    &shader::Emission,
    &shader::Background,
    &shader::TransparentBsdf,
    &shader::MixShader,
    &shader::AddShader,
    &input::Rgb,
    &input::Value,
    &input::TextureCoordinate,
    &color::MixLegacy,
    &color::InvertColor,
    &color::Gamma,
    &color::BrightContrast,
    &vector::Mapping,
    &vector::NormalMap,
    &vector::Bump,
    &converter::Math,
    &converter::ColorRamp,
];

/// The type registered under `idname`, if the catalog carries it.
pub fn find(idname: &str) -> Option<&'static dyn BShaderNode> {
    CATALOG.iter().copied().find(|t| t.idname() == idname)
}

/// The types whose label contains `query`, case-insensitively, in catalog order
/// — the Add menu's own order. An empty query is every type, which is the menu
/// with nothing typed yet.
///
/// Blender's search is a fuzzy ranked match (`string_search`); a substring match
/// is the honest smaller version and keeps the menu's order rather than a score
/// phiber would have to invent.
pub fn search(query: &str) -> Vec<&'static dyn BShaderNode> {
    let q = query.trim().to_lowercase();
    CATALOG.iter().copied().filter(|t| q.is_empty() || t.label().to_lowercase().contains(&q)).collect()
}

#[cfg(test)]
mod tests;
