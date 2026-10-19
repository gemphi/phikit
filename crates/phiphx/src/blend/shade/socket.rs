//! A node's sockets — `bNode.inputs` and `bNode.outputs`, each a `ListBase` of
//! `bNodeSocket` (`makesdna/DNA_node_types.h:1426`).
//!
//! `graph.rs` reads the nodes and the links between them, which is enough to draw
//! cards and say which pairs are joined. It is not enough to draw a node editor:
//! Blender's links run from a socket to a socket, and where those sockets sit is
//! what gives a node its height and its link endpoints.
//!
//! The one thing a socket does NOT carry is its position.
//! `bNodeSocket.runtime->location` is where Blender keeps it, and `runtime` is a
//! pointer rebuilt on load — nothing of it is in the file. So a reader can only
//! give the sockets, in order, with their type and flags, and the position has to
//! be LAID OUT the way Blender lays it out (`node_draw.cc`'s
//! `node_update_basis_from_socket_lists`, ported in phiber's
//! `canvas/node/socket.rs`). That split is deliberate and is why this file
//! carries no coordinates at all.
//!
//! | field | what it gives |
//! |---|---|
//! | `identifier[64]` | the stable id a `bNodeLink` names its ends by |
//! | `name[64]` | what the row prints |
//! | `type` | `eNodeSocketDatatype` — decides the socket's colour |
//! | `flag` | `eNodeSocketFlag`: `SOCK_UNAVAIL`, `SOCK_HIDDEN`, … |
//! | `in_out` | input or output; the lists already separate them
//! | `display_shape` | `eNodeSocketDisplayShape` — the shape it is DRAWN as |
//!
//! `type` is read through `int_field` rather than at a fixed width because it is
//! a C++ enum in 5.x and was a `short` in older files, so the DNA is the only
//! honest source for how wide it is in THIS file.

use super::nodes::read_cstr;
use crate::blend::file::types::*;

/// One socket of a node, as the editor needs it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlendSocket {
    /// `bNodeSocket.identifier` — unique on its node and stable across renames,
    /// which is what makes it the id a link's end is resolved by.
    pub identifier: String,
    /// `bNodeSocket.name` — what the socket's row prints.
    pub name: String,
    /// `bNodeSocket.type`, an `eNodeSocketDatatype`. `SOCK_CUSTOM` is `-1`.
    pub kind: i32,
    /// `bNodeSocket.flag`, an `eNodeSocketFlag`.
    pub flag: i32,
    /// `bNodeSocket.limit` — how many links this socket accepts.
    ///
    /// Set at creation to `in_out == SOCK_IN ? 1 : 0xFFF`
    /// (`blenkernel/intern/node.cc:3090`), so an input takes ONE link and an
    /// output takes 4095. That one number is why dropping a link on a
    /// connected input replaces what is there instead of adding to it.
    pub limit: i32,
    /// `bNodeSocket.display_shape`, an `eNodeSocketDisplayShape`
    /// (`DNA_node_types.h:116`): circle 0, square 1, diamond 2, the three
    /// `_DOT` variants 3–5, line 6, volume grid 7, list 8. It is what the
    /// editor draws the socket AS, and it is a different question from
    /// `kind`, which is what colours it: a Geometry Nodes field input is a
    /// diamond whatever its type, and a socket carrying a single value is a
    /// circle. Defaults to circle, as the DNA member does.
    pub display_shape: i32,
    /// The saved address, so a link can name this end.
    pub address: u64,
    /// `bNodeSocket.default_value` — the value an UNCONNECTED input shows in its
    /// row (a number, a colour swatch). `None` for a socket with no stored
    /// default, or a type whose default phiber does not read yet. The editor
    /// draws it only on an unconnected input (`socket::shows_value`).
    pub default: Option<BSocketDefault>,
}

/// The value an unconnected input socket carries — Blender's type-specific
/// `default_value` struct (`bNodeSocketValueFloat`/`RGBA`/`Vector`/… in
/// `DNA_node_types.h`), read into the few shapes the shader editor draws.
#[derive(Clone, Debug, PartialEq)]
pub enum BSocketDefault {
    /// `bNodeSocketValueFloat.value`.
    Float(f32),
    /// `bNodeSocketValueRGBA.value[4]` — RGBA.
    Color([f32; 4]),
    /// `bNodeSocketValueVector.value` — the first three of its `value[4]`.
    Vector([f32; 3]),
    /// `bNodeSocketValueBoolean.value`.
    Bool(bool),
    /// `bNodeSocketValueInt.value`.
    Int(i32),
}

/// `eNodeSocketFlag` (`DNA_node_types.h`), the bits that decide whether a socket
/// is drawn at all.
pub const SOCK_HIDDEN: i32 = 1 << 1;
pub const SOCK_IS_LINKED: i32 = 1 << 2;
pub const SOCK_UNAVAIL: i32 = 1 << 3;
pub const SOCK_MULTI_INPUT: i32 = 1 << 11;

/// `eNodeSocketDisplayShape` (`DNA_node_types.h:116–126`) — all nine, in
/// their declared order, which is the order the GPU shader's `switch` takes
/// them in (`gpu_shader_2D_node_socket_frag.glsl`).
pub const SOCK_DISPLAY_SHAPE_CIRCLE: i32 = 0;
pub const SOCK_DISPLAY_SHAPE_SQUARE: i32 = 1;
pub const SOCK_DISPLAY_SHAPE_DIAMOND: i32 = 2;
pub const SOCK_DISPLAY_SHAPE_CIRCLE_DOT: i32 = 3;
pub const SOCK_DISPLAY_SHAPE_SQUARE_DOT: i32 = 4;
pub const SOCK_DISPLAY_SHAPE_DIAMOND_DOT: i32 = 5;
pub const SOCK_DISPLAY_SHAPE_LINE: i32 = 6;
pub const SOCK_DISPLAY_SHAPE_VOLUME_GRID: i32 = 7;
pub const SOCK_DISPLAY_SHAPE_LIST: i32 = 8;

impl BlendSocket {
    /// `bNodeSocket::is_available` (`BKE_node_runtime.hh:1024`) — exactly
    /// `(flag & SOCK_UNAVAIL) == 0`. An unavailable socket is one the node's type
    /// has switched off (a Math node's second operand in single-operand mode),
    /// and Blender's layout skips it, so it takes no row and no height.
    pub fn available(&self) -> bool {
        self.flag & SOCK_UNAVAIL == 0
    }

    /// Whether this socket gets a row.
    ///
    /// Blender's `is_visible()` is `!is_user_hidden() && is_available() &&
    /// inferred_socket_visibility()`. The third term is a runtime inference over
    /// the tree's topology and is not in the file, so it cannot be answered from
    /// here; the first two are flags and are. A socket this calls visible is
    /// therefore one Blender would at most also hide, never one it would show.
    pub fn visible(&self) -> bool {
        self.available() && self.flag & SOCK_HIDDEN == 0
    }

    /// `SOCK_IS_LINKED`: something is plugged in, so the row shows no value.
    pub fn linked(&self) -> bool {
        self.flag & SOCK_IS_LINKED != 0
    }

    /// `SOCK_MULTI_INPUT`: takes any number of links (a Join Geometry input).
    /// Blender gives one of these a taller row — `NODE_MULTI_INPUT_LINK_GAP` per
    /// link past the second — which the layout accounts for.
    pub fn multi(&self) -> bool {
        self.flag & SOCK_MULTI_INPUT != 0
    }

    /// `node_socket_link_limit` (`blenkernel/intern/node.cc:5409`).
    ///
    /// A multi-input socket takes 4095 whatever its stored `limit` says, which
    /// is the first thing that function checks. After that Blender prefers the
    /// socket TYPE's limit when `use_link_limits_of_type` is set, and falls back
    /// to the stored field otherwise — `typeinfo` is a registry pointer and is
    /// not in the file, so the stored field is what a reader has. It is the same
    /// answer for every stock socket, whose type limits match the defaults.
    pub fn link_limit(&self) -> i32 {
        if self.multi() { MULTI_INPUT_LIMIT } else { self.limit }
    }
}

/// What `node_socket_link_limit` returns for a multi-input socket.
pub const MULTI_INPUT_LIMIT: i32 = 4095;

impl BlendFile {
    /// One of a node's two socket lists, in the file's own order.
    ///
    /// Order matters and is not sorted: it is the order Blender lays the rows out
    /// in, so reordering here would move every socket on the card.
    pub(crate) fn node_sockets(
        &self,
        node: &BlendBlock,
        which: &'static str,
        dna: &DnaType,
    ) -> Vec<BlendSocket> {
        let first = self.ptr(node, self.off("bNode", which, dna));
        let next = self.off("bNodeSocket", "*next", dna);
        let identifier = self.off("bNodeSocket", "identifier", dna);
        let name = self.off("bNodeSocket", "name", dna);
        self.list(first, next)
            .into_iter()
            .map(|s| {
                let kind = self.int_field(s, "bNodeSocket", "type", dna) as i32;
                BlendSocket {
                    identifier: identifier.map_or_else(String::new, |o| read_cstr(&s.data, o, 64)),
                    name: name.map_or_else(String::new, |o| read_cstr(&s.data, o, 64)),
                    // `type` is an enum in 5.x and was a `short` before it; the DNA
                    // says which, so the width is never assumed.
                    kind,
                    flag: self.int_field(s, "bNodeSocket", "flag", dna) as i32,
                    limit: self.int_field(s, "bNodeSocket", "limit", dna) as i32,
                    // A `char` enum in the DNA, so `int_field` reads its one
                    // byte; a file old enough not to carry the member at all
                    // gets the circle the member itself defaults to.
                    display_shape: self
                        .off("bNodeSocket", "display_shape", dna)
                        .map_or(SOCK_DISPLAY_SHAPE_CIRCLE, |_| self.int_field(s, "bNodeSocket", "display_shape", dna) as i32),
                    address: s.old_address,
                    default: self.socket_default(s, kind, dna),
                }
            })
            .collect()
    }

    /// The value an input socket shows when nothing is plugged into it —
    /// `bNodeSocket.default_value`, a pointer to a type-specific struct.
    ///
    /// Only the shapes the shader editor draws are read; any other socket type
    /// (string, image, a geometry-nodes type) comes back `None` rather than a
    /// guessed value.
    fn socket_default(&self, sock: &BlendBlock, kind: i32, dna: &DnaType) -> Option<BSocketDefault> {
        let addr = self.ptr(sock, self.off("bNodeSocket", "*default_value", dna));
        let value = self.block_at(addr)?;
        let f = |st: &str| {
            // Array fields are stored as `value[N]`; a scalar as `value`.
            let o = self.off(st, "value[4]", dna).or_else(|| self.off(st, "value", dna))?;
            value.data.get(o..o + 4).map(|b| self.read_f32(b))
        };
        let floats = |st: &str, n: usize| -> Option<Vec<f32>> {
            let o = self.off(st, "value[4]", dna).or_else(|| self.off(st, "value", dna))?;
            (0..n).map(|i| value.data.get(o + i * 4..o + i * 4 + 4).map(|b| self.read_f32(b))).collect()
        };
        match kind {
            0 => f("bNodeSocketValueFloat").map(BSocketDefault::Float),
            1 => floats("bNodeSocketValueVector", 3).map(|v| BSocketDefault::Vector([v[0], v[1], v[2]])),
            2 => floats("bNodeSocketValueRGBA", 4).map(|v| BSocketDefault::Color([v[0], v[1], v[2], v[3]])),
            4 => {
                let o = self.off("bNodeSocketValueBoolean", "value", dna)?;
                value.data.get(o).map(|b| BSocketDefault::Bool(*b != 0))
            }
            6 => {
                let o = self.off("bNodeSocketValueInt", "value", dna)?;
                value.data.get(o..o + 4).map(|b| BSocketDefault::Int(self.read_i32(b)))
            }
            _ => None,
        }
    }
}
#[cfg(test)]
#[path = "socket_tests.rs"]
mod tests;
