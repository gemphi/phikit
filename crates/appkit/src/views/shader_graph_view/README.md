# ShaderGraphView — Visual Shader Editor

## Overview

`ShaderGraphView` provides a node-based visual editor for creating shaders. Users connect nodes representing operations, textures, and outputs. Custom component for rust-kit-pro's material system.

## Architecture Flow

```
┌─────────────────────────────────────────────────────────────┐
│                 Shader Graph Structure                       │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              Node Types                                        │
│  ├─ Input nodes: UV, Time, Vertex Color                     │
│  ├─ Texture nodes: Sample 2D, Sample Cube                   │
│  ├─ Math nodes: Add, Multiply, Sine, Power                │
│  ├─ Color nodes: RGB, HSV, Gradient                       │
│  └─ Output nodes: Albedo, Normal, Metallic, Roughness       │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                 Connection System                              │
│  ├─ Each node has input and output sockets                  │
│  ├─ Connections link output → input                         │
│  ├─ Data flows: left (inputs) → right (outputs)             │
│  └─ Graph compiled to WGSL shader code                      │
└─────────────────────────────────────────────────────────────┘
```

## Node Categories

| Category | Nodes | Purpose |
|----------|-------|---------|
| **Input** | UV, Time, Vertex Color, Normal | Base data |
| **Texture** | Sample 2D, Sample Cube, Normal Map | Texture sampling |
| **Math** | Add, Multiply, Divide, Power, Lerp | Calculations |
| **Trigonometry** | Sin, Cos, Tan, Noise | Procedural patterns |
| **Color** | RGB, HSV, Gradient, Desaturate | Color operations |
| **Output** | Albedo, Emission, Normal, Roughness | Final material |

## Connection to Other Modules

```
ShaderGraphView
├── Uses: NSView (canvas for nodes)
├── Uses: NSStackView (node internals)
├── Uses: NSBezierPath (connections)
├── Generates: WGSL code (shaders/)
└── Used by: rust-kit-pro material editor
```

## Usage Examples

### Create Simple Shader
```
[UV] → [Sample Texture] → [Albedo Output]
         ↑
   [Texture Asset]

Result: Textured material
```

### Procedural Pattern
```
[UV] → [Multiply by 10] → [Sin] → [Multiply] → [Albedo]
        ↑                              ↑
   [Time] → [Multiply]          [Color Red]
   
Result: Animated stripes
```

### Normal Mapping
```
[UV] → [Sample Normal Map] → [Normal Blend] → [Normal Output]
                                    ↑
                          [Vertex Normal]

Result: Bump mapped surface
```

### Node Creation
```rust
use rust_appkit::prelude::*;
use rust_appkit::shader_graph_view::*;

let mut graph_view = ShaderGraphView::new(NSRect::new(0.0, 0.0, 800.0, 600.0));

// Add nodes
let uv_node = graph_view.add_node(NodeType::UV, NSPoint::new(100.0, 300.0));
let texture_node = graph_view.add_node(NodeType::SampleTexture, NSPoint::new(300.0, 300.0));
let output_node = graph_view.add_node(NodeType::AlbedoOutput, NSPoint::new(500.0, 300.0));

// Connect
graph_view.connect(uv_node.output("uv"), texture_node.input("uv"));
graph_view.connect(texture_node.output("color"), output_node.input("albedo"));

// Compile
let shader_code = graph_view.compile_to_wgsl();
```

## Graph View Layout

```
┌─────────────────────────────────────────────────────────────────┐
│  Shader Graph - Material_01.mat                                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────┐         ┌─────────────┐         ┌──────────────┐     │
│  │   UV    │──uv────→│  Sample     │──color─→│   Albedo     │     │
│  │         │         │  Texture    │         │   Output     │     │
│  └─────────┘         │             │         └──────────────┘     │
│                      │  [Preview]  │                             │
│                      └─────────────┘                             │
│                           ↑                                      │
│                      ┌─────────┐                                 │
│                      │ Texture │                                 │
│                      │ Asset   │                                 │
│                      └─────────┘                                 │
│                                                                  │
│  [+ Add Node]  [Compile]  [Preview]                             │
└─────────────────────────────────────────────────────────────────┘
```

## Node Graph Operations

| Operation | Action |
|-----------|--------|
| **Double-click canvas** | Add node menu |
| **Drag from output** | Create connection |
| **Right-click node** | Delete, duplicate |
| **Scroll wheel** | Zoom view |
| **Middle drag** | Pan view |
| **Compile button** | Generate WGSL |

## Testing & Verification

```rust
let mut graph = ShaderGraphView::new(frame);

// Create UV → Albedo passthrough
let uv = graph.add_node(NodeType::UV, NSPoint::new(100.0, 100.0));
let out = graph.add_node(NodeType::AlbedoOutput, NSPoint::new(300.0, 100.0));

graph.connect(uv.output("uv"), out.input("albedo"));

let shader = graph.compile_to_wgsl();
assert!(shader.contains("@fragment"));
assert!(shader.contains("fs_main"));
```

## File Structure

```
src/shader_graph_view/
├── mod.rs          # ShaderGraphView
├── node.rs         # ShaderNode, NodeType, Socket
├── connection.rs   # NodeConnection
├── compiler.rs     # WgslCompiler
├── preview.rs      # ShaderPreview (3D preview sphere)
└── palette.rs      # Node palette (add node menu)
```

## Missing from ShaderGraphView

- **Live preview**: Real-time material preview
- **Sub-graphs**: Reusable node groups
- **Custom nodes**: User-defined HLSL/WGSL
- **Comments**: Sticky notes on graph
- **Undo/redo**: Graph history
- **Templates**: Preset shader graphs

These are stubbed for current use cases.

## Related

- **GPU module**: Executes compiled shaders
- **Shaders folder**: Generated WGSL output
- **Material system**: Uses graph outputs
