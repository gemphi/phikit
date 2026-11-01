//! Event dispatch — routes events through the view hierarchy.
//!
//! Submodules:
//! - `router` — top-level dispatch_event_to_view and captured view handling
//! - `children` — recursive child hit-testing and propagation
//! - `claim` — delegate claim checks and direct delivery
//! - `drag_drop` — drag session routing

mod router;
mod children;
mod claim;
mod drag_drop;
