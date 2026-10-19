//! The world's shader graph (`World.*nodetree`): the Background node that
//! lights every render, and the Environment or Sky Texture behind it.
//!
//! Blender ref (`node_shader_background.cc`, `node_shader_tex_environment.cc`,
//! `background.h`): the World Output's Surface input takes a Background node
//! whose `Color` × `Strength` is the radiance arriving from every direction.
//! `World.horr/horg/horb` is the 2.7x horizon colour and is *not* what a 2.8+
//! render uses — reading it instead of the graph is why our ambient light was
//! wrong.

use super::nodes::read_cstr;
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
        let graph = super::eval::BGraph::owned(self, wo, "World", dna);
        let named = |n: &BlendBlock| {
            self.off("bNode", "idname", dna).map(|o| read_cstr(&n.data, o, 64)).unwrap_or_default()
        };
        let mut world = BlendWorld { address: wo.old_address, ..Default::default() };
        let Some(bg) = nodes.iter().copied().find(|n| named(n) == "ShaderNodeBackground") else {
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
        let env = nodes.iter().copied().find(|n| named(n) == "ShaderNodeTexEnvironment");
        if let Some(env) = env {
            let im = self.block_at(self.ptr(env, self.off("bNode", "*id", dna)));
            let present = im.is_none_or(|im| self.image_present(im, dna));
            // An image replaces the socket, so Color is the tint it is
            // multiplied by — white. A file Blender cannot find is magenta
            // in Blender, the same `IMB` "invalid" colour every other
            // missing image takes (`eval.rs`'s `ShaderNodeTexImage`), and it
            // lights the whole scene that colour because the world is a
            // dome. That is recorded in `missing` rather than reproduced:
            // the renderer stands Blender's studio light in for the file
            // the HDRI points at on another machine, and the read report
            // names it (`SceneReadReport::warnings`).
            world.texture = im.filter(|_| present).map(|im| self.image_key(im, dna));
            world.missing = im.filter(|_| !present).map(|im| self.image_key(im, dna));
            world.color = [1.0; 3];
        }
        // A Sky Texture takes the same place as an Environment Texture: it
        // replaces `Color`, so the socket's value becomes the white tint it
        // is multiplied by. Blender does not evaluate it per ray either —
        // `node_shader_tex_sky.cc` bakes it and the kernel samples the bake.
        if let Some(node) = nodes.iter().copied().find(|n| named(n) == "ShaderNodeTexSky") {
            world.sky = self.sky(node, dna);
            if world.sky.is_some() {
                world.color = [1.0; 3];
            }
        }
        if let Some(map) = nodes.iter().copied().find(|n| named(n) == "ShaderNodeMapping") {
            world.rotation = self.mapping_spin(map, dna);
        }
        world
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
        let Some(off) = self.off("bNodeSocketValueVector", "value[3]", dna) else { return 0.0 };
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
