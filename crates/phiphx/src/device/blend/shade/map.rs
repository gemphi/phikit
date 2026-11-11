//! Image Texture nodes linked to Principled inputs other than Base Color:
//! Roughness, Metallic and Emission Color. A linked image replaces the
//! socket value (Blender ignores the slider once something is linked).

use super::node::read_cstr;
use crate::device::blend::file::kind::*;

/// Image keys (see `image.rs`) per textured Principled input.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlendMapList {
    pub rough: Option<String>,
    pub metal: Option<String>,
    pub emit: Option<String>,
}

/// The Principled socket each map reads, in [`BlendMapList`] field order.
pub const MAPPED: [&str; 3] = ["Roughness", "Metallic", "Emission Color"];

impl BlendMapList {
    /// The three keys in field order.
    pub fn each(&self) -> [&Option<String>; 3] {
        [&self.rough, &self.metal, &self.emit]
    }

    fn each_mut(&mut self) -> [&mut Option<String>; 3] {
        [&mut self.rough, &mut self.metal, &mut self.emit]
    }

    pub fn is_empty(&self) -> bool {
        self.each().iter().all(|k| k.is_none())
    }
}

impl BlendFile {
    /// Every Image Texture linked to a mapped Principled input. Only a direct
    /// link is a map: behind a Mix or a Color Ramp the image is one term of
    /// what reaches the socket, and using it alone as the value turned a
    /// mostly black pattern mask into a mirror. Those keep the value the
    /// graph folds to (`principled.rs`).
    pub(crate) fn shade_maps(&self, ma: &BlendBlock, dna: &DnaType) -> BlendMapList {
        let mut maps = BlendMapList::default();
        for (slot, socket) in maps.each_mut().into_iter().zip(MAPPED) {
            *slot = self.direct_image(ma, dna, socket);
        }
        maps
    }

    /// The image linked straight into the Principled input `socket`, through
    /// reroutes only.
    fn direct_image(&self, ma: &BlendBlock, dna: &DnaType, socket: &str) -> Option<String> {
        let nodes = self.shader_nodes(ma, dna);
        let links = self.links(ma, dna);
        let idname = self.off("bNode", "idname", dna)?;
        let bsdf = self.surface_principled(ma, dna)?;
        let mut sock = self.socket_named(bsdf, socket, dna)?;
        loop {
            let link = links.iter().find(|l| l.to_sock == sock)?;
            let node = nodes.iter().copied().find(|n| n.old_address == link.from)?;
            match read_cstr(&node.data, idname, 64).as_str() {
                "ShaderNodeTexImage" => {
                    let im = self.block_at(self.ptr(node, self.off("bNode", "*id", dna)))?;
                    return Some(self.image_key(im, dna)).filter(|k| !k.is_empty());
                }
                "NodeReroute" => sock = *self.input_sockets(node, dna).first()?,
                _ => return None,
            }
        }
    }
}
