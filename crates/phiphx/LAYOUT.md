# phiphx layout: scene files come in variants

A scene file comes in variants. `Phx` owns import and export and delegates to a
**device**, the way Cycles delegates to render devices, without entangling them.
Three devices for now: **USD**, **Blend**, **PHX**. glTF, PLY, OBJ, STL, PCD,
LAS and XYZ stay exactly where they are and keep working.

Phases: **1** trait + USD (done), **2** Blend (done), **3** PHX + scene + names, sizes, tests.

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
  blend/               (phase 2, done)
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
| `blend/file/*` | `device/blend/file/` (`types.rs` is now `kind.rs`) | 2 done |
| `blend/{mesh,object,curve,text,scene,asset}`, `blend/idprop.rs` | `device/blend/read/<kind>/`, `read/idprop.rs` | 2 done |
| `blend/write/*` | `device/blend/write/` | 2 done |
| `blend/{modifier,rig,anim,physics,shade}` | `device/blend/{modifier,rig,anim,physics,shade}/` (the driver code stays in `anim/driver/`, `rig/driver/`, `modifier/driver*.rs`, `shade/driver*.rs`, as it is split by what it drives) | 2 done |
| `blend/tests/*`, `*_tests.rs` beside code | `device/blend/test/<folder tested>/`, reached by `#[path]` from the module they test (so they still see its private items); inline `mod tests` is `mod test` | 2 done |
| `export/blend.rs`, `export/blend/*` (script writers: chunks, config, script_phx, script_gltf, types) | `device/blend/write/script/` (`chunk.rs`, `helper.rs`, `kind.rs`); `export/blend.rs` is a `pub use` shim | 2 done |
| `phx.rs` functions `blend`, `blend_bytes`, `blend_progressive`, `guarded`, `settle`, `classify_native_error` | `device/blend/open.rs`; `BlendDevice` (`impl Device`: detect, peek, read, read_progressive; write says it cannot) in `device/blend/mod.rs`, registered after USD. `Phx::open*` still call `open::*` | 2 done |
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

## Phase 2 (Blend): done
`device/blend/` holds `BlendDevice`, the container (`file/`), the readers
(`read/{asset,curve,mesh,object,scene,text}/`, `read/idprop.rs`), the writer
(`write/`, `write/script/`), Blender's evaluation (`modifier/`, `rig/`, `anim/`,
`physics/`, `shade/`) and the unit tests (`test/`). Old paths
(`phiphx::blend::*`, `phiphx::export::blend`) are two small `pub use` shims
(`src/blend.rs`, `src/export/blend.rs`); renamed modules keep no alias, so
`phiber` was moved to the new names (branch `variant-layout` in its repo).
### The 8 files that were over 300 code lines
| Was | File | Now |
|---|---|---|
| 589 | `modifier/bbone.rs` | `modifier/bbone/{mod 53, curve 57, roll 45, spline 139, shape 64, pose 172, read 75}.rs` |
| 518 | `rig/math.rs` | `rig/math/{mod 40, euler 93, quat 129, vec 85, mat4 119, ortho 38, symmetric 38}.rs` |
| 451 | `shade/eval.rs` | `shade/eval.rs` 197 + `shade/emit.rs` 256 (the bake walk) |
| 424 | `modifier/mod.rs` | `mod.rs` 106 + `kind.rs` 30, `source.rs` 60, `stack.rs` 42, `job.rs` 140, `deferred.rs` 66 |
| 419 | `rig/ik/mod.rs` | `mod.rs` 42 + `tree.rs` 116, `build.rs` 83, `execute.rs` 185 |
| 398 | `write/edit.rs` | `write/edit/{mod 127, apply 163, structure 113}.rs` |
| 358 | `write/mesh.rs` | `write/mesh/{mod 84, new 106, add 95, storage 82}.rs` |
| 328 | `rig/ik/segment.rs` | `segment.rs` 201 + `rotation.rs` 129 (`BIkMath`) |
No file in `device/blend` is over 300 code lines now (largest: `read/curve/bevel.rs` 291).
(`spatial/spatial_object.rs` 535 and `spatial/glyph.rs` 331 are outside the
three devices and stay as they are.)
### Names
Plural files, folders and types in `device/blend` are singular or take a
collective: `anim/{channel,driver_list,handle}`, `file/{name,reader,kind}`,
`read/mesh/{attr,face,group,normal,offset,position}`, `read/scene/{prop,kind}`,
`read/text/{character,font/name}`, `read/object/instance_member`,
`modifier/{merge_vertex,node*,solidify_crease,solidify_normal}`,
`shade/{link,map,node,slot}`, `write/key`, `write/script/{chunk,helper,kind}`;
types `BlendDriverList`, `BlendEditList`, `BlendSceneProp`, `BFaceList`,
`BNlaStackList`, `BModifierDriverList`, `PhxChunkList`, ... (36 in all, see git
history). Kept on purpose: `physics` and `multires` are not plurals (an
uncountable noun, a contraction), and every folder's `jobs.md` is the repo's
status-file name.
## Left for phase 3
- **PHX device**: `export/phx*`, `import/phx.rs`, `export/scene*`, `project.rs`
  into `device/phx/`; `read/asset/phx.rs` (`.phx` sidecar, called by
  `BlendScene::write_sidecar`) and `import/usd.rs::write_phx_sidecar` into
  `device/phx/sidecar.rs`; `PhxChunkList` (now in `device/blend/write/script/`)
  belongs with the PHX scene.
- **`scene/`**: trim to `SceneModel`. What is Blender reading and moves to
  `device/blend/read/scene/`: `scene/import.rs`, `import_extras.rs`,
  `progressive*.rs`, `build.rs` (`SceneModel::from_blend_owned`,
  `from_blend_progressive`), `rig.rs`, `motion.rs`, `collection.rs`,
  `relation.rs`, `share.rs`-style Blender glue. `scene/corners.rs`, `extras.rs`,
  `types_ext.rs` and `tests_types.rs` still have plural or `tests_` names.
- **`Phx` delegation**: `Phx::open*`, `open_bytes*`, `open_progressive`, `peek*`
  and `detect` still call `device::blend::open::*`, `BlendFile` and
  `FileFormat`; they should ask `DeviceList` (`detect` by head then extension,
  then `device.read`), with `Phx::use_skinner`/`modifiers` going through the
  Blend device. `FileFormat::detect` does not know `.blend` (by extension only).
- **Remaining names and tests**: `phx_tests.rs` and `scene/*_tests*.rs` into
  `test/` folders; `export/scene/types.rs`, `export/phx` plural names; the
  generic `import/*`/`export/*` stay as they are.
- **Shims**: once `phiber` and the examples use `device::blend` paths, drop
  `src/blend.rs`, `src/export/blend.rs` and the old USD shims.