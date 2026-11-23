//! Open Phiogy in a browser. `wasm-bindgen` calls `start` once the module
//! is loaded, which is a browser's `main`. The app is the same: appkit's
//! screen picks the browser's window and renderer from the target. A
//! browser cannot listen, so there is no line there, and no disk, so no
//! site is read: the window says so.
//!
//! Build it and open it as `web/index.html` says.

use wasm_bindgen::prelude::wasm_bindgen;

/// Start Phiogy in the page.
#[wasm_bindgen(start)]
pub fn start() {
    appkit::screen::init();
    crate::run(&[]);
}
