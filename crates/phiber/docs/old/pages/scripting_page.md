# Page Documentation: Scripting Workspace

## Overview
The **Scripting** workspace page is a development and debugging layout for interacting with the application API.

## UI Layout Composition
```
┌──────────────────────────────┬──────────────────────────────┬────────────────┐
│ Top Menu Bar Header          │ Workspace Tabs Strip         │                │
├──────────────────────────────┼──────────────────────────────┼────────────────┤
│                              │                              │ Outliner Space │
│ 3D Viewport Space            │ Text Editor Space            │ (Optional)     │
│ (Live updates)               │ (Python / Rust Scripts)      ├────────────────┤
│                              │                              │ Console Space  │
├──────────────────────────────┼──────────────────────────────┼────────────────┤
│ Info Space (Operator logs)   │                              │                │
└──────────────────────────────┴──────────────────────────────┴────────────────┘
```

## Key Interactions
- **Text Editor**: Write, save, and execute scripts (`Run Script` button). Supports syntax highlighting.
- **Info Space**: Displays the command history (e.g., `bpy.ops.mesh.primitive_cube_add()`). Useful for copy-pasting API calls into scripts.
- **Console Space**: Interactive REPL for testing API snippets before committing them to a script.
