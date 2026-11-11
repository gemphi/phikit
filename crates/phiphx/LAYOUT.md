# phiphx layout: scene files come in variants

A scene file comes in variants. `Phx` owns import and export and delegates to a
**device**, the way Cycles delegates to render devices, without entangling them.
Three devices for now: **USD**, **Blend**, **PHX**. glTF, PLY, OBJ, STL, PCD,
LAS and XYZ stay exactly where they are and keep working.

Phases (one commit each): **1** trait + USD, **2** Blend, **3** PHX + scene + names, sizes, tests.

## Target tree (`crates/phiphx/src`)

```text
phx.rs                 Phx::open / peek / detect: find the device by bytes or
                       extension and delegate; no format logic of its own
device/
  mod.rs               trait Device, DeviceKind, Source
  list.rs              DeviceList: usd, blend, phx in that order, feature-gated
  usd/                 usdkit's object model + everything USD
    stage.rs prim.rs attribute.rs xform.rs error.rs
    read/              parser (usda text), mesh/scene builder, usdc fallback
    write/             printer, SceneModel -> stage writer (scene/), point cloud, mesh
    package/           usdz
    test/              unit tests
  blend/               (phase 2)
    file/              container, SDNA, zstd
    read/<kind>/       one folder per data kind: mesh, object, curve, text, ...
    write/             .blend writer, script writers
    modifier/ rig/ anim/ driver/ physics/ shade/   Blender's own evaluation
    test/
  phx/                 (phase 3)
    cloud.rs           the PHX point cloud
    scene/             the PHX scene
    sidecar.rs         the .phx sidecar
scene/                 SceneModel only (phase 3 trims what is not the model)
```

Everything else (`cloud`, `geom`, `format`, `sky`, `spatial`, `uv`, `comp`,
`import/{gltf,ply,asset,format}`, `export/{gltf,ply,pcd,las,xyz,json,binary}`)
is **untouched for now**.

## The `Device` trait

```rust
pub enum DeviceKind { Usd, Blend, Phx }

/// Where a scene comes from: a name, an optional path (for sibling files such
/// as textures), and the bytes.
pub struct Source<'a> { pub name: &'a str, pub path: Option<&'a Path>, pub bytes: &'a [u8] }

pub trait Device: Sync {
    fn kind(&self) -> DeviceKind;
    /// File extensions this device owns, lower case, no dot.
    fn extension(&self) -> &'static [&'static str];
    /// Does this head of a file belong to the device?
    fn detect(&self, head: &[u8]) -> bool;
    /// Header-only look: version, render info, thumbnail.
    fn peek(&self, bytes: &[u8]) -> crate::Result<Peek>;
    /// Read into a scene (evaluated, for Blend).
    fn read(&self, source: &Source, progress: Progress) -> Result<SceneModel, PhxError>;
    /// The progressive open path: base scene now, deferred work after.
    /// Default: `read` with nothing deferred.
    fn read_progressive(&self, source: &Source, progress: Progress)
        -> Result<(SceneModel, Option<DeferredModifiers>), PhxError>;
    /// Write a scene as this device's file.
    fn write(&self, scene: &SceneModel) -> Result<Vec<u8>, PhxError>;
}
```

`DeviceList::get(kind)`, `DeviceList::detect(head)`, `DeviceList::by_extension(ext)`,
`DeviceList::iter()`.

## Map: current -> target

| Current (`src/`) | Target | Phase |
|---|---|---|
| `import/usd.rs`, `import/usd/*` (importer, parse, scene, usda*, usdc, usdz, tests) | `device/usd/read/mesh/` (heuristic readers, `usdc_fallback.rs`), `device/usd/package/mesh.rs`, `device/usd/test/import.rs`; `import/usd.rs` is now a `pub use` shim | 1 done |
| `export/usd.rs`, `export/usd/{mesh,point_cloud,writer,tests}.rs` | `device/usd/write/{exporter,mesh,point_cloud,writer}.rs`, `test/export.rs`; `export/usd.rs` is a `pub use` shim | 1 done |
| `export/usd/stage.rs`, `export/usd/stage/*` (SceneModel writer) | `device/usd/write/scene/` (`UsdStage` becomes `UsdSceneWriter`, old name re-exported; still writes text itself, not through the usdkit printer) | 1 done |
| `export/blend/script_usd.rs` (`BlenderScene::to_usda`) | `device/usd/write/script.rs` | 1 done |
| usdkit (`phikit/crates/apple/src/macos/usdkit`, copied) | `device/usd/{stage,prim,attribute,xform,error}.rs`, `read/`, `write/`, `package/`, `test/` (+ `matrix4d` value) | 1 done |
| `blend/file/*` | `device/blend/file/` | 2 |
| `blend/{mesh,object,curve,text,scene,asset (reader half)}` | `device/blend/read/<kind>/` | 2 |
| `blend/write/*` | `device/blend/write/` | 2 |
| `blend/{modifier,rig,anim,physics,shade}` | `device/blend/{modifier,rig,anim,driver,physics,shade}/` | 2 |
| `blend/tests/*`, `*_tests.rs` beside code | `device/blend/test/` | 2 |
| `export/blend.rs`, `export/blend/*` (script writers: chunks, config, script_phx, script_gltf, types) | `device/blend/write/script/` | 2 |
| `phx.rs` functions `blend`, `blend_bytes`, `blend_progressive`, `guarded`, `settle` | `device/blend/mod.rs` (`impl Device`) | 2 |
| `export/phx.rs`, `export/phx/*` (PHX point cloud) | `device/phx/cloud.rs` + `cloud/` | 3 |
| `import/phx.rs` (`PhxImporter`) | `device/phx/read.rs` | 3 |
| `export/scene.rs`, `export/scene/*` (PHX scene) | `device/phx/scene/` | 3 |
| `blend/asset/phx.rs`, `import/usd.rs::write_phx_sidecar` | `device/phx/sidecar.rs` | 3 |
| `project.rs` | `device/phx/project.rs` | 3 |
| `scene/*` | `scene/` (SceneModel only; `scene/import.rs`, `import_extras.rs`, `progressive*.rs` go to `device/blend/read/`) | 3 |
| `phx_tests.rs`, `scene/*_tests*.rs` | `test/` folders | 3 |

Old public paths (`phiphx::export::usd`, `phiphx::import::usd`,
`phiphx::export::UsdExporter`, `phiphx::blend::*`, ...) keep working through
`pub use` re-exports until phber moves to the device paths.

## Untouched for now

glTF (`import/gltf*`, `export/gltf*`), PLY (`import/ply.rs`, `export/ply.rs`),
OBJ (`import/format.rs`), STL, PCD (`export/pcd.rs`), LAS (`export/las.rs`),
XYZ (`export/xyz.rs`), JSON/binary point cloud exports, and the shared pieces
`cloud`, `geom`, `format`, `sky`, `spatial`, `uv`, `comp`, `import/asset.rs`.

## Files over 300 code lines (non-blank, non-comment) in scope

All Blend, so phase 2 splits them; none in USD or PHX at start.

| Lines | File | Fix |
|---|---|---|
| 589 | `blend/modifier/bbone.rs` | split by stage: segment, ease, curve |
| 518 | `blend/rig/math.rs` | split: matrix, quaternion, roll |
| 451 | `blend/shade/eval.rs` | split by node family |
| 424 | `blend/modifier/mod.rs` | move dispatch to `stack.rs`, per-modifier `apply` stays |
| 419 | `blend/rig/ik/mod.rs` | split solver / chain / pole |
| 398 | `blend/write/edit.rs` | split by data kind |
| 358 | `blend/write/mesh.rs` | split layers / attributes |
| 328 | `blend/rig/ik/segment.rs` | split bend / limit |

(`spatial/spatial_object.rs` 535 and `spatial/glyph.rs` 331 are outside the
three devices and stay as they are.)

## Plural names in scope, with their fix

Files: `blend/anim/{channels,drivers,handles}.rs`, `blend/curve/lists.rs` (-> `list.rs`),
`blend/file/{names,readers,types}.rs`, `blend/mesh/{attrs,faces,groups,normals,offsets,positions}.rs`,
`blend/modifier/{merge_verts,multires,nodes*,solidify_creases,solidify_normals}.rs`,
`blend/scene/{props,types}.rs`, `blend/shade/{links,maps,nodes,slots}.rs`,
`blend/text/{chars,font/names}.rs`, `blend/write/keys.rs`, `blend/object/instance_members.rs`,
`export/blend/{chunks,helpers,types}.rs`, `export/scene/types.rs`, `scene/{corners,extras}.rs`,
all `tests.rs` and `*_tests.rs` (-> `test/<name>.rs`), `blend/tests/`.
Fix: the singular (`channel`, `driver`, `handle`, `name`, `reader`, `type` -> `kind`,
`attr`, `face`, `group`, `normal`, `offset`, `position`, `node`, `link`, `map`, `slot`,
`char`, `key`), or a collective (`list`, `stack`, `book`) where the file holds many.
Types: `...s` named types (`Skinner`-style agent nouns are fine) are renamed in the phase
that moves the file; USD types arrive singular (`UsdSceneWriter`, `UsdStage`).
Folders: `blend/tests` -> `test`. Modules called `tests` -> `test`.
In USD: `import/usd/tests.rs`, `export/usd/tests.rs` -> `device/usd/test/`;
usdkit's `parser/arrays.rs` -> `read/array.rs`, `tests/generators.rs` -> `test/generator.rs`,
`tests/` -> `test/`, `parser/` -> `read/`, `printer/` -> `write/`, `usdz.rs` -> `package/`.
