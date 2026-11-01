//! Image types and symbol constants.

use crate::graphics::NSSize;
use std::sync::Arc;

/// An image with logical and physical size, stored as RGBA pixels.
pub struct NSImage {
    pub(super) name: String,
    /// Logical size (what the layout system uses for positioning).
    pub(super) size: NSSize,
    /// Physical pixel dimensions (size × scale_factor).
    pub(super) physical_size: NSSize,
    pub(super) pixels: Vec<u8>, // RGBA pixels at physical resolution
}

impl NSImage {
    /// Creates an `NSImage` from raw SVG data, rasterised at `target_size × scale_factor`.
    ///
    /// The image stores pixels at the physical resolution for crisp rendering on
    /// HiDPI displays. The `size()` method returns the logical size for layout,
    /// while `physical_size()` and `pixels()` give the actual pixel data.
    ///
    /// SVGs that use `currentColor` (the SF Symbols convention) are patched to
    /// render using the active theme's `CLR_TEXT_LABEL` token — so icons
    /// automatically match dark/light mode.
    pub fn new_with_svg(name: &str, svg_data: &[u8], target_size: NSSize) -> Option<Self> {
        super::loading::new_with_svg(name, svg_data, target_size)
    }

    pub fn new_with_png(name: &str, png_data: &[u8]) -> Option<Self> {
        super::loading::new_with_png(name, png_data)
    }

    /// Creates an `NSImage` from raw RGBA pixel data.
    pub fn from_rgba(pixels: &[u8], width: u32, height: u32) -> Self {
        let size = NSSize::new(width as f32, height as f32);
        Self {
            name: String::from("rgba_icon"),
            size,
            physical_size: size,
            pixels: pixels.to_vec(),
        }
    }

    /// Logical size (for layout positioning).
    pub fn size(&self) -> NSSize {
        self.size
    }

    /// Physical pixel dimensions (actual buffer size).
    pub fn physical_size(&self) -> NSSize {
        self.physical_size
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Look up a named icon from the asset search paths.
    pub fn named(name: &str) -> Option<Arc<Self>> {
        println!("[NSImage] Looking for icon: '{}'", name);
        let paths = [
            format!("assets/icons/{}.svg", name),
            format!("assets/icons/{}.png", name),
            format!("phi/assets/icons/{}.svg", name),
            format!("phi/assets/icons/{}.png", name),
            format!("../../assets/icons/{}.svg", name),
            format!("../../assets/icons/{}.png", name),
        ];

        for p in &paths {
            let path = std::path::Path::new(p);
            if path.exists() {
                println!("[NSImage] Found icon '{}' at path: {:?}", name, path);
                let data = std::fs::read(path).ok()?;
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if ext == "svg" {
                    return Self::new_with_svg(name, &data, NSSize::new(24.0, 24.0))
                        .map(Arc::new);
                } else if ext == "png" {
                    return Self::new_with_png(name, &data).map(Arc::new);
                }
            }
        }
        println!(
            "[NSImage] WARNING: Icon '{}' not found in any of the search paths!",
            name
        );
        None
    }
}

/// System symbol icons (RCP style) — maps to SVGs in phi/assets/icons/
pub struct NSImageSymbol;

impl NSImageSymbol {
    // Geometry
    pub const CUBE: &str = "cube";
    pub const SPHERE: &str = "sphere";

    // Scene elements
    pub const LIGHT: &str = "light";
    pub const CAMERA: &str = "camera";

    // Actions
    pub const PLUS: &str = "plus";
    pub const TRASH: &str = "trash";
    pub const SEARCH: &str = "search";

    // Viewport tools
    pub const CURSOR: &str = "cursor";
    pub const MOVE: &str = "move";
    pub const ROTATE: &str = "rotate";

    // Navigation
    pub const CHEVRON_RIGHT: &str = "chevron_right";
    pub const CHEVRON_DOWN: &str = "chevron_down";

    // Visibility
    pub const EYE: &str = "eye";
    pub const EYE_OFF: &str = "eye_off";

    // Project
    pub const FOLDER: &str = "folder";
    pub const PROJECT: &str = "project";
    pub const HIERARCHY: &str = "hierarchy";
    pub const INSPECTOR: &str = "inspector";

    // Bottom tabs
    pub const SHADER_GRAPH: &str = "shader_graph";
    pub const TIMELINE: &str = "timeline";
    pub const AUDIO: &str = "audio";
    pub const STATISTICS: &str = "statistics";

    // Transport
    pub const PLAY: &str = "play";
    pub const PAUSE: &str = "pause";
    pub const STOP: &str = "stop";

    // Layout toggle (panel visibility)
    pub const LAYOUT_LEFT: &str = "layout_left";
    pub const LAYOUT_RIGHT: &str = "layout_right";
    pub const LAYOUT_BOTTOM: &str = "layout_bottom";
    pub const LAYOUT_ALL: &str = "layout_all";

    // Misc
    pub const FILE: &str = "file";
    pub const PARTICLES: &str = "particles";
}
