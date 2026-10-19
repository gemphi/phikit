# Jobs — `phiphx/phx`

| State | Open | Done |
|---|---|---|
| 🟡 in progress | 4 | 3 |

## Todo

- [ ] USD scenes through `Phx::open` (read with the `openusd` crate: usda/usdc/usdz, composition)
- [ ] glTF scenes through `Phx::open` into the same `SceneModel`
- [ ] Progressive open: return rest meshes first, evaluated meshes when the modifier stack finishes
- [ ] WebAssembly build of `Phx`: the byte door and the Zstd decoder are in (below); what is left is `rayon` with no threads

## Done

- [x] `Phx::open` / `open_with` (progress + cancel), `peek`, `detect`, `modifiers`
- [x] Errors categorised (`Inaccessible` / `Malformed` / `Unsupported` / `Cancelled`); parser panics caught
- [x] phiber reaches phiphx only through `Phx` and the `scene` data types
- [x] `Phx::open_bytes` / `open_bytes_with` / `peek_bytes` — a `.blend` from memory, for a caller with no filesystem (browser file picker, HTTP body, asset server). The parser was already `parse_bytes_with_progress`; the path was only doing `std::fs::read`, so this split IO from parsing rather than adding a second reader. The compression sniff (`decode`) is generic over the reader now. Images named by path cannot resolve (no folder) and the source dir is cleared rather than inherited, so one file's `//textures/` never resolves against another's. Test: `scene/bytes_tests.rs` — both doors over one of Blender's own test files, object for object and vertex for vertex, plus `peek` parity on version/frames/thumbnail
- [x] Zstd decoding on a target with no C toolchain (`blend/file/zst.rs`): `zstd` (C) only under `cfg(not(target_family = "wasm"))`, `ruzstd` (pure Rust) on wasm, chosen by target with a `pure-zstd` feature so the Rust path is exercised by the suite on a desktop host. A plain `[dependencies]` entry would not do — it puts `zstd-sys` in the wasm crate graph and the build dies in `cc-rs` with `failed to find tool "clang"` before any Rust is reached. Blender writes a `.blend` as SEVERAL concatenated Zstd frames (`tests/fixtures/curve_paths.blend` has three); the C library spans them, `ruzstd::StreamingDecoder` stops at the first and the block parser then fails with `UnexpectedEof`, so `MultiFrame` starts a new decoder per frame, peeking with `fill_buf` so `read_quick_info` still does not decode a whole file for a thumbnail. Tests: round trip, three concatenated frames, and the real multi-frame fixture — all run under both decoders
- [x] `SceneModel::heap_bytes` — the open report's memory figure, moved off a free function in `phx.rs` onto the type it measures (`scene/heap.rs`)

Format: one line per job, `- [ ]` open / `- [x]` done; move a line from Todo to Done when it lands, with its test.
