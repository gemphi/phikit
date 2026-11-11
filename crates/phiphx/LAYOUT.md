# phiphx layout: scene files come in variants (done)

A scene file comes in variants. `Phx` owns import and export and delegates to a
**device**, the way Cycles delegates to render devices, without entangling them.
Three devices: **USD**, **Blend**, **PHX**. glTF, PLY, OBJ, STL, PCD, LAS and
XYZ stay where they were and keep working.

Phases: **1** trait + USD (done), **2** Blend (done), **3** PHX + scene + names,
sizes, tests, shims, docs (done).

## Tree (`crates/phiphx/src`)

```text
phx.rs                 Phx::open / peek / detect: asks DeviceList which device owns
                       the file (bytes, then extension; Blend if none) and delegates
error.rs lib.rs        crate error, module list
device/                (469 .rs files)
  mod.rs               trait Device, DeviceKind, Source
  list.rs              DeviceList: usd, blend, phx in that order
  peek.rs              Peek, PeekScene, PeekThumbnail: what any device can say about a file
  usd/ (61)            usdkit's object model + everything USD
    stage.rs prim.rs attribute.rs xform.rs error.rs
    read/              parser (usda text), mesh/scene builder, usdc fallback
    write/             printer, SceneModel -> stage writer (scene/), point cloud, mesh
    package/           usdz
    test/
  blend/ (386)         file/ read/<kind>/ write/ (write/script/) modifier/ rig/ anim/
                       driver/ physics/ shade/ test/ ; read/scene/model/ is the
                       SceneModel built from Blender data
  phx/ (19)            the PHX device
    open.rs            PHX <-> SceneModel, peek
    read.rs            PhxImporter (asset pipeline)
    sidecar.rs         the .phx sidecar (from a .blend or a USD file)
    project.rs         ProjectScaffold
    cloud/             the PHX point cloud: chunk.rs (PhxChunkList), read, write, mesh
    scene/             the PHX scene: part.rs (PhxScene and its parts), build, read, write
    test/
scene/ (24)            SceneModel, the generic model (+ surface, extra, corner, material,
                       texture*, report, polyfill, geometry, ui, uv, test/)
cloud/ (5) comp/ (6) export/ (12) format/ (2) geom/ (6) import/ (11) sky/ (3)
spatial/ (18, glyph/ spatial_object/ test/) test/ (1: Phx tests) uv/ (10)
```

Total 572 `.rs` files. No file in `src` is over 300 code lines. No plural file,
folder or type name (collective noun, `...List`, `P`/`B`-prefixed singular), apart
from `jobs.md` (the status-file name), `physics` and `multires` (not plurals), and
`PointCloudStat`-style singulars for `stats`.

## The `Device` trait

```rust
pub enum DeviceKind { Usd, Blend, Phx }
pub struct Source<'a> { pub name: &'a str, pub path: Option<&'a Path>, pub bytes: &'a [u8] }
pub trait Device: Sync {
    fn kind(&self) -> DeviceKind;
    fn extension(&self) -> &'static [&'static str];        // lower case, no dot
    fn detect(&self, head: &[u8]) -> bool;                  // by magic bytes
    fn peek(&self, bytes: &[u8]) -> Result<Peek>;           // header only
    fn peek_path(&self, path: &Path) -> Result<Peek>;       // default: read, then peek
    fn streams(&self) -> bool;                              // read() opens Source::path itself
    fn read(&self, source: &Source, progress: Progress) -> Result<SceneModel, SceneLoadError>;
    fn read_progressive(&self, source: &Source, progress: Progress)
        -> Result<(SceneModel, Option<DeferredModifierList>), SceneLoadError>;
    fn write(&self, scene: &SceneModel) -> Result<Vec<u8>, SceneLoadError>;
}
```

`DeviceList::get(kind)`, `detect(head)`, `by_extension(ext)`, `iter()`.

| Device | Extension | Detects | `write` |
|---|---|---|---|
| `UsdDevice` | usd usda usdc usdz | `#usda`, `PXR-USDC`, zip | `SceneModel::to_usda` |
| `BlendDevice` | blend | `BLENDER`, zstd frame | refuses (edit an existing file with `write::`) |
| `PhxDevice` | phx | `PHX_SCENE\x01`, `PHX\0` | `PhxScene` bytes |

## `Phx` delegation

`Phx::open*`, `open_progressive`, `open_bytes*`, `peek*` ask `DeviceList` for the
device: its first 16 bytes by `detect`, then the extension, and a file no device
claims goes to Blend, so its errors read as before. `Phx::detect` returns a
`FileFormat`: the old magic checks (glTF, USD, OBJ), then the devices' (so
`BLENDER` and `PHX` heads are known), then the extension. `Phx::use_skinner` and
`modifiers` go through `BlendDevice`.

## Map: where things went

| Was | Now |
|---|---|
| `import/usd*`, `export/usd*`, `export/blend/script_usd.rs` | `device/usd/` (phase 1) |
| `blend/*`, `export/blend*` | `device/blend/` (phase 2) |
| `export/phx*` (point cloud), `PhxChunkList` and the chunk format | `device/phx/cloud/` |
| `export/scene*` (PHX scene), `types.rs` | `device/phx/scene/` (`part.rs`) |
| `import/phx.rs` | `device/phx/read.rs` |
| `blend/asset/phx.rs`, `write_phx_sidecar` | `device/phx/sidecar.rs` |
| `project.rs` | `device/phx/project.rs` |
| `scene/{import,import_extras,object,share,progressive*,build,rig,motion,collection,relation}.rs` | `device/blend/read/scene/model/` |
| `scene/{corners,extras,types_ext}.rs` | `scene/{corner,extra,surface}.rs` |
| `BlendQuickInfo`, `BlendRenderInfo`, `BlendThumbnail` | `device::{Peek, PeekScene, PeekThumbnail}` |
| `DeferredModifiers` | `DeferredModifierList` |
| `SceneRigs` `SceneCorners` `SceneMaps` `SceneNormals` `SceneDrivers` `STriangles` `VertexAttributes` `PointCloudStats` | `...List` / `PointCloudStat` |
| `phx_tests.rs`, `scene/tests_*.rs`, inline `mod tests { }` | `test/` folders reached by `#[path]` |
| `spatial/spatial_object.rs` (535), `spatial/glyph.rs` (331) | `spatial/spatial_object/{mod,solid,round}.rs`, `spatial/glyph/{mod,low,high}.rs` |

## Shims

Removed: `src/blend.rs`, `src/export/blend.rs`, `import::usd`, `export::usd`.
`phiber`, `phipro`, the examples, tests and benches use `phiphx::device::{blend,usd}`
paths. Kept on purpose: `phiphx::scene::{build,collection,motion,progressive,relation,rig}`
and the `SceneModel` types they define are re-exported from `scene/` so
`phiphx::scene::*` paths hold; `export::*` still re-exports the PHX scene types
(`PhxScene`, `PhxMesh`, ...) and `lib.rs` still re-exports `project`, so
`phiphx::PhxScene` and `phiphx::project::ProjectScaffold` resolve.

## Untouched

glTF (`import/gltf*`, `export/gltf*`), PLY, OBJ (`import/format.rs`), STL, PCD,
LAS, XYZ, JSON/binary point cloud exports, and the shared pieces `cloud`, `geom`,
`format`, `sky`, `spatial`, `uv`, `comp`, `import/asset.rs`. Only their plural
names and inline tests changed (`cloud/{method,data}.rs`, `export/gltf/buffer.rs`,
`geom/bound.rs`, `test/` folders).
