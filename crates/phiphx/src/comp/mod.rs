//! The scene's compositor, read and run.
//!
//! Blender renders, then puts the result through the scene's compositor node
//! tree before anything sees it: the Image editor's Render Result *is* the
//! composite's output, not the raw render. phiber drew those nodes from
//! batch 142 and ran none of them, so a file whose look came out of the
//! compositor — a grade, an exposure trim, an alpha-over of two layers —
//! rendered here as the ungraded render, and looked like a lighting bug.
//!
//! * [`node`] — which node a `bNode` is, and the settings it carries.
//! * [`read`] — the tree out of the file, both spellings of where it lives.
//! * [`tree`] — the nodes in the order they run.
//! * [`pixel`] — the per-pixel operations, each ported from its node's own
//!   `node_build_multi_function`.
//! * [`eval`] — the tree resolved to a straight-line program and run.
//!
//! What runs is every node whose output at a pixel depends on its inputs at
//! that same pixel. What does not is listed by [`tree::BCompTree::declined`]
//! rather than quietly skipped: the neighbourhood filters (Blur, Glare,
//! Defocus, Denoise, Dilate/Erode, Kuwahara, Despeckle), the transforms
//! (Translate, Scale, Rotate, Transform, Crop, Flip, Lens Distortion), the
//! keyers, the curve and balance nodes that need a `CurveMapping`, and
//! anything reading a second datablock (Mask, Movie Clip, Cryptomatte).

pub mod eval;
pub mod node;
pub mod pixel;
pub mod read;
pub mod tree;

pub use eval::BCompPlan;
pub use node::{BCompKind, BCompSetting};
pub use pixel::BCompPixel;
pub use read::R_DOCOMP;
pub use tree::BCompTree;
