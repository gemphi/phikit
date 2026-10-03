//! Animation — task 17. Reading actions and evaluating them at a frame.
//!
//! (a) object keyframes: [`read`] takes layered (4.4+) and legacy actions to
//!     [`fcurve::BFCurve`]s; [`channels`] writes them over a transform.
//! (b) pose-bone keyframes: [`pose`] builds each bone's basis and the plain
//!     `BKE_pose_where_is` chain.
//! (c) [`motion`]: every object's rest transform, bound curves and parent,
//!     solved to world matrices at a frame — what a player asks for.
//! (d) constraints and drivers: a rig's pose solver, `blend/rig` (batches
//!     85 and 89).
//! (e) [`follow`]: objects that follow others — their constraints and bone
//!     parents — solved with the motion, rigs posed on demand (batch 90),
//!     and children of a curve riding its path (`blend/curve`, batch 91).
//!
//! Every step is checked against Blender at several frames
//! (`tests/truth/thymus_anim.txt`, made by `truth/blender_anim.py`).

pub mod channels;
pub mod drivers;
pub mod easing;
pub mod fcurve;
pub mod follow;
pub mod handles;
pub mod motion;
pub mod nla;
pub mod nla_edit;
pub mod nla_eval;
pub mod nla_stack;
pub mod pose;
pub mod read;
pub mod smooth;
mod tridiagonal;
pub mod summary;


#[cfg(test)]
mod follow_tests;
#[cfg(test)]
mod tests;
