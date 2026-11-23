//! Animation manager (< 200 lines)
use std::sync::{Arc, Mutex};

/// Animation manager for multiple players
pub struct AnimationManager {
    active_players: Vec<Box<dyn AnimationPlayerBase>>,
    global_speed: f32,
}

pub trait AnimationPlayerBase {
    fn update(&mut self, dt: f32);
    fn is_finished(&self) -> bool;
}

impl AnimationManager {
    pub fn new() -> Self {
        Self {
            active_players: Vec::new(),
            global_speed: 1.0,
        }
    }

    pub fn add_player(&mut self, player: Box<dyn AnimationPlayerBase>) {
        self.active_players.push(player);
    }

    pub fn update(&mut self, dt: f32) {
        let scaled_dt = dt * self.global_speed;
        for player in &mut self.active_players {
            player.update(scaled_dt);
        }
        self.active_players.retain(|p| !p.is_finished());
    }

    pub fn stop_all(&mut self) {
        self.active_players.clear();
    }

    pub fn active_count(&self) -> usize {
        self.active_players.len()
    }
}

/// Thread-safe handle
pub type AnimationManagerHandle = Arc<Mutex<AnimationManager>>;

pub fn create_animation_manager() -> AnimationManagerHandle {
    Arc::new(Mutex::new(AnimationManager::new()))
}
