//! The extras last built, and what for: the viewport, its picks and the
//! extras list all ask every frame, and a rig is a few thousand lines, so
//! [`scene_extras_now`](super::scene_extras_now) keeps one list until the
//! frame, the rigs' pose or the session's edits change.

use crate::phiphx::scene::SceneModel;
use super::BExtraLine;

/// What the extras were last built for: the model, the frame in scope, the
/// frame the rigs were posed at, and the session's edits and transforms.
#[derive(Clone)]
pub(super) struct BExtrasKey {
    model: std::sync::Arc<SceneModel>,
    at: (Option<f32>, Option<f32>, u64),
}

impl PartialEq for BExtrasKey {
    fn eq(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.model, &other.model) && self.at == other.at
    }
}

/// The extras last built, and what for.
pub(super) struct BExtrasMemo;

thread_local! {
    static MEMO: std::cell::RefCell<Option<(BExtrasKey, std::rc::Rc<Vec<BExtraLine>>)>> = const { std::cell::RefCell::new(None) };
}

impl BExtrasMemo {
    pub(super) fn key(s: &crate::v2::state::BState, model: std::sync::Arc<SceneModel>) -> BExtrasKey {
        let posed = crate::v2::scene::pose::BPosed::shown(s).frame;
        let edits = crate::v2::scene::edit::BSceneEdit::now(s).fingerprint() ^ crate::v2::scene::transform::BTransforms::fingerprint(s).rotate_left(17)
            ^ Self::resolution().rotate_left(41);
        BExtrasKey { model, at: (crate::v2::scene::motion::scoped_frame(), posed, edits) }
    }

    /// The camera shapes follow the panel's resolution (`render_set`).
    fn resolution() -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        crate::v2::scene::render_set::BResolution::scoped().hash(&mut h);
        h.finish()
    }

    pub(super) fn get(key: &BExtrasKey) -> Option<std::rc::Rc<Vec<BExtraLine>>> {
        MEMO.with(|m| m.borrow().as_ref().filter(|(k, _)| k == key).map(|(_, lines)| std::rc::Rc::clone(lines)))
    }

    pub(super) fn put(key: BExtrasKey, lines: std::rc::Rc<Vec<BExtraLine>>) {
        MEMO.with(|m| *m.borrow_mut() = Some((key, lines)));
    }
}
