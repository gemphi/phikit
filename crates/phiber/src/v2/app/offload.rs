//! Moving work off the thread that asked for it — where there is another
//! thread to move it to.
//!
//! Every background job in Phiber is a warm-up or a worker started with
//! `std::thread::Builder::spawn`. Neither half of that survives a browser:
//!
//! * `spawn` returns `Err` on `wasm32-unknown-unknown`, so the work would
//!   never run even if it compiled.
//! * It usually would not compile. A warm-up that touches the GPU captures a
//!   `wgpu` handle, and on the web a `wgpu` handle is a JavaScript object —
//!   `Rc<RefCell<WebBufferMapState>>`, `Rc<Cell<u32>>` — which is not `Send`.
//!   `spawn` requires `Send`, so the build fails at the closure:
//!   *"`Rc<RefCell<wgpu::backend::webgpu::WebBufferMapState>>` cannot be sent
//!   between threads safely"*.
//!
//! So the two versions below differ in their *bounds*, not only their
//! bodies — that is the whole point of the split. The desktop one asks for
//! `Send`; the browser one does not, because it never hands the closure to a
//! thread. Writing this once means a call site reads the same on both
//! targets and the reason lives here instead of in eight `cfg` blocks.
//!
//! This is for fire-and-forget work only. A caller that needs the
//! `JoinHandle` — `BWarm::start` joins its one — keeps `spawn`, and gets
//! `None` on the web from the failed spawn. Returning a handle here would
//! mean returning a different type per target, which is exactly the
//! divergence this module exists to avoid.
//!
//! Callers must treat `false` as "it did not run". Every site converted so
//! far already did: they were warm-ups whose failure path is to fall back to
//! the CPU, which is exactly what a browser build should do until the
//! readback path is `async` (`phiray::Gpu::wait_for`).

/// Run `work` on a background thread. `false` when there is no thread to run
/// it on, and the work has **not** run.
#[cfg(not(target_family = "wasm"))]
pub fn offload(name: &str, work: impl FnOnce() + Send + 'static) -> bool {
    std::thread::Builder::new().name(name.to_string()).spawn(work).is_ok()
}

/// A browser has one thread, and it is drawing. Declines without running.
///
/// Note the missing `Send`: a closure holding a WebGPU handle cannot satisfy
/// it, and requiring it here would fail the build on call sites that are
/// merely *unused* on the web.
#[cfg(target_family = "wasm")]
pub fn offload(name: &str, work: impl FnOnce()) -> bool {
    let _ = (name, work);
    false
}
