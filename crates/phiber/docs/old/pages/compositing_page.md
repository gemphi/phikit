# Page Documentation: Compositing Workspace

## Overview
The **Compositing** workspace page allows for post-processing and 2D node-based image manipulation of render outputs.

## UI Layout Composition
```
┌──────────────────────────────┬──────────────────────────────┬────────────────┐
│ Top Menu Bar Header          │ Workspace Tabs Strip         │                │
├──────────────────────────────┼──────────────────────────────┼────────────────┤
│                              │                              │                │
│ Image Editor Space           │ Node Editor Space            │ Properties     │
│ (Viewer Node Output)         │ (Compositor Nodes)           │ (Optional)     │
│                              │                              │                │
└──────────────────────────────┴──────────────────────────────┴────────────────┘
```

## Key Interactions
- **Node Editor**: Builds the compositing tree. Activating "Use Nodes" replaces the direct render output with the composite output.
- **Image Editor**: Acts as a live preview for the active `Viewer` node in the graph, providing immediate feedback on blur, glare, and color balance nodes.
