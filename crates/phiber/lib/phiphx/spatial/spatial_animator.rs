//! 60fps Spring Interpolation & 3D Spatial Matrix Projection Engine for PhiQui

#[derive(Debug, Clone, Copy)]
pub struct SpatialSpring {
    pub value: f32,
    pub target: f32,
    pub velocity: f32,
    pub stiffness: f32,
    pub damping: f32,
}

impl SpatialSpring {
    pub fn new(initial_value: f32, stiffness: f32, damping: f32) -> Self {
        Self {
            value: initial_value,
            target: initial_value,
            velocity: 0.0,
            stiffness,
            damping,
        }
    }

    pub fn set_target(&mut self, new_target: f32) {
        self.target = new_target;
    }

    pub fn update(&mut self, delta_time: f32) -> f32 {
        let force = (self.target - self.value) * self.stiffness;
        let damping_force = -self.velocity * self.damping;
        let acceleration = force + damping_force;

        self.velocity += acceleration * delta_time;
        self.value += self.velocity * delta_time;

        self.value
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SpatialTransform3D {
    pub pos_x: f32,
    pub pos_y: f32,
    pub pos_z: f32,
    pub rot_x: f32,
    pub rot_y: f32,
    pub rot_z: f32,
    pub scale: f32,
}

impl SpatialTransform3D {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            pos_x: x,
            pos_y: y,
            pos_z: z,
            rot_x: 0.0,
            rot_y: 0.0,
            rot_z: 0.0,
            scale: 1.0,
        }
    }

    /// Project 3D World Coordinates (X, Y, Z) to 2D Screen Space (X, Y) with Perspective Scale
    pub fn project_to_screen(
        &self,
        viewport_w: f32,
        viewport_h: f32,
        fov_distance: f32,
    ) -> (f32, f32, f32) {
        let distance = (self.pos_z + fov_distance).max(1.0);
        let perspective_scale = fov_distance / distance;

        let screen_x = (viewport_w / 2.0) + (self.pos_x * perspective_scale);
        let screen_y = (viewport_h / 2.0) - (self.pos_y * perspective_scale);
        let final_scale = self.scale * perspective_scale;

        (screen_x, screen_y, final_scale)
    }

    /// Calculate 3D Parallax Tilt Offset based on mouse pointer delta
    pub fn compute_parallax_tilt(&self, mouse_x: f32, mouse_y: f32) -> (f32, f32) {
        let tilt_x = self.rot_x + (mouse_y * 0.05);
        let tilt_y = self.rot_y + (mouse_x * 0.05);
        (tilt_x, tilt_y)
    }
}

impl Default for SpatialTransform3D {
    fn default() -> Self {
        Self {
            pos_x: 0.0,
            pos_y: 0.0,
            pos_z: 0.0,
            rot_x: 0.0,
            rot_y: 0.0,
            rot_z: 0.0,
            scale: 1.0,
        }
    }
}
