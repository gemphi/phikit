# .blend fixtures

| File | Origin | Header | Notes |
|---|---|---|---|
| `FLORIS_BOLDRIOD_RIG.blend` (5.5 MB) | moved from `crates/phiber/assets/production` | legacy `BLENDER-v306` | 331 objects, 32 parented to an armature |
| `THYMUS-BOLDROID-WAVE-003.blend` (6.9 MB) | same | legacy `BLENDER-v306` | 9 cameras, 4 lights, curve-parented camera |
| `TEXTURED-PACKED.blend` (88 KB) + `tex/magenta.png` | saved by Blender 5.0.1 (`pip install bpy`) | v1, zstd | plane with a **packed** 2×2 image (BL red, BR green, TL blue, TR yellow) and a cube using `//tex/magenta.png` (#26) |
| `REFS/blender/blender/tests/files/modeling/operators.blend` | Blender upstream | **v1 `BLENDER17-01v0502`** (Blender 5.2) + LargeBHead8 | used by `importer::blend::tests::loads_v1_header_large_bhead_file` |
| `REFS/blender/blender/tests/files/usd/*.blend` | Blender upstream | zstd | existing tests |

The five large production files (120–200 MB: FLOWER-CITY…, HEXA-COMBINED-T-48, LAND-TYPE-15-01, Mannequins-Female--Pose-05, Smile&Driveaway-Animation) stay in `crates/phiber/assets/production` and are referenced by path from benchmarks only.
Measured with `examples/bench_open.rs` (release, cloud container): LAND-TYPE-15-01 (178 MB, 756k verts) opens in 0.38 s — it previously hit the 30 s watchdog.

## Blender ground truth

`../truth/mesh_digest.txt` holds per-mesh digests (vertex/face counts, position, face, UV, material-index and smooth sums) that **Blender 5.0.1 itself** produced for every mesh object in the opened scene of: the three fixtures above, `LAND-TYPE-15-01` and `Mannequins-Female--Pose-05` (from `crates/phiber/assets/production`), and the upstream `operators.blend`, `usd_simple_scene.blend`, `empty.blend`. `tests/blender_truth.rs` recomputes them with phiphx and must match exactly (1,276 meshes); files missing on a machine are skipped. Regenerate with `tests/truth/blender_mesh_digest.py`.

Open times on the same cloud machine (release, warm): LAND-TYPE-15-01 — phiphx 0.52 s vs Blender 5.0.1 2.3 s; Mannequins-Female--Pose-05 — phiphx 0.47 s vs Blender 7.0 s.
