//! An armature's deform matrices, read once per armature.
//!
//! Every mesh an armature deforms carries its own Armature modifier, and
//! reading one used to read the *whole rig*: `armature::read` called
//! [`BlendFile::pose_deform_mats`], which walks every pose channel and every
//! bone of the armature it points at. A dressed figure has forty-odd meshes
//! on one rig, and the stack is built twice over an open — once by
//! `modifier_jobs` for the evaluated meshes, once by `skin_sources` for the
//! ones the app re-poses — so the same few hundred bones were read eighty
//! times. Measured on `FLORIS_BOLDRIOD_RIG.blend`: **84 reads, 144 ms of a
//! 213 ms open**, the largest single cost in the whole file read.
//!
//! Blender does not do this. `BKE_modifier_deform_verts` for an Armature
//! modifier reads `md->object->pose`, which the depsgraph evaluated once;
//! the modifier only points at it. This is that: the read keyed by the
//! armature's own block address, so forty modifiers share one.
//!
//! The memo is sound because the read is a pure function of its armature
//! and the DNA. Both halves of it — `pose_shapes` and the channel walk —
//! open with `self.scoped(arm, …)` (`file/scope.rs`), so neither depends on
//! which object's scope was ambient when the modifier was read; and the DNA
//! does not change for the life of a [`BlendFile`].

use super::armature::BDeformBone;
use crate::blend::file::types::*;
use std::collections::HashMap;
use std::sync::Arc;

/// An armature's deforming bones by name — what an Armature modifier reads.
pub(crate) type BDeformMats = HashMap<String, BDeformBone>;

impl BlendFile {
    /// [`Self::pose_deform_mats`] for `arm`, read once and shared.
    ///
    /// The lock is taken to look and taken again to store, never held
    /// across the read: holding it would serialise a parallel caller behind
    /// a file read, and the read re-enters the file. Two callers racing on
    /// a cold entry both read and the second's store wins, which costs one
    /// duplicate read and is still correct — the two results are equal.
    pub(crate) fn deform_mats(&self, arm: &BlendBlock, dna: &DnaType) -> Arc<BDeformMats> {
        let key = arm.old_address;
        if let Some(hit) = self.deform_memo.lock().ok().and_then(|seen| seen.get(&key).cloned()) {
            return hit;
        }
        let made = Arc::new(self.pose_deform_mats(arm, dna));
        if let Ok(mut seen) = self.deform_memo.lock() {
            seen.insert(key, Arc::clone(&made));
        }
        made
    }
}

#[cfg(test)]
#[path = "deform_tests.rs"]
mod deform_tests;
