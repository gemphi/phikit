> [JINGLE_REALITYCOMPOSERPRO](../../JINGLE_REALITY_COMPOSER_PRO_GUIDE.md) > [BUILDING MATERIALS](./--jingle-building-materials.md)

---

### 🗺 Documentation Map

- [🏠 RealityKit Home](../../../realitykit/-realitykit.md)
- [🛠 Jingle Reality Composer](../../JINGLE_REALITY_COMPOSER_PRO_GUIDE.md)
- [🕶 visionOS Home](../../../visionos/-visionos.md)

---

# Building Jingle materials

Design and create high-performance materials for your 3D assets in the Jingle environment using Rust.

## Topics

### Jingle Materials

- [Applying Jingle materials to your assets](../applying-realitykit-materials-on-imported-assets/--jingle-applying-materials.md)
- [Working with PBR Materials](../../building-materials/--building-materials.md)

### Shader Graph (Node-based)

- [Designing materials with Jingle Shader Graph](../editor_shadergraph/--jingle-editor-shadergraph.md)
- [Loading entities with ShaderGraph materials](../loading-entities-with-shadergraph-materials/--jingle-loading-shadergraph-entities.md)
- [Creating a Fresnel outline effect with Shader Graph](../creating-a-fresnel-outline-effect-with-shader-graph/--jingle-creating-fresnel-outline.md)
- [Creating a vertex displacement material with Shader Graph](../creating-a-vertex-displacement-material-with-shader-graph/--jingle-creating-vertex-displacement.md)
- [Correcting normals after vertex displacement with Shader Graph](../correcting-normals-after-vertex-displacement-with-shader-graph/--jingle-correcting-normals.md)

---

## Code Example: Material Quick-Start

```rust
use jingle::reality_kit::Material;
use glam::Vec3;

fn create_standard_materials() {
    // 1. Simple PBR Material
    let plastic = Material::pbr()
        .with_base_color(Vec3::new(0.8, 0.8, 0.8))
        .with_roughness(0.2);
        
    // 2. Emissive Material (Glow)
    let neon = Material::emissive(Vec3::new(0.0, 1.0, 0.0), 5.0);
    
    // 3. Diffuse Material (Matte)
    let wood = Material::diffuse(Vec3::new(0.4, 0.2, 0.1));
}
```

---

*Generated for Jingle RealityKit Implementation v0.1.0*
