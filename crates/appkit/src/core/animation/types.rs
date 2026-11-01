//! Animation types and traits (< 200 lines)

/// Animation state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SCNAnimationState {
    Stopped,
    Running,
    Paused,
}

/// Timing function for animations
#[derive(Debug, Clone, Copy)]
pub enum SCNTimingFunction {
    Linear,
    EaseIn,
    EaseOut,
    EaseInEaseOut,
}

impl SCNTimingFunction {
    pub fn apply(&self, t: f32) -> f32 {
        match self {
            Self::Linear => t,
            Self::EaseIn => t * t,
            Self::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            Self::EaseInEaseOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
        }
    }
}

/// Animation event callback
pub type SCNAnimationEvent = Box<dyn Fn(f32) + Send + Sync>;

/// Interpolatable trait for animated types
pub trait Interpolatable: Clone {
    fn interpolate(a: &Self, b: &Self, t: f32) -> Self;
    fn default_value() -> Self;
}

impl Interpolatable for f32 {
    fn interpolate(a: &Self, b: &Self, t: f32) -> Self {
        a + (b - a) * t
    }
    fn default_value() -> Self {
        0.0
    }
}

impl Interpolatable for f64 {
    fn interpolate(a: &Self, b: &Self, t: f32) -> Self {
        a + (b - a) * t as f64
    }
    fn default_value() -> Self {
        0.0
    }
}

/// Keyframe for animation
#[derive(Debug, Clone)]
pub struct SCNKeyframe<T: Interpolatable> {
    pub time: f32,
    pub value: T,
}

impl<T: Interpolatable> SCNKeyframe<T> {
    pub fn new(time: f32, value: T) -> Self {
        Self { time, value }
    }
}
