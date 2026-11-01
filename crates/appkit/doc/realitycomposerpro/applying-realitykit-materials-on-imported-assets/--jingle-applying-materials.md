> [JINGLE_REALITYCOMPOSERPRO](../../JINGLE_REALITY_COMPOSER_PRO_GUIDE.md) > [APPHXING JINGLE MATERIALS TO ASSETS](./--jingle-applying-materials.md)

---

### 🗺 Documentation Map

- [🏠 RealityKit Home](../../../realitykit/-realitykit.md)
- [🛠 Jingle Reality Composer](../../JINGLE_REALITY_COMPOSER_PRO_GUIDE.md)
- [🕶 visionOS Home](../../../visionos/-visionos.md)

---

# Applying Jingle materials to your assets

Work with materials in the Jingle environment to enhance the appearance of your spatial models using Rust.

## Overview

When you import a USDZ or OBJ model into Jingle, the `AssetManager` extracts material information and creates corresponding `Material` components. These materials are managed within your project and can be modified at runtime to change the appearance of your entities.

In the PhiPro editor, materials are organized within the hierarchy view, allowing you to quickly identify which entities share the same visual properties.

![Jingle Material Hierarchy](./../../assets/rcpro-materialhierarchy.png)

## Editing Materials in Rust

You can programmatically adjust material properties such as Roughness, Metallic, and Emissive Color using the Jingle API. This is equivalent to using the inspector in Reality Composer Pro.

![Jingle Material Inspector](./../../assets/rcpro-materialinspector-pbr.png)

### Physically Based Rendering (PBR) Materials

Jingle's PBR materials allow you to define high-quality surface properties. You can replace textures, colors, or values directly in your code:

```rust
use jingle::reality_kit::Material;
use glam::Vec3;

fn configure_metal_material(material: &mut Material) {
    // Set base color (Reality Composer Pro "Diffuse Color" equivalent)
    material.set_base_color(Vec3::new(0.8, 0.8, 0.8));
    
    // Adjust physical properties
    material.set_metallic(0.9);
    material.set_roughness(0.1);
    
    // Enable emissive glow
    material.set_emissive(Vec3::new(0.1, 0.2, 1.0), 2.0);
}
```

### Material Sections

Jingle materials store properties in logical sections similar to the Reality Composer Pro inspector:

- **Material Properties**: Core attributes like Shader type, Face Culling, Diffuse Color, and Roughness.
- **Advanced Attributes**: Fine-tuning parameters like Clearcoat, Ambient Occlusion, and Opacity Threshold.
- **References**: Managed by the ECS, allowing you to see which entities are bound to a specific material resource.

## Creating Materials from Scratch

You can create new materials dynamically and apply them to any entity in your scene.

```rust
use jingle::reality_kit::{Scene, Entity, ModelComponent, Material};

fn apply_new_material(scene: &mut Scene, entity: Entity) {
    // Create a new PBR material
    let mut gold_material = Material::pbr()
        .with_base_color(Vec3::new(1.0, 0.84, 0.0))
        .with_metallic(1.0)
        .with_roughness(0.2);
    
    // Apply to an entity (updates the ModelComponent)
    if let Some(mut model) = scene.get_component_mut::<ModelComponent>(entity) {
        model.materials = vec![gold_material];
    }
}
```

## Material Reuse and Binding

Any changes you make to a shared material resource will affect all entities bound to that material. This allows for efficient global updates to your scene's appearance.

```rust
fn update_shared_material(manager: &mut AssetManager, material_handle: AssetHandle) {
    if let Some(material) = manager.get_material_mut(material_handle) {
        // This update will be reflected in all entities using this handle
        material.set_roughness(0.5);
    }
}
```

---

## Next Steps

- [Designing Materials with Shader Graph](../editor_shadergraph/--editor_shadergraph.md)
- [Adding Assets to Your Scene](../adding-assets-into-your-reality-composer-pro-scene/--jingle-adding-assets-to-scene.md)
- [API Comparison](../../JINGLE_REALITY_COMPOSER_PRO_GUIDE.md#api-comparison)

---

*Generated for Jingle RealityKit Implementation v0.1.0*
