> [JINGLE_REALITYCOMPOSERPRO](../../JINGLE_REALITY_COMPOSER_PRO_GUIDE.md) > [CREATING A VERTEX DISPLACEMENT MATERIAL](./--jingle-creating-vertex-displacement.md)

---

### 🗺 Documentation Map

- [🏠 RealityKit Home](../../../realitykit/-realitykit.md)
- [🛠 Jingle Reality Composer](../../JINGLE_REALITY_COMPOSER_PRO_GUIDE.md)
- [🎨 Shader Graph Guide](../STEP_BY_STEP_SHADER_GRAPH_GUIDE.md)

---

# Creating a vertex displacement material with Jingle Shader Graph

Animate the vertices of your Jingle meshes over time with procedural 3D noise using the Rust implementation.

## Overview

Procedurally distorting the shape of a mesh within a shader is a powerful technique for creating organic, dynamic effects like water ripples, waving grass, or magical distortions. This effect, known as **Vertex Displacement**, moves vertices from their original positions based on mathematical functions or noise.

In Jingle, you achieve this by connecting a displacement calculation to the **Position Offset** output of your `ShaderGraphMaterial`.

![Vertex Displacement Example](./../../assets/vertex-displacement-4-5.png)

## 1. Set Up the Geometry Modifier

The first step is to define how the vertex position will be modified. In Jingle, we use the `ShaderGraphMaterial` to calculate an offset that is added to the base vertex position.

![Main Shader Body](./../../assets/vertex-displacement-3.png)

```rust
use jingle::material::shader_graph::{ShaderGraphMaterial, ShaderNode, NodeId};

fn create_base_displacement_graph() -> ShaderGraphMaterial {
    let mut graph = ShaderGraphMaterial::new();
    
    // Get original vertex position
    let position = graph.add_node(ShaderNode::Position);
    
    // Calculate offset logic here...
    let offset = build_displacement_logic(&mut graph, position);
    
    // Connect to Position Offset output (RealityKit Geometry Modifier)
    graph.connect_position_output(offset);
    
    graph
}
```

## 2. Displace the Vertices with Noise

Using a `Noise 3D` node is the most common way to create smooth, natural-looking displacement.

![Noise Interior](./../../assets/vertex-displacement-4.png)

```rust
fn build_noise_displacement(graph: &mut ShaderGraphMaterial, pos: NodeId) -> NodeId {
    // Generate Perlin noise based on vertex position
    let noise = graph.add_node(ShaderNode::Noise3D {
        position: pos,
        octaves: 3,
    });
    
    noise
}
```

## 3. Controlling Displacement Properties

To make the effect adjustable, we use parameters for Amplitude, Scale, and Rate.

### Amplitude (Intensity)
Control how far the vertices are displaced.

![Amplitude Graph](./../../assets/vertex-displacement-5.png)

```rust
let amplitude = graph.add_node(ShaderNode::Scalar { value: 0.5 });
let displaced = graph.add_node(ShaderNode::Multiply { a: noise, b: amplitude });
```

### Scale (Spatial Frequency)
Control the size of the distortion patterns across the mesh.

![Scale Graph](./../../assets/vertex-displacement-6.png)

```rust
let scale = graph.add_node(ShaderNode::Scalar { value: 2.0 });
let scaled_pos = graph.add_node(ShaderNode::Multiply { a: position, b: scale });
// Pass scaled_pos to the noise node
```

### Rate (Temporal Frequency)
Control the speed of the animation over time.

![Rate Graph](./../../assets/vertex-displacement-8.png)

```rust
let time = graph.add_node(ShaderNode::Time);
let rate = graph.add_node(ShaderNode::Scalar { value: 1.0 });
let time_offset = graph.add_node(ShaderNode::Multiply { a: time, b: rate });

// Combine spatial and temporal inputs
let noise_input = graph.add_node(ShaderNode::Add { a: scaled_pos, b: time_offset });
```

## 4. Final Implementation

Combining all elements into a single Jingle shader graph:

```rust
pub fn build_full_displacement_shader() -> ShaderGraphMaterial {
    let mut graph = ShaderGraphMaterial::new();
    
    let position = graph.add_node(ShaderNode::Position);
    let time = graph.add_node(ShaderNode::Time);
    
    // Uniform parameters
    let amplitude = graph.add_node(ShaderNode::Scalar { value: 0.4 });
    let scale = graph.add_node(ShaderNode::Scalar { value: 3.0 });
    let rate = graph.add_node(ShaderNode::Scalar { value: 1.5 });
    
    // Calculate animated noise input
    let scaled_pos = graph.add_node(ShaderNode::Multiply { a: position, b: scale });
    let time_vec = graph.add_node(ShaderNode::Multiply { a: time, b: rate });
    let noise_pos = graph.add_node(ShaderNode::Add { a: scaled_pos, b: time_vec });
    
    // Generate noise
    let noise = graph.add_node(ShaderNode::Noise3D { position: noise_pos, octaves: 4 });
    
    // Apply intensity
    let offset = graph.add_node(ShaderNode::Multiply { a: noise, b: amplitude });
    
    // Output
    graph.connect_position_output(offset);
    
    // Add a simple color for the surface
    let color = graph.add_node(ShaderNode::Color { color: Vec4::new(0.2, 0.5, 0.8, 1.0) });
    graph.connect_albedo_output(color);
    
    graph
}
```

---

## Important Note: Correcting Normals

When you displace vertices, the surface normals are no longer accurate for the new shape, which can lead to flat-looking lighting. To fix this, you must also calculate new normals.

See [Correcting Normals After Displacement](./--jingle-correcting-normals.md) for the complete solution.

---

## Next Steps

- [Correcting Normals After Displacement](./--jingle-correcting-normals.md)
- [Creating a Fresnel Outline Effect](./--jingle-creating-fresnel-outline.md)
- [Jingle Shader Graph Basics](../editor_shadergraph/--jingle-editor-shadergraph.md)

---

*Generated for Jingle RealityKit Implementation v0.1.0*
