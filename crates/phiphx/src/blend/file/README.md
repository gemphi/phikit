# blend/file

The `.blend` container. Header, block headers, block table and pointer lookup (`scope`), DNA and struct layout, quick peek, names.

Callers enter through [`phx::Phx`](../../phx/README.md); this folder is one of its parts.

## Files

| File | What it is |
|---|---|
| `bhead.rs` | Block headers — `BHead4`, `SmallBHead8`, `LargeBHead8` (`blenloader_core/BLO_core_bhead.hh`), chosen by `BlendHeader::bhead_type`. |
| `dna.rs` | DNA block parsing (SDNA structure definitions). |
| `header.rs` | `.blend` file header — port of `BLO_readfile_blender_header_decode` (`blenloader_core/intern/blo_core_blend_header.cc`): the legacy 12-byte… |
| `layout.rs` | DNA field/struct layout helpers: field sizes, offsets, pointer reads. |
| `mod.rs` | The `.blend` container. |
| `names.rs` | `ID.name` decoding (DNA-driven, with a heuristic fallback). |
| `parse.rs` | Top-level .blend parsing: file header and data blocks. |
| `parse_tests.rs` | Header / BHead decoding tests (split from `parse.rs`). |
| `quick.rs` | Quick `.blend` peek for file browsers — Blender's `BLO_thumbnail_from_file` / `read_file_thumbnail` (readfile.cc) and the `REND` `RenderInfo`… |
| `readers.rs` | Low-level little-endian byte readers. |
| `scope.rs` | Per-ID pointer resolution (Blender `readfile.cc` `fd->datamap`). |
| `types.rs` | Shared types, constants and trait impls for the native .blend parser. |
| `types_tests.rs` | Primary-object selection tests (split from `types.rs`). |

Status and next steps: [`jobs.md`](jobs.md).
