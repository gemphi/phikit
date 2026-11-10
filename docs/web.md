# web

An object on a web page that a visitor can turn, pan and zoom, the way a
USD model opens on Apple. Planned from what exists (read 2026-11-10,
nothing built or run). Solid boxes are in the code today; dashed boxes are
to build. Paths are under `crates/`.

## What exists

```mermaid
graph TB
    p_page["phiber/web/index.html<br/>checks navigator.gpu, loads pkg/phiber.js"]
    p_pkg["phiber/web/pkg<br/>the whole editor as wasm, 37.8 MB, built 22 Oct"]
    p_start["phiber/src/wasm.rs:22-26<br/>start: app::run, the embedded demo"]
    p_web["appkit/gpu/gpui_web<br/>makes the canvas, pointer · wheel · keyboard"]
    p_wgpu["appkit/gpu/gpui_wgpu<br/>WebGPU or GL surface from the canvas"]
    p_bytes["phiphx Phx::open_bytes, src/phx.rs:58<br/>a .blend from bytes, Zstd in pure Rust"]
    p_async["phiray Gpu::open_shared, src/gpu/mod.rs:138<br/>the async door to a device: no caller"]
    p_orbit["phiber/src/scene/orbit.rs · navigate.rs · key.rs<br/>turntable, pan, zoom and its clamp, as plain code"]
    p_page --> p_pkg --> p_start --> p_web --> p_wgpu
    p_start --> p_bytes
    p_start -.->|"never reached"| p_async
    p_start --> p_orbit
```

Whether that page runs in a browser today is not verified: no log of a run
is on disk.

## What stops an object appearing

| # | Blocker | Where | To build |
|---|---|---|---|
| 1 | phiray's web device is never opened: the async door has no caller, and the wasm `shared()` only reads | phiray `src/gpu/mod.rs:127-138` | open it during `web_init`, before the first view |
| 2 | No pixels come back: on wasm `wait_for` returns false, so `read_range` and `Tracer::pixels` are empty | phiray `src/gpu/mod.rs:318-322`, `gpu/array.rs:174-181`, `trace/mod.rs:985` | do not read back at all: see 3 |
| 3 | phiray has no surface, and cannot adopt the device gpui already holds | phiray `src/gpu/mod.rs:82-95, 241`, `gpu/raster.rs:53-83` | a constructor that takes a device and queue; the traced image handed to gpui as a texture. One device, no copy: the same change the desktop's present path needs |
| 4 | The traced viewport starts a worker thread | phiber `src/view/draw/canvas/view3d/rendered/worker.rs:45-56` | on the web, run one slice a frame from the page's frame callback |
| 5 | A page cannot be given a file: only `embedded:` opens, there is no picker, drops carry no path | phiber `src/app/load.rs:91-94`; `appkit/gpu/gpui_web/src/platform.rs:234-240`, `events.rs:347` | bytes in: a URL fetched, a file input, a drop |
| 6 | USD, glTF and PLY meshes are read from paths; the byte readers are private | phiphx `src/import/gltf/full.rs:24-25`, `usd/importer.rs:93-94`, `ply.rs:12-16`, `usd/usdz.rs:71` | `open_bytes` for every format `Phx::open` reads |
| 7 | The only wasm target is the whole editor | phiber `src/wasm.rs:22-26` | a viewer entry: one object, one view, no panels |

Also open, not blockers: rayon is unconditional in phiray and phiphx (its
serial fallback on wasm is not verified); Material Preview's kernel took
248 s to compile natively, so the viewer starts in Solid.

## The viewer, in order

```mermaid
graph TB
    v1["1 open the web device"]
    v2["2 one device for gpui and phiray"]
    v3["3 present a texture, no readback"]
    v4["4 a viewer entry in phiber/web<br/>Solid mode: the raster prepass"]
    v5["5 bytes in: URL, file input, drop<br/>every format from bytes"]
    v6["6 orbit · pan · zoom · fit"]
    v7["7 Rendered: a slice a frame, no thread"]
    v8["8 the world in the page:<br/>the same Plugin set, an item fetched from phiora"]
    v1 --> v2 --> v3 --> v4 --> v5 --> v6 --> v7 --> v8
    classDef plan stroke-dasharray: 6 4;
    class v1,v2,v3,v4,v5,v6,v7,v8 plan;
```

## Moving it around: the interaction to match

Apple's is written down in SceneKit and RealityFoundation
(`gemphi/phi/apple-source/ruskit/Interfaces/MacOSX/`); Quick Look's own
headers are not on disk.

| Behaviour | Apple | phiber today | To build |
|---|---|---|---|
| drag orbits, scroll or alt-drag pans, pinch or alt-scroll dollies | `SceneKit/Headers/SCNView.h:92-102` | `scene/orbit.rs:18-43`, `scene/navigate.rs:59-174` | touch: one finger orbits, two pan and pinch |
| turntable about a target, world up fixed | `SCNCameraController.h:17-27, 47-55` | turntable | - |
| frame the object on load | `frameNodes:` `SCNCameraController.h:92` | - | distance = largest side / (2 tan(fov / 2)) x 1.18, near and far from it, as MHR's viewer does (`phiphx/reference/MHR/web-viewer/static/app.js:648-678`) |
| inertia, off by default | `SCNCameraController.h:59-63`, friction 0.05 | - | optional; MHR damps at 0.07 (`app.js:100-103`) |
| angle limits | `SCNCameraController.h:75-85` | zoom clamp `scene/key.rs:102-127` | vertical limit |
| which controls a view allows | `CameraControls { none, tilt, pan, orbit, dolly }`, RealityFoundation interface `:11624-11637` | - | one setting on the viewer |
| which files open | `MDLAsset.canImportFileExtension:` `ModelIO/Headers/MDLAsset.h:140` | `Phx::detect` | from bytes: blocker 6 |

## How the references put 3D on a page

```mermaid
graph LR
    key["all of these send triangles to a renderer written in JavaScript"]
    r_mhr["MHR web-viewer<br/>three.js, a Flask server:<br/>faces as raw u32, vertices as raw f32, over HTTP"]
    r_viser["viser: momentum, vggt, fast3r<br/>three.js client, arrays over a websocket"]
    r_rerun["rerun: momentum, hot3d, project_superdex<br/>meshes logged to its viewer"]
    r_plotly["plotly: pytorch3d, home-robot<br/>Mesh3d in a notebook or a Dash page"]
    r_hab["habitat-sim: no browser build on disk"]
    o_ours["phikit: the renderer itself runs in the page.<br/>The file is read by phiphx in wasm,<br/>drawn by phiray on WebGPU, shown by gpui_web"]
    o_take["what to take: the fit-to-view arithmetic, damping,<br/>and raw typed buffers when a server does send geometry"]
    r_mhr --> o_take
    r_viser --> o_take
    o_ours --- o_take
    classDef plan stroke-dasharray: 6 4;
    class o_take plan;
```

## One viewer, not a second renderer

```mermaid
graph TB
    a_file["bytes of a .blend, usd, gltf or ply"]
    a_phx["phiphx: the scene"]
    a_world["phiecs World, with the same plugins as the desktop"]
    a_ray["phiray: Solid first, then Rendered"]
    a_tex["a texture on the shared device"]
    a_canvas["gpui_web: the canvas"]
    a_in["pointer and touch: orbit, pan, zoom"]
    a_cam["the camera: a word with a Pose"]
    a_file --> a_phx --> a_world --> a_ray --> a_tex --> a_canvas
    a_canvas --> a_in --> a_cam --> a_world
    classDef plan stroke-dasharray: 6 4;
    class a_world,a_tex,a_in,a_cam plan;
```
