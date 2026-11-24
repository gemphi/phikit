# Jobs — `phiphx/import`

| State | Open | Done |
|---|---|---|
| 🟡 in progress | 3 | 3 |

## Todo

- [ ] USD via `openusd` (replace the ASCII-only scanner): composition, references, `.usdc` beyond the heuristic, materials read back (the scanner reads points / faces / normals / transforms only)
- [ ] glTF scenes through `Phx::open` need the `gltf-import` feature (off by default); materials and node names are not carried yet
- [ ] Remove the duplicate reader copy in `crates/apple/src/phiphx`

## Done

- [x] Asset router (`AssetImporter`), format detection, glTF, USD (ASCII), PLY, PHX, OBJ
- [x] Every format into a `SceneModel` (`scene.rs`, `scene/import_mesh.rs`): `Phx::open` matches on the detected format, so USD / glTF / OBJ / PLY / PHX open into the same model as `.blend`. Test: `tests/usd_scene.rs`
- [x] USDA transforms read the right way round (`usd/usda_mesh.rs`): USD's rows are glam's columns; the scanner had them transposed, so a translated prim came back with its translation in the bottom row. Two more scanner slips fixed with it: the `4` of `matrix4d` split numbers (`0.4` read as `0`), and the `3` of `point3f[]` / `normal3f[]` was glued onto the first coordinate (`30.05`); numbers are now read only after `=` / inside a tuple. Test: `tests/usd_scene.rs` (world boxes equal after a round trip)

Format: one line per job, `- [ ]` open / `- [x]` done; move a line from Todo to Done when it lands, with its test.
