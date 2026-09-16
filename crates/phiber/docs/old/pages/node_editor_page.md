# Page Documentation: Node Editor Space

## Overview
The **Node Editor** page space is a visual scripting interface for Shaders, Geometry Nodes, and Compositing.

## UI Elements
- **Node Canvas**: Infinite 2D grid allowing panning and zooming.
- **Nodes**: Visual blocks with headers (title, color-coded by category) and body (properties like sliders, color pickers).
- **Sockets**: Circular connection points on the left (Inputs) and right (Outputs) of nodes, color-coded by data type (e.g., yellow for color, purple for vector, green for shader, gray for float).
- **Links**: Bezier curves connecting an output socket to an input socket.

## Context Types
The Node Editor adapts its available nodes based on its active context:
1. **Shader Nodes**: For material authoring (e.g., Principled BSDF).
2. **Geometry Nodes**: For procedural modeling (e.g., Instance on Points).
3. **Compositing Nodes**: For post-processing renders (e.g., Glare, Color Balance).
