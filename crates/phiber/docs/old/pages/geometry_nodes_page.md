# Page Documentation: Geometry Nodes Workspace

## Overview
The **Geometry Nodes** workspace page is a procedural modeling environment driven by node networks.

## UI Layout Composition
```
┌──────────────────────────────┬──────────────────────────────┬────────────────┐
│ Top Menu Bar Header          │ Workspace Tabs Strip         │                │
├──────────────────────────────┼──────────────────────────────┼────────────────┤
│ Spreadsheet Editor           │ 3D Viewport Space            │ Outliner Space │
│ (Attribute Inspection)       │ (Procedural Output Preview)  │                │
├──────────────────────────────┼──────────────────────────────┼────────────────┤
│                              │ Node Editor Space            │ Properties     │
│                              │ (Geometry Nodes)             │ (Modifiers)    │
└──────────────────────────────┴──────────────────────────────┴────────────────┘
```

## Key Interactions
- **Spreadsheet**: Invaluable for debugging mathematical operations on geometry by displaying raw vertex/face attribute values.
- **Node Editor**: Connects mesh primitives, math nodes, and instances to procedurally generate geometry evaluated by the Geometry Nodes Modifier on the active object.
