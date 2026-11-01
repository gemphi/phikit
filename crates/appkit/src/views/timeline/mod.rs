//! # NSTimelineView — Keyframe Animation Timeline
//!
//! A timeline editor view for keyframe animations and time-based editing.
//!
//! **Custom component (no direct Apple equivalent)** — This component provides
//! timeline editing functionality similar to Final Cut Pro, Motion, or other
//! professional video/animation editing applications.
//!
//! ## Apple Documentation
//! - [NSView](https://developer.apple.com/documentation/appkit/nsview)
//! - [NSScrollView](https://developer.apple.com/documentation/appkit/nsscrollview)
//!
//! ## Usage
//!
//! ```rust,ignore
//! let mut timeline = NSTimelineView::new(frame);
//! timeline.set_duration(10.0);
//! timeline.add_track(track);
//! timeline.set_playhead_position(5.0);
//! timeline.play();
//! ```
//!
//! ## See Also
//! - Custom component for animation editing
//! - Similar to Final Cut Pro / Motion timeline

pub mod builder;
pub mod delegate;
pub mod track;

use self::track::TimelineTrackBase;
use crate::{NSRect, NSView};
use std::sync::{Arc, Mutex};

use std::sync::OnceLock;

pub struct TimelineState {
    pub playhead_pos: f32,
    pub is_scrubbing: bool,
    pub is_playing: bool,
    pub current_time: f32,
    pub duration: f32,
    pub tracks: Vec<Box<dyn TimelineTrackBase + Send + Sync>>,
}

static ACTIVE_TIMELINES: OnceLock<Mutex<Vec<Arc<Mutex<TimelineState>>>>> = OnceLock::new();

pub fn get_active_timelines() -> &'static Mutex<Vec<Arc<Mutex<TimelineState>>>> {
    ACTIVE_TIMELINES.get_or_init(|| Mutex::new(Vec::new()))
}

/// Timeline view control
#[derive(Clone)]
pub struct NSTimelineView {
    pub(crate) view: NSView,
    pub state: Arc<Mutex<TimelineState>>,
}

impl NSTimelineView {
    pub fn new(frame: NSRect) -> Self {
        let state = Arc::new(Mutex::new(TimelineState {
            playhead_pos: 80.0,
            is_scrubbing: false,
            is_playing: false,
            current_time: 0.0,
            duration: 10.0,
            tracks: vec![],
        }));

        // Register this active timeline state so other crates can tick/drive it
        get_active_timelines().lock().unwrap().push(state.clone());

        let mut view = NSView::new(frame).with_name("NSTimelineView");
        view.set_delegate(delegate::TimelineDelegate {
            state: state.clone(),
        });
        Self { view, state }
    }

    pub fn view(&self) -> &NSView {
        &self.view
    }
    pub fn view_mut(&mut self) -> &mut NSView {
        &mut self.view
    }

    pub fn add_track(&mut self, track: impl TimelineTrackBase + Send + Sync + 'static) {
        if let Ok(mut state) = self.state.lock() {
            state.tracks.push(Box::new(track));
        }
    }

    pub fn play(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.is_playing = true;
        }
    }

    pub fn pause(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.is_playing = false;
        }
    }

    pub fn stop(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.is_playing = false;
            state.current_time = 0.0;
        }
    }
}
