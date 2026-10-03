//! `Aspect` — the component of this ECS.
//!
//! Apple writes `impl<T: Send + Sync + 'static> Component for T` (a blanket
//! impl, `reality-foundation/component.rs:21`), which makes every type a
//! component and, by coherence, makes its `on_attach` / `on_detach` hooks
//! impossible to override — they are dead code in that crate. Here the impl is
//! explicit, so the hooks are real and a type is an aspect only when its
//! author says it is.

use crate::word::Word;

/// Something a word can carry: a phasor, a condition, a count.
pub trait Aspect: Send + Sync + 'static {
    /// Called after this aspect is bound to `word`.
    fn on_bind(&self, _word: Word) {}

    /// Called before it is taken off again.
    fn on_unbind(&self, _word: Word) {}
}
