//! Animation player implementation (< 200 lines)
use crate::animation::types::*;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Core animation struct
pub struct SCNAnimation<T: Interpolatable + Clone> {
    pub duration: Duration,
    pub timing_function: SCNTimingFunction,
    pub keyframes: Vec<SCNKeyframe<T>>,
    pub events: Vec<(f32, SCNAnimationEvent)>,
}

impl<T: Interpolatable + Clone> std::fmt::Debug for SCNAnimation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SCNAnimation")
            .field("duration", &self.duration)
            .field("timing_function", &self.timing_function)
            .field("keyframes_count", &self.keyframes.len())
            .field("events_count", &self.events.len())
            .finish()
    }
}

impl<T: Interpolatable + Clone> Clone for SCNAnimation<T> {
    fn clone(&self) -> Self {
        Self {
            duration: self.duration,
            timing_function: self.timing_function,
            keyframes: self.keyframes.clone(),
            events: Vec::new(), // Events cannot be cloned, so we create a new empty vec
        }
    }
}

impl<T: Interpolatable + Clone> SCNAnimation<T> {
    pub fn new(duration_secs: f32) -> Self {
        Self {
            duration: Duration::from_secs_f32(duration_secs),
            timing_function: SCNTimingFunction::Linear,
            keyframes: vec![
                SCNKeyframe::new(0.0, T::default_value()),
                SCNKeyframe::new(1.0, T::default_value()),
            ],
            events: Vec::new(),
        }
    }

    pub fn with_keyframe(mut self, time: f32, value: T) -> Self {
        self.keyframes.push(SCNKeyframe::new(time, value));
        self.keyframes
            .sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
        self
    }

    /// Evaluate the animation at a specific time (in seconds)
    pub fn evaluate(&self, time_secs: f32) -> T {
        let t = (time_secs / self.duration.as_secs_f32()).clamp(0.0, 1.0);
        self.sample(t)
    }

    fn sample(&self, t: f32) -> T {
        let t = self.timing_function.apply(t.clamp(0.0, 1.0));
        for i in 0..self.keyframes.len().saturating_sub(1) {
            let k0 = &self.keyframes[i];
            let k1 = &self.keyframes[i + 1];
            if t >= k0.time && t <= k1.time {
                let local_t = (t - k0.time) / (k1.time - k0.time);
                return T::interpolate(&k0.value, &k1.value, local_t);
            }
        }
        self.keyframes
            .last()
            .map(|k| k.value.clone())
            .unwrap_or_else(T::default_value)
    }
}

/// Animation player
pub struct SCNAnimationPlayer<T: Interpolatable + Clone> {
    animation: Arc<Mutex<SCNAnimation<T>>>,
    state: SCNAnimationState,
    current_time: f32,
    start_time: Option<Instant>,
}

impl<T: Interpolatable + Clone> std::fmt::Debug for SCNAnimationPlayer<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SCNAnimationPlayer")
            .field("state", &self.state)
            .field("current_time", &self.current_time)
            .field("start_time", &self.start_time)
            .finish()
    }
}

impl<T: Interpolatable + Clone + 'static> SCNAnimationPlayer<T> {
    pub fn new(animation: SCNAnimation<T>) -> Self {
        Self {
            animation: Arc::new(Mutex::new(animation)),
            state: SCNAnimationState::Stopped,
            current_time: 0.0,
            start_time: None,
        }
    }

    pub fn play(&mut self) {
        self.state = SCNAnimationState::Running;
        self.start_time = Some(Instant::now());
    }

    pub fn pause(&mut self) {
        self.state = SCNAnimationState::Paused;
    }

    pub fn stop(&mut self) {
        self.state = SCNAnimationState::Stopped;
        self.current_time = 0.0;
    }

    pub fn set_current_time(&mut self, time_secs: f32) {
        self.current_time = time_secs;
    }

    pub fn update(&mut self, dt: f32) -> Option<T> {
        if self.state != SCNAnimationState::Running {
            return None;
        }

        self.current_time += dt;
        let anim = self.animation.lock().unwrap();
        let duration = anim.duration.as_secs_f32();
        let t = (self.current_time / duration).clamp(0.0, 1.0);
        let value = anim.sample(t);

        if self.current_time >= duration {
            self.state = SCNAnimationState::Stopped;
        }
        Some(value)
    }
}
