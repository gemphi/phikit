//! # NSTimelineViewController — Animation Sequencer Panel
//!
//! Implements the assembly and playback logic for the animation sequencer.
//! Mirrors Apple's view controller pattern for modular temporal editors.
//!
//! ## Apple Documentation
//! - [NSViewController](https://developer.apple.com/documentation/appkit/nsviewcontroller)
//! - [NSView](https://developer.apple.com/documentation/appkit/nsview)
//! - [NSScrollView](https://developer.apple.com/documentation/appkit/nsscrollview)
//! - [NSTabView](https://developer.apple.com/documentation/appkit/nstabview)
//!
//! ## Architecture
//!
//! This panel uses NSTabView to organize timeline content into tabs.
//! The panel provides a toolbar with transport controls (play, pause, stop)
//! and a tabbed content area for different timeline views.
//!
//! ## See Also
//! - [NSTimelineView](crate::timeline::NSTimelineView)
//! - [NSTabView](crate::views::tab_view::NSTabView)
//! - Custom component for keyframe animation editing

use super::model::*;
use crate::animation::{Interpolatable, SCNAnimation};
use crate::graphics::NSImage;
use crate::prelude::*;
use crate::tab_view::{NSTabView, NSTabViewItem};
use crate::theme;
use crate::layout::{spacing, hierarchy};
use crate::timeline::track::TimelineTrack;
use std::sync::Arc;

pub struct NSTimelineViewController {
    pub view: NSView,
    pub tab_view: NSTabView,
    pub timeline_view: NSTimelineView,
    pub state: NSTimelineState,
}

impl NSTimelineViewController {
    pub fn new(
        rect: NSRect,
        play_icon: Option<Arc<NSImage>>,
        pause_icon: Option<Arc<NSImage>>,
        stop_icon: Option<Arc<NSImage>>,
    ) -> Self {
        let toolbar_h = NSLayoutConstants::TOOLBAR_HEIGHT;
        let mut view = NSView::new(rect).with_name("TimelinePanel");
        view.set_background_color(Some(hierarchy::bg_color("panel")));
        view.set_corner_radius(crate::theme::ThemeToken::RADIUS_PANEL());
        view.set_glass(true);
        view.set_clips_to_bounds(true);

        let mut toolbar = NSView::new(NSRect::sized(rect.width(), toolbar_h))
            .with_name("TimelineToolbar");
        toolbar.set_background_color(Some(hierarchy::bg_color("toolbar")));
        toolbar.set_autoresizing_mask(
            NSAutoresizingMask::WIDTH_SIZABLE | NSAutoresizingMask::MAX_Y_MARGIN,
        );

        // Create tab view for organizing timeline content
        let tab_rect = NSRect::new(
            0.0,
            toolbar_h,
            rect.width(),
            (rect.height() - toolbar_h).max(0.0),
        );
        let mut tab_view = NSTabView::new(tab_rect);
        tab_view.view_mut().set_autoresizing_mask(
            NSAutoresizingMask::WIDTH_SIZABLE | NSAutoresizingMask::HEIGHT_SIZABLE,
        );

        // Create timeline view as the default content
        let timeline_rect = NSRect::new(0.0, 0.0, tab_rect.width(), tab_rect.height());
        let mut timeline_view = NSTimelineView::new(timeline_rect);
        timeline_view.view_mut().set_name("TimelineTracks");
        timeline_view.view_mut().set_autoresizing_mask(
            NSAutoresizingMask::WIDTH_SIZABLE | NSAutoresizingMask::HEIGHT_SIZABLE,
        );

        // Note: Tabs should be added by the application layer using add_tab()
        // This keeps the framework generic and reusable

        add_transport_controls(
            &mut toolbar,
            &timeline_view,
            play_icon,
            pause_icon,
            stop_icon,
        );
        add_time_label(&mut toolbar, rect.width());

        view.add_subview(toolbar);
        view.add_subview(tab_view.view().clone());

        Self {
            view,
            tab_view,
            timeline_view,
            state: NSTimelineState::new(),
        }
    }

    /// Add a tab to the timeline panel.
    /// Applications should use this to configure tabs with their specific content.
    ///
    /// # Example
    /// ```rust,ignore
    /// timeline.add_tab("Animation", animation_view);
    /// timeline.add_tab("Audio", audio_view);
    /// timeline.set_selected_tab(0);
    /// ```
    pub fn add_tab(&mut self, label: &str, content: NSView) {
        let tab = NSTabViewItem::new(label, content);
        self.tab_view.add_tab_view_item(tab);
    }

    /// Set the selected tab by index.
    pub fn set_selected_tab(&mut self, index: usize) {
        self.tab_view.set_selected_index(index);
    }

    pub fn play(&mut self) {
        self.timeline_view.play();
        *self.state.is_playing.lock().unwrap() = true;
    }

    pub fn pause(&mut self) {
        self.timeline_view.pause();
        *self.state.is_playing.lock().unwrap() = false;
    }

    pub fn stop(&mut self) {
        self.timeline_view.stop();
        *self.state.is_playing.lock().unwrap() = false;
        *self.state.current_time.lock().unwrap() = 0.0;
    }

    pub fn add_track<T: Interpolatable + Clone + Send + Sync + 'static>(
        &mut self,
        name: &str,
        animation: SCNAnimation<T>,
    ) {
        let track = TimelineTrack {
            name: name.to_string(),
            animation,
            player: None,
            keyframes: vec![],
            color: {
                let c = theme::color(theme::CLR_ACCENT);
                [c.r, c.g, c.b, c.a]
            },
        };
        self.timeline_view.add_track(track);
    }
}

fn add_transport_controls(
    toolbar: &mut NSView,
    timeline: &NSTimelineView,
    play: Option<Arc<NSImage>>,
    pause: Option<Arc<NSImage>>,
    stop: Option<Arc<NSImage>>,
) {
    let btn_sz = NSLayoutConstants::ICON_BUTTON_SIZE + NSLayoutConstants::ICON_BUTTON_INSET;
    let mut x = spacing::margin::TOOLBAR;

    let icons = [
        (play, "PlayButton"),
        (pause, "PauseButton"),
        (stop, "StopButton"),
    ];
    for (icon, name) in icons {
        let mut btn = NSButton::new(NSRect::new(x, spacing::padding::TOOLBAR_V, btn_sz, btn_sz));
        btn.set_image(icon);
        btn.view_mut().set_name(name);
        let t = timeline.clone();
        match name {
            "PlayButton" => btn.set_action(move || t.play()),
            "PauseButton" => btn.set_action(move || t.pause()),
            "StopButton" => btn.set_action(move || t.stop()),
            _ => {}
        }
        toolbar.add_subview(btn.view_clone());
        x += btn_sz + spacing::gap::TOOLBAR;
    }
}

fn add_time_label(toolbar: &mut NSView, win_w: f32) {
    let label_w = NSLayoutConstants::LABEL_HEIGHT * 3.0;
    let label_x = win_w - label_w - spacing::margin::TOOLBAR;
    let label_y = spacing::padding::TOOLBAR_V;
    let label_frame = NSRect::new(label_x, label_y, label_w, NSLayoutConstants::LABEL_HEIGHT);

    let mut label = NSTextField::new(label_frame);
    label.set_string_value("0.00s");
    label.set_text_alignment(NSTextAlignment::Right);
    label.set_text_color(theme::color(theme::CLR_TEXT_LABEL));
    label
        .view_mut()
        .set_autoresizing_mask(NSAutoresizingMask::MIN_X_MARGIN | NSAutoresizingMask::MAX_Y_MARGIN);
    toolbar.add_subview(label.view().clone());
}
