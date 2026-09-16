# PHX → Rust-Kit-Pro Integration Notes

## Goals
- Surface the newly extracted PHX metadata (objects, modifiers, instances, placeholders for vertices/faces) to the Rust-Kit-Pro asset pipeline.
- Ensure the Swift/RealityKit side can reference the same data through the existing Apple-oriented export/import surface.
- Document how PHX assets map back to RealityKit components (entities, meshes, materials) and how to consume them from `rust-kit-pro`.

## Current Status
1. Native Rust parser now outputs `PhxScene` metadata with:
   - Scene/collection/object names
   - Modifier and instance flags for each object
   - Triangle/vertex blobs (per-object placeholder or scanned data) written into the PHX chunk as soon as a mesh is found
2. The generated `.phx` files live next to the `.blend` files and can already be imported via `phx::importers::phx_scene::import_phx_scene`.
3. `CONVERSION_SUMMARY.md`, `BLENDER_API_COVERAGE.md`, and per-asset `.md` documents describe metadata availability, modifiers, and Blender feature coverage.

## Integration Plan
1. **Rust-Kit-Pro importer**
   - Ensure `rust-kit-pro` depends on `phx = { path = "../../ext/phx" }` (already true).
   - Update `rust-kit-pro/src/main.rs` `import_asset_from_path` to treat `.blend` files as follows:
     1. Run the native parser (`phx::importers::blend_native::import_blend_native`).
     2. Serialize the resulting `BlendScene` into a `phx_scene::PhxScene` (or reuse `phx::exporters::phx_scene` helper) that includes metadata + placeholder geometry.
     3. Persist the resulting `.phx` path back into the project assets so the Pro Composer UI sees the converted file.
   - Add unit test verifying `rust-kit-pro` can import a PHX generated from one of the assets (e.g., `LAND-TYPE-15-01`).

2. **Swift/RealityKit Bridging**
   - The PHX exports contain entity metadata (names/types/collections) that map to RealityKit `Entity`/`MeshResource`/`MaterialResource`.
   - Update the `phi/crates/rust-kit-pro/doc/read/realitycomposerpro/STEP_BY_STEP_RUST_KIT_PRO_EDITOR.md` (or a new section) to describe which PHX chunks correspond to:
     - `Entity.name` ← Blender `ID.name`
     - `MeshResource.positions` ← PHX mesh chunk vertex buffer
     - `MeshResource.indices` ← PHX index buffer
     - Modifier/instance flags ← annotate RealityKit Entities via `MetadataComponent`
   - Consider adding a new `phx::importers::phx_scene::PhxMetadata` struct to represent modifiers/instances for downstream Swift code.

3. **Swift/Objective-C (AppKit/Foundation) APIs**
   - The PHX file includes JSON metadata that can be wrapped in `NSDictionary` and consumed by VisualKit (already previously documented). Ensure the importer populates:
     - `metadata.scene` → `SceneEntity.name`
     - `metadata.collections` → `Entity` hierarchy for RealityKit anchors
   - Provide the same metadata to AppKit/Swift via JSON bridging so the macOS visuals can respond to modifiers and instancing (e.g., by setting `Entity.isInstance = true`).

4. **Validation & Testing**
   - Run `cargo run -p phx --example convert_blend_to_phx -- <blend>` for each asset and ensure PHX outputs contain non-zero vertex counts.
   - Load generated `.phx` files into Rust-Kit-Pro's import dialog and verify the metadata surfaces (names, collections, modifiers).
   - Document known gaps in this file (e.g., actual face loops remain heuristic until full `MPOLY` parsing is implemented).

## Next Steps
- Implement proper `MPOLY`/`MLOOP` reader so faces and UVs are accurate.
- Expose `BlendSceneObject.modifiers` and `.is_instance` via `PhxMetadata` for RealityKit components.
- Wire `PHX` metadata to Swift `VisualKit` items (e.g., `AnimationTrack` references) to ensure feature coverage for modifiers/instances.
