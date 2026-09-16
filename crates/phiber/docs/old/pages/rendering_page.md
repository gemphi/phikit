# Page Documentation: Rendering Workspace

## Overview
The **Rendering** workspace page is dedicated strictly to evaluating final pixel output and managing render properties without the overhead of the 3D Viewport.

## UI Layout Composition
```
┌──────────────────────────────────────────────┬───────────────────────────────┐
│ Top Menu Bar Header                          │ Workspace Tabs Strip          │
├──────────────────────────────────────────────┼───────────────────────────────┤
│                                              │ Outliner Space                │
│ Image Editor Space                           │ (Optional / Hidden)           │
│ (Locked to "Render Result")                  ├───────────────────────────────┤
│                                              │ Properties Space              │
│                                              │ (Render, Output, View Layer)  │
└──────────────────────────────────────────────┴───────────────────────────────┘
```

## Key Interactions
- **Image Editor**: Inspects render passes (Albedo, Z-Depth, Normals, Emission) from the render buffer. Allows saving the result to disk.
- **Properties Space**: Quickly iterate on sampling, resolution, file format, and engine settings (Cycles/Eevee).
