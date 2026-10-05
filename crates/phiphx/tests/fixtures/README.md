# .blend fixtures

| File | Origin | Header | Notes |
|---|---|---|---|
| `FLORIS_BOLDRIOD_RIG.blend` (5.5 MB) | moved from `crates/phiber/assets/production` | legacy `BLENDER-v306` | 331 objects, 32 parented to an armature |
| `THYMUS-BOLDROID-WAVE-003.blend` (6.9 MB) | same | legacy `BLENDER-v306` | 9 cameras, 4 lights, curve-parented camera |
| `TEXTURED-PACKED.blend` (88 KB) + `tex/magenta.png` | saved by Blender 5.0.1 (`pip install bpy`) | v1, zstd | plane with a **packed** 2×2 image (BL red, BR green, TL blue, TR yellow) and a cube using `//tex/magenta.png` (#26) |
| `PROCEDURAL.blend` (104 KB) | saved by Blender 5.0.1 (`make_procedural.py`) | v1, zstd | three planes: Principled Base Color ← Color Ramp ← Noise over a Mapping of Object coordinates with a Voronoi Bump on the Normal; a Mix Shader whose Fac is a Noise through a reroute; a Voronoi's Color over UVs (`tests/procedural_truth.rs`); and eight planes with one texture node each — Checker, Brick, Gradient, Magic, Wave, a Ridged Noise, Distance to Edge, Smooth F1 Minkowski (`tests/texture_nodes_truth.rs`) |
| `LIGHTS.blend` (81 KB) | saved by Blender 5.0.1 (`make_lights.py`) | v1, zstd | one light of each kind, each with a shape no default would give: a Sun at 10°, a Point at radius 0.4, an Area at size 2.5, a Spot at radius 0.2 and a 60° cone, and three area shapes — a Rectangle 3 × 0.5 on an object scaled 2 in X, a Disk at 1.5 whose Size Y stays at the default, an Ellipse 2 × 0.75 (`tests/light_truth.rs`) |
| `REFS/blender/blender/tests/files/modeling/operators.blend` | Blender upstream | **v1 `BLENDER17-01v0502`** (Blender 5.2) + LargeBHead8 | used by `importer::blend::tests::loads_v1_header_large_bhead_file` |
| `REFS/blender/blender/tests/files/usd/*.blend` | Blender upstream | zstd | existing tests |

The five large production files (120–200 MB: FLOWER-CITY…, HEXA-COMBINED-T-48, LAND-TYPE-15-01, Mannequins-Female--Pose-05, Smile&Driveaway-Animation) stay in `crates/phiber/assets/production` and are referenced by path from benchmarks only.
Measured with `examples/bench_open.rs` (release, cloud container): LAND-TYPE-15-01 (178 MB, 756k verts) opens in 0.38 s — it previously hit the 30 s watchdog.

## Blender ground truth

`../truth/mesh_digest.txt` holds per-mesh digests (vertex/face counts, position, face, UV, material-index and smooth sums) that **Blender 5.0.1 itself** produced for every mesh object in the opened scene of: the three fixtures above, `LAND-TYPE-15-01` and `Mannequins-Female--Pose-05` (from `crates/phiber/assets/production`), and the upstream `operators.blend`, `usd_simple_scene.blend`, `empty.blend`. `tests/blender_truth.rs` recomputes them with phiphx and must match exactly (1,276 meshes); files missing on a machine are skipped. Regenerate with `tests/truth/blender_mesh_digest.py`.

Open times on the same cloud machine (release, warm): LAND-TYPE-15-01 — phiphx 0.52 s vs Blender 5.0.1 2.3 s; Mannequins-Female--Pose-05 — phiphx 0.47 s vs Blender 7.0 s.
