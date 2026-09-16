# Blender → PHX Conversion Summary

This document tracks the seven Blender assets in `ext/phx/assets` plus the native parser status. Each row records the original blend size, the PHX output size (if generated), the high-level scene hierarchy counts, and the current status.

> **📋 Complete API Coverage**: See [`BLENDER_API_COVERAGE.md`](BLENDER_API_COVERAGE.md) for the full mapping of Blender DNA fields to Apple frameworks (RealityKit, VisualKit, Foundation), implementation status, and priority phases.

> **📁 Per-Asset Details**: Each `.blend` file has a corresponding `.md` file (e.g., [`HEXA-COMBINED-T-48.md`](assets/HEXA-COMBINED-T-48.md)) with scene-specific modifiers, instances, and Blender feature analysis.

| Blender File | Blend Size (MB) | PHX Size (bytes) | Scenes | Collections | Objects | Status | Notes |
|--------------|-----------------|------------------|--------|-------------|---------|--------|-------|
| FLORIS_BOLDRIOD_RIG.blend | 5.27 | 67,802 | 1 | 8 | 331 | ✅ converted | PHX in `assets/FLORIS_BOLDRIOD_RIG.phx` |
| FLOWER-CITY-BUILDINGS-DRIJKE-BLUE-CAFFEINE-NEW-CITY.blend | 114.27 | — | — | — | — | ❌ failed | Parser hit `UnexpectedEof` before scene metadata |
| HEXA-COMBINED-T-48.blend | 193.13 | 263,128 | 1 | 250 | 1,224 | ✅ converted | Scene summary stored in log output |
| LAND-TYPE-15-01.blend | 169.86 | 43,732 | 1 | 11 | 219 | ✅ converted | PHX asset covers stylized garden/lighting rig |
| Mannequins-Female--Pose-05.blend | 130.51 | 253,468 | 1 | 25 | 1,222 | ✅ converted | Very high object count (1222) recorded |
| Smile&Driveaway-Animation.blend | 168.99 | — | — | — | — | ❌ failed | Same `UnexpectedEof` reading last chunk |
| THYMUS-BOLDROID-WAVE-003.blend | 6.55 | 56,406 | 1 | 15 | 273 | ✅ converted | PHX made for smaller rig iteration |

## PHX Format Overview
- Magic header: `PHX_SCENE\x01` followed by a little-endian u32 length and JSON metadata.
- Metadata contains scene name, collections, meshes, and transforms.
- Geometry chunks follow the JSON (rare for these scenes because the native parser only emits metadata placeholders).
- PHX files produced above live alongside the original `.blend` files so the Pro Composer can watch the same directory.

## Current Implementation Status

### Phase 1: Core Metadata 
- Object names, types, transforms (loc/rot/scale)
- Collection hierarchy (SC/GR/OB block detection)
- Scene structure
- **Modifier detection** (field presence, active/inactive)
- **Instance detection** (dup_group, instance_collection pointers)

### Phase 2: Geometry Data 
- Vertex positions (MVert.co) — *code ready, needs activation*
- Face indices (MPoly + MLoop) — *structure in place*
- Normals (MVert.no) — *parsing ready*
- UVs (MLoopUV) — *pointer detection ready*

### Phase 3+: Materials, Animation, Rigging 
See [`BLENDER_API_COVERAGE.md`](BLENDER_API_COVERAGE.md) for complete roadmap.

## Workflow & Next Steps
1. **Validate**: Run `cargo run -p phx --example convert_blend_to_phx -- <blend_path>` and observe the `(X instances, Y with modifiers)` summary.
2. **Fix Failing Assets**: For the two EOF-failing files, either:
   - Update parser to handle truncated blocks gracefully
   - Implement Blender CLI fallback for complex files
3. **Activate Geometry**: Enable vertex/face extraction from ME blocks (code exists, needs activation).
4. **Apple Integration**: PHX files are ready for `rust-kit-pro` via `phx::importers::phx_scene::import_phx_scene`.
5. **Extend Coverage**: Pick next feature from `BLENDER_API_COVERAGE.md` Phase 2/3 list.

"""
<p>Generated via native Rust converter on Apr 27, 2026.</p>
"""
