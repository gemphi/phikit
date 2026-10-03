//! The Armature constraint (`armdef_evaluate`): the owner moved as if
//! skinned to its target bones by their weights — each bone's deform
//! (`chan_mat`, pose · rest⁻¹) blended linearly and applied on top.
//! Not yet: dual quaternions (Preserve Volume), envelopes, and a B-Bone
//! target's segments (its whole-bone deform is used).

use super::{BCob, BTarget};
use glam::Mat4;

impl BCob {
    /// `targets` with this frame's `weights` (drivers set them); `deform`
    /// a target bone's deform in world space.
    pub(super) fn armature(cob: &mut BCob, targets: &[(BTarget, f32)], weights: &[f32], deform: impl Fn(&BTarget) -> Option<Mat4>) {
        let mut sum = Mat4::ZERO;
        let mut total = 0.0;
        for (i, (target, _)) in targets.iter().enumerate() {
            let weight = weights.get(i).copied().unwrap_or(0.0);
            if weight <= 0.0 {
                continue;
            }
            // A target that is not a bone ends the constraint.
            let Some(m) = deform(target) else { return };
            sum += m * weight;
            total += weight;
        }
        if total > 0.0 {
            let blended = sum * (1.0 / total);
            cob.matrix = blended * cob.matrix;
        }
    }
}
