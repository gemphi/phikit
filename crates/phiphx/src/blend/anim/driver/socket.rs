//! The half of a datablock's drivers that is the same for every datablock
//! carrying a node tree: the socket path Blender writes, and the socket
//! list an index in it counts through.
//!
//! A driver on a node socket is stored on the **tree's** `AnimData`, not
//! the datablock's, and pathed
//!
//! ```text
//! nodes["<node name>"].inputs[<i>].default_value
//! ```
//!
//! with the component in the curve's `array_index`. The index counts
//! through `bNode.inputs` as the file stores it — every socket, including
//! the ones the panel hides — so it is not a count of what the UI shows.
//!
//! Materials ([`shade::driver`]), worlds and anything else with a tree all
//! read it this way; only the datablock's own property table differs.
//!
//! [`shade::driver`]: crate::blend::shade::driver

use crate::blend::file::types::*;
use crate::blend::shade::nodes::read_cstr;

/// `eNodeSocketDatatype::SOCK_RGBA`.
const SOCK_RGBA: u16 = 2;
const NODES_PREFIX: &str = "nodes[\"";
const INPUTS_MID: &str = "].inputs[";
const VALUE_SUFFIX: &str = "].default_value";

/// One input socket as the file stores it — enough to clamp a write to it
/// and to know whether the write can reach a pixel.
#[derive(Clone, Debug, PartialEq)]
pub struct BDriverSocket {
    pub name: String,
    /// `SOCK_RGBA`: its components clamp at 0, and it carries a colour.
    pub rgba: bool,
    /// A link feeds it, so the shader reads the link and not
    /// `default_value`. A driver still writes it, as Blender does.
    pub linked: bool,
}

/// `nodes["Principled BSDF"].inputs[2].default_value` as
/// `("Principled BSDF", 2)`. The name is taken up to its own closing quote,
/// so a node called `a].inputs[0` cannot be mistaken for an index.
pub fn socket_path(path: &str) -> Option<(String, usize)> {
    let rest = path.strip_prefix(NODES_PREFIX)?;
    let quote = rest.find('"')?;
    let (name, tail) = (&rest[..quote], &rest[quote + 1..]);
    let index = tail.strip_prefix(INPUTS_MID)?.strip_suffix(VALUE_SUFFIX)?;
    if name.is_empty() {
        return None;
    }
    Some((name.to_string(), index.parse().ok()?))
}

impl BlendFile {
    /// `bNode.name` — the name a socket path spells.
    pub(crate) fn node_name(&self, node: &BlendBlock, dna: &DnaType) -> Option<String> {
        self.off("bNode", "name", dna).map(|o| read_cstr(&node.data, o, 64))
    }

    /// One node's input sockets in storage order — the order the index in
    /// `inputs[<i>]` counts through.
    pub(crate) fn driver_socket_list(
        &self,
        node: &BlendBlock,
        graph: &crate::blend::shade::eval::BGraph<'_>,
        dna: &DnaType,
    ) -> Vec<BDriverSocket> {
        let type_o = self.off("bNodeSocket", "type", dna);
        self.input_sockets(node, dna)
            .into_iter()
            .map(|addr| {
                let rgba = type_o
                    .zip(self.block_at(addr))
                    .and_then(|(o, b)| b.data.get(o..o + 2).map(|bytes| self.read_u16(bytes) == SOCK_RGBA))
                    .unwrap_or(false);
                BDriverSocket { name: self.socket_name(addr, dna), rgba, linked: graph.linked(addr) }
            })
            .collect()
    }

    /// The input sockets of every node named in `wanted`, for a tree owned
    /// by `id` through its `owner`'s `*nodetree` field.
    pub(crate) fn driver_sockets_of(
        &self,
        id: &BlendBlock,
        owner: &str,
        wanted: &[String],
        dna: &DnaType,
    ) -> Vec<(String, Vec<BDriverSocket>)> {
        if wanted.is_empty() {
            return Vec::new();
        }
        let nodes = self.tree_nodes(id, owner, dna);
        let graph = crate::blend::shade::eval::BGraph::owned(self, id, owner, dna);
        wanted
            .iter()
            .filter_map(|name| {
                let node = nodes.iter().copied().find(|n| self.node_name(n, dna).as_deref() == Some(name.as_str()))?;
                Some((name.clone(), self.driver_socket_list(node, &graph, dna)))
            })
            .collect()
    }
}

/// `raw` as a socket's own write path leaves it: a float or vector socket
/// takes it whole even where RNA declares a range, and an RGBA socket's
/// component clamps at 0 below and is free above 1. Measured against
/// Blender's evaluated values, not read off the RNA ranges, which do not
/// predict it.
pub fn socket_clamp(sockets: &[BDriverSocket], input: usize, raw: f32) -> f32 {
    match sockets.get(input) {
        Some(sock) if sock.rgba => raw.max(0.0),
        _ => raw,
    }
}

#[cfg(test)]
mod tests {
    use super::socket_path;

    #[test]
    fn a_socket_path_is_a_node_name_and_an_input_index() {
        assert_eq!(
            socket_path("nodes[\"Principled BSDF\"].inputs[2].default_value"),
            Some(("Principled BSDF".to_string(), 2))
        );
        assert_eq!(socket_path("nodes[\"Background\"].inputs[1].default_value"), Some(("Background".to_string(), 1)));
        assert_eq!(socket_path("nodes[\"Noise Texture\"].inputs[11].default_value"), Some(("Noise Texture".to_string(), 11)));
    }

    #[test]
    fn a_path_that_is_not_a_socket_is_not_read_as_one() {
        assert_eq!(socket_path("nodes[\"Mapping\"].rotation"), None, "a node's own property");
        assert_eq!(socket_path("roughness"), None, "the datablock's own property");
        assert_eq!(socket_path("nodes[\"RGB\"].outputs[0].default_value"), None, "an output cannot be driven");
        assert_eq!(socket_path("nodes[\"N\"].inputs[].default_value"), None);
        assert_eq!(socket_path("nodes[\"N\"].inputs[x].default_value"), None);
        assert_eq!(socket_path("nodes[\"\"].inputs[0].default_value"), None);
    }

    #[test]
    fn the_node_name_ends_at_its_own_quote() {
        assert_eq!(
            socket_path("nodes[\"a].inputs[0\"].inputs[3].default_value"),
            Some(("a].inputs[0".to_string(), 3))
        );
    }
}
