# reference

What phiecs reads and is measured against. None is built.

Apple's interfaces are SDK files and stay where they are on disk; they are
listed here by path (from `gemphi/`) so there is one place to look.

- **RealityFoundation, the ECS contract** (Entity, Component, System,
  EntityQuery, events, Transform, animation, physics, sync):
  `phi/crates/kitpro/src/examples/headers/MacOSX-20260419T054331Z-3-001/MacOSX/RealityFoundation/SwiftInterfaces/arm64e-apple-macos.swiftinterface`
  - the same for every platform: `phi/apple-source/ruskit/Interfaces/{MacOSX,iPhoneOS,XROS}/RealityFoundation`
- **SceneKit headers** (the documented update order: update, animation,
  physics, constraints, render): `phi/crates/kitpro/src/examples/headers/SceneKit.framework-20260419T035550Z-3-001/`
- **The Rust transcriptions of those** (shapes only, nothing runs):
  `phikit/crates/apple/src/{realitykit,reality-foundation,services/ecs}`,
  `phi/crates/macos/src/{realitykit,realityfoundation,scenekit}`
- **The full phiecs** (tree, state, conditions, queries, kit, quest):
  `phinet/crates/phiecs`
- **phiora, the table shape a saved world follows:**
  `puikit/app/phiora-server`

The row-by-row mapping and the order of work are in `../docs/APPLE.md`.
