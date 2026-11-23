use crate::animation::{Interpolatable, SCNAnimation, SCNAnimationPlayer, SCNKeyframe};
use std::any::Any;

pub trait TimelineTrackBase: Send + Sync {
    fn name(&self) -> &str;
    fn color(&self) -> [f32; 4];
    fn update(&mut self, dt: f32);
    fn as_any(&self) -> &dyn Any;
}

pub struct TimelineTrack<T: Clone + Interpolatable + 'static> {
    pub name: String,
    pub animation: SCNAnimation<T>,
    pub player: Option<SCNAnimationPlayer<T>>,
    pub keyframes: Vec<SCNKeyframe<T>>,
    pub color: [f32; 4],
}

impl<T: Clone + Interpolatable + Send + Sync + 'static> TimelineTrackBase for TimelineTrack<T> {
    fn name(&self) -> &str {
        &self.name
    }
    fn color(&self) -> [f32; 4] {
        self.color
    }
    fn update(&mut self, dt: f32) {
        if let Some(ref mut p) = self.player {
            p.update(dt);
        }
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<T: Clone + Interpolatable + Send + Sync + 'static> TimelineTrack<T> {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            animation: SCNAnimation::new(5.0),
            player: None,
            keyframes: vec![],
            color: [0.0, 0.48, 1.0, 1.0],
        }
    }

    pub fn with_keyframe(mut self, time: f32, value: T) -> Self {
        self.keyframes.push(SCNKeyframe { time, value });
        self
    }

    pub fn build(&mut self) {
        // Sort keyframes by time
        self.keyframes
            .sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());

        // Update animation duration if needed
        if let Some(last) = self.keyframes.last() {
            self.animation.duration = std::time::Duration::from_secs_f32(last.time);
        }

        // Create player
        self.player = Some(SCNAnimationPlayer::new(self.animation.clone()));
    }
}
