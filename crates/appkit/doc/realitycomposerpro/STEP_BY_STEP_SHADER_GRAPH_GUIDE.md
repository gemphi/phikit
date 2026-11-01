# Step-by-Step: Designing Materials with Shader Graph

> Complete guide to creating materials using Jingle's node-based Shader Graph system

---

## 🗺 Documentation Map

- [← Back to Reality Composer Pro Guide](./JINGLE_REALITY_COMPOSER_PRO_GUIDE.md)
- [📦 Importing Assets](STEP_BY_STEP_ASSET_IMPORT_GUIDE.md)
- [🏠 RealityKit Home](../realitykit/-realitykit.md)
- [🎨 Shader Graph API](../../../../../phi/ext/jingle/src/material/shader_graph.rs)

---

## 📊 Implementation Status

| Feature | Status | Notes |
|---------|--------|-------|
| **Node Editor UI** | ✅ Working | Drag-drop node canvas |
| **10 Node Types** | ✅ Working | Position, Texture, Add, Multiply, etc. |
| **Pin Connections** | ✅ Working | Visual wire connections |
| **Material Preview** | ✅ Working | Live preview sphere |
| **Real-time Updates** | ✅ Working | Instant material refresh |
| **Node Parameters** | ✅ Working | Editable values per node |
| **Delete/Connect** | ✅ Working | Full interaction |
| **Custom Nodes** | ⚠️ Partial | Basic node registration |
| **Material Export** | ⚠️ Partial | PHX material format |
| **Shader Code Gen** | ❌ Not Started | Planned for future |

**Last Updated:** 2024-01-20  
**App Status:** ✅ Running & Ready  
**Crate:** `phi/crates/rust-kit-pro/src/shader_graph_editor.rs`

---

## Overview

Jingle's Shader Graph provides a node-based visual programming interface for creating materials, equivalent to Reality Composer Pro's Shader Graph editor. This guide walks you through creating materials from simple to complex.

![Shader Graph Interface](./../assets/shader-graph.png)

---

## Table of Contents

1. [Shader Graph Basics](#shader-graph-basics)
2. [Node Types Reference](#node-types-reference)
3. [Step 1: Creating Your First Material](#step-1-creating-your-first-material)
4. [Step 2: Adding Textures](#step-2-adding-textures)
5. [Step 3: Creating a Fresnel Effect](#step-3-creating-a-fresnel-effect)
6. [Step 4: Vertex Displacement](#step-4-vertex-displacement)
7. [Step 5: Procedural Patterns](#step-5-procedural-patterns)
8. [Step 6: Animating Materials](#step-6-animating-materials)
9. [Advanced Techniques](#advanced-techniques)
10. [Full Examples](#full-examples)

---

## Shader Graph Basics

### What is a Shader Graph?

A Shader Graph is a visual node-based system for creating materials. Instead of writing shader code, you connect nodes that represent mathematical operations, texture samples, and material properties.

### Core Concepts

| Concept | Description | Example |
|---------|-------------|---------|
| **Nodes** | Individual operations | Position, Texture, Add, Multiply |
| **Pins** | Connection points on nodes | Input pins, Output pins |
| **Edges** | Connections between nodes | Data flow from one node to another |
| **Graph** | Complete network of nodes | Entire material definition |

---

## Step 1: Creating Your First Material

Let's create a simple colored material.

```rust
use jingle::material::shader_graph::{
    ShaderGraphMaterial, ShaderNode, NodeId
};
use glam::Vec4;

fn create_simple_colored_material() -> ShaderGraphMaterial {
    println!("🎨 Creating simple colored material...");
    
    // Step 1: Create empty graph
    let mut graph = ShaderGraphMaterial::new();
    
    // Step 2: Create color node
    let color_node = graph.add_node(ShaderNode::Color {
        color: Vec4::new(0.2, 0.5, 1.0, 1.0), // Nice blue
    });
    
    // Step 3: Connect to output
    graph.connect_albedo_output(color_node);
    
    println!("   ✅ Simple material created");
    
    graph
}
```

---

## Step 2: Adding Textures

### Basic Texture Sampling

```rust
use jingle::material::shader_graph::{
    ShaderGraphMaterial, ShaderNode
};

fn create_textured_material(
    texture_id: u64
) -> ShaderGraphMaterial {
    println!("🖼️  Creating textured material...");
    
    let mut graph = ShaderGraphMaterial::new();
    
    // Step 1: Get UV coordinates
    let uv = graph.add_node(ShaderNode::UV);
    
    // Step 2: Sample texture
    let texture = graph.add_node(ShaderNode::Texture {
        texture_id,
        uv: uv,
    });
    
    // Step 3: Output
    graph.connect_albedo_output(texture);
    
    println!("   ✅ Textured material created");
    
    graph
}
```

---

## Step 3: Creating a Fresnel Effect

The Fresnel effect makes surfaces brighter at glancing angles (like rim lighting).

```rust
fn build_fresnel_logic(graph: &mut ShaderGraphMaterial, falloff: f32) -> NodeId {
    let normal = graph.add_node(ShaderNode::Normal);
    let view_dir = graph.add_node(ShaderNode::ViewDirection);
    
    // Fresnel = pow(1.0 - clamp(dot(normal, view), 0, 1), falloff)
    let dot = graph.add_node(ShaderNode::DotProduct { a: view_dir, b: normal });
    let clamped = graph.add_node(ShaderNode::Clamp { value: dot, min: 0.0, max: 1.0 });
    let inverted = graph.add_node(ShaderNode::OneMinus { value: clamped });
    let fresnel = graph.add_node(ShaderNode::Power { 
        base: inverted, 
        exponent: graph.add_node(ShaderNode::Scalar { value: falloff }) 
    });
    
    fresnel
}
```

---

## Step 4: Vertex Displacement

Vertex displacement modifies mesh geometry based on shader calculations.

```rust
fn create_displacement_material() -> ShaderGraphMaterial {
    let mut graph = ShaderGraphMaterial::new();
    
    // Get position and normal
    let position = graph.add_node(ShaderNode::Position);
    let time = graph.add_node(ShaderNode::Time);
    
    // Create animated noise
    let noise = graph.add_node(ShaderNode::Noise3D {
        position: graph.add_node(ShaderNode::Add { 
            a: position, 
            b: graph.add_node(ShaderNode::Multiply { 
                a: time, 
                b: graph.add_node(ShaderNode::Scalar { value: 0.5 }) 
            }) 
        }),
        octaves: 3,
    });
    
    // Scale displacement amount
    let offset = graph.add_node(ShaderNode::Multiply {
        a: noise,
        b: graph.add_node(ShaderNode::Scalar { value: 0.2 }),
    });
    
    // Set as vertex position output
    graph.connect_position_output(offset);
    
    // Set surface color
    let color = graph.add_node(ShaderNode::Color { color: Vec4::new(0.2, 0.4, 0.8, 1.0) });
    graph.connect_albedo_output(color);
    
    graph
}
```

---

## Next Steps

- [Importing and Organizing Content](editor_projectbrowser/--jingle-editor-projectbrowser.md)
- [Building Materials](building-materials/--jingle-building-materials.md)
- [Correcting Normals After Displacement](correcting-normals-after-vertex-displacement-with-shader-graph/--jingle-correcting-normals.md)

---

*Generated for Jingle RealityKit Implementation v0.1.0*
