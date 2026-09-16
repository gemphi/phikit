# Page Documentation: Shading Workspace

## Overview
The **Shading** page is BUIApp's specialized material authoring workspace (`workspace_idx = 4`). It tiles 6 space editors across 3 columns to facilitate asset browsing, material node graph editing, and real-time shader previewing.

## UI Layout Composition

```
┌──────────────────────────────┬──────────────────────────────┬──────────────────────────────┐
│ File Browser Space           │ 3D Viewport Space            │ Outliner Space               │
│ (Asset / Texture ingest)     │ (Material Preview mode)      │ (Scene tree)                 │
├──────────────────────────────┼──────────────────────────────┼──────────────────────────────┤
│ Image / Texture Editor       │ Shader Node Editor Space     │ Properties Space             │
│ (2D Texture inspection)      │ (Node tree graph connections)│ (Material & Shader tab)      │
└──────────────────────────────┴──────────────────────────────┴──────────────────────────────┘
```

## Data Bindings
- **Node Graph**: Edits `blenda::NodeTree` inside the selected material data block.
- **Material Preview**: Real-time rendering via `appkit` WGPU pipelines reflecting Principled BSDF metallic, roughness, base color, and specular parameters.
