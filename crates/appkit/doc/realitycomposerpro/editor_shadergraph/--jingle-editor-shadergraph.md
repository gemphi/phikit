> [JINGLE_REALITYCOMPOSERPRO](../../JINGLE_REALITY_COMPOSER_PRO_GUIDE.md) > [DESIGNING MATERIALS WITH SHADER GRAPH](./--jingle-editor-shadergraph.md)

---

### 🗺 Documentation Map

- [🏠 RealityKit Home](../../../realitykit/-realitykit.md)
- [🛠 Jingle Reality Composer](../../JINGLE_REALITY_COMPOSER_PRO_GUIDE.md)
- [🎨 Shader Graph Guide](../STEP_BY_STEP_SHADER_GRAPH_GUIDE.md)

---

# Designing materials with Jingle Shader Graph

Create dynamic, stylized, and high-performance materials using Jingle's node-based visual editor in Rust.

## Overview

In Jingle, you can create two types of materials for your spatial entities:

1. **`Material::pbr()`**: Optimized for realistic real-world surfaces (metal, plastic, glass).
2. **`ShaderGraphMaterial`**: Provides full control through a node-based logic system for stylized or dynamic effects.

While PBR materials are excellent for standard assets, Jingle's Shader Graph allows you to implement custom logic, such as cartoon shading, animated textures, or complex procedural patterns, without writing raw WGSL or Metal code.

![Jingle Shader Graph Editor](./../../assets/rcpro-shadergraphwindow.png.png)

## Designing with Nodes

Jingle materials built in the editor can affect both the appearance (**Surface**) and the shape (**Geometry**) of an entity.

- **Custom Surface**: Nodes connected here control the final color, roughness, and emissive properties of each pixel. This is equivalent to a fragment shader.
- **Geometry Modifier**: Nodes connected here modify the vertex positions and normals of the mesh. This is equivalent to a vertex shader.

### Building a Graph in Rust

Each node in the Jingle Shader Graph represents a mathematical operation or a data lookup (like a texture sample).

```rust
use jingle::material::shader_graph::{ShaderGraphMaterial, ShaderNode, NodeId};

fn create_animated_glow_material() -> ShaderGraphMaterial {
    let mut graph = ShaderGraphMaterial::new();
    
    // 1. Get current time
    let time = graph.add_node(ShaderNode::Time);
    
    // 2. Calculate a pulsing value using Sine
    let pulsing = graph.add_node(ShaderNode::Sin {
        input: graph.add_node(ShaderNode::Multiply {
            a: time,
            b: graph.add_node(ShaderNode::Scalar { value: 2.0 }), // Frequency
        }),
    });
    
    // 3. Connect to Emissive Color output
    let color = graph.add_node(ShaderNode::Color {
        color: Vec4::new(0.0, 1.0, 0.5, 1.0), // Teal glow
    });
    let intensity = graph.add_node(ShaderNode::Multiply { a: color, b: pulsing });
    
    graph.connect_emissive_output(intensity);
    
    graph
}
```

## Update Material Values at Runtime

One of the most powerful features of Jingle's Shader Graph is the ability to expose parameters as **Promoted Inputs** (uniforms) that can be changed dynamically from your Rust code.

### Exposing Parameters

In the Jingle editor, any constant node can be "Promoted" to an input parameter. In code, you define these as uniform inputs:

```rust
// In your shader construction
let health_input = graph.add_node(ShaderNode::Uniform {
    name: "Health".to_string(),
    default_value: 1.0,
});
```

### Setting Parameters at Runtime

You can update these parameters during your game loop to reflect changes in your application state (e.g., a robot glowing red when damaged).

```rust
fn update_robot_material(scene: &mut Scene, robot_entity: Entity, health: f32) {
    if let Some(mut model) = scene.get_component_mut::<ModelComponent>(robot_entity) {
        if let Some(material) = model.materials.get_mut(0) {
            // Update the "Health" parameter on the ShaderGraphMaterial
            // Equivalent to Swift's setParameter(name:value:)
            material.set_parameter("Health", health);
        }
    }
}
```

## Node Library

Jingle includes a comprehensive library of nodes:
- **Math**: Add, Multiply, Sin, Cos, DotProduct, CrossProduct, Mix (Lerp).
- **Texture**: Texture2D, TextureCube, NormalMap.
- **Geometry**: Position, Normal, UV, Tangent, ViewDirection.
- **Procedural**: Noise3D, Voronoi, Checkerboard.

For a full list of available nodes and their functions, see the [Shader Graph API Reference](../../../../../phi/ext/jingle/src/material/shader_graph.rs).

---

## Next Steps

- [Creating a Fresnel Outline Effect](../creating-a-fresnel-outline-effect-with-shader-graph/--jingle-creating-fresnel-outline.md)
- [Creating a Vertex Displacement Material](../creating-a-vertex-displacement-material-with-shader-graph/--jingle-creating-vertex-displacement.md)
- [Loading ShaderGraph Entities](../loading-entities-with-shadergraph-materials/--jingle-loading-shadergraph-entities.md)

---

*Generated for Jingle RealityKit Implementation v0.1.0*
