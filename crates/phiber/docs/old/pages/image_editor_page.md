# Page Documentation: Image Editor Space

## Overview
The **Image Editor** page space is used for viewing and modifying 2D image buffers, such as textures or rendered frames.

## UI Elements
- **View Canvas**: Displays the image with pan/zoom capabilities. Overlays like grid or UV bounds can be enabled.
- **Header Options**:
  - Image selection dropdown (choose from loaded textures, Viewer Node, or Render Result).
  - Slot selector (for comparing multiple renders).
  - Channel toggles (View RGB, Alpha, Z-Depth).
- **Sidebar (N-Panel)**: Contains image metadata, color space settings, dimensions, and annotations.

## Interaction Modes
- **View**: Standard inspection.
- **Paint**: 2D texture painting.
- **Mask**: 2D spline masking.
- **UV Editor**: When combined with 3D Viewport edit mode, allows manipulating UV layout over the image.
