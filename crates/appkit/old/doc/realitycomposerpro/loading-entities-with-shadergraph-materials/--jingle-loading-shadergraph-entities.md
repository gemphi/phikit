> [JINGLE_REALITYCOMPOSERPRO](../../JINGLE_REALITY_COMPOSER_PRO_GUIDE.md) > [LOADING SHADERGRAPH ENTITIES](./--jingle-loading-shadergraph-entities.md)

---

### 🗺 Documentation Map

- [🏠 RealityKit Home](../../../realitykit/-realitykit.md)
- [🛠 Jingle Reality Composer](../../JINGLE_REALITY_COMPOSER_PRO_GUIDE.md)
- [🎨 Shader Graph Guide](../STEP_BY_STEP_SHADER_GRAPH_GUIDE.md)

---

# Loading entities with Jingle ShaderGraph materials

Bring Jingle entities containing complex node-based materials into your spatial application using the Rust implementation.

## Overview

Jingle allows you to create high-fidelity entities with ShaderGraph materials that can be easily loaded and interacted with in your spatial apps. This guide demonstrates how to load these entities from a project package and display them in a dynamic layout using Jingle's `RealityView` and `JingUI` components.

![Shader Graph Samples](./../../assets/creating-materials-overview-image.jpg)

## 1. Create Materials in Jingle ShaderGraph

First, design your materials in the Jingle editor. Your project's `AssetManifest` will track these shaders along with their corresponding 3D models.

![Open Jingle Package](./../../assets/open-package-in-reality-composer-pro.png)

The project package typically includes a variety of shaders:
- **Fresnel Outline**: Highlighting edges.
- **Vertex Displacement**: Organic movement.
- **Procedural Lava**: Dynamic noise patterns.

## 2. Create a Volumetric View in Rust

To display these shaders, we'll use a volumetric window configuration in Jingle.

```rust
use jingle::reality_kit::{WindowGroup, VolumetricWindow};
use crate::projects::botanist::views::ShaderSamplesView;

fn main() {
    // Equivalent to Swift's WindowGroup { ... }.windowStyle(.volumetric)
    let app = VolumetricWindow::new("Shader Samples")
        .with_content(ShaderSamplesView::new())
        .with_size(0.9, 0.5, 0.9); // meters
}
```

## 3. Load Shaders from the Content Package

Use the `AssetManager` to asynchronously load your shader entities.

```rust
use jingle::asset::{AssetManager, AssetHandle};
use jingle::reality_kit::{Entity, Scene};

pub struct ShaderLoader {
    pub shader_entities: Vec<Entity>,
}

impl ShaderLoader {
    async fn load_shaders(&mut self, manager: &mut AssetManager, scene: &mut Scene) {
        println!("📦 Loading shader samples...");
        
        // 1. Resolve the scene root from manifest
        if let Some(scene_root) = manager.load_scene("ShaderSamplesScene").await {
            // 2. Extract child entities containing unique shaders
            for child in scene_root.children() {
                self.shader_entities.push(child);
            }
            
            println!("✅ Loaded {} shader samples", self.shader_entities.len());
        }
    }
}
```

## 4. Add Shader Entities to RealityView Content

Position and scale the loaded entities so they fit within your UI layout.

```rust
use jingle::reality_kit::{Transform, RealityView};
use glam::Vec3;

fn create_shader_preview(entity: Entity, scene: &mut Scene) -> RealityView {
    // 1. Configure the entity for preview
    if let Some(mut transform) = scene.get_component_mut::<Transform>(entity) {
        transform.position = Vec3::ZERO;
        transform.scale = Vec3::splat(0.75);
    }
    
    // 2. Wrap in a RealityView for the UI
    RealityView::new(entity)
        .with_background_effect(true) // Equivalent to .glassBackgroundEffect()
}
```

## 5. Load Entities into a ScrollView

Use Jingle's `ScrollView` (part of `JingUI`) to display the shader samples in a horizontal list.

```rust
use projects::botanist::jingui::{View, VStack, HStack, ScrollView, Text};

impl View for ShaderSamplesView {
    fn render(&mut self, ui: &mut egui::Ui) {
        // Equivalent to Swift's ScrollView(.horizontal) { HStack { ... } }
        ScrollView::horizontal().show(ui, |ui| {
            ui.horizontal(|ui| {
                for entity in &self.loader.shader_entities {
                    ui.vertical(|ui| {
                        // 1. Render the 3D Preview
                        let preview = create_shader_preview(*entity, &mut self.scene);
                        preview.render(ui);
                        
                        // 2. Display the shader name
                        let name = self.scene.get_name(*entity).unwrap_or("Unknown");
                        ui.add(Text::new(name).font_size(14.0));
                    });
                }
            });
        });
    }
}
```

## Result

Your application will now display a scrolling list of 3D objects, each demonstrating a unique ShaderGraph material, fully interactive within the spatial environment.

![Scrolling through Shaders](./../../assets/scrolling-through-shaders.mp4)

---

## Next Steps

- [Designing Materials with Shader Graph](../editor_shadergraph/--jingle-editor-shadergraph.md)
- [Creating a Fresnel Outline Effect](../creating-a-fresnel-outline-effect-with-shader-graph/--jingle-creating-fresnel-outline.md)
- [Jingle Project Setup](../JINGLE_PROJECT_SETUP.md)

---

*Generated for Jingle RealityKit Implementation v0.1.0*
