# .blend fixtures

| File | Origin | Header | Notes |
|---|---|---|---|
| `FLORIS_BOLDRIOD_RIG.blend` (5.5 MB) | moved from `gemphi/phi/crates/phiphx/assets` | legacy `BLENDER-v306` | 331 objects, 32 parented to an armature |
| `THYMUS-BOLDROID-WAVE-003.blend` (6.9 MB) | same | legacy `BLENDER-v306` | 9 cameras, 4 lights, curve-parented camera |
| `REFS/blender/blender/tests/files/modeling/operators.blend` | Blender upstream | **v1 `BLENDER17-01v05xx` + LargeBHead8** | used by `importer::blend::tests::loads_v1_header_large_bhead_file` |
| `REFS/blender/blender/tests/files/usd/*.blend` | Blender upstream | zstd | existing tests |

The five large production files (120–200 MB: FLOWER-CITY…, HEXA-COMBINED-T-48, LAND-TYPE-15-01, Mannequins-Female--Pose-05, Smile&Driveaway-Animation) stay in `gemphi/phi/crates/phiphx/assets` and are referenced by path from benchmarks only.
Measured with `examples/bench_open.rs` (release, cloud container): LAND-TYPE-15-01 (178 MB, 756k verts) opens in 0.38 s — it previously hit the 30 s watchdog.
