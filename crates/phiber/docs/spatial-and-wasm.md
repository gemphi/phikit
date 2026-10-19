# Spatial, and the web interface — what is built, what is not, what next

Read against the tree on 2026-10-18. Every claim here names the file that
proves it, because the last five docs I checked in this repo were stale in
the same direction — tracked as open, actually done — and the Warp job list
was stale for seven of its nine lines.

## 1. The headline: the groundbreaking ideas are already in

The ask was to take NVIDIA Warp's ideas and make them work on any GPU. That
work is done, and it was done before this session. Warp's own analysis
(`crates/phiray/reference/warp/ANALYSIS.md`) states its conclusion plainly:

> Warp's speed is *not* exotic. It is (1) no per-call allocation, (2) no
> per-call argument marshalling, (3) replaying a recorded launch list.

All three are in phiray, on wgpu, on any adapter:

| Warp idea | Where it lives now |
|---|---|
| Kernel = source + typed params, pipeline cached by hash | `phiray/src/kernel.rs`, `gpu/mod.rs::pipeline` |
| 256-wide blocks, lean x→y grid, no 3-D grid maths | `kernel.rs::Dispatch` (tested to 20 M items) |
| Record the launch list once, replay per frame | `gpu/pass.rs` — `add`, `set`, `run` |
| Buffers grown by doubling, never per frame | `gpu/array.rs:50` |
| LBVH: bounds → Morton → links | `bvh/kernels.rs` (Morton, Karras) |
| Refit a posed frame instead of rebuilding | `bvh::Refit`, `GpuBvh::update` |
| 16-byte node halves, leaf bit in the index | `bvh/node.rs:9` |
| 32-slot stack, near child first | `trace/wgsl/bvh.wgsl:55` |
| Skinning with CSR influences and B-Bone segments | `kernels/skin.rs` |

So the correct next move was never to build this again. It was to find what
is genuinely missing. `reference/warp/jobs.md` now lists that, with
evidence, and it is seven lines rather than nine.

**The bottleneck that is left, and it is the big one:** the frame comes back
to the CPU. `phiber/src/v2/scene/skin.rs:13` says the quiet part out loud —
*"below this many vertices the CPU is faster than an upload + readback"* —
and `GpuArray::read_first` makes a staging buffer per read and throws it
away. Warp needs `RegisteredGLBuffer` to avoid this because CUDA and GL are
two worlds.

**Correction (batch 199).** An earlier draft of this file said *we do not
need anything — the compute buffer and the buffer the display reads are on
one wgpu device.* That is wrong on the desktop, and the error is worth
leaving visible. phiray is wgpu; gpui on Windows renders through
**DirectX** (`gpui_windows/directx_renderer.rs`, `directx_atlas.rs`, HLSL
shaders). Two devices, two APIs — which is the very split Warp needs
`RegisteredGLBuffer` for. A traced frame crosses them, and today it crosses
through the CPU.

It **is** free in two places, and that decides the order of work: *inside*
phiray, where every buffer is already on one wgpu device; and in the **web**
build, because `gpui_web` renders through `gpui_wgpu`. So the change that
makes the readback free on the desktop — gpui rendering through
`gpui_wgpu` — is the same change that makes the web build work. One move,
two payoffs. The alternative on Windows is DXGI shared handles: fiddly,
Windows-only, and worth nothing to the web.

Note also what this readback is *not*. `scene/skin.rs` runs at **import**,
filling a CPU-side phiphx model, so it is not a per-frame viewport cost and
cannot be removed without changing phiphx's model. The per-frame cost is the
traced frame itself, which is what `spatial/cost.rs` counts.
once in the viewport, once in the browser, where the copy is worse.

## 2. The spatial module (`phiber/src/v2/spatial/`) — new, this session

What phiray cannot decide for itself is **how much frame to ask for**. A path
tracer will spend a second on one frame happily; a spatial view has about
sixteen milliseconds. So the module is a ladder and an instrument, and
nothing else, because nothing else was missing:

* `set.rs` — the four tiers. The divisors are Blender's own
  `pixel_size_items` (`rna_scene.cc:6922`: 1×, 2×, 4×, 8×, i.e. full, 50%,
  25%, 12.5%). Powers of two so a tier change resamples cleanly.
* `tier.rs` — `BSpatialTier::step`: one rung down over budget, one rung up
  when the frame fits in half of it. One rung at a time in both directions,
  so a single slow frame (a shader compiling) does not cost a visible
  second of climbing back.
* `cost.rs` — `BSpatialCost`: pixels, samples, rays, and bytes crossing the
  bus. **No milliseconds-per-ray constant**, deliberately: that number
  belongs to an adapter and is measured, and a guessed one turns an
  instrument into a fortune teller. `rays_per_ms` gives the number to put
  *next to* a measurement.
* `mod.rs` — `BSpatial`: the running state, plus `reuse()` = replays per
  record. That ratio is the diagnosis Warp's third win is worth exactly:
  a spatial view that records as often as it replays is getting nothing
  from the recorded launch list, whatever its frame time says.

**Where this parts from Blender, and why.** Blender has the same dial and
holds it still — `preview_pixel_size` is a user preference, set once. A
spatial view moves on the ladder while it flies, because a budget held by
hand is a budget missed on the frame the scene gets heavy.

## 3. The web interface — most of it is already in the tree

One line changed in phiray to make a browser possible at all:
`Gpu::open()` is now `pub async` (`gpu/mod.rs:154`). `Gpu::new()` wraps it
in `pollster::block_on`, which is right on a desktop and impossible on the
web: blocking the page's one thread is what WebGPU's async API exists to
prevent, and `block_on` there does not stall, it panics.

**What a `phiweb` crate can be built from today, with no new dependency.**
`wasm-bindgen`, `wasm-bindgen-futures` and `js-sys` are already in
`[workspace.dependencies]`. So the first version is:

```
PhiWeb::open()        → Gpu::open().await
PhiWeb::scene(tris)   → phiray::Scene from data JS passes in
PhiWeb::render(w, h)  → Tracer::render → RGBA8 → putImageData
```

That renders a Phiber scene in a browser, on the customer's own GPU,
through the same kernels the desktop uses. It is enough to share.

**Correction (batch 199): both "blockers" below were wrong, and the web is
much closer than this said.**

What is actually in the tree:

* **`crates/phigpu/gpui_web`** — 2,816 lines: a web platform for gpui, with
  its own dispatcher, display, events, keyboard, window and http client.
* **`crates/phigpu/gpui_wgpu`** — gpui rendered through **wgpu**, with
  `[target.'cfg(target_family = "wasm")'.dependencies]` already declaring
  `web-sys = { version = "0.3", features = ["HtmlCanvasElement"] }`,
  `wasm-bindgen`, `wasm-bindgen-futures` and `js-sys`.
* **`gpui_platform::application()`** already branches on
  `cfg(target_family = "wasm")` and builds a `gpui_web::WebPlatform`, and
  there are `single_threaded_web()` and `web_init()` beside it for exactly a
  `wasm_bindgen` entry point.

So the two things this file called blockers were not:

1. ~~A canvas surface needs `web-sys`, which is not in the workspace deps.~~
   A crate declares `web-sys` itself under its own wasm target, and
   `gpui_wgpu` already does. The workspace list was never the gate.
2. ~~phiber's UI cannot run in a browser, so the web build must be a
   separate pixel-pushing crate.~~ The UI framework has a web platform and a
   wgpu renderer, both written. The web build is **phiber itself**, compiled
   for `wasm32-unknown-unknown`, not a `phiweb` that re-renders scenes.

Neither crate is a workspace member, so neither was being built, so nobody
knew whether they still worked. `gpui_wgpu` was checked in batch 199 and
compiles **clean, with no warnings**; it is now a member so it cannot rot.
`gpui_web` is wasm-only and cannot be a member on a desktop host — it is
checked by building for the wasm target, which is the next thing to try.

**The one blocker that survived, and it is the real one.** phiphx reads from
paths, not bytes: `Phx::open(&Path)`, `open_with`, `open_progressive`,
`peek`, `detect` (`phx.rs:23-64`). A browser has bytes. 35 files under
`phiphx/src` touch `std::fs`, but that number flatters the problem — about
twenty of them are tests or the `blend/write/*` save path, which a viewer
does not need. The read path is roughly a dozen files: `phx.rs`,
`blend/mod.rs`, `blend/file/quick.rs`, `blend/asset/{import,phx}.rs`,
`import/{format,phx,ply,usd}.rs`, `import/usd/importer.rs`,
`import/gltf/full.rs`, `project.rs`, `scene/texture{,_find}.rs`. A byte
source threaded through those is a bounded job, and it is what lets a
customer open their own `.blend` in a browser.

Three more things will want attention when the wasm target is first built,
and all three are known rather than guessed: `rayon` (13 files use it, and
threads on `wasm32-unknown-unknown` need `wasm-bindgen-rayon` or a serial
fallback), `std::thread::spawn` (9 files in phiber, including
`RaySkin::install`, which returns `Err` on wasm rather than failing to
compile), and `zstd`, which is C. `flate2` is already on `rust_backend`, so
somebody was thinking about this.

## 4. What to do next, in order

Re-ordered in batch 199, because §3 changed what the shortest path is.

1. **Build for `wasm32-unknown-unknown`.** `rustup target add
   wasm32-unknown-unknown`, then `cargo check -p phiber --target
   wasm32-unknown-unknown`. Everything in §3 says this is close; nothing
   says it passes, because nobody has run it. The error list from that one
   command is worth more than any further reading of this tree, and it is
   the thing that turns "the web is close" into a number.
2. **gpui through `gpui_wgpu` on the desktop.** One wgpu device shared with
   phiray, which makes the traced frame's readback unnecessary (§1) and is
   the same renderer the web build uses. The crate is a workspace member now
   and compiles clean, so this is wiring rather than writing.
3. ~~A byte source for phiphx~~ — **done, batch 200.** It was far shallower
   than this file guessed: `import_blend_native_with_progress` did
   `std::fs::read(path)` and then handed the bytes to
   `parse_bytes_with_progress`, so **the parser was already byte-based** and
   the path was doing nothing but the read. Splitting the two gives
   `Phx::open_bytes(name, data)` / `open_bytes_with`, and `Phx::peek_bytes`
   for the thumbnail and frame range (the magic-byte sniff that picks
   Zstd/gzip/raw is now generic over the reader, so one sniff serves a file
   and a buffer). `scene/bytes_tests.rs` reads one of Blender's own test
   files through both doors and asserts the scenes match object for object,
   vertex for vertex. The estimate of "roughly a dozen files" above was
   wrong in the right direction — the read path funnels through one
   function, and the other ~20 `std::fs` files are tests and the
   `blend/write/*` save path.

   What it cannot do, and says so: resolve an image a file names by path
   (`//textures/bark.png`), because bytes carry no folder. The source
   directory is explicitly *cleared* rather than left over from the last
   open, or one file's `//textures/` would resolve against another's
   folder and load the wrong image. Packed images are unaffected.
4. **Leaf packing in compute** — the one hop left in the LBVH build now the
   sort has moved (`bvh/radix.rs`), and the last readback in it.
5. **Mapped staging ring** for the reads that remain.
6. `rayon`, `std::thread` and `zstd` on wasm, once step 1 says which of them
   actually bite.


## 5. Not resolved

* ~~"hogpot"~~ — settled: it is a separate idea (a fictional fashion /
  runway game), not a name for this. The module is `spatial`, and nothing
  here is named after it.
* **The fly-through video** — `Screen Recording 2024-10-18 at 15.42.43.mov`
  (29 MB) in `Downloads/phiber/this-is-phiber-topo-flesh-and-welcome-video`
  has not been opened. The six terrains in `modal/splash/terrain` came from
  its still frames, which is why they are real curves and not noise, but
  the camera path in the video is a separate thing and is not extracted.
