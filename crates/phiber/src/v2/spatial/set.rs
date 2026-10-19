//! The ladder — data only; what it means and how it is climbed is in
//! `mod.rs` and `tier.rs`.

use super::BSpatialTier;

/// What a spatial frame is allowed to take: 60 frames a second.
///
/// Not a round 16: the budget is `1000 / 60`, and rounding it down to 16
/// would reject a frame that in fact made 60 fps.
pub const BUDGET_MS: f32 = 1000.0 / 60.0;

/// A frame must fit in this fraction of the budget before the ladder will
/// climb back up — the hysteresis that stops it oscillating between two
/// tiers on a scene sitting right at the boundary.
///
/// Half, because a step up doubles the pixels: a frame at 0.5 × budget is
/// the one with room for the tier above it, and nothing closer is.
pub const CLIMB: f32 = 0.5;

/// The tiers, finest first.
///
/// The divisors are Blender's own `pixel_size_items` (`rna_scene.cc:6922`:
/// 1×, 2×, 4×, 8×, described there as full, 50%, 25% and 12.5%
/// resolution). Powers of two, so a tier change resamples cleanly and the
/// image does not shimmer as the ladder moves.
///
/// Samples fall with resolution rather than rising to compensate. That is
/// deliberate and it is the opposite of what a still render wants: when a
/// spatial view is in trouble it is because the camera is moving, and a
/// moving frame hides noise far better than it hides a stale frame. The
/// finest tier is the one that gets the samples, because that is the tier
/// a view settles on when the camera stops.
pub static TIER: &[BSpatialTier] = &[
    BSpatialTier { name: "Full", pixel: 1, samples: 4 },
    BSpatialTier { name: "Half", pixel: 2, samples: 2 },
    BSpatialTier { name: "Quarter", pixel: 4, samples: 1 },
    BSpatialTier { name: "Eighth", pixel: 8, samples: 1 },
];
