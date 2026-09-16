# Page Documentation: Animation Workspace

## Overview
The **Animation** workspace page provides the tools necessary for keyframe authoring, rigging, and playback timing.

## UI Layout Composition
```
┌──────────────────────────────────────────────┬───────────────────────────────┐
│ Top Menu Bar Header                          │ Workspace Tabs Strip          │
├──────────────────────────────────────────────┼───────────────────────────────┤
│                                              │ Outliner Space                │
│ 3D Viewport Space                            │ (Pose/Bone focus)             │
│ (Pose Mode)                                  ├───────────────────────────────┤
│                                              │ Properties Space              │
├──────────────────────────────────────────────┤ (Bone, Constraints tabs)      │
│ Dope Sheet / Graph Editor Space              │                               │
├──────────────────────────────────────────────┴───────────────────────────────┤
│ Timeline Space                                                               │
└──────────────────────────────────────────────────────────────────────────────┘
```

## Key Interactions
- **Dope Sheet**: Visualizes keyframes across objects hierarchically for timing adjustments.
- **Graph Editor**: F-Curve editor for precise interpolation and easing (Bezier handles).
- **Auto-Keying**: Transport header allows enabling auto-keyframe insertion during 3D Viewport transformation.
