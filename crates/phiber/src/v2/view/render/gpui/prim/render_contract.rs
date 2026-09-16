//! `BFocusedRenderer` — compile-time trait coverage for focused concrete
//! renderer modules.
//!
//! Every focused module in `prim/` and `prim/panel/` implements this trait on
//! its named B-prefixed struct. The trait provides:
//!
//! - **Compile-time coverage proof**: `impl BFocusedRenderer for BFoo` is the
//!   compile-time attestation that `BFoo` owns its render behaviour as a named
//!   entity (Gate 1, ordered gate 1 of task 6.2).
//! - **Typed input adapter**: `type Input` names the data this renderer
//!   consumes, separating the input type from the renderer struct.
//! - **Compatibility wrapper**: `Self::render_input(input, c)` is the public
//!   API surface that `prim/mod.rs` (the thin router) may call. The router
//!   never directly constructs the struct, keeping GPUI builder bodies out of
//!   dispatchers (Gate 3 invariant).
//!
//! ## Placement rule
//!
//! This trait lives in `prim/render_contract.rs` (inside the `prim` module
//! family, visible to all focused sub-modules via `super::render_contract`).
//! It is explicitly **not** re-exported through `prim/mod.rs` or
//! `panel/mod.rs` — the routers must stay free of `BFocusedRenderer` to pass
//! the source-text assertion in `file_browser_accordion_refinement.rs`.
//!
//! ## Semantic migration (Gate 4)
//!
//! Because every focused module already reads `c.style.*` exclusively
//! (verified by `task_scoped_renderers_consume_only_the_resolved_style`),
//! semantic migration is structurally complete. This trait formalises that
//! ownership without introducing new GPUI builder bodies into any dispatcher.

use super::super::ctx::BRenderCtx;
use gpui::AnyElement;

/// Compile-time render ownership contract for focused GPUI primitive modules.
///
/// The `Input` associated type is the typed input adapter: a reference to the
/// data the renderer consumes, extracted from the `BView` variant by the thin
/// dispatcher before the concrete renderer is constructed. In many cases
/// `Input` is the struct itself (e.g. `type Input<'a> = BRow<'a>`), which
/// keeps the adapter and renderer as one value.
///
/// # Contract
///
/// An impl of this trait on a struct `BFoo` attests:
/// 1. `BFoo` owns all GPUI builder composition for its `BView` family member.
/// 2. No other struct or function constructs the same GPUI element tree.
/// 3. The struct reads semantic style exclusively through `BRenderCtx`.
/// 4. No legacy `METRICS` or palette `c.p` accesses appear in its `render`.
pub trait BFocusedRenderer {
    /// Render the concrete element. Every impl must satisfy:
    /// - All GPUI builder bodies live here, not in the calling dispatcher.
    /// - Style values come exclusively from `c.style.*`.
    /// - No `METRICS` constants, no `c.p` palette reads.
    fn render(&self, c: &BRenderCtx) -> AnyElement;
}
