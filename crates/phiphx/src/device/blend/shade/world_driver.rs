//! Drivers on the world — `World.adt->drivers` for its own `color`, and
//! `World.nodetree->adt->drivers` for the Background node's `Strength` and
//! `Color`, which is what actually lights a 2.8+ render.
//!
//! The split and the socket path are the material's exactly
//! ([`anim::driver::socket`]), so only the datablock's own property table
//! is new here: `World.color` is a three-component colour with hard range
//! 0 .. ∞, clamping at 0 and free above 1.
//!
//! The socket clamps are the shared ones: the Background's `Strength` is a
//! float socket and takes a negative from a driver, its `Color` is RGBA and
//! clamps each component at 0.
//!
//! [`anim::driver::socket`]: crate::device::blend::anim::driver::socket

use crate::device::blend::shade::world::BlendWorld;
use crate::device::blend::anim::driver::{BDriverSocket, BlendDriver, BlendDriverScene, socket_clamp};

/// The node a world's background is read from (`world.rs`).
pub const BACKGROUND: &str = "Background";
/// `Background`'s input indices: `Color`, then `Strength`.
pub const BG_COLOR: usize = 0;
pub const BG_STRENGTH: usize = 1;

/// What one driver on a world writes.
#[derive(Clone, Debug, PartialEq)]
pub enum BWorldWrite {
    /// `World.color[c]` — the viewport's horizon colour, hard 0 .. ∞.
    Color(usize),
    /// A node socket's `default_value` on the world's tree.
    Socket { node: String, input: usize, component: usize },
}

/// One driver on a world.
#[derive(Clone, Debug, PartialEq)]
pub struct BWorldDriver {
    pub write: BWorldWrite,
    pub driver: BlendDriver,
}

/// Every driver the world carries, over both its IDs.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BWorldDriverList {
    pub world: String,
    /// The `World` datablock's saved address. A file may hold several
    /// worlds while the scene uses one, so the address is what says which
    /// set belongs to the scene's own world (`anim/driver/wire.rs`).
    pub address: u64,
    pub drivers: Vec<BWorldDriver>,
    /// The input sockets of each node a driver writes, in storage order.
    pub inputs: Vec<(String, Vec<BDriverSocket>)>,
}

impl BWorldDriverList {
    pub fn is_empty(&self) -> bool {
        self.drivers.is_empty()
    }

    fn sockets(&self, node: &str) -> &[BDriverSocket] {
        self.inputs.iter().find(|(name, _)| name == node).map_or(&[], |(_, list)| list.as_slice())
    }

    /// Every driver's value at `frame`, clamped as Blender clamps it.
    pub fn values(&self, scene: &BlendDriverScene, frame: f32) -> Vec<(BWorldWrite, f32)> {
        self.drivers
            .iter()
            .filter(|d| !d.driver.muted())
            .map(|d| {
                let raw = d.driver.value(scene, frame);
                let value = match &d.write {
                    // A colour property clamps at 0 and is free above 1.
                    BWorldWrite::Color(_) => raw.max(0.0),
                    BWorldWrite::Socket { node, input, .. } => socket_clamp(self.sockets(node), *input, raw),
                };
                (d.write.clone(), value)
            })
            .collect()
    }

    /// Write this frame's driven values over `world`.
    ///
    /// `BlendWorld::color` is the Background node's folded colour, not
    /// `World.color` — the reader prefers the tree, because the tree is
    /// what lights the render. A driver on `World.color` therefore moves
    /// the viewport's horizon and not the lighting, which is what Blender
    /// does too; it is kept in [`Self::values`] and does not land here.
    ///
    /// A socket on a node the world's background does not read is skipped,
    /// and so is one a link feeds — the link reaches the shader instead.
    pub fn apply(&self, world: &mut BlendWorld, scene: &BlendDriverScene, frame: f32) {
        for (write, value) in self.values(scene, frame) {
            let BWorldWrite::Socket { node, input, component } = write else { continue };
            if node != BACKGROUND {
                continue;
            }
            let Some(sock) = self.sockets(&node).get(input).filter(|sock| !sock.linked) else { continue };
            let _ = sock;
            match input {
                BG_COLOR if component < 3 => world.color[component] = value,
                BG_STRENGTH => world.strength = value,
                _ => {}
            }
        }
    }
}

#[cfg(test)]
#[path = "../test/shade/world_driver.rs"]
mod world_driver_test;
