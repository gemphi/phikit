# phiphx

Phiber's asset core: read Blender, USD, glTF, PLY and PHX files into one
runtime scene, evaluate what Blender evaluates for display, and write PHX,
glTF, USD and point-cloud formats.

**Start at [`src/phx.rs`](src/phx.rs)** — `Phx::open`, `Phx::peek`,
`Phx::detect`. Everything else is a part of it:

| Folder | Role |
|---|---|
| [`src/phx.rs`](src/phx.rs) | entry point (`Phx`; tests in `src/phx_tests.rs`) |
| [`src/scene`](src/scene/README.md) | `SceneModel` — what every view draws |
| [`src/blend`](src/blend/README.md) | native `.blend` reader |
| [`src/import`](src/import/README.md) | other readers + asset router |
| [`src/export`](src/export/README.md) | writers |
| [`src/cloud`](src/cloud/README.md) | point clouds |
| [`src/geom`](src/geom/README.md) | geometry primitives |
| [`src/format`](src/format/README.md) | output formats, materials |
| [`src/spatial`](src/spatial/README.md) | spatial UI objects |

Rules: types own behaviour (methods and traits, no loose `pub fn`), short
names, at most 200 substantive lines per file, every Blender-derived reader
checked against Blender (`tests/truth`). Status: [`jobs.md`](jobs.md) here and
in every folder.
