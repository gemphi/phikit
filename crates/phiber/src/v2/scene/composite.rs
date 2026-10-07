//! The scene's compositor over a finished render.
//!
//! Blender runs the compositor as part of the **render pipeline**, not as
//! part of a viewport's shading: F12 renders, the result goes through the
//! scene's node tree, and what the Image editor shows is the composite's
//! output. The Rendered viewport shows the raw render. So this is wired
//! into the still path (`view3d/rendered/still.rs`) and deliberately not
//! into the viewport's worker.
//!
//! And it runs on the render's **scene-linear** pixels, before the view
//! transform. Compositing the displayed pixels instead would put a grade
//! after AgX rather than before it — plausible-looking and wrong — so the
//! tracer's own display step is re-run over the composited radiance
//! (`Tracer::show`, `phiray`'s `show.wgsl`) rather than the transform being
//! implemented a second time on the CPU.

use phiphx::comp::BCompTree;

/// What a composite did, for the journal and for a panel to report.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BCompositeRun {
    /// How many nodes ran.
    pub ran: usize,
    /// The `idname` of each node that reached the output and was not run
    /// (`BCompTree::declined`) — named rather than silently skipped.
    pub declined: Vec<String>,
}

impl BCompositeRun {
    /// Whether anything happened.
    pub fn any(&self) -> bool {
        self.ran > 0
    }

    /// One line for the journal.
    pub fn line(&self) -> String {
        match self.declined.is_empty() {
            true => format!("Compositor: {} nodes over the render", self.ran),
            false => format!("Compositor: {} nodes over the render; {} not run ({})", self.ran, self.declined.len(), self.declined.join(", ")),
        }
    }
}

/// Put a scene's compositor over a finished render.
pub trait BComposite {
    /// Run `tree` over this image's scene-linear pixels and show the
    /// result, so the bytes, the display floats and the radiance all come
    /// back composited. Does nothing when the tree is switched off or
    /// nothing reaches its output.
    fn composite(&mut self, gpu: &phiray::Gpu, tree: &BCompTree) -> BCompositeRun;
}

impl BComposite for phiray::Tracer {
    fn composite(&mut self, gpu: &phiray::Gpu, tree: &BCompTree) -> BCompositeRun {
        let Some(plan) = tree.plan() else { return BCompositeRun::default() };
        let mut linear: Vec<f32> = self.radiance(gpu).iter().flat_map(|c| [c[0], c[1], c[2], 1.0]).collect();
        let (w, h) = self.size();
        plan.apply(w, h, &mut linear);
        let out: Vec<[f32; 3]> = linear.chunks_exact(4).map(|c| [c[0], c[1], c[2]]).collect();
        if !self.show(gpu, &out) {
            return BCompositeRun::default();
        }
        BCompositeRun { ran: plan.len(), declined: tree.declined() }
    }
}

/// The loaded file's compositor tree, when it has one that would run.
pub fn tree() -> Option<BCompTree> {
    super::source::with_loaded(|m| m.and_then(|m| m.compositor.clone())).filter(BCompTree::runs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_run_says_what_it_did_and_what_it_left() {
        let quiet = BCompositeRun::default();
        assert!(!quiet.any());
        let ran = BCompositeRun { ran: 4, declined: Vec::new() };
        assert_eq!(ran.line(), "Compositor: 4 nodes over the render");
        let partly = BCompositeRun { ran: 4, declined: vec!["CompositorNodeBlur".into()] };
        assert!(partly.any());
        assert_eq!(partly.line(), "Compositor: 4 nodes over the render; 1 not run (CompositorNodeBlur)");
    }
}
