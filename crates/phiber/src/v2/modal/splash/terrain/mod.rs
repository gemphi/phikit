//! Six terrains, traced off the reference frames — the types.
//!
//! The curves themselves are `set.rs`, as every level here keeps its
//! content beside its structure.
//!
//! Not invented and not noise-generated: each curve is the **top edge of a
//! colour band** in one frame of the landscape fly-through, sampled at 64
//! points across the frame and normalised so 0 is the top of the frame and
//! 1 the bottom. The frames are in
//! `Downloads/phiber/this-is-phiber-topo-flesh-and-welcome-video`.
//!
//! The extraction was looked at, not assumed. Drawing the first pass back
//! over the originals showed three things, and each became a rule: the sky
//! is a band whose top edge is the frame's own (dropped); a hill's dapple
//! breaks into bands whose edge zigzags, where a ridge is smooth (dropped
//! on roughness); and a band that goes behind a nearer ridge leaves a gap
//! whose fill reads as a cliff (dropped on step size).
//!
//! A fourth rule came from checking the first three rather than trusting
//! them: a terrain is **layered**, so a frame with one band against the
//! sky is a horizon and not a landscape. One of the frames is a room from
//! the keynote; it has exactly one band and leaves here. It had already
//! been excluded by ranking, which is not the same as being excluded by a
//! rule — the rule is what makes it stay out of a re-extraction.
//!
//! `rgb` is the band's own mean colour, scene-linear 0..1, so a painter
//! can lay the ridges back down in the colours they were traced from and
//! get the haze for free: the far bands are already the paler, bluer ones.

/// Samples across one ridge — the width every curve here is stored at.
pub const SAMPLES: usize = 64;

/// One ridge: its silhouette and the colour of the land below it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BRidge {
    /// Height at each of [`SAMPLES`] points, 0 at the frame's top edge.
    pub curve: [f32; SAMPLES],
    /// The band's mean colour, linear RGB.
    pub rgb: [f32; 3],
}

/// One terrain: its ridges, far to near.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BTerrain {
    /// The frame it was traced from.
    pub name: &'static str,
    /// Its ridges, far to near.
    pub ridges: &'static [BRidge],
}

/// The ridges of every terrain, far to near (`set.rs`).
pub use set::TERRAIN;

/// The traced curves.
pub mod set;

#[cfg(test)]
mod tests;
