//! Builder extensions for styling NSView as button-like containers.
//!
//! These methods allow fluent styling of NSView instances used as
//! button containers in panels (activity bar, toolbar, etc.).

use super::properties::NSView;
use crate::theme::{self, keys};

impl NSView {
    /// Style as an activity bar icon button with automatic hover/selection.
    ///
    /// # Example
    /// ```ignore
    /// let btn = NSView::new(frame)
    ///     .activity_bar_style(is_active)
    ///     .with_name("hierarchy");
    /// ```
    pub fn activity_bar_style(mut self, is_active: bool) -> Self {
        if is_active {
            self.set_background_color(Some(theme::color(keys::CLR_BG_SELECTED)));
        } else {
            self.set_background_color(None);
        }
        self.set_corner_radius(crate::layout_constants::radius_button());
        self.set_hover_color(Some(theme::color(keys::CLR_BG_HOVER)));
        self
    }
    
    /// Style as a toolbar toggle button with automatic active/inactive colors.
    ///
    /// # Example
    /// ```ignore
    /// let btn = NSView::new(frame)
    ///     .toolbar_toggle_style(is_visible)
    ///     .with_name("sidebar");
    /// ```
    pub fn toolbar_toggle_style(mut self, is_active: bool) -> Self {
        self.set_background_color(Some(if is_active {
            theme::color(keys::CLR_ACCENT)
        } else {
            theme::color(keys::CLR_BG_CONTROL)
        }));
        self.set_corner_radius(5.0);
        self.set_hover_color(Some(theme::color(keys::CLR_BG_HOVER)));
        self
    }
    
    /// Style as a circular play control button.
    ///
    /// # Example
    /// ```ignore
    /// let btn = NSView::new(frame)
    ///     .play_control_style(is_playing)
    ///     .with_name("play");
    /// ```
    pub fn play_control_style(mut self, is_active: bool) -> Self {
        let radius = self.frame().width() / 2.0;
        self.set_background_color(Some(if is_active {
            theme::color(keys::CLR_ACCENT)
        } else {
            theme::color(keys::CLR_BG_CONTROL)
        }));
        self.set_corner_radius(radius);
        self.set_hover_color(Some(theme::color(keys::CLR_BG_HOVER)));
        self
    }
}
