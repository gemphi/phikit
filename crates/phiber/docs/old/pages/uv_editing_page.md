# Page Documentation: UV Editing Workspace

## Overview
The **UV Editing** workspace page provides a dual-pane interface for mapping 3D geometry surfaces to 2D textures.

## UI Layout Composition
```
┌──────────────────────────────┬──────────────────────────────┬────────────────┐
│ Top Menu Bar Header          │ Workspace Tabs Strip         │                │
├──────────────────────────────┼──────────────────────────────┼────────────────┤
│                              │                              │ Outliner Space │
│ Image Editor Space           │ 3D Viewport Space            │ (Optional)     │
│ (UV Editor Mode)             │ (Edit Mode)                  ├────────────────┤
│                              │                              │ Properties     │
└──────────────────────────────┴──────────────────────────────┴────────────────┘
```

## Key Interactions
- **Selection Sync**: Selecting faces in the 3D Viewport highlights corresponding UV islands in the UV Editor.
- **Operations**: Unwrapping tools are executed from the 3D Viewport, while UV manipulation (pin, stitch, scale) occurs in the UV Editor.
