# Jobs — `phiphx/phx`

| State | Open | Done |
|---|---|---|
| 🟡 in progress | 4 | 3 |

## Todo

- [ ] USD scenes through `Phx::open` (read with the `openusd` crate: usda/usdc/usdz, composition)
- [ ] glTF scenes through `Phx::open` into the same `SceneModel`
- [ ] Progressive open: return rest meshes first, evaluated meshes when the modifier stack finishes
- [ ] WebAssembly build of `Phx` (no threads fallback for `rayon`)

## Done

- [x] `Phx::open` / `open_with` (progress + cancel), `peek`, `detect`, `modifiers`
- [x] Errors categorised (`Inaccessible` / `Malformed` / `Unsupported` / `Cancelled`); parser panics caught
- [x] phiber reaches phiphx only through `Phx` and the `scene` data types

Format: one line per job, `- [ ]` open / `- [x]` done; move a line from Todo to Done when it lands, with its test.
