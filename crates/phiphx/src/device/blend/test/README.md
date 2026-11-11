# blend/test
Unit tests of the Blend device, in one folder per folder they test (`test/anim/` tests `anim/`,
`test/read/object/` tests `read/object/`). Each is declared by `#[path]` from the module it tests, so it
sees that module's private items. Integration tests are in `crates/phiphx/tests/`.
## Files
| Folder | What it tests |
|---|---|
| `blend/` | The device: `device.rs` (the `Device` contract), `scene.rs` (upstream `tests/files`), `file.rs` (textures, DNA layout). |
| `anim/`, `file/`, `modifier/`, `read/`, `rig/`, `shade/`, `write/` | The folder of the same name. |
Status and next steps: [`jobs.md`](jobs.md).
