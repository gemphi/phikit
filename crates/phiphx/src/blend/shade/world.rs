//! The world's shader graph (`World.*nodetree`): the Background node that
//! lights every render, and the Environment or Sky Texture behind it.
//!
//! Blender ref (`node_shader_background.cc`, `node_shader_tex_environment.cc`,
//! `background.h`): the World Output's Surface input takes a Background node
//! whose `Color` × `Strength` is the radiance arriving from every direction.
//! `World.horr/horg/horb` is the 2.7x horizon colour and is *not* what a 2.8+
//! render uses — reading it instead of the graph is why our ambient light was
//! wrong.

use super::links::BLink;
use crate::blend::file::types::*;

/// `NodeSkyType`: the two models this atmosphere is (`sky_single_scattering
/// .cpp`). Preetham (0) and Hosek/Wilkie (1) are a different model with
/// their own precomputed coefficient dataset and are declined.
const SKY_SINGLE_SCATTERING: i32 = 2;
const SKY_MULTIPLE_SCATTERING: i32 = 3;

/// The world as it lights the scene.
#[derive(Clone, Debug, PartialEq)]
pub struct BlendWorld {
    /// Background `Color`, folded: white when an image feeds it, and
    /// Blender's missing-image magenta when that image is not there.
    pub color: [f32; 3],
    /// Background `Strength`.
    pub strength: f32,
    /// Environment Texture image key (see `image.rs`), when one feeds
    /// `Color` *and* Blender would find it; `None` for a missing file, so
    /// the renderer lights with `color` rather than hunting for it.
    pub texture: Option<String>,
    /// Mapping node `Rotation` around Z, in radians (the usual HDRI spin).
    pub rotation: f32,
    /// The Environment Texture image Blender could NOT find — its path as
    /// the file spells it — when the world asks for one that is not there.
    /// Blender lights the whole scene its missing-image magenta in that
    /// case; phiber lights it with Blender's own studio light instead and
    /// says so in the read report (`scene/import.rs`), because a demo lit
    /// magenta by a path from another machine is a broken picture, not a
    /// faithful one. `color` is then the white tint the stand-in is
    /// multiplied by, and `texture` is `None`.
    pub missing: Option<String>,
    /// A Sky Texture on `Color` instead of an image, with its own settings
    /// (`crate::sky`). The sky is not evaluated per ray in Blender either —
    /// it is baked and sampled as an image — so this is what the renderer
    /// bakes and uses as the environment.
    pub sky: Option<crate::sky::BSky>,
    /// The `World` datablock's saved address, or 0 for the default world
    /// of a file that has none. A file may hold several worlds while the
    /// scene uses one, so this is what says which driver set is the
    /// scene's (`anim/driver/wire.rs`).
    pub address: u64,
}

impl Default for BlendWorld {
    fn default() -> Self {
        // Blender's default world: a near-black grey at full strength.
        Self { color: [0.05; 3], strength: 1.0, texture: None, missing: None, rotation: 0.0, sky: None, address: 0 }
    }
}

impl BlendWorld {
    /// Radiance arriving from the world (colour × strength).
    pub fn radiance(&self) -> [f32; 3] {
        self.color.map(|c| c * self.strength)
    }
}

impl BlendFile {
    /// The scene's world, read from its node tree.
    pub(crate) fn world(&self, wo: &BlendBlock, dna: &DnaType) -> BlendWorld {
        let nodes = self.tree_nodes(wo, "World", dna);
        let links = self.tree_links(wo, "World", dna);
        let graph = super::eval::BGraph::owned(self, wo, "World", dna);
        let mut world = BlendWorld { address: wo.old_address, ..Default::default() };
        // The Background the World Output's Surface reaches, not the first
        // in the tree: a world holding spare Backgrounds and HDRIs (pose
        // 23_2 has three of each, the one listed first linked to nothing)
        // was lit by whichever the file happened to save first.
        let reached = self.lighting_background(&nodes, &links, &graph, dna);
        let Some(bg) = reached.or_else(|| nodes.iter().copied().find(|n| self.kind(n, dna) == "ShaderNodeBackground")) else {
            return world;
        };
        for sock in self.input_sockets(bg, dna) {
            let name = self.socket_name(sock, dna);
            let Some(v) = graph.input(sock, 0) else { continue };
            match name.as_str() {
                "Color" => world.color = [v.rgba[0], v.rgba[1], v.rgba[2]],
                "Strength" => world.strength = v.value(),
                _ => {}
            }
        }
        // What feeds that Background's Color: an Environment or Sky Texture,
        // through reroutes. A Background reached by the walk takes only
        // what is linked to it; the fallback (no World Output) keeps the
        // old reading of the first such node in the tree.
        let feeding = |kind: &str| match reached {
            Some(bg) => self.upstream(&nodes, &links, bg, "Color", dna).filter(|n| self.kind(n, dna) == kind),
            None => nodes.iter().copied().find(|n| self.kind(n, dna) == kind),
        };
        let env = feeding("ShaderNodeTexEnvironment");
        if let Some(env) = env {
            let im = self.block_at(self.ptr(env, self.off("bNode", "*id", dna)));
            let present = im.is_none_or(|im| self.image_present(im, dna));
            world.texture = im.filter(|_| present).map(|im| self.image_key(im, dna));
            world.missing = im.filter(|_| !present).map(|im| self.image_key(im, dna));
            // An image replaces the socket, so Color is the tint it is
            // multiplied by — white. A file Blender cannot find is magenta
            // in Blender, and lights the whole scene that colour because
            // the world is a dome. That is recorded in `missing` rather than
            // reproduced: the renderer stands Blender's studio light in for
            // the file the HDRI points at on another machine, and the read
            // report names it (`SceneReadReport::warnings`).
            world.color = [1.0; 3];
        }
        // A Sky Texture takes the same place as an Environment Texture: it
        // replaces `Color`, so the socket's value becomes the white tint it
        // is multiplied by. Blender does not evaluate it per ray either —
        // `node_shader_tex_sky.cc` bakes it and the kernel samples the bake.
        if let Some(node) = feeding("ShaderNodeTexSky") {
            world.sky = self.sky(node, dna);
            if world.sky.is_some() {
                world.color = [1.0; 3];
            }
        }
        // The Mapping that turns the texture: the one on its Vector input,
        // through reroutes (or, without a walk, the first in the tree).
        let map = match reached {
            Some(_) => env.and_then(|e| self.upstream(&nodes, &links, e, "Vector", dna)).filter(|n| self.kind(n, dna) == "ShaderNodeMapping"),
            None => nodes.iter().copied().find(|n| self.kind(n, dna) == "ShaderNodeMapping"),
        };
        if let Some(map) = map {
            world.rotation = self.mapping_spin(map, dna);
        }
        world
    }

    /// The Background node that lights the scene: from the active World
    /// Output's Surface, through reroutes and Add Shaders (first input) and
    /// Mix Shaders. A Mix keyed on Light Path's `Is Camera Ray` — the usual
    /// "one HDRI to see, another to light with" setup — takes its first
    /// shader, the one every ray but the camera's meets, because that is
    /// the one that lights. Any other Mix takes the side its folded factor
    /// leans to.
    fn lighting_background<'a>(&self, nodes: &[&'a BlendBlock], links: &[BLink], graph: &super::eval::BGraph, dna: &DnaType) -> Option<&'a BlendBlock> {
        let out = self.active_output(nodes, "OutputWorld", dna)?;
        let mut node = self.linked_named(nodes, links, out, "Surface", dna)?;
        for _ in 0..32 {
            let kind = self.kind(node, dna);
            let next = match kind.as_str() {
                "ShaderNodeBackground" => return Some(node),
                "NodeReroute" | "ShaderNodeAddShader" => 0,
                // Inputs: 0 Fac, 1 the shader at Fac 0, 2 the one at Fac 1.
                "ShaderNodeMixShader" => {
                    let fac = *self.input_sockets(node, dna).first()?;
                    let camera = links.iter().find(|l| l.to_sock == fac).is_some_and(|l| {
                        nodes.iter().any(|n| n.old_address == l.from && self.kind(n, dna) == "ShaderNodeLightPath")
                            && self.socket_name(l.from_sock, dna) == "Is Camera Ray"
                    });
                    match camera {
                        true => 1,
                        false => if graph.input(fac, 0).map_or(0.5, |v| v.value()) < 0.5 { 1 } else { 2 },
                    }
                }
                _ => return None,
            };
            node = self.linked_node(nodes, links, node, next, dna)?;
        }
        None
    }

    /// The node feeding `node`'s input `name`, through reroutes.
    fn upstream<'a>(&self, nodes: &[&'a BlendBlock], links: &[BLink], node: &BlendBlock, name: &str, dna: &DnaType) -> Option<&'a BlendBlock> {
        let mut up = self.linked_named(nodes, links, node, name, dna)?;
        for _ in 0..16 {
            if self.kind(up, dna) != "NodeReroute" {
                return Some(up);
            }
            up = self.linked_node(nodes, links, up, 0, dna)?;
        }
        None
    }

    /// A Sky Texture node's settings (`NodeTexSky`). `None` for the two
    /// analytic models — Preetham and Hosek/Wilkie — which are a different
    /// model with a precomputed coefficient dataset behind them, not this
    /// one; a file naming either keeps the Background colour it folded to
    /// rather than being given the wrong sky.
    fn sky(&self, node: &BlendBlock, dna: &DnaType) -> Option<crate::sky::BSky> {
        let st = self.block_at(self.ptr(node, self.off("bNode", "*storage", dna)))?;
        let at = |field: &str| -> Option<f32> {
            let o = self.off("NodeTexSky", field, dna)?;
            st.data.get(o..o + 4).map(|b| self.read_f32(b))
        };
        // The Dust slider's DNA field is `dust_density` in the file even
        // though Blender's own source calls it `aerosol_density` — the
        // struct was renamed and the DNA name kept, so both are tried and
        // the file decides. Reading only the source's name lost the slider
        // silently: every other field read, so a sky with heavy dust came
        // out as a sky with none.
        let dust = || at("dust_density").or_else(|| at("aerosol_density"));
        let model = self.off("NodeTexSky", "sky_model", dna).and_then(|o| st.data.get(o..o + 4)).map(|b| self.read_i32(b))?;
        // `NodeSkyType` (`DNA_node_types.h`): Preetham 0, Hosek 1, single
        // scattering 2, multiple scattering 3 — the analytic pair first,
        // which is the order to read twice before trusting.
        //
        // The multiple-scattering model is read as the single-scattering
        // one: the same atmosphere with the inter-scattering term left out,
        // so a sky asking for it comes out darker and less blue in the
        // shadowed parts than Blender's. That is the honest thing to say
        // until that term is ported too.
        if !(SKY_SINGLE_SCATTERING..=SKY_MULTIPLE_SCATTERING).contains(&model) {
            return None;
        }
        let disc = self.off("NodeTexSky", "sun_disc", dna).and_then(|o| st.data.get(o)).is_none_or(|b| *b != 0);
        Some(crate::sky::BSky {
            elevation: at("sun_elevation")?,
            rotation: at("sun_rotation")?,
            altitude: at("altitude")?,
            air: crate::sky::atmosphere::BSkyAir {
                air: at("air_density")?,
                aerosol: dust()?,
                ozone: at("ozone_density")?,
            },
            disc,
            size: at("sun_size")?,
            intensity: at("sun_intensity")?,
        })
    }

    /// A Mapping node's `Rotation` around Z, in radians.
    fn mapping_spin(&self, map: &BlendBlock, dna: &DnaType) -> f32 {
        // `value[3]` until Blender 4.2 made vector sockets 2–4 wide and the
        // field `value[4]`; asking for the old name alone read every newer
        // file's HDRI spin as 0.
        let Some(off) = self.off("bNodeSocketValueVector", "value[3]", dna).or_else(|| self.off("bNodeSocketValueVector", "value[4]", dna)) else { return 0.0 };
        for sock in self.input_sockets(map, dna) {
            if self.socket_name(sock, dna) != "Rotation" {
                continue;
            }
            let Some(v) = self.block_at(sock).map(|s| self.ptr(s, self.off("bNodeSocket", "*default_value", dna))) else { continue };
            let Some(val) = self.block_at(v) else { continue };
            if let Some(b) = val.data.get(off + 8..off + 12) {
                return self.read_f32(b);
            }
        }
        0.0
    }
}
