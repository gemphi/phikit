//! Something made once per mesh and kept while the mesh lives — a pick tree
//! (`pick/tree.rs`), the extreme points of an outline (`mesh/extreme.rs`) —
//! by the mesh's address, shared across threads so a file's meshes can be
//! prepared as it loads (`warm.rs`) and found by the UI thread after.
//!
//! The lock is held for the lookup and the insert, never for the making:
//! a two-million-triangle tree takes a second in a debug build, and the UI
//! thread must not wait on another thread's.

use crate::phiphx::scene::SceneMesh;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};

/// The things of one kind, by the mesh each was made for.
pub struct BMeshKeep<T> {
    /// Made on first use: a `HashMap` cannot be built in a `const`.
    kept: Mutex<Option<HashMap<usize, (Weak<SceneMesh>, Arc<T>)>>>,
}

impl<T> BMeshKeep<T> {
    pub const fn new() -> Self {
        Self { kept: Mutex::new(None) }
    }

    /// The thing of `mesh`, made with `make` once and kept while the mesh
    /// lives. Two threads asking at once for a mesh no one has made yet each
    /// make it; the first to finish is the one kept.
    pub fn of(&self, mesh: &Arc<SceneMesh>, make: impl FnOnce(&Arc<SceneMesh>) -> T) -> Arc<T> {
        let key = Arc::as_ptr(mesh) as usize;
        if let Some(found) = self.found(key, mesh) {
            return found;
        }
        let made = Arc::new(make(mesh));
        let mut kept = self.kept.lock().unwrap_or_else(|e| e.into_inner());
        let kept = kept.get_or_insert_with(HashMap::new);
        if let Some((weak, first)) = kept.get(&key)
            && weak.ptr_eq(&Arc::downgrade(mesh))
        {
            return Arc::clone(first);
        }
        // Meshes gone from the scene leave their entries behind: drop them.
        kept.retain(|_, (weak, _)| weak.strong_count() > 0);
        kept.insert(key, (Arc::downgrade(mesh), Arc::clone(&made)));
        made
    }

    /// Whether `mesh` has its thing already.
    pub fn has(&self, mesh: &Arc<SceneMesh>) -> bool {
        self.found(Arc::as_ptr(mesh) as usize, mesh).is_some()
    }

    fn found(&self, key: usize, mesh: &Arc<SceneMesh>) -> Option<Arc<T>> {
        let kept = self.kept.lock().unwrap_or_else(|e| e.into_inner());
        kept.as_ref()?.get(&key).filter(|(weak, _)| weak.ptr_eq(&Arc::downgrade(mesh))).map(|(_, t)| Arc::clone(t))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phiphx::scene::SceneNormals;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn mesh(n: usize) -> Arc<SceneMesh> {
        Arc::new(SceneMesh { vertices: vec![[0.0; 3]; n], indices: Vec::new(), normals: SceneNormals::PerFace(Vec::new()), corners: Default::default() })
    }

    #[test]
    fn made_once_per_living_mesh_and_dropped_with_it() {
        static KEEP: BMeshKeep<usize> = BMeshKeep::new();
        let made = AtomicUsize::new(0);
        let make = |m: &Arc<SceneMesh>| {
            made.fetch_add(1, Ordering::Relaxed);
            m.vertices.len()
        };
        let a = mesh(3);
        assert_eq!(*KEEP.of(&a, make), 3);
        assert_eq!(*KEEP.of(&a, make), 3);
        assert!(KEEP.has(&a));
        assert_eq!(made.load(Ordering::Relaxed), 1, "made once");
        drop(a);
        // A new mesh at the same address (likely, after the drop) is not the
        // old one's thing.
        let b = mesh(5);
        assert!(!KEEP.has(&b));
        assert_eq!(*KEEP.of(&b, make), 5);
        assert_eq!(made.load(Ordering::Relaxed), 2);
        // Found from another thread.
        let c = Arc::clone(&b);
        assert_eq!(std::thread::spawn(move || *KEEP.of(&c, |_| unreachable!("kept"))).join().unwrap(), 5);
    }
}
