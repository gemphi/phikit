//! Motion played from code: what a rig's or an object's own animation
//! would give at a frame, supplied by something outside the file — a game
//! (`phiplay`) that has simulated its match and now poses the players and
//! moves the ball frame by frame.
//!
//! It plugs in where the file's own animation is read: a rig source asks
//! its play for the bone channels before its NLA and action
//! (`BlendRigSource::pose_at`), an object's motion asks for its transform
//! before its curves (`BlendMotion::raw_at`). Everything after — the
//! constraints, the solve, the skin, the tracer's repose — is the same
//! path a keyed file takes, so a played frame is drawn exactly as a keyed
//! one would be.

use crate::blend::object::transform::ObjectTransformRaw;
use crate::blend::rig::{BRig, BRigState};
use std::sync::Arc;

/// A source of motion at any frame. Pure in the frame: the same frame
/// gives the same answer, so the viewport may pose ahead of the playhead
/// on another thread and scrub backwards.
pub trait BPlay: Send + Sync + std::fmt::Debug {
    /// The rig's channels at `frame`, when this play drives a rig —
    /// usually `rig.rest_state()` with the bones it moves written over.
    fn rig(&self, _rig: &BRig, _frame: f32) -> Option<BRigState> {
        None
    }
    /// The object's transform at `frame`, when this play moves an object.
    fn object(&self, _frame: f32) -> Option<ObjectTransformRaw> {
        None
    }
    /// The frames the play covers.
    fn range(&self) -> (f32, f32);
}

/// A play handle, as a source holds one: compared by identity, so a
/// source that holds the same play is the same source.
#[derive(Clone, Debug)]
pub struct BPlayRef(pub Arc<dyn BPlay>);

impl PartialEq for BPlayRef {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
