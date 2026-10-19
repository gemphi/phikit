//! The browser entry point.
//!
//! `wasm-bindgen` calls [`start`] once the module is instantiated, which is
//! the browser's equivalent of `main`. Everything it needs is already in
//! the tree: `gpui_web` is the platform, `gpui_wgpu` the renderer, and
//! `app::run` picks both through `cfg(target_family = "wasm")`.
//!
//! Build it with
//! `wasm-pack build crates/phiber --target web`, or
//! `cargo build -p phiber --target wasm32-unknown-unknown` followed by
//! `wasm-bindgen --target web`.

use wasm_bindgen::prelude::wasm_bindgen;

/// Start Phiber in the page.
///
/// `web_init` installs the panic hook that puts a Rust panic in the
/// browser console with its location — without it a panic is an unhelpful
/// `unreachable executed`, which is most of what makes a wasm build hard to
/// debug. The desktop binary installs its own hook in `main.rs` for the same
/// reason.
#[wasm_bindgen(start)]
pub fn start() {
    crate::app::run();
}
