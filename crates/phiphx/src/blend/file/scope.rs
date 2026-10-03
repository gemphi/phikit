//! Per-ID pointer resolution (Blender `readfile.cc` `fd->datamap`).
//!
//! Up to 4.x every saved pointer was a unique memory address, so one global
//! OldNewMap sufficed. Blender 5.x writes *generated* addresses
//! (`BLO_write_generated_pointer_tag`, implicitly shared arrays) that repeat
//! across IDs, and resolves DATA pointers only among the blocks written
//! after their owning ID. We mirror that: while an ID's data is read inside
//! [`BlendFile::scoped`], a repeated address resolves to the first copy
//! written after that ID block.

use crate::blend::file::types::*;
use std::sync::atomic::Ordering;

impl BlendFile {
    /// Position of `block` inside `self.blocks` (it must come from there).
    pub(crate) fn index_of(&self, block: &BlendBlock) -> Option<usize> {
        let size = std::mem::size_of::<BlendBlock>();
        let base = self.blocks.as_ptr() as usize;
        let at = block as *const BlendBlock as usize;
        let i = at.checked_sub(base)? / size;
        (i < self.blocks.len() && std::ptr::eq(&self.blocks[i], block)).then_some(i)
    }

    /// Run `f` with pointer lookups scoped to `owner`'s data; restores the
    /// previous scope afterwards (scopes nest: object → mesh → material).
    pub(crate) fn scoped<R>(&self, owner: &BlendBlock, f: impl FnOnce() -> R) -> R {
        let Some(i) = self.index_of(owner) else { return f() };
        let prev = self.scope.swap(i, Ordering::Relaxed);
        let out = f();
        self.scope.store(prev, Ordering::Relaxed);
        out
    }

    /// For a repeated address: the first copy after the current scope's ID.
    pub(crate) fn scoped_candidate(&self, addr: u64) -> Option<usize> {
        let all = self.dup_index.get(&addr)?;
        let scope = self.scope.load(Ordering::Relaxed);
        if scope == usize::MAX {
            return None;
        }
        all.iter().copied().find(|&i| i > scope)
    }
}
