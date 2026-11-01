//! # View Styles
//!
//! Style descriptors for layout engine view nodes.
//! All colors are expressed as theme token strings so they resolve
//! dynamically when the active theme changes.

/// Text alignment within its frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
}

impl Default for TextAlignment {
    fn default() -> Self {
        TextAlignment::Left
    }
}

/// Style descriptor for Text nodes.
#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    /// Font size in points.
    pub font_size: f32,
    /// Theme token string for the text color (e.g. "clr.text.label").
    pub color: String,
    /// Horizontal text alignment.
    pub alignment: TextAlignment,
}

impl TextStyle {
    pub fn new(font_size: f32, color: &str) -> Self {
        Self {
            font_size,
            color: color.to_string(),
            alignment: TextAlignment::Left,
        }
    }

    pub fn with_alignment(mut self, alignment: TextAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Convenience: title style (15pt, primary label).
    pub fn title() -> Self {
        Self::new(15.0, "clr.text.label")
    }

    /// Convenience: body style (13pt, primary label).
    pub fn body() -> Self {
        Self::new(13.0, "clr.text.label")
    }

    /// Convenience: caption style (11pt, secondary label).
    pub fn caption() -> Self {
        Self::new(11.0, "clr.text.secondary")
    }

    /// Convenience: section header style (12pt, tertiary label, left-aligned).
    pub fn section_header() -> Self {
        Self::new(12.0, "clr.text.tertiary")
    }
}

impl Default for TextStyle {
    fn default() -> Self {
        Self::body()
    }
}

/// Bezel style for buttons (mirrors NSBezelStyle).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BezelStyle {
    Rounded,
    RegularSquare,
    TexturedRounded,
    SmallSquare,
    Toolbar,
    AccessoryBarAction,
    Inline,
    Recessed,
    Push,
}

impl Default for BezelStyle {
    fn default() -> Self {
        BezelStyle::Rounded
    }
}

/// Style descriptor for Button nodes.
#[derive(Debug, Clone, PartialEq)]
pub struct ButtonStyle {
    /// Bezel shape.
    pub bezel_style: BezelStyle,
    /// Optional fill color (theme token). None = transparent/default.
    pub fill_color: Option<String>,
    /// Optional tint/text color (theme token). None = system default.
    pub tint_color: Option<String>,
    /// Corner radius in points.
    pub corner_radius: f32,
}

impl ButtonStyle {
    pub fn new(bezel_style: BezelStyle) -> Self {
        Self {
            bezel_style,
            fill_color: None,
            tint_color: None,
            corner_radius: 6.0,
        }
    }

    pub fn with_fill(mut self, token: &str) -> Self {
        self.fill_color = Some(token.to_string());
        self
    }

    pub fn with_tint(mut self, token: &str) -> Self {
        self.tint_color = Some(token.to_string());
        self
    }

    pub fn with_corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    /// Convenience: accent-filled button (primary action).
    pub fn accent_filled() -> Self {
        Self::new(BezelStyle::Rounded)
            .with_fill("clr.accent")
            .with_tint("clr.text.label")
            .with_corner_radius(6.0)
    }

    /// Convenience: secondary button (no fill, bordered).
    pub fn secondary() -> Self {
        Self::new(BezelStyle::Rounded)
            .with_tint("clr.text.secondary")
            .with_corner_radius(6.0)
    }

    /// Convenience: toolbar button (compact, no fill).
    pub fn toolbar() -> Self {
        Self::new(BezelStyle::Toolbar).with_corner_radius(4.0)
    }
}

impl Default for ButtonStyle {
    fn default() -> Self {
        Self::new(BezelStyle::Rounded)
    }
}

/// Style descriptor for Panel nodes (bordered containers).
#[derive(Debug, Clone, PartialEq)]
pub struct PanelStyle {
    /// Background color (theme token).
    pub background_color: String,
    /// Corner radius in points.
    pub corner_radius: f32,
    /// Border width in points (0 = no border).
    pub border_width: f32,
    /// Border color (theme token).
    pub border_color: String,
}

impl PanelStyle {
    pub fn new(background_color: &str) -> Self {
        Self {
            background_color: background_color.to_string(),
            corner_radius: 8.0,
            border_width: 1.0,
            border_color: "clr.border".to_string(),
        }
    }

    pub fn with_corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    pub fn with_border(mut self, width: f32, color: &str) -> Self {
        self.border_width = width;
        self.border_color = color.to_string();
        self
    }

    /// Convenience: standard bordered panel.
    pub fn bordered() -> Self {
        Self::new("clr.bg.panel")
    }

    /// Convenience: content area panel (no border).
    pub fn content() -> Self {
        Self {
            background_color: "clr.bg.content".to_string(),
            corner_radius: 0.0,
            border_width: 0.0,
            border_color: "clr.border".to_string(),
        }
    }

    /// Convenience: sidebar panel.
    pub fn sidebar() -> Self {
        Self::new("clr.bg.sidebar").with_corner_radius(0.0)
    }
}

impl Default for PanelStyle {
    fn default() -> Self {
        Self::bordered()
    }
}

/// Style descriptor for Image nodes.
#[derive(Debug, Clone, PartialEq)]
pub struct ImageStyle {
    /// Corner radius for rounded image display.
    pub corner_radius: f32,
    /// Whether the image should scale to fill its frame.
    pub scale_to_fill: bool,
    /// Optional tint color (theme token) for template images.
    pub tint_color: Option<String>,
}

impl ImageStyle {
    pub fn new() -> Self {
        Self {
            corner_radius: 0.0,
            scale_to_fill: false,
            tint_color: None,
        }
    }

    pub fn with_corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    pub fn with_scale_to_fill(mut self) -> Self {
        self.scale_to_fill = true;
        self
    }

    pub fn with_tint(mut self, token: &str) -> Self {
        self.tint_color = Some(token.to_string());
        self
    }
}

impl Default for ImageStyle {
    fn default() -> Self {
        Self::new()
    }
}
