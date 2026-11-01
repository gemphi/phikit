> [JINGLE_REALITYCOMPOSERPRO](../../JINGLE_REALITY_COMPOSER_PRO_GUIDE.md) > [CORRECTING NORMALS AFTER DISPLACEMENT](./--jingle-correcting-normals.md)

---

### 🗺 Documentation Map

- [🏠 RealityKit Home](../../../realitykit/-realitykit.md)
- [🛠 Jingle Reality Composer](../../JINGLE_REALITY_COMPOSER_PRO_GUIDE.md)
- [🎨 Shader Graph Guide](../STEP_BY_STEP_SHADER_GRAPH_GUIDE.md)

---

# Correcting normals after vertex displacement with Jingle Shader Graph

Approximate new vertex normals for Jingle materials that displace a mesh's vertices to ensure correct lighting and shading.

## Overview

Vertex displacement shaders in Jingle can create dynamic effects like water waves, terrain undulation, and wind-blown foliage. However, displacing vertex positions without updating the vertex normals leads to incorrect lighting calculations, as the shader still uses the original mesh normals.

To correct this, you can approximate the new normal direction by sampling the displacement of neighboring points on the surface and calculating their cross product.

## 1. Create a Displacement Node Graph

Start by defining a reusable displacement function. This graph takes a position and outputs a model position offset.

![Displacement Node Graph](./../../assets/correcting-normals-4.png)

```rust
use jingle::material::shader_graph::{ShaderGraphMaterial, ShaderNode, NodeId};

fn build_displacement_graph(graph: &mut ShaderGraphMaterial) -> NodeId {
    let position = graph.add_node(ShaderNode::Position);
    let time = graph.add_node(ShaderNode::Time);
    
    // Parameters (RealityKit uniform inputs)
    let amplitude = graph.add_node(ShaderNode::Scalar { value: 0.5 });
    let frequency = graph.add_node(ShaderNode::Scalar { value: 2.0 });
    let rate = graph.add_node(ShaderNode::Scalar { value: 1.0 });
    
    // Calculate noise offset: noise(pos * freq + time * rate) * amp
    let spatial_part = graph.add_node(ShaderNode::Multiply { a: position, b: frequency });
    let temporal_part = graph.add_node(ShaderNode::Multiply { a: time, b: rate });
    let noise_input = graph.add_node(ShaderNode::Add { a: spatial_part, b: temporal_part });
    
    let noise = graph.add_node(ShaderNode::Noise { uv: noise_input, scale: 1.0, octaves: 3 });
    let offset = graph.add_node(ShaderNode::Multiply { a: noise, b: amplitude });
    
    offset
}
```

## 2. Set Up the Neighbors Node Graph

To calculate the new normal, we need two orthogonal neighboring positions.

![Neighbors Node Graph](./../../assets/correcting-normals-8.png)

```rust
fn build_neighbors_graph(graph: &mut ShaderGraphMaterial, pos: NodeId, dist: f32) -> (NodeId, NodeId) {
    let normal = graph.add_node(ShaderNode::Normal);
    
    // Find tangent and bitangent using an arbitrary vector
    let arb = graph.add_node(ShaderNode::Vector3 { value: Vec3::X });
    let tangent = graph.add_node(ShaderNode::CrossProduct { a: normal, b: arb });
    let bitangent = graph.add_node(ShaderNode::CrossProduct { a: normal, b: tangent });
    
    // Neighbor 1: pos + tangent * dist
    let n1_offset = graph.add_node(ShaderNode::Multiply { a: tangent, b: graph.add_node(ShaderNode::Scalar { value: dist }) });
    let n1 = graph.add_node(ShaderNode::Add { a: pos, b: n1_offset });
    
    // Neighbor 2: pos + bitangent * dist
    let n2_offset = graph.add_node(ShaderNode::Multiply { a: bitangent, b: graph.add_node(ShaderNode::Scalar { value: dist }) });
    let n2 = graph.add_node(ShaderNode::Add { a: pos, b: n2_offset });
    
    (n1, n2)
}
```

## 3. Sample Displacement at Neighbors

We must apply the same displacement logic to the original vertex and its two neighbors.

![Sample Displacement at Neighbors](./../../assets/correcting-normals-10.png)

```rust
fn build_displaced_positions(graph: &mut ShaderGraphMaterial) -> (NodeId, NodeId, NodeId) {
    let p0 = graph.add_node(ShaderNode::Position);
    let (p1, p2) = build_neighbors_graph(graph, p0, 0.01);
    
    // Displace all three points
    let d0 = build_displacement_at(graph, p0);
    let d1 = build_displacement_at(graph, p1);
    let d2 = build_displacement_at(graph, p2);
    
    // New positions
    let pos0 = graph.add_node(ShaderNode::Add { a: p0, b: d0 });
    let pos1 = graph.add_node(ShaderNode::Add { a: p1, b: d1 });
    let pos2 = graph.add_node(ShaderNode::Add { a: p2, b: d2 });
    
    (pos0, pos1, pos2)
}
```

## 4. Compute the Corrected Normal

Finally, calculate the new normal by taking the cross product of the vectors pointing to the displaced neighbors.

![Corrected Normal Graph](./../../assets/correcting-normals-11.png)

```rust
fn compute_corrected_normal(graph: &mut ShaderGraphMaterial) {
    let (pos0, pos1, pos2) = build_displaced_positions(graph);
    
    // Vectors to displaced neighbors
    let v1 = graph.add_node(ShaderNode::Subtract { a: pos1, b: pos0 });
    let v2 = graph.add_node(ShaderNode::Subtract { a: pos2, b: pos0 });
    
    // Corrected normal
    let new_normal = graph.add_node(ShaderNode::CrossProduct { a: v1, b: v2 });
    let normalized = graph.add_node(ShaderNode::Normalize { value: new_normal });
    
    // Connect to material outputs
    graph.connect_position_output(pos0);
    graph.connect_normal_output(normalized);
}
```

## Summary of Benefits

- **Accurate Lighting**: Specular highlights and diffuse shading respond correctly to the displaced surface.
- **Improved Visual Fidelity**: Avoids "flat" lighting on complex displaced geometry.
- **Dynamic Response**: Normal updates in real-time as the displacement changes (e.g., with time).

---

## Next Steps

- [Creating a Fresnel Outline Effect](./--jingle-creating-fresnel-outline.md)
- [Creating a Vertex Displacement Material](./--jingle-creating-vertex-displacement.md)
- [Jingle Material Basics](../building-materials/--jingle-building-materials.md)

---

*Generated for Jingle RealityKit Implementation v0.1.0*
