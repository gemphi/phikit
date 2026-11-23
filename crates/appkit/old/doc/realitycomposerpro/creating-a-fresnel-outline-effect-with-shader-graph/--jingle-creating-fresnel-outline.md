> [JINGLE_REALITYCOMPOSERPRO](../../JINGLE_REALITY_COMPOSER_PRO_GUIDE.md) > [CREATING A FRESNEL OUTLINE EFFECT](./--jingle-creating-fresnel-outline.md)

---

### 🗺 Documentation Map

- [🏠 RealityKit Home](../../../realitykit/-realitykit.md)
- [🛠 Jingle Reality Composer](../../JINGLE_REALITY_COMPOSER_PRO_GUIDE.md)
- [🎨 Shader Graph Guide](../STEP_BY_STEP_SHADER_GRAPH_GUIDE.md)

---

# Creating a Fresnel outline effect with Jingle Shader Graph

Highlight your Jingle entities by adding a dynamic emissive outline effect using the Fresnel approximation.

## Overview

Emphasizing the edges of a spatial entity is an effective way to provide visual feedback for selection, focus, or interaction. In Jingle, you can achieve this using the Fresnel effect, which calculates shading based on the angle between the viewer and the surface normal.

By applying an emissive color where the viewer's direction is perpendicular to the surface normal, you create a "rim light" or outline effect that scales naturally with distance and orientation.

## 1. Prepare the Fresnel Node Graph

In Jingle, you can build this logic using the `ShaderGraphMaterial` API. We'll create a reusable function to calculate the Fresnel falloff.

![Main Shader Graph](./../../assets/fresnel-1.png)

```rust
use jingle::material::shader_graph::{ShaderGraphMaterial, ShaderNode, NodeId};
use glam::Vec4;

fn build_fresnel_logic(graph: &mut ShaderGraphMaterial, falloff: f32) -> NodeId {
    let normal = graph.add_node(ShaderNode::Normal);
    let view_dir = graph.add_node(ShaderNode::ViewDirection);
    
    // 1. Calculate dot product (cosine of the angle)
    let dot = graph.add_node(ShaderNode::DotProduct { a: view_dir, b: normal });
    
    // 2. Clamp to 0..1 to avoid negative values
    let clamped = graph.add_node(ShaderNode::Clamp { value: dot, min: 0.0, max: 1.0 });
    
    // 3. Invert the value (Fresnel is stronger at edges where dot is 0)
    let inverted = graph.add_node(ShaderNode::OneMinus { value: clamped });
    
    // 4. Apply falloff power for sharpness
    let falloff_node = graph.add_node(ShaderNode::Scalar { value: falloff });
    let fresnel = graph.add_node(ShaderNode::Power { base: inverted, exponent: falloff_node });
    
    fresnel
}
```

## 2. Compose the Outline Effect

Now use the Fresnel value to drive the emissive component of your material.

![Interior Fresnel Graph](./../../assets/fresnel-2.png)

```rust
fn create_outline_material(outline_color: Vec4, falloff: f32) -> ShaderGraphMaterial {
    let mut graph = ShaderGraphMaterial::new();
    
    // Calculate Fresnel falloff
    let fresnel = build_fresnel_logic(&mut graph, falloff);
    
    // Define the outline color
    let color_node = graph.add_node(ShaderNode::Color { color: outline_color });
    
    // Multiply color by Fresnel intensity
    let emissive = graph.add_node(ShaderNode::Multiply { a: color_node, b: fresnel });
    
    // Connect to material outputs
    // 1. Emissive glow around edges
    graph.connect_emissive_output(emissive);
    
    // 2. Base surface color (optional, keep dark for highlight only)
    let base = graph.add_node(ShaderNode::Color { color: Vec4::new(0.1, 0.1, 0.1, 1.0) });
    graph.connect_albedo_output(base);
    
    graph
}
```

## 3. Results and Tuning

The Fresnel effect creates a glow that is parallel to the viewer at the center of the object and becomes more intense toward the edges where the surface normal becomes perpendicular to the view direction.

![Fresnel Result](./../../assets/fresnel-6.png)

### Key Parameters

| Parameter | Jingle Node | Impact |
|-----------|-------------|---------|
| **Outline Color** | `ShaderNode::Color` | Changes the hue of the highlight (e.g., Gold for selection, Red for danger). |
| **Falloff** | `ShaderNode::Power` | Controls the "thickness" of the outline. Higher values make a thinner, sharper edge. |
| **Emissive Intensity** | `ShaderNode::Multiply` | Controls the brightness of the glow. |

---

## Next Steps

- [Correcting Normals After Displacement](./--jingle-correcting-normals.md)
- [Creating a Vertex Displacement Material](./--jingle-creating-vertex-displacement.md)
- [Jingle Project Setup](../JINGLE_PROJECT_SETUP.md)

---

*Generated for Jingle RealityKit Implementation v0.1.0*
