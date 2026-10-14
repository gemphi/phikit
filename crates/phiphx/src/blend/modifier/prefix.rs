//! The mesh a stack has reached just before its Armature modifier, computed
//! once and handed to whichever of the two consumers asks second.
//!
//! A mesh an Armature deforms is built twice over an open, and the two are
//! the same mesh:
//!
//! * [`super::BlendFile::modifier_jobs`] captures the **whole** stack for
//!   every mesh object, and [`super::apply_eval_jobs`] applies it — the
//!   evaluated mesh the viewport draws at the saved pose.
//! * [`super::BlendFile::skin_sources`] splits the stack at the Armature and
//!   applies the part **before** it, keeping that as `base` so the rig can
//!   pose the mesh at any frame (`scene/rig.rs`).
//!
//! Everything before the Armature is therefore evaluated twice from the same
//! vertices, with the same modifiers, under the same matrices — and on a
//! dressed figure that prefix is a Subdivision Surface per garment. Measured
//! on `FLORIS_BOLDRIOD_RIG.blend`: 28 of 32 armature-deformed meshes have a
//! non-empty prefix, 39 modifiers in all, and `skin_sources` spent 20.7 ms
//! re-applying what `apply_eval_jobs` had already applied inside its 39.8 ms.
//!
//! Blender has no equivalent step because it does not have the two
//! consumers: the depsgraph evaluates each object's stack once and the pose
//! comes from the armature it already evaluated.
//!
//! So this is a drop box, not a cache: each entry is **put by whichever
//! consumer runs first and taken by the other**, which keeps the ordering out
//! of it. An eager open applies the stack during the read, before the rigs
//! are read, so `apply_eval_jobs` puts and `skin_sources` takes; a
//! progressive open defers the stack to a worker thread, so the rigs are read
//! first and it goes the other way. Taking removes the entry, so a
//! subdivided mesh is not held once its second reader has it, and nothing has
//! to clear the box afterwards.

use super::BlendEvalMesh;
use std::collections::HashMap;
use std::sync::Mutex;

/// What makes two prefixes the same prefix.
///
/// The object and the number of modifiers are the obvious part. `weights` is
/// here because the two consumers decide separately whether to load the
/// mesh's vertex groups — `skin_sources` always does, `modifier_jobs` only
/// when some modifier in the stack asks (`wants_weights`) — and a mesh whose
/// groups were never loaded cannot be skinned by them. Today an Armature
/// always asks, so the flag is always true on both sides; it is in the key so
/// that a modifier which stops asking makes the keys miss rather than hand
/// the rig a mesh with no weights in it.
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub(crate) struct BPrefixKey {
    /// Index into the read scene's objects.
    pub object: usize,
    /// How many modifiers were applied — the Armature's own position in the
    /// stack. `PHIPHX_MODIFIERS` truncates one consumer's stack and not the
    /// other's, so this is also what keeps a truncated run from matching.
    pub applied: usize,
    /// Whether the mesh carries vertex-group weights.
    pub weights: bool,
}

/// The meshes waiting for their second reader.
#[derive(Default)]
pub(crate) struct BPrefixBox {
    held: Mutex<HashMap<BPrefixKey, BlendEvalMesh>>,
}

impl BPrefixBox {
    /// Leave `mesh` for the other consumer. Called by whichever runs first;
    /// the mesh is cloned because the caller goes on using it.
    ///
    /// A second put under the same key would mean a third consumer appeared,
    /// which is a bug rather than a cache miss — it keeps the first, since
    /// that is the one its partner is waiting for.
    pub(crate) fn put(&self, key: BPrefixKey, mesh: &BlendEvalMesh) {
        let mut held = self.held.lock().unwrap_or_else(|e| e.into_inner());
        held.entry(key).or_insert_with(|| mesh.clone());
    }

    /// Take the mesh the other consumer left, if it ran first. Removes it:
    /// each prefix has exactly two readers, so once the second has it nothing
    /// else wants it and a subdivided mesh should not sit in memory.
    pub(crate) fn take(&self, key: BPrefixKey) -> Option<BlendEvalMesh> {
        let mut held = self.held.lock().unwrap_or_else(|e| e.into_inner());
        held.remove(&key)
    }

    /// How many meshes are waiting — for the probe in `prefix_tests.rs`, and
    /// so a leak shows up as a number rather than as memory.
    #[cfg(test)]
    pub(crate) fn waiting(&self) -> usize {
        self.held.lock().unwrap_or_else(|e| e.into_inner()).len()
    }
}

impl std::fmt::Debug for BPrefixBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let n = self.held.try_lock().map(|h| h.len());
        match n {
            Ok(n) => write!(f, "BPrefixBox {{ waiting: {n} }}"),
            Err(_) => write!(f, "BPrefixBox {{ locked }}"),
        }
    }
}

/// Whether the two consumers share a prefix at all.
///
/// `PHIPHX_NOSHARE=1` turns the sharing off, so both apply the prefix as they
/// did before this module existed. It is how the saving is measured — one
/// binary, the switch flipped, the same machine state
/// (`docs/blender-head-to-head.md`) — and how a suspected difference in an
/// evaluated mesh is bisected against the old path. Nothing in the app sets
/// it, as with `PHIPHX_MODIFIERS`.
///
/// Read once: the consumers ask per object, and this is a `getenv` behind a
/// lock. It is deliberately **not** inside [`BPrefixBox`], so the box stays a
/// plain data structure whose own tests cannot be perturbed by a variable
/// left in someone's environment.
pub(crate) fn sharing() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("PHIPHX_NOSHARE").is_none())
}

/// Where a stack splits for skinning: the position of the first Armature
/// modifier, which is the one `skin_sources` splits at. Both consumers ask
/// this so they cannot disagree about where the prefix ends.
pub(crate) fn split_at(stack: &[Box<dyn super::BModifier>]) -> Option<usize> {
    stack.iter().position(|m| m.armature_source().is_some())
}
