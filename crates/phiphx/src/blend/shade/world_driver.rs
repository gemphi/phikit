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
//! [`anim::driver::socket`]: crate::blend::anim::driver::socket

use super::world::BlendWorld;
use crate::blend::anim::driver::{BDriverSocket, BlendDriver, BlendDriverScene, socket_clamp};

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
pub struct BWorldDrivers {
    pub world: String,
    /// The `World` datablock's saved address. A file may hold several
    /// worlds while the scene uses one, so the address is what says which
    /// set belongs to the scene's own world (`anim/driver/wire.rs`).
    pub address: u64,
    pub drivers: Vec<BWorldDriver>,
    /// The input sockets of each node a driver writes, in storage order.
    pub inputs: Vec<(String, Vec<BDriverSocket>)>,
}

impl BWorldDrivers {
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
mod tests {
    use super::*;
    use crate::blend::anim::driver::BDriverState;

    fn sockets(linked: bool) -> Vec<(String, Vec<BDriverSocket>)> {
        vec![(
            BACKGROUND.to_string(),
            vec![
                BDriverSocket { name: "Color".into(), rgba: true, linked },
                BDriverSocket { name: "Strength".into(), rgba: false, linked: false },
            ],
        )]
    }

    fn sock(input: usize, component: usize) -> BWorldWrite {
        BWorldWrite::Socket { node: BACKGROUND.to_string(), input, component }
    }

    /// A driver that is just a constant: no variables, so a Sum comes to
    /// 0, and one key at x = 0 maps that 0 to `value`.
    fn constant(value: f32) -> BlendDriver {
        use crate::blend::anim::fcurve::{BFCurve, BKey, IPO_CONST};
        let key = BKey::new([0.0, value], [0.0, value], [0.0, value], IPO_CONST);
        BlendDriver {
            kind: crate::blend::rig::driver::DRIVER_SUM,
            expression: String::new(),
            vars: Vec::new(),
            parsed: None,
            curve: BFCurve::new("", 0, vec![key]),
        }
    }

    fn state() -> crate::blend::anim::driver::BDriverState {
        BDriverState::default()
    }

    #[test]
    fn the_background_colour_and_strength_land_on_the_world() {
        let set = BWorldDrivers {
            inputs: sockets(false),
            drivers: vec![
                BWorldDriver { write: sock(BG_COLOR, 0), driver: constant(0.25) },
                BWorldDriver { write: sock(BG_COLOR, 2), driver: constant(0.75) },
                BWorldDriver { write: sock(BG_STRENGTH, 0), driver: constant(3.0) },
                // A driver on `World.color` is kept and does not land on
                // `BlendWorld::color`, which is the Background's.
                BWorldDriver { write: BWorldWrite::Color(0), driver: constant(0.9) },
            ],
            ..Default::default()
        };
        let st = state();
        let ds = st.scene();
        assert_eq!(set.values(&ds, 1.0).len(), 4, "all four are read");

        let mut world = BlendWorld::default();
        set.apply(&mut world, &ds, 1.0);
        assert_eq!(world.color, [0.25, 0.05, 0.75], "the Background's Color, component by component");
        assert_eq!(world.strength, 3.0);
    }

    #[test]
    fn a_link_into_the_background_keeps_the_world_where_it_was() {
        let set = BWorldDrivers {
            inputs: sockets(true),
            drivers: vec![BWorldDriver { write: sock(BG_COLOR, 0), driver: constant(0.9) }],
            ..Default::default()
        };
        let st = state();
        let ds = st.scene();
        let mut world = BlendWorld::default();
        set.apply(&mut world, &ds, 1.0);
        assert_eq!(world.color, [0.05; 3], "an Environment Texture feeds Color, not the driver");
    }

    #[test]
    fn a_strength_socket_takes_a_negative_and_a_colour_does_not() {
        let set = BWorldDrivers { inputs: sockets(false), ..Default::default() };
        assert_eq!(socket_clamp(set.sockets(BACKGROUND), BG_STRENGTH, -2.0), -2.0, "a float socket is unclamped");
        assert_eq!(socket_clamp(set.sockets(BACKGROUND), BG_COLOR, -2.0), 0.0, "an RGBA socket clamps at 0");
        assert_eq!(socket_clamp(set.sockets(BACKGROUND), BG_COLOR, 4.0), 4.0, "and is free above 1");
    }

    #[test]
    fn the_worlds_own_colour_clamps_at_zero_and_is_free_above_one() {
        // `World.color` is hard 0..∞, like every colour property.
        assert_eq!((-1.0f32).max(0.0), 0.0);
        assert_eq!(3.0f32.max(0.0), 3.0);
    }
}
