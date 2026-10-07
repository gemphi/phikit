//! Running the tree over an image.
//!
//! Every node in scope is per-pixel (`node.rs`), so the whole tree is a
//! straight-line program: the nodes in dependency order, each reading its
//! inputs' results at the same pixel. [`BCompPlan`] resolves the graph's
//! names to indices **once** — the graph is searched by string, and doing
//! that per pixel turned a 1080p frame into minutes — and then each pixel is
//! a loop over a few dozen floats.
//!
//! A float output feeding a colour input is broadcast to grey with alpha 1,
//! which is what Blender's implicit socket conversion does, and a colour
//! feeding a float input takes its first channel.

use super::node::{sock, BCompKind, BCompSet, BCompSetting};
use super::pixel::{self, BCompPixel};
use super::tree::BCompTree;
use crate::blend::shade::graph::BlendNode;

/// The tree resolved to a straight-line program.
pub struct BCompPlan<'a> {
    step: Vec<BCompPlanStep<'a>>,
}

struct BCompPlanStep<'a> {
    node: &'a BlendNode,
    kind: BCompKind,
    /// What `bNode.storage` said (`read.rs`).
    set: BCompSet,
    /// Input socket identifier → the step whose result feeds it.
    from: Vec<(String, usize)>,
}

impl BCompTree {
    /// This tree as a program, or `None` when it would not change a render.
    pub fn plan(&self) -> Option<BCompPlan<'_>> {
        if !self.runs() {
            return None;
        }
        let order = self.order();
        let at = |name: &str| order.iter().position(|s| s.node.name == name);
        let step = order
            .iter()
            .map(|s| BCompPlanStep {
                node: s.node,
                kind: s.kind,
                set: self.set(&s.node.name),
                from: s
                    .node
                    .inputs
                    .iter()
                    .filter(|sock| sock.available())
                    .filter_map(|sock| {
                        let from = self.feeding(&s.node.name, &sock.identifier)?;
                        Some((sock.identifier.clone(), at(&from.name)?))
                    })
                    .collect(),
            })
            .collect();
        Some(BCompPlan { step })
    }
}

impl BCompPlan<'_> {
    /// How many nodes run.
    pub fn len(&self) -> usize {
        self.step.len()
    }

    pub fn is_empty(&self) -> bool {
        self.step.is_empty()
    }

    /// The colour the tree's output ends up with, given the render's pixel.
    pub fn pixel(&self, render: BCompPixel) -> BCompPixel {
        let mut out: Vec<BCompPixel> = Vec::with_capacity(self.step.len());
        for step in &self.step {
            let value = self.one(step, render, &out);
            out.push(value);
        }
        out.last().copied().unwrap_or(render)
    }

    /// Run the tree over `px` in place: `w × h` straight RGBA quadruples,
    /// row 0 first. Anything past `w * h` is left alone.
    pub fn apply(&self, w: u32, h: u32, px: &mut [f32]) {
        let n = (w as usize * h as usize).min(px.len() / 4);
        for i in 0..n {
            let at = i * 4;
            let was = [px[at], px[at + 1], px[at + 2], px[at + 3]];
            let now = self.pixel(was);
            px[at..at + 4].copy_from_slice(&now);
        }
    }

    /// One step's colour.
    fn one(&self, step: &BCompPlanStep<'_>, render: BCompPixel, done: &[BCompPixel]) -> BCompPixel {
        let node = step.node;
        let set = step.set;
        // A linked input, else the value on the socket, else `fallback`.
        let linked = |id: &[&str]| {
            id.iter()
                .find_map(|want| step.from.iter().find(|(s, _)| s == want).map(|(_, k)| done[*k]))
        };
        let color = |id: &[&str], fallback: BCompPixel| linked(id).or_else(|| node.color(id)).unwrap_or(fallback);
        let num = |id: &[&str], fallback: f32| linked(id).map_or_else(|| node.number_or(id, fallback), |c| c[0]);
        let image = || color(sock::IMAGE, render);
        // A muted node is a wire: Blender runs its first input straight to
        // its first output (`NODE_MUTED`).
        if node.muted() && !step.kind.source() {
            return image();
        }
        match step.kind {
            BCompKind::RenderLayer | BCompKind::Image => render,
            BCompKind::Constant => self.constant(node),
            BCompKind::Composite | BCompKind::Viewer | BCompKind::GroupOutput | BCompKind::Reroute => {
                // The output's socket is named "Image" but identified
                // `Socket_1` in a 5.x group; take whatever is linked.
                linked(&[]).or_else(|| step.from.first().map(|(_, k)| done[*k])).unwrap_or(render)
            }
            BCompKind::BrightContrast => pixel::bright_contrast(image(), num(sock::BRIGHT, 0.0), num(sock::CONTRAST, 0.0)),
            BCompKind::Exposure => pixel::exposure(image(), num(sock::EXPOSURE, 0.0)),
            BCompKind::Gamma => pixel::gamma(image(), num(sock::GAMMA, 1.0)),
            BCompKind::Invert => pixel::invert(
                image(),
                num(sock::FAC, 1.0),
                self.invert_channel(node, 0),
                self.invert_channel(node, 1),
            ),
            BCompKind::HueSat => pixel::hue_sat(
                image(),
                num(sock::HUE, 0.5),
                num(sock::SATURATION, 1.0),
                num(sock::VALUE, 1.0),
                num(sock::FAC, 1.0),
            ),
            BCompKind::Posterize => pixel::posterize(image(), num(sock::STEPS, 8.0)),
            BCompKind::RgbToBw => {
                let v = pixel::rgb_to_bw(image());
                [v, v, v, 1.0]
            }
            BCompKind::SetAlpha => pixel::set_alpha(image(), num(sock::ALPHA, 1.0), self.menu(node, sock::SET_ALPHA_TYPE, pixel::SET_ALPHA_APPLY)),
            BCompKind::AlphaConvert => pixel::alpha_convert(image(), self.menu(node, sock::ALPHA_MAPPING, pixel::ALPHA_PREMULTIPLY)),
            BCompKind::AlphaOver => pixel::alpha_over(
                image(),
                color(sock::IMAGE_2, image()),
                num(sock::FAC, 1.0),
                // Straight Alpha: off by default, and off means the
                // foreground is already premultiplied and goes over as it
                // is. Measured against Blender's own render — premultiplying
                // by default put a foreground over black at 0.6 of itself.
                num(sock::STRAIGHT, 0.0) != 0.0,
                self.menu(node, sock::OVER_OPERATION, pixel::OVER_PLAIN),
            ),
            BCompKind::Mix => pixel::mix(
                set.blend,
                num(sock::FAC, 0.5),
                image(),
                color(sock::IMAGE_2, render),
                set.clamp || num(sock::CLAMP, 0.0) != 0.0,
            ),
            BCompKind::Declined => image(),
        }
    }

    /// An RGB or Value node's constant: the value is on its *output*
    /// socket, not on an input row.
    fn constant(&self, node: &BlendNode) -> BCompPixel {
        use crate::blend::shade::socket::BSocketDefault;
        match node.outputs.first().and_then(|s| s.default.as_ref()) {
            Some(BSocketDefault::Color(c)) => *c,
            Some(BSocketDefault::Float(v)) => [*v, *v, *v, 1.0],
            _ => [0.0, 0.0, 0.0, 1.0],
        }
    }

    /// Invert's two switches. 5.x carries them as boolean sockets; before
    /// that they were `custom1`'s bits — `CMP_CHAN_RGB` is 1 and
    /// `CMP_CHAN_A` is 2, and a new node sets **only** `CMP_CHAN_RGB`, so
    /// the alpha is left alone until the user asks for it. A reader that
    /// defaults both on inverts an alpha Blender does not, which is what
    /// the truth test caught.
    fn invert_channel(&self, node: &BlendNode, bit: u32) -> bool {
        let name = if bit == 0 { sock::INVERT_COLOR } else { sock::INVERT_ALPHA };
        match node.number(name) {
            Some(v) => v != 0.0,
            None if node.custom1 == 0 => bit == 0,
            None => node.custom1 & (1 << bit) != 0,
        }
    }

    /// A menu 5.x keeps as a socket and earlier files keep in `custom1`.
    fn menu(&self, node: &BlendNode, name: &[&str], fallback: i32) -> i32 {
        match node.number(name) {
            Some(v) => v as i32,
            None if node.custom1 != 0 => node.custom1,
            None => fallback,
        }
    }
}
