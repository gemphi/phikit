//! Timeline model.
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct NSTimelineState {
    pub current_time: Arc<Mutex<f64>>,
    pub duration: Arc<Mutex<f64>>,
    pub is_playing: Arc<Mutex<bool>>,
    pub fps: f32,
}

impl NSTimelineState {
    pub fn new() -> Self {
        Self {
            current_time: Arc::new(Mutex::new(0.0)),
            duration: Arc::new(Mutex::new(10.0)),
            is_playing: Arc::new(Mutex::new(false)),
            fps: 30.0,
        }
    }
}
