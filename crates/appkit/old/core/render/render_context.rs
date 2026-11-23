//! Core render context struct and state management for software 2D rendering.

use crate::{NSColor, NSPoint, NSRect};

/// Rounded clip region: a rectangular clip with corner radius for content clipping.
#[derive(Clone, Copy, Debug)]
pub struct RoundedClip {
    pub rect: NSRect,
    pub radius: f32,
}

impl RoundedClip {
    /// Check if a pixel at (x, y) in absolute coordinates is inside the rounded clip region.
    pub fn contains(&self, x: f32, y: f32) -> bool {
        // First check rectangular bounds
        if x < self.rect.min_x()
            || x > self.rect.max_x()
            || y < self.rect.min_y()
            || y > self.rect.max_y()
        {
            return false;
        }
        if self.radius <= 0.0 {
            return true;
        }
        // Check corner regions
        let r = self.radius;
        let cx = match x < self.rect.min_x() + r {
            true => self.rect.min_x() + r,
            false => match x > self.rect.max_x() - r {
                true => self.rect.max_x() - r,
                false => x,
            },
        };
        let cy = match y < self.rect.min_y() + r {
            true => self.rect.min_y() + r,
            false => match y > self.rect.max_y() - r {
                true => self.rect.max_y() - r,
                false => y,
            },
        };
        let d_sq = (x - cx).powi(2) + (y - cy).powi(2);
        d_sq <= r * r
    }
}

/// Software render context for UI drawing
pub struct RenderContext<'a> {
    pub width: usize,
    pub height: usize,
    pub(crate) buffer: &'a mut [u32],
    pub current_color: NSColor,
    pub font: Option<fontdue::Font>,
    pub offset: NSPoint,
    pub(crate) offset_stack: Vec<NSPoint>,
    pub clip_rect: Option<NSRect>,
    pub(crate) clip_stack: Vec<Option<NSRect>>,
    /// Stack of rounded clip regions for panel content clipping.
    pub(crate) rounded_clip: Option<RoundedClip>,
    pub(crate) rounded_clip_stack: Vec<Option<RoundedClip>>,
    /// Display scale factor (1.0 = standard, 2.0 = Retina/HiDPI).
    pub scale_factor: f32,
    /// Above zero while drawing inside a glass container, which has already
    /// rendered the glass of everything in it.
    pub(crate) glass_batched: u32,
}

impl<'a> RenderContext<'a> {
    pub fn new(width: usize, height: usize, buffer: &'a mut [u32]) -> Self {
        Self {
            width,
            height,
            buffer,
            current_color: NSColor::white(),
            font: Some(crate::render::text::system_font().clone()),
            offset: NSPoint::new(0.0, 0.0),
            offset_stack: Vec::new(),
            clip_rect: None,
            clip_stack: Vec::new(),
            rounded_clip: None,
            rounded_clip_stack: Vec::new(),
            scale_factor: 1.0,
            glass_batched: 0,
        }
    }

    /// Set the display scale factor for HiDPI rendering.
    pub fn set_scale_factor(&mut self, scale: f32) {
        self.scale_factor = scale;
    }

    /// Returns a mutable reference to the underlying pixel buffer.
    pub fn buffer_mut(&mut self) -> &mut [u32] {
        self.buffer
    }

    pub fn push_offset(&mut self, offset: NSPoint) {
        self.offset_stack.push(self.offset);
        self.offset.x += offset.x;
        self.offset.y += offset.y;
    }

    pub fn pop_offset(&mut self, _offset: NSPoint) {
        if let Some(old_offset) = self.offset_stack.pop() {
            self.offset = old_offset;
        }
    }

    pub fn push_clip(&mut self, clip: NSRect) {
        self.clip_stack.push(self.clip_rect);
        let mut new_clip = clip;
        new_clip.origin.x += self.offset.x;
        new_clip.origin.y += self.offset.y;
        if let Some(existing) = self.clip_rect {
            self.clip_rect = Some(existing.intersection(&new_clip));
        } else {
            self.clip_rect = Some(new_clip);
        }
    }

    pub fn pop_clip(&mut self) {
        if let Some(old_clip) = self.clip_stack.pop() {
            self.clip_rect = old_clip;
        }
    }

    /// Push a rounded clip region that clips all subsequent drawing to a rounded rectangle.
    pub fn push_rounded_clip(&mut self, clip: NSRect, radius: f32) {
        self.rounded_clip_stack.push(self.rounded_clip);
        let mut abs_rect = clip;
        abs_rect.origin.x += self.offset.x;
        abs_rect.origin.y += self.offset.y;
        self.rounded_clip = Some(RoundedClip { rect: abs_rect, radius });
    }

    /// Pop the most recent rounded clip region, restoring the previous one.
    pub fn pop_rounded_clip(&mut self) {
        if let Some(old) = self.rounded_clip_stack.pop() {
            self.rounded_clip = old;
        }
    }

    /// Check if a pixel at absolute coordinates (x, y) passes the current rounded clip.
    /// Returns true if no rounded clip is active or if the pixel is inside the rounded region.
    #[inline]
    pub fn pixel_in_rounded_clip(&self, x: f32, y: f32) -> bool {
        match self.rounded_clip {
            Some(ref rc) => rc.contains(x, y),
            None => true,
        }
    }

    pub fn set_color(&mut self, color: NSColor) {
        self.current_color = color;
    }
}
