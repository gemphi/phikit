# phiphx

Phiber's asset core: read Blender, USD, glTF, PLY and PHX files into one
runtime scene, evaluate what Blender evaluates for display, and write PHX,
glTF, USD and point-cloud formats.

**Start at [`src/phx.rs`](src/phx.rs)**: `Phx::open`, `Phx::peek`,
`Phx::detect`. `Phx` has no format logic of its own. It asks the device list
which **device** owns the file (by its bytes, then by its extension) and the
device does the work, the way Cycles delegates to render devices. The layout
and how it got there are in [`LAYOUT.md`](LAYOUT.md).

| Folder | Role |
|---|---|
| [`src/phx.rs`](src/phx.rs) | entry point (`Phx`; tests in `src/test/phx.rs`) |
| [`src/device`](src/device/mod.rs) | the `Device` trait, `DeviceList`, `Peek`; one folder per scene-file variant |
| [`src/device/usd`](src/device/usd/mod.rs) | the USD device: object model, parser, printer, usdz, `SceneModel` writer |
| [`src/device/blend`](src/device/blend/README.md) | the Blend device: native `.blend` reader, writer and Blender evaluation |
| [`src/device/phx`](src/device/phx/README.md) | the PHX device: point cloud, scene, `.phx` sidecar, project folder |
| [`src/scene`](src/scene/README.md) | `SceneModel`, the generic runtime scene every view draws |
| [`src/import`](src/import/README.md) | glTF, PLY, OBJ readers + asset router |
| [`src/export`](src/export/README.md) | glTF, PLY, PCD, LAS, XYZ, JSON, binary writers |
| [`src/cloud`](src/cloud/README.md) | point clouds |
| [`src/geom`](src/geom/README.md) | geometry primitives |
| [`src/format`](src/format/README.md) | output formats, materials |
| [`src/spatial`](src/spatial/README.md) | spatial UI objects |

Rules: types own behaviour (methods and traits, no loose `pub fn`), short
singular names (a collective noun or `...List` where there are many), at most
300 code lines per file, unit tests in `test/` folders, every Blender-derived
reader checked against Blender (`tests/truth`). Status: [`jobs.md`](jobs.md)
here and in every folder.